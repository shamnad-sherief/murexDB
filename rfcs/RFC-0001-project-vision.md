# RFC-0001: Project Vision

* **Status:** Accepted (Amended 2026-11-24)
* **Created:** 2026-07-21
* **Authors:** MurexDB Engineering
* **Category:** Vision

---

# Summary

This RFC defines the long-term vision and guiding principles of MurexDB.

MurexDB is an embedded, crash-safe storage engine written in Rust for **lossless AI agent context**. It stores every message an agent sees verbatim and immutably. It stores summaries as a DAG whose provenance edges always point back to the original messages. It serves context to agent harnesses under a token budget.

This RFC establishes **why** MurexDB exists. Future RFCs define **how** it will be built.

---

# Motivation

Agent sessions keep getting longer. Even models with very large context windows get worse as the window fills ("context rot"), so harnesses compact older history into summaries. Most compaction is lossy: once the originals are summarized, they are gone, and the agent cannot recover what it was told.

The LCM paper (*Lossless Context Management*, Voltropy, 2026) shows that lossless compaction works. It keeps an immutable message store and a hierarchical summary DAG with pointers back to every original. The paper states the storage layer needs only three properties:

1. Transactional writes, so a compaction is atomic.
2. Referential integrity, so no summary is orphaned.
3. Indexed full-text search over the full history.

Its reference implementation embeds PostgreSQL inside a TypeScript application. Existing agent memory systems take a different approach: mem0 and cortex-mem extract facts with an LLM call on every write and depend on an external vector database.

MurexDB provides the LCM storage layer as a single, embeddable Rust component, with no external database and no LLM on the write path.

---

# Goals

MurexDB aims to:

* Store agent history immutably and losslessly.
* Store summaries as a DAG with enforced provenance.
* Provide a small, purpose-built context API: `append`, `compact`, `expand`, `grep`, `context`, `describe`.
* Be crash-safe, and prove it with fault-injection tests.
* Be embeddable in Rust harnesses and reachable from others via MCP.
* Beat a SQLite + FTS5 baseline on measurable criteria, and publish the results honestly.
* Remain understandable, extensible and contributor-friendly.

---

# Non-Goals

The following are intentionally out of scope:

* Summarizing text. The harness or its LLM writes summaries; MurexDB stores and serves them.
* Becoming a general-purpose or SQL database, or offering a query language.
* LLM-based memory or fact extraction.
* Vector search in the MVP.
* Distributed clustering before there are users who need it.
* Premature performance optimizations.

These may be revisited in future RFCs as the project evolves.

---

# Guiding Principles

MurexDB follows a small set of core principles:

* Build incrementally.
* Every milestone must make the project more usable.
* Keep the architecture modular.
* Avoid unnecessary complexity.
* Prefer correctness over feature count.
* Make architectural decisions through RFCs.
* Design for future expansion without implementing future milestones early.

---

# Revision History

### 2026-11-24 — Refocus on lossless agent context

The original vision (2026-07-21) was to build a general-purpose, production-quality database engine first, including a storage engine abstraction, secondary indexes, a query language and full ACID transactions, and only then add AI memory features.

That plan delayed any usable product by many months and led toward a general database with no clear advantage over SQLite or Redis. The vision now targets a specific niche, the LCM storage layer, from the next milestone onward.

The completed work (TCP server, binary protocol, snapshots, WAL and crash recovery) carries forward. See `ROADMAP.md` for the milestones that were dropped.

---

# Related Documents

* `README.md`
* `ROADMAP.md`
* `RFC-0002 — Network Protocol`
* `RFC-0003 — Project Architecture`
* `RFC-0006 — Write-Ahead Log`
* `RFC-0007 — Context Data Model` (planned, Milestone 4)
