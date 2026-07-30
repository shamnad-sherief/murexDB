# RFC-0005: Binary Snapshot File Format & Storage Persistence

* **Status:** Accepted
* **Created:** 2026-07-30
* **Category:** Storage & Persistence

---

# 1. Summary

This RFC specifies the binary file format, atomic save mechanism, and startup state recovery process for **MurexDB** (Milestone 2). The persistence engine enables the in-memory `Database` (`Arc<RwLock<HashMap<Key, Value>>>`) to be written to a compact binary snapshot file on disk (`data.db`) and restored automatically when the database process restarts.

All snapshot encoding, decoding, and file I/O utilities reside in `murex-server` (or a dedicated storage module).

---

# 2. Motivation

In Milestone 1, MurexDB operated purely in-memory. Any process shutdown or server crash resulted in total data loss.

To achieve data durability without sacrificing runtime execution speed, MurexDB implements a **Snapshot Persistence Engine**.

Human-readable text formats (JSON/TOML) were evaluated and rejected due to:
1. $O(N)$ character-by-character string parsing on server boot.
2. Inability to store raw binary key/value bytes without Base64 encoding bloat (33% disk penalty).
3. Heap memory allocation churn during deserialization.

A custom binary format provides:
- Fast, $O(1)$ header validation.
- Direct binary byte slice decoding.
- Compact length-prefixed records with zero formatting overhead.

---

# 3. Snapshot File Layout Specification

A MurexDB snapshot file (`data.db`) consists of a **File Header** followed by a stream of **Length-Prefixed Records**. All multi-byte integers are encoded in **Big-Endian (BE)** network byte order.

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Magic Bytes ("MXDB")                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Version (u16)         |       Entry Count (u32)       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Entry Count                           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   Record 1 (Length-Prefixed)                  |
|                               ...                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   Record N (Length-Prefixed)                  |
|                               ...                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

## 3.1 File Header (10 Bytes)

| Field | Size | Type | Value / Description |
|---|---|---|---|
| **Magic Bytes** | 4 Bytes | `[u8; 4]` | `[0x4D, 0x58, 0x44, 0x42]` (ASCII `"MXDB"`) |
| **Version** | 2 Bytes | `u16 BE` | `0x0001` (Version 1) |
| **Entry Count** | 4 Bytes | `u32 BE` | Number of key-value records in the snapshot |

---

## 3.2 Record Layout

Each key-value record is encoded sequentially:

```text
+-------------------+-------------------+---------------------+---------------------+
| Key Length (u16)  | Key Bytes         | Value Length (u32)  | Value Bytes         |
| (2 Bytes BE)      | (KeyLen Bytes)    | (4 Bytes BE)        | (ValLen Bytes)      |
+-------------------+-------------------+---------------------+---------------------+
```

1. **Key Length:** 2-byte unsigned integer in Big-Endian (`u16`). Max key length is 65,535 bytes (64 KB).
2. **Key Bytes:** Raw binary key slice of exact length `Key Length`.
3. **Value Length:** 4-byte unsigned integer in Big-Endian (`u32`). Max value length is 67,108,864 bytes (64 MB).
4. **Value Bytes:** Raw binary value slice of exact length `Value Length`.

---

# 4. Atomic Save & Recovery Workflow

## 4.1 Atomic Save Algorithm (Crash Protection)

To prevent data corruption if the server crashes or loses power while writing a snapshot, MurexDB uses an **Atomic Rename Write Strategy**:

1. Open a temporary file: `data.db.tmp`.
2. Write the 10-byte File Header (`"MXDB"` magic bytes, version 1, entry count).
3. Acquire a read lock on `Database` (`storage.read().await`).
4. Iterate over `HashMap` entries and write each length-prefixed record to `data.db.tmp`.
5. Flush and sync file buffers to physical disk storage (`file.sync_all()`).
6. Atomically replace the old database file with the temporary file (`std::fs::rename("data.db.tmp", "data.db")`).

> **Safety Guarantee:** Operating systems guarantee that `rename` is an atomic filesystem operation. If power fails mid-write, the original `data.db` remains undamaged.

---

## 4.2 Startup Recovery Algorithm

When `murex-server` initializes:

1. Check if `data.db` exists. If the file does not exist, start with an empty `HashMap`.
2. Open `data.db` for reading.
3. Read and validate the 10-byte File Header:
   - Check `magic == [0x4D, 0x58, 0x44, 0x42]`. Reject invalid files.
   - Check `version == 1`. Reject unsupported schema versions.
4. Loop through `Entry Count`:
   - Read `key_len` (`u16`).
   - Read `key_len` bytes into `key`.
   - Read `val_len` (`u32`).
   - Read `val_len` bytes into `val`.
   - Insert `(key, val)` into the in-memory `HashMap`.
5. Log recovery statistics (e.g. `"Loaded 1,500 records from data.db"`).

---

# 5. Security & Bounds Checking

- **Magic Byte Validation:** Prevents parsing arbitrary or corrupted non-database files.
- **Maximum Bounds Checking:** Rejects any record with `key_len > 65_535` or `val_len > 67_108_864` to protect against memory exhaustion attacks.
- **Unexpected EOF Handling:** If file reading encounters EOF before `Entry Count` is satisfied, report file truncation error.

---

# 6. Future Compatibility

Future milestones will introduce:
- **Write-Ahead Logging (Milestone 3):** Replaying logs captured after the last snapshot.
- **Checksums (CRC32):** Adding checksum verification to record headers.
