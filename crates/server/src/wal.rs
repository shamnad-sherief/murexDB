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
    next_lsn: u64,
    sync_mode: FsyncMode,
}

pub struct WalReader<R> {
    reader: R,
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
        let mut file = std::fs::OpenOptions::new().read(true).open(&path)?;
        if path.as_ref().exists() {
            let mut header = [0u8; 8];
            file.read_exact(&mut header)?;
            if header[0..4] != [0x4D, 0x58, 0x57, 0x4C] {
                return Err(murex_common::MurexError::InvalidFrame(
                    "Invalid WAL magic bytes".into(),
                ));
            }
        } else {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;

            let mut header = [0; 8];
            header[0..4].copy_from_slice(b"MXWL");
            header[4..6].copy_from_slice(&1u16.to_be_bytes());
            header[6..8].copy_from_slice(&0u8.to_be_bytes());

            file.write_all(&header)?;
        }
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
        todo!("append the delete to the wal file");
    }

    pub fn checkpoint<P: AsRef<Path>>(path: P) -> Result<Self> {
        todo!("checkpoint")
    }
}

impl<R: std::io::Read> WalReader<R> {
    pub fn new(reader: R) -> Result<Self> {
        todo!("new reader")
    }
    pub fn replay_into(&mut self, db: &Database) -> Result<usize> {
        todo!("replay")
    }
}

enum FsyncMode {
    Full,
    WAL,
    None,
}
