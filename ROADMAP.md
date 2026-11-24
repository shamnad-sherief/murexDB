# MurexDB Roadmap

> **Mission**
>
> MurexDB is an embedded, crash-safe storage engine in Rust for **lossless AI agent context**.
>
> It keeps every message an agent sees, stores summaries as a graph that always points back to the original messages, and serves context to agent harnesses. It runs as a single binary: no PostgreSQL, no vector database, and no LLM calls on the write path.

---

# Vision

Agent sessions keep getting longer. Even with very large context windows, models get worse as the window fills, and harnesses respond by compacting history. Compaction usually throws the original messages away.

MurexDB is the storage layer that makes compaction **lossless**:

* **Immutable history.** Every user message, assistant response and tool result is stored verbatim and never modified.
* **Summary DAG.** Summaries of messages, and summaries of summaries, are stored as derived nodes with provenance edges back to their sources. A summary can always be expanded back into its originals.
* **Context assembly.** Given a session and a token budget, return the recent messages plus the summary nodes that fit.
* **Retrieval.** Regex and full-text search over the full history, grouped by the summary that covers each match.
* **Large-file references.** Store a path, metadata and a short summary instead of duplicating large files.

The storage model follows the requirements published in the LCM paper (*Lossless Context Management*, Voltropy, 2026): an immutable message store, transactional writes with referential integrity, and indexed full-text search.

### Who it is for

* People building agent harnesses (Claude Code, OpenCode, Codex CLI, Goose, custom agents) who need long sessions without losing history.
* Rust-native harnesses in particular, which can embed MurexDB as a library crate.
* Teams that need an auditable record of what an agent saw and when.

### What MurexDB does not do

* **Summarize text.** The harness or its LLM writes summaries. MurexDB stores and serves them.
* **General-purpose database features.** No SQL or custom query language, no general secondary indexes.
* **LLM-based memory extraction.** mem0 and cortex-mem already cover that.
* **Vector search in the MVP.** It may be added later as an optional retrieval path.

### The question every milestone must answer

> *Why not just use SQLite with FTS5?*

SQLite already provides transactions, foreign keys and full-text search. MurexDB must win on something measurable:

* a purpose-built context API (`compact`, `expand`, `context(budget)`), not a schema everyone designs themselves;
* immutability and no-orphan invariants enforced by the engine;
* crash-tested, benchmarked append and search performance.

The benchmarks are kept honest. If SQLite wins somewhere, the README says so.

---

# Engineering Principles

Every milestone must:

* Solve one engineering problem.
* Teach one infrastructure concept.
* Move the project closer to a usable MVP.
* Be independently releasable.
* Include tests and documentation.

Future milestones should influence architecture, **not implementation**.

---

# Phase 1 — Core Engine *(Completed)*

## Milestone 0 — Foundation *(Completed, v0.0.1)*

Cargo workspace, repository structure, RFC process, CI, rustfmt, Clippy, CHANGELOG, CONTRIBUTING.

## Milestone 1 — In-Memory Database *(Completed, v0.1.0)*

Async Tokio TCP server, CLI client, in-memory key-value store (`SET`, `GET`, `DELETE`, `PING`, `HELP`), concurrent clients, graceful shutdown.

## Milestone 2 — Persistence *(Completed, v0.2.0)*

Binary snapshot format (RFC-0005), atomic rename, startup recovery.

## Milestone 3 — Write-Ahead Log *(Completed, v0.3.0)*

Append-only WAL with CRC32 checksums (RFC-0006), log replay, crash recovery, checkpoint truncation. Follow-up hardening of torn writes at EOF is in progress.

**Carried forward:** the WAL, crash recovery, the Tokio server, the binary protocol and the RFC process.

---

# Phase 2 — Context Store MVP

Target: a public MVP in about six weeks at 15–20 hours per week.

## Milestone 4 — Context Data Model *(Current)*

### Goal

Define and store the core context entities.

### Deliverables

