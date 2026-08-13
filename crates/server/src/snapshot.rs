use bytes::BufMut;

use crate::Database;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

pub const MAGIC_BYTES: [u8; 4] = [0x4D, 0x58, 0x44, 0x42]; // "MXDB" magic byte
pub const DB_VERSION: u16 = 1;

pub async fn load_snapshot<P: AsRef<Path>>(path: P) -> murex_common::Result<Database> {
    if !path.as_ref().exists() {
        Ok(Database::new())
    } else {
        let mut reader = BufReader::new(File::open(path)?);
        let mut header = [0; 10];
        reader.read_exact(&mut header)?;
        if header[0..4] != MAGIC_BYTES {
            return Err(murex_common::MurexError::InvalidFrame(
                "Invalid snapshot magic bytes".into(),
            ));
        }
        let db = Database::new();
        let entry_count = u32::from_be_bytes(header[6..10].try_into().unwrap());

        for _ in 0..entry_count {
            // read 2 byte key length
            let mut key_len_bytes = [0u8; 2];
            reader.read_exact(&mut key_len_bytes)?;

            let key_len = u16::from_be_bytes(key_len_bytes) as usize;

            let mut key = vec![0u8; key_len];
            reader.read_exact(&mut key)?;

            // read 4 byte value lenght
            let mut val_len_bytes = [0u8; 4];
            reader.read_exact(&mut val_len_bytes)?;

            let val_len = u32::from_be_bytes(val_len_bytes) as usize;
            let mut val = vec![0u8; val_len];
            reader.read_exact(&mut val)?;

            db.set(key, val).await;
        }
        Ok(db)
    }
}

pub async fn save_snapshot<P: AsRef<Path>>(db: &Database, path: P) -> murex_common::Result<()> {
    // create a temporary file
    let tmp_path = path.as_ref().with_extension("tmp");

    let mut writer = BufWriter::new(File::create(&tmp_path)?);
    let entries = db.entries().await;

    let mut header = [0; 10];
    header[0..4].copy_from_slice(&MAGIC_BYTES);
    header[4..6].copy_from_slice(&DB_VERSION.to_be_bytes());
    header[6..10].copy_from_slice(&(entries.len() as u32).to_be_bytes());

    writer.write_all(&header)?;

    for (key, val) in entries {
        let mut record = bytes::BytesMut::with_capacity(2 + key.len() + 4 + val.len());
        record.put_u16(key.len() as u16);
        record.put_slice(&key);

        record.put_u32(val.len() as u32);
        record.put_slice(&val);

        writer.write_all(&record)?;
    }

    writer.flush()?;

    writer.get_ref().sync_all()?;

    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_murex_snapshot.db");

        // Clean up any old test file if present
        let _ = std::fs::remove_file(&db_path);

        // 1. Create DB and insert keys
        let db = Database::new();
        db.set(b"user:1".to_vec(), b"Alice".to_vec()).await;
        db.set(b"user:2".to_vec(), b"Bob".to_vec()).await;

        // 2. Save snapshot to disk
        save_snapshot(&db, &db_path).await.unwrap();

        // 3. Load snapshot back into a new Database
        let loaded_db = load_snapshot(&db_path).await.unwrap();

        // 4. Verify data survived disk roundtrip!
        assert_eq!(loaded_db.get(b"user:1").await, Some(b"Alice".to_vec()));
        assert_eq!(loaded_db.get(b"user:2").await, Some(b"Bob".to_vec()));
        assert_eq!(loaded_db.get(b"user:3").await, None);

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
    }
}
