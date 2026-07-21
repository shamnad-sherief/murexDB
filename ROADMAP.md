# MurexDB Roadmap

> **Mission**
>
> MurexDB is an open-source, modular database written in Rust that serves as the storage foundation for AI agents.
>
> The project is built incrementally from first principles. It begins as a production-quality database engine and gradually evolves into a specialized storage platform for AI agent memory and infrastructure.

---

# Vision

The long-term vision of MurexDB is to provide the storage layer required by modern AI agents.

Future capabilities include:

* Conversation history
* Long-term memory
* Agent session checkpoints
* Tool execution history
* Policy storage
* Context retrieval
* Semantic retrieval
* Embedding storage

The database engine is built first. AI-specific capabilities are introduced only after the core engine is complete.

---

# Phase 1 — Database Engine

## Milestone 0 — Foundation *(Current)*

### Goal

Establish the project foundation and engineering workflow.

### Deliverables

* [x] Cargo Workspace
* [x] Repository Structure
* [x] README
* [x] ROADMAP
* [x] `.gitignore`
* [x] RFC Process
* [x] RFC-0002 — Network Protocol
* [ ] RFC-0003 — Project Architecture
* [ ] GitHub Actions
* [ ] Code Quality Configuration (`rustfmt`, Clippy)
* [ ] CHANGELOG
* [ ] CONTRIBUTING
* [ ] Initial Release (`v0.0.1`)

---

## Milestone 1 — In-Memory Database

### Goal

Build the smallest usable database.

### Deliverables

* Async Tokio TCP server
* CLI client
* Human-readable protocol
* In-memory key-value storage
* Commands:

  * `SET`
  * `GET`
  * `DELETE`
  * `PING`
  * `HELP`
* Multiple concurrent client connections
* Graceful shutdown

### Out of Scope

* Persistence
* Write-Ahead Log
* Transactions
* Secondary indexes
* Query language
* Replication
* AI memory features

---

## Milestone 2 — Persistence

### Goal

Persist data across process restarts.

### Deliverables

* Snapshot persistence
* Binary serialization
* Startup recovery

---

## Milestone 3 — Write-Ahead Log (WAL)

### Goal

Protect data against unexpected crashes.

### Deliverables

* Append-only WAL
* Log replay
* Crash recovery

---

## Milestone 4 — Storage Engine

### Goal

Separate storage implementation from database logic.

### Deliverables

* `StorageEngine` trait
* Pluggable storage interface
* Initial storage implementation

---

## Milestone 5 — Secondary Indexes

### Goal

Improve lookup performance.

### Deliverables

* Secondary indexes
* Lookup optimization
* Performance benchmarks

---

## Milestone 6 — Query Language

### Goal

Introduce a structured query language.

### Deliverables

* Lexer
* Parser
* Abstract Syntax Tree (AST)
* Query execution engine

---

## Milestone 7 — Transactions

### Goal

Guarantee data consistency.

### Deliverables

* `BEGIN`
* `COMMIT`
* `ROLLBACK`
* Isolation guarantees
* Concurrency control

---

## Milestone 8 — Observability & Benchmarking

### Goal

Operate MurexDB like production infrastructure.

### Deliverables

* Structured logging
* Tracing
* Metrics interface
* Benchmark suite

---

## Milestone 9 — Production Foundation (v1.0)

### Goal

Release a stable, production-quality single-node database engine.

### Deliverables

* Stability improvements
* Configuration
* Comprehensive testing
* Documentation polish
* Release automation

### Outcome

A reliable database engine that serves as the foundation for AI agent infrastructure.

---

# Phase 2 — AI Agent Memory

With the core database engine complete, MurexDB evolves into a storage platform optimized for AI agents.

---

## v1.1 — Conversation Storage

* Conversation history
* Thread management
* Conversation retrieval

---

## v1.2 — Tool Execution History

* Tool invocation logs
* Tool outputs
* Audit history

---

## v1.3 — Agent Sessions

* Session state
* Workflow checkpoints
* Task execution tracking

---

## v1.4 — Memory Management

* TTL policies
* Expiration
* Memory pinning
* Memory importance

---

## v1.5 — Metadata & Tagging

* Labels
* Categories
* Ownership
* Confidence scores

---

## v1.6 — Semantic Retrieval

* Context retrieval
* Similar memory lookup
* Relevance ranking

---

## v1.7 — Embedding Support

* Embedding storage
* Vector index integration

---

## v2.0 — Distributed Memory

* Replication
* High availability
* Distributed retrieval
* Multi-node storage

---

# Development Workflow

Every significant feature follows the same process:

1. Define the problem
2. Research existing approaches
3. Write an RFC (when architecture changes)
4. Review the design
5. Implement
6. Test
7. Benchmark (when applicable)
8. Document
9. Release

---

# Definition of Done

A milestone is complete only when:

* Planned features are implemented.
* Unit and integration tests pass.
* Documentation is updated.
* Relevant RFCs are updated.
* `cargo fmt` passes.
* `cargo clippy` passes.
* `cargo test` passes.
* The milestone is independently releasable.

---

# Guiding Principles

* Build for today's milestone.
* Design for tomorrow's expansion.
* Never implement future milestones early.
* Prefer simplicity over unnecessary complexity.
* Prioritize correctness, maintainability, and reliability.
