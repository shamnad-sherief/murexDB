# RFC-0004: Server Concurrency & State Management Model

* **Status:** Accepted
* **Created:** 2026-07-21
* **Authors:** MurexDB Engineering
* **Category:** System Design & Concurrency

---

# 1. Summary

This RFC specifies the Tokio asynchronous network model, connection lifecycle, and thread-safe shared state concurrency model for **MurexDB**.

---

# 2. Network Concurrency Model (Tokio Async)

MurexDB utilizes Tokio's multi-threaded asynchronous runtime (`#[tokio::main]`).

```text
 [ TCP Listener Loop ]
         │
         ├── Connection 1 ──► tokio::spawn(handle_client(stream_1, state.clone()))
         ├── Connection 2 ──► tokio::spawn(handle_client(stream_2, state.clone()))
         └── Connection N ──► tokio::spawn(handle_client(stream_N, state.clone()))
```

### Connection Handling Lifecycle
1. **Listener Task:** The main thread binds to a TCP address (e.g., `127.0.0.1:6379`) and executes an infinite `accept()` loop.
2. **Task Per Client:** Upon accepting a TCP socket connection, the server invokes `tokio::spawn` to hand off the `TcpStream` to an isolated, lightweight async task.
3. **Session Processing:** The client task reads fixed 8-byte binary headers via `murex-protocol`, parses the `Command`, executes the operation on shared storage state, and writes binary response frames back to the client socket.

---

# 3. State Concurrency Model (`Arc<RwLock<HashMap>>`)

To enable concurrent state access across multiple client connection tasks, state is shared using thread-safe reference counting and read-write locking:

```rust
pub type DbState = Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>;
```

### Design Justification

Milestone 1 uses `Arc<RwLock<HashMap>>` as the initial concurrency strategy because it provides a simple, idiomatic, and sufficiently concurrent implementation for an in-memory key-value store. 

* **Read-Heavy Optimization:** Database workloads are typically read-heavy (`GET` requests outnumber `SET`/`DELETE`). An `RwLock` allows **multiple concurrent readers** to access the `HashMap` simultaneously without blocking each other.
* **Direct Access without Channel Overhead:** Passing messages over `mpsc` channels to a single actor task serializes all read/write operations through one thread bottleneck. `Arc<RwLock<HashMap>>` leverages all CPU cores directly.
* **Evolution:** Future milestones may replace this implementation as the storage engine evolves (e.g., WAL in Milestone 3, `StorageEngine` trait in Milestone 4).

---

# 4. Storage Extension Point

To prevent coupling server networking logic directly to raw `HashMap` primitives, shared state is encapsulated within a database wrapper struct:

```rust
pub struct Database {
    storage: DbState,
}
```

This abstraction ensures that future milestones (Write-Ahead Logging in Milestone 3, StorageEngine traits in Milestone 4, and Secondary Indexes in Milestone 5) can replace the internal storage engine implementation without altering client connection handlers.

---

# 5. Related Documents

* `README.md`
* `ROADMAP.md`
* `rfcs/RFC-0001-project-vision.md`
* `rfcs/RFC-0002-network-protocol.md`
* `rfcs/RFC-0003-project-architecture.md`
