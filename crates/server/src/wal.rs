use std::{
    io::{Read, Write},
    path::Path,
};

use crate::Database;
use bytes::BufMut;
use murex_common::Result;

pub const MAX_KEY_LEN: usize = 65_535; // 64KB
pub const MAX_VAL_LEN: u32 = 67_108_864; // 64MB

pub struct WalWriter {
    file: std::fs::File,
    pub next_lsn: u64,
    #[allow(dead_code)]
    sync_mode: FsyncMode,
}

pub struct WalReader<R> {
    reader: R,
    pub last_lsn: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalRecord {
    Set {
        lsn: u64,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        lsn: u64,
        key: Vec<u8>,
    },
}

impl WalWriter {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = if path.as_ref().exists() {
            let mut f = std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .open(&path)?;
            let mut header = [0u8; 8];
            f.read_exact(&mut header)?;
            if header[0..4] != [0x4D, 0x58, 0x57, 0x4C] {
                return Err(murex_common::MurexError::InvalidFrame(
                    "Invalid WAL magic bytes".into(),
                ));
            }
            f
        } else {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;

            let mut header = [0; 8];
            header[0..4].copy_from_slice(b"MXWL");
            header[4..6].copy_from_slice(&1u16.to_be_bytes());
            header[6..8].copy_from_slice(&0u16.to_be_bytes());

            f.write_all(&header)?;
            f
        };
        Ok(WalWriter {
            file,
            next_lsn: 0,
            sync_mode: FsyncMode::WAL,
        })
    }

    pub fn append_set(&mut self, key: &[u8], value: &[u8]) -> Result<u64> {
        if key.is_empty() {
            return Err(murex_common::MurexError::InvalidFrame(
                "Key cannot be empty".to_owned(),
            ));
        }

        if key.len() > MAX_KEY_LEN {
            return Err(murex_common::MurexError::InvalidFrame(
                "Key length exceeds maximum allowed size".to_owned(),
            ));
        }

        if value.len() > MAX_VAL_LEN as usize {
            return Err(murex_common::MurexError::InvalidFrame(
                "Value length exceeds maximum allowed size".to_owned(),
            ));
        }

        let lsn = self.next_lsn;

        let mut record = bytes::BytesMut::with_capacity(15 + key.len() + value.len());

        record.put_u64(lsn);
        record.put_u8(0x01);
        record.put_u16(key.len() as u16);
        record.put_u32(value.len() as u32);

        record.put_slice(key);
        record.put_slice(value);

        let crc = crc32fast::hash(&record);

        self.file.write_all(&crc.to_be_bytes())?;
        self.file.write_all(&record)?;

        self.file.sync_all()?;

        self.next_lsn += 1;

        Ok(lsn)
    }

    pub fn append_delete(&mut self, key: &[u8]) -> Result<u64> {
        if key.is_empty() {
            return Err(murex_common::MurexError::InvalidFrame(
                "Key cannot be empty".to_owned(),
            ));
        }

        if key.len() > MAX_KEY_LEN {
            return Err(murex_common::MurexError::InvalidFrame(
                "Key length exceeds maximum allowed size".to_owned(),
            ));
        }

        let lsn = self.next_lsn;

        let mut record = bytes::BytesMut::with_capacity(15 + key.len());

        record.put_u64(lsn);
        record.put_u8(0x02);
        record.put_u16(key.len() as u16);
        record.put_u32(0);

        record.put_slice(key);

        let crc = crc32fast::hash(&record);

        // write crc first
        self.file.write_all(&crc.to_be_bytes())?;
        self.file.write_all(&record)?;

        self.file.sync_all()?;

        self.next_lsn += 1;

        Ok(lsn)
    }

    pub fn checkpoint<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;

        let mut header = [0; 8];
        header[0..4].copy_from_slice(b"MXWL");
        header[4..6].copy_from_slice(&1u16.to_be_bytes());
        header[6..8].copy_from_slice(&0u16.to_be_bytes());

        file.write_all(&header)?;

        Ok(WalWriter {
            file,
            next_lsn: 0,
            sync_mode: FsyncMode::Full,
        })
    }
}

