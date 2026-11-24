# MurexDB

> **An embedded, crash-safe storage engine for lossless AI agent context, written in Rust.**

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Overview

**MurexDB** keeps everything an AI agent sees. It stores every message verbatim and never modifies it. It stores summaries as a graph whose edges always point back to the original messages, so any summary can be expanded back into its sources. It serves context to agent harnesses under a token budget.

It runs as a single binary or an embeddable Rust crate: no PostgreSQL, no vector database, and no LLM calls on the write path.

The storage model follows the requirements of the LCM paper (*Lossless Context Management*, Voltropy, 2026): an immutable message store, transactional writes with referential integrity, and indexed full-text search.

> **Status:** pre-MVP. The core engine (TCP server, snapshots, write-ahead log, crash recovery) is done. The context store is being built now. See [ROADMAP.md](ROADMAP.md).

### Planned API

| Operation | What it does |
|---|---|
| `append` | Store a message in a session, verbatim and immutable |
| `compact` | Atomically store a summary node and its provenance edges |
| `expand` | Return the messages or summaries a summary was built from |
| `grep` | Regex search over the full history, grouped by covering summary |
| `context` | Recent messages plus summaries that fit a token budget |
| `describe` | Metadata for any message, summary or file reference |

### Who it is for

People building agent harnesses (Claude Code, OpenCode, Codex CLI, Goose, custom agents) who need long sessions without losing history, and teams that need an auditable record of what an agent saw.

### What it is not

MurexDB does not summarize text (the harness does), is not a general-purpose or SQL database, and does not do LLM-based memory extraction like mem0 or cortex-mem.

---

## Workspace Architecture

MurexDB is organized as a modular Rust Cargo workspace:

```text
murexDB/
├── crates/
│   ├── client/       # (murex-client) CLI client interactive binary
│   ├── server/       # (murex-server) Async TCP database server binary
│   ├── protocol/     # (murex-protocol) Wire protocol definitions & serialization
│   └── common/       # (murex-common) Shared types, primitives, and error handling
├── benchmarks/       # Micro-benchmarks and performance test suites
├── docs/             # Technical design documents and architectural specs
├── examples/         # Usage examples and code samples
├── rfcs/             # Request for Comments (RFC) architectural decisions
└── tests/            # End-to-end and integration test suites
```

---

## Getting Started

### Prerequisites
* **Rust**: `1.85.0` or higher (Edition 2024).

### Building the Project
Clone the repository and build all workspace crates:

```bash
cargo build --workspace
```

### Running Tests and Verification
Ensure code quality, formatting, and tests pass:

```bash
# Type check all workspace crates
cargo check --workspace

# Run unit and integration tests
cargo test --workspace

# Run Clippy lints
cargo clippy --workspace -- -D warnings

# Check code formatting
cargo fmt --check
```

### Running Server and Client

Start the database server:
```bash
cargo run -p murex-server
```

In a separate terminal, launch the CLI client:
```bash
cargo run -p murex-client
```

---

## Roadmap

MurexDB follows an incremental, RFC-driven workflow. Full details are in [ROADMAP.md](ROADMAP.md).

**Phase 1 — Core Engine** *(Completed)*
* **M0 Foundation:** Cargo workspace, CI, code quality standards, RFC process.
* **M1 In-Memory Database:** Tokio async TCP server, CLI client, in-memory key-value store.
* **M2 Persistence:** binary snapshots (RFC-0005), atomic rename, startup recovery.
* **M3 Write-Ahead Log:** CRC32-checked append-only log (RFC-0006), replay, crash recovery.

**Phase 2 — Context Store MVP**
* **M4 Context Data Model** *(Current)*: RFC-0007, append-only message segments, SQLite + FTS5 baseline.
* **M5 Transactions & Summary DAG:** multi-record WAL transactions, `compact`, `expand`.
* **M6 Retrieval:** `grep`, `context`, `describe`.
* **M7 Harness Integration:** MCP server and one real harness integration.
* **M8 Proving It Works:** crash-test harness, benchmarks against SQLite.
* **M9 Public MVP (v0.9.0).**

---

## License

This project is licensed under the [MIT License](LICENSE).