* **Research:** how OpenCode, Codex CLI and Goose store sessions today, and the pain points MurexDB would solve.
* **Baseline:** a small SQLite + FTS5 prototype of the same data model, used as the benchmark reference from now on.
* **RFC-0007 — Context Data Model:** sessions, messages, summaries (leaf and condensed), provenance edges and file references, with their identifiers and invariants.
* Append-only message storage in log segments, with an index from message ID to segment offset.
* An embeddable library crate exposing `append`.

### Out of scope

Summaries, search, MCP.

### Learning

Data modelling, log-structured storage, segment files, immutability.

---

## Milestone 5 — Transactions & the Summary DAG

### Goal

Store summaries atomically, with no orphaned nodes.

### Deliverables

* WAL transactions spanning several records, ending with a commit marker. Recovery discards uncommitted transactions.
* `compact`: one transaction that writes a summary node and all of its provenance edges.
* `expand(summary_id)`: return the messages or summaries a node was built from.
* An integrity checker that verifies every edge points to an existing node.

### Learning

Atomicity, commit protocols, referential integrity, crash consistency.

---

## Milestone 6 — Retrieval

### Goal

Let agents find and assemble context.

### Deliverables

* `grep(pattern, scope?)`: regex search over the full history, optionally limited to one summary's messages, with results grouped by covering summary and paginated.
* `context(session, token_budget)`: recent messages plus the summary nodes that fit the budget.
* `describe(id)`: metadata for any message, summary or file reference.

### Learning

Scan performance, pagination, budget-constrained selection, API design.

---

## Milestone 7 — Harness Integration

### Goal

Make MurexDB usable from a real agent harness.

### Deliverables

* A stdio MCP server exposing `grep`, `expand` and `describe` as tools.
* One real harness integration (a Rust crate integration, an OpenCode plugin or a proxy), because MCP tools alone cannot control what the harness puts in the context window.
* A working end-to-end demo.

### Learning

Protocol integration, API ergonomics, what users actually need.

---

## Milestone 8 — Proving It Works

### Goal

Show that MurexDB is correct and fast, with evidence.

### Deliverables

* A crash-test harness that kills the process mid-write in a loop and verifies invariants after every restart.
* Criterion benchmarks for append, compact, grep and context assembly against the SQLite + FTS5 baseline.
* Structured logging and basic metrics.

### Learning

Fault injection, benchmarking methodology, observability.

---

## Milestone 9 — Public MVP *(v0.9.0)*

### Goal

Ship something people can use.

### Deliverables

* A README with a quick start, a demo GIF and published benchmark results.
* Published to crates.io.
* A launch post on r/rust, the Rust users' forum and Show HN.

### Outcome

An embeddable, crash-safe, lossless context store for agent harnesses, backed by published benchmarks and crash tests.

---

# Phase 3 — After the MVP

The order is decided by user feedback. Nothing here starts before the MVP ships.

* An inverted full-text index to replace brute-force grep.
* Segment compaction and garbage collection.
* An HTTP API, plus Python and TypeScript SDKs.
* Concurrent writers to the same session.
* An optional embedding index over summary nodes.
* Retention and export policies for audit use cases.
* Replication, only if users ask for it.

---

# Removed from the previous roadmap

These milestones were dropped on 2026-11-24 when the project refocused on lossless agent context:

* **StorageEngine trait abstraction.** Replaced by the context data model.
* **General secondary indexes.** Only the indexes the context store needs are built.
* **Query language** (lexer, parser, AST). Not needed; the API is a small set of context operations.
* **General ACID transactions** (`BEGIN`, `COMMIT`, `ROLLBACK`). Replaced by the narrow, multi-record WAL transactions in Milestone 5.
* **The v1.x AI memory series** (conversation storage, TTL, tagging, semantic retrieval, embeddings). Replaced by Phase 2, which delivers a usable product much earlier.

---

# Development Workflow

Every significant feature follows the same process:

1. Define the problem
2. Research existing approaches (including the SQLite baseline)
3. Write an RFC (when architecture changes)
4. Review the design
5. Implement
6. Test, including crash tests for anything on the write path
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
* Every milestone must make the project more usable.
* Prefer simplicity over unnecessary complexity.
* Prioritize correctness, maintainability and reliability.