impl<R: std::io::Read> WalReader<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;

        // check if it is a valid WAL file
        if header[0..4] != [0x4D, 0x58, 0x57, 0x4C] {
            return Err(murex_common::MurexError::InvalidFrame(
                "Invalid WAL magic bytes".into(),
            ));
        }

        // check the version matches
        let version = u16::from_be_bytes(header[4..6].try_into().unwrap());
        if version != 1 {
            return Err(murex_common::MurexError::InvalidFrame(
                "Invalid WAL version".into(),
            ));
        }
        Ok(WalReader {
            reader,
            last_lsn: None,
        })
    }

    pub async fn replay_into(&mut self, db: &Database) -> Result<usize> {
        let reader = &mut self.reader;

        let mut count = 0;

        loop {
            let mut crc_buf = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut crc_buf) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(e.into());
            }
            let crc = u32::from_be_bytes(crc_buf);

            let mut lsn_buffer = [0u8; 8];
            reader.read_exact(&mut lsn_buffer)?;
            let lsn = u64::from_be_bytes(lsn_buffer);

            let mut op_code_buff = [0u8; 1];
            reader.read_exact(&mut op_code_buff)?;
            let op_code = u8::from_be_bytes(op_code_buff);

            let mut key_len_buff = [0u8; 2];
            reader.read_exact(&mut key_len_buff)?;
            let key_len = u16::from_be_bytes(key_len_buff);

            let mut val_len_buff = [0u8; 4];
            reader.read_exact(&mut val_len_buff)?;
            let val_len = u32::from_be_bytes(val_len_buff);

            let mut key_buff = vec![0u8; key_len as usize];
            reader.read_exact(&mut key_buff)?;

            let mut val_buff = vec![];

            if val_len > 0 {
                val_buff = vec![0u8; val_len as usize];
                reader.read_exact(&mut val_buff)?;
            }

            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&lsn_buffer);
            hasher.update(&op_code_buff);
            hasher.update(&key_len_buff);
            hasher.update(&val_len_buff);
            hasher.update(&key_buff);
            if val_len > 0 {
                hasher.update(&val_buff);
            }
            let generated_crc = hasher.finalize();

            if generated_crc != crc {
                return Err(murex_common::MurexError::InvalidFrame(
                    "CRC mismatch - WAL corrupted".into(),
                ));
            }

            match op_code {
                0x01 => {
                    db.set(key_buff, val_buff).await;
                }
                0x02 => {
                    db.delete(&key_buff).await;
                }
                _ => {
                    return Err(murex_common::MurexError::UnknownOpCode(op_code));
                }
            }
            count += 1;
            self.last_lsn = Some(lsn);
        }

        Ok(count)
    }
}

#[allow(dead_code, clippy::upper_case_acronyms)]
enum FsyncMode {
    Full,
    WAL,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn temp_wal_path(test_name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "murex_test_{}_{}.wal",
            test_name,
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[tokio::test]
    async fn test_wal_append_and_replay_roundtrip() {
        let path = temp_wal_path("roundtrip");
        let mut writer = WalWriter::open(&path).unwrap();

        let lsn0 = writer.append_set(b"user:1", b"Alice").unwrap();
        assert_eq!(lsn0, 0);

        let lsn1 = writer.append_set(b"user:2", b"Bob").unwrap();
        assert_eq!(lsn1, 1);

        let lsn2 = writer.append_delete(b"user:1").unwrap();
        assert_eq!(lsn2, 2);

        // Replay into a fresh DB
        let file = std::fs::File::open(&path).unwrap();
        let mut reader = WalReader::new(file).unwrap();
        let db = Database::new();

        let count = reader.replay_into(&db).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(reader.last_lsn, Some(2));

        assert_eq!(db.get(b"user:1").await, None);
        assert_eq!(db.get(b"user:2").await, Some(b"Bob".to_vec()));

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_wal_checkpoint_truncates_file() {
        let path = temp_wal_path("checkpoint");
        let mut writer = WalWriter::open(&path).unwrap();

        writer.append_set(b"k1", b"v1").unwrap();
        writer.append_set(b"k2", b"v2").unwrap();

        let metadata_before = std::fs::metadata(&path).unwrap();
        assert!(metadata_before.len() > 8);

        // Checkpoint should truncate back to 8 bytes
        let checkpoint_writer = WalWriter::checkpoint(&path).unwrap();
        assert_eq!(checkpoint_writer.next_lsn, 0);

        let metadata_after = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata_after.len(), 8);

        // Replaying after checkpoint should yield 0 records
        let file = std::fs::File::open(&path).unwrap();
        let mut reader = WalReader::new(file).unwrap();
        let db = Database::new();
        let count = reader.replay_into(&db).await.unwrap();
        assert_eq!(count, 0);
        assert_eq!(reader.last_lsn, None);

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_wal_crc_corruption_detection() {
        let path = temp_wal_path("corruption");
        let mut writer = WalWriter::open(&path).unwrap();

        writer
            .append_set(b"important_key", b"important_val")
            .unwrap();
        drop(writer);

        // Read the file, corrupt a byte in the payload, write back
        let mut bytes = std::fs::read(&path).unwrap();
        let last_idx = bytes.len() - 1;
        bytes[last_idx] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let mut reader = WalReader::new(file).unwrap();
        let db = Database::new();

        let res = reader.replay_into(&db).await;
        assert!(res.is_err(), "Corrupted WAL record should fail CRC check");
        match res.unwrap_err() {
            murex_common::MurexError::InvalidFrame(msg) => {
                assert!(msg.contains("CRC mismatch") || msg.contains("corrupted"));
            }
            other => panic!("Expected InvalidFrame error, got {:?}", other),
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_wal_invalid_magic_bytes() {
        let invalid_header = b"BADW\x00\x01\x00\x00";
        let cursor = Cursor::new(invalid_header);
        let res = WalReader::new(cursor);
        assert!(res.is_err());
    }

    #[test]
    fn test_wal_invalid_version() {
        let invalid_version = b"MXWL\x00\x02\x00\x00";
        let cursor = Cursor::new(invalid_version);
        let res = WalReader::new(cursor);
        assert!(res.is_err());
    }

    #[test]
    fn test_wal_bounds_validation() {
        let path = temp_wal_path("bounds");
        let mut writer = WalWriter::open(&path).unwrap();

        // Empty key
        assert!(writer.append_set(b"", b"value").is_err());
        assert!(writer.append_delete(b"").is_err());

        // Key too large (> 64KB)
        let large_key = vec![b'k'; MAX_KEY_LEN + 1];
        assert!(writer.append_set(&large_key, b"value").is_err());
        assert!(writer.append_delete(&large_key).is_err());

        let _ = std::fs::remove_file(&path);
    }
}
