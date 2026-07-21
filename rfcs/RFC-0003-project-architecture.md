# RFC-0003: Project Architecture

* **Status:** Accepted
* **Created:** 2026-07-21
* **Authors:** MurexDB Engineering
* **Category:** Architecture & System Design

---

# 1. Summary

This RFC specifies the workspace crate dependency graph, system architecture, and module responsibilities for **MurexDB**.

---

# 2. Crate Dependency Architecture

MurexDB strictly enforces clean layer separation across four workspace member crates:

```text
       ┌────────────────┐           ┌────────────────┐
       │  murex-server  │           │  murex-client  │
       └───────┬────────┘           └───────┬────────┘
               │                            │
               ├────────────────────┐       │
               ▼                    ▼       ▼
       ┌───────────────┐          ┌───────────────────┐
       │ murex-protocol│─────────►│   murex-common    │
       └───────────────┘          └───────────────────┘
```

### 1. `murex-common` (Foundation Layer)
* Contains shared primitives, error types (`MurexError`), configuration structs, and common data structures.
* Dependencies: Zero internal crate dependencies.

### 2. `murex-protocol` (Wire Layer)
* Handles binary wire frame encoding, decoding, magic byte verification, and command parsing.
* Exposes the unified `Command` and `Response` enums.
* Dependencies: `murex-common`.

### 3. `murex-server` (Engine & Network Layer)
* Executable binary hosting the async TCP database server.
* Manages client connection sessions, executes commands against shared storage, and handles graceful shutdown signals.
* Dependencies: `murex-protocol`, `murex-common`, `tokio`.

### 4. `murex-client` (CLI & SDK Layer)
* Executable REPL CLI binary and client connection driver.
* Serializes user terminal commands into binary frames and displays responses.
* Dependencies: `murex-protocol`, `murex-common`, `tokio`.

---

# 3. Related Documents

* `README.md`
* `ROADMAP.md`
* `rfcs/RFC-0001-project-vision.md`
* `rfcs/RFC-0002-network-protocol.md`
* `rfcs/RFC-0004-concurrency-model.md`
