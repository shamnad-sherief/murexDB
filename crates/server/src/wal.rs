use std::{
    io::{Read, Write},
    path::Path,
};

use crate::Database;
use bytes::BufMut;
use murex_common::Result;

pub const MAX_KEY_LEN: usize = 65_535 as usize; // 64KB
pub const MAX_VAL_LEN: u32 = 67_108_864; // 64MB

pub struct WalWriter {
    file: std::fs::File,
    pub next_lsn: u64,
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
        record.put_u32(0 as u32);

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

enum FsyncMode {
    Full,
    WAL,
    None,
}
