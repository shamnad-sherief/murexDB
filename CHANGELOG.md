# Changelog

All notable changes to MurexDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
