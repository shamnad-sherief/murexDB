# Changelog

All notable changes to MurexDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.3.0] - 2026-09-22

### Added
- **Milestone 3 — Write-Ahead Log (WAL) & Crash Recovery:**
  - **RFC-0006 Specification:** Comprehensive WAL architecture, binary frame layout, LSN sequencing, and crash-recovery protocol (`rfcs/RFC-0006-write-ahead-log.md`).
  - **Binary Write-Ahead Log Engine (`murex_server::wal`):**
    - 8-byte WAL File Header (`0x4D 0x58 0x57 0x4C` `"MXWL"`, version `0x0001`, reserved bytes).
    - Length-prefixed per-record binary frames with IEEE 802.3 CRC32 checksum verification.
    - Monotonically increasing 64-bit Log Sequence Numbers (LSN).
    - In-place checksum verification using streaming `crc32fast::Hasher`.
    - Key bounds (`MAX_KEY_LEN = 65,535`) and value bounds (`MAX_VAL_LEN = 67,108,864`) validation.
  - **Durability Invariant in Request Pipeline (`murex_server::handler`):**
    - Mutations (`SET`, `DELETE`) are logged to WAL and synced before in-memory `Database` application.
    - Thread-safe concurrent access via `Arc<tokio::sync::Mutex<WalWriter>>`.
  - **Crash Recovery & Replay on Startup (`murex_server::main`):**
    - Automatic inspection and sequential replay of `wal.log` via `WalReader::replay_into` upon server boot.
    - Restoration of sequence counter (`next_lsn`) to `last_lsn + 1`.
  - **Checkpoint-Based Truncation:**
    - Truncation of `wal.log` back to 8-byte header on successful snapshot saving during graceful shutdown.
  - **Test Suite:**
    - Comprehensive unit tests covering roundtrip replay, file truncation on checkpoint, CRC corruption detection, invalid magic/version header validation, and bounds checking.

## [v0.2.0] - 2026-08-21

### Added
- **Milestone 2 — Persistence & Binary Snapshot Engine:**
  - **RFC-0005 Storage Specification:** Binary snapshot file format specification (`rfcs/RFC-0005-storage-persistence.md`).
  - **Binary Snapshot Codec (`murex_server::snapshot`):**
    - 10-byte File Header framing (`0x4D 0x58 0x44 0x42` `"MXDB"` magic bytes, 2-byte schema version `0x0001`, 4-byte entry count).
    - Length-prefixed binary key-value record format (`u16 BE` key length, key bytes, `u32 BE` value length, value bytes).
    - Atomic rename write strategy (`data.db.tmp` $\rightarrow$ `data.db` with `BufWriter` & `sync_all` `fsync` protection).
    - Startup state recovery (`load_snapshot`) loading snapshots off disk into `Database` on server boot.
    - Snapshot save & load roundtrip unit test suite.

## [v0.1.0] - 2026-07-25


### Added
- **Milestone 1 — In-Memory Database Engine & CLI Client:**
  - **Binary Wire Protocol Codec (`murex-protocol`):**
    - Fixed 8-byte binary header framing (`0x4D 0x58` `"MX"` magic bytes, OpCode, Flags, 4-byte payload length) per RFC-0002.
    - Payload serialization/deserialization for `PING`, `GET`, `SET`, `DELETE`, `HELP`, `OK`, `NOT_FOUND`, `ERR_INVALID_FRAME`, `ERR_SERVER_ERROR`, `HELP`.
    - Async stream helpers `read_command`, `write_command`, `read_response`, `write_response`.
    - Roundtrip binary unit test suite.
  - **In-Memory Storage & Async TCP Server (`murex-server`):**
    - Thread-safe `Database` state model using `Arc<RwLock<HashMap<Key, Value>>>` per RFC-0004.
    - Session handler loop (`handle_client`) reading command frames and sending binary response frames.
    - Tokio multi-threaded TCP listener (`127.0.0.1:6739`) with graceful shutdown (`ctrl_c`).
    - Multi-client concurrent TCP integration test suite (`tests/server_test.rs`).
  - **CLI Client Application (`murex-client`):**
    - `MurexClient` connection wrapper over TCP streams.
    - Single-command CLI runner mode (`cargo run --bin murex-client -- set key val`).
    - Interactive REPL terminal prompt mode (`murex> `).

## [v0.0.1] - 2026-07-21


### Added
- **Workspace Infrastructure:** Initialized Cargo workspace with `murex-client`, `murex-server`, `murex-protocol`, and `murex-common`.
- **Architectural RFCs:**
  - `RFC-0001`: Project Vision & Goals.
  - `RFC-0002`: Binary Wire Protocol & 8-byte framing layout.
  - `RFC-0003`: Workspace Crate Hierarchy.
  - `RFC-0004`: Server Concurrency & `Arc<RwLock<HashMap>>` State Model.
- **Code Quality & CI:** Configured `rustfmt.toml`, `clippy.toml`, and `.github/workflows/ci.yml`.
- **Documentation:** Initial `README.md` and `ROADMAP.md`.
