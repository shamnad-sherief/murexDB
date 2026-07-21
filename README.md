# MurexDB

> **A modular database engine for AI agent memory, written in Rust.**

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Overview

**MurexDB** is an open-source database written in Rust that is being built incrementally from first principles. Its long-term vision is to become the storage foundation for AI agents, supporting conversations, long-term memory, tool execution history, and context retrieval.

Rather than implementing AI-specific capabilities immediately, MurexDB first focuses on building a production-quality database engine. AI memory features are introduced only after the core storage engine is complete.


### Core Focus Areas
* **Phase 1 — Database Engine:** In-memory key-value store, persistence, Write-Ahead Logging (WAL), modular storage engine traits, secondary indexes, query lexer/parser, ACID transactions, and observability.
* **Phase 2 — AI Agent Infrastructure:** Structured conversation threads, agent session checkpoints, tool execution logs, TTL lifecycle policies, metadata indexing, semantic retrieval, and embedding storage.

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

## Roadmap & Engineering Principles

MurexDB follows an incremental, RFC-driven development workflow:

* **Milestone 0 — Foundation** *(Current)*: Cargo workspace, code quality standards, repository architecture.
* **Milestone 1 — In-Memory Database**: Tokio async TCP server, CLI client, in-memory KV engine (`GET`, `SET`, `DELETE`, `PING`).
* **Milestone 2 — Persistence**: Disk snapshotting, serialization, and recovery.
* **Milestone 3 — Write-Ahead Log (WAL)**: Crash durability, log replay, append-only storage.
* **Milestone 4 — Storage Engine Abstraction**: Modular `StorageEngine` trait interface.

---

## License

This project is licensed under the [MIT License](LICENSE).
