# RFC-0001: Project Vision

* **Status:** Accepted
* **Created:** 2026-07-21
* **Authors:** MurexDB Engineering
* **Category:** Vision

---

# Summary

This RFC defines the long-term vision and guiding principles of MurexDB.

MurexDB is an open-source, modular database written in Rust that serves as the storage foundation for AI agents. The project is built incrementally from first principles, beginning with a production-quality database engine before evolving into a specialized platform for AI agent memory and infrastructure.

This RFC establishes **why** MurexDB exists. Future RFCs define **how** it will be built.

---

# Motivation

Modern AI agents require persistent, reliable, and efficient storage for:

* Conversation history
* Long-term memory
* Agent sessions
* Tool execution history
* Context retrieval

While existing databases can store this information, MurexDB is designed specifically to provide a modular storage foundation that can evolve alongside AI systems.

Rather than building AI-specific capabilities immediately, MurexDB first focuses on creating a solid database engine. AI-oriented features are introduced only after the core engine is complete.

---

# Goals

MurexDB aims to:

* Build a modular database engine in Rust.
* Prioritize correctness, maintainability, and reliability.
* Grow incrementally through well-defined milestones.
* Serve as the storage foundation for AI agents.
* Remain understandable, extensible, and contributor-friendly.

---

# Non-Goals

The following are intentionally out of scope for the initial versions of MurexDB:

* Competing directly with PostgreSQL.
* Becoming a general-purpose SQL database.
* Distributed clustering in early milestones.
* Premature performance optimizations.
* AI-specific features before the database engine is mature.

These may be revisited in future RFCs as the project evolves.

---

# Guiding Principles

MurexDB follows a small set of core principles:

* Build incrementally.
* Keep the architecture modular.
* Avoid unnecessary complexity.
* Prefer correctness over feature count.
* Make architectural decisions through RFCs.
* Design for future expansion without implementing future milestones early.

---

# Related Documents

* `README.md`
* `ROADMAP.md`
* `RFC-0002 — Network Protocol`
* `RFC-0003 — Project Architecture`
