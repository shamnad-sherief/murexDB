# RFC-0002: Binary Wire Protocol & Framing Specification

* **Status:** Accepted
* **Created:** 2026-07-21
* **Authors:** MurexDB Engineering
* **Category:** Protocol & Networking

---

# 1. Summary

This RFC specifies the binary wire protocol and frame layout for **MurexDB**. The protocol is designed to provide high-throughput, low-latency, zero-copy deserialization for key-value payloads, structured conversation logs, and raw binary vector embeddings (`Vec<f32>`).

All network framing logic resides in the `murex-protocol` crate.

---

# 2. Motivation

Text-based protocols (like HTTP/JSON or plain ASCII strings) require character-by-character scanning, delimiter searching, and CPU float-to-string parsing. For AI infrastructure handling large context windows and vector embeddings, text parsing introduces significant network bandwidth overhead and heap allocation latency.

A binary fixed-header protocol guarantees:
1. $O(1)$ constant-time header parsing.
2. Direct zero-copy byte slice references (`&[u8]`).
3. 60%+ reduction in network payload size for binary vectors.
4. Native support for arbitrary binary keys and values containing newline characters or non-ASCII data.

---

# 3. Binary Frame Specification

Every request and response frame over the TCP stream begins with a **fixed 8-byte header**, followed by a variable-length payload.

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Magic Bytes          |    OpCode     |     Flags     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Payload Length                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                            Payload                            |
|                              ...                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

## 3.1 Header Fields

| Field | Type | Size | Description |
| :--- | :--- | :--- | :--- |
| **Magic Bytes** | `[u8; 2]` | 2 Bytes | Protocol identifier. Must be `0x4D 0x58` (ASCII `"MX"`). |
| **OpCode** | `u8` | 1 Byte | Command or response operation type. |
| **Flags** | `u8` | 1 Byte | Bitmask for frame options (bit 0 = Compressed, bit 1 = Encrypted). |
| **Payload Length** | `u32` (BE) | 4 Bytes | Length of the payload section in bytes (Big-Endian). |

---

# 4. Command OpCodes (Milestone 1)

| OpCode | Command | Description | Payload Encoding |
| :--- | :--- | :--- | :--- |
| `0x01` | `PING` | Health check / heartbeat. | Optional byte string echo. |
| `0x02` | `GET` | Retrieve value for key. | `[Key Length: u16][Key Bytes]` |
| `0x03` | `SET` | Store key-value pair. | `[Key Length: u16][Key Bytes][Val Length: u32][Value Bytes]` |
| `0x04` | `DELETE` | Remove key from database. | `[Key Length: u16][Key Bytes]` |
| `0x05` | `HELP` | Request server command list. | Empty payload (`Length = 0`). |

---

# 5. Response Framing

Server responses use the same 8-byte header format. The `OpCode` field in a response header indicates status:

| OpCode Value | Status | Description |
| :--- | :--- | :--- |
| `0x80` | `OK / SUCCESS` | Command executed successfully. Payload contains result data. |
| `0x81` | `NOT_FOUND` | Key does not exist in storage. |
| `0x82` | `ERR_INVALID_FRAME` | Malformed frame or unknown Magic Bytes. |
| `0x83` | `ERR_SERVER_ERROR` | Internal server execution error. |

---

# 6. Unified Architecture (`crates/protocol`)

To keep protocol handling decoupled from storage engine logic:

1. `murex-protocol` reads bytes from Tokio TCP streams and decodes binary frames into a clean, strongly-typed Rust `Command` enum:

```rust
pub enum Command {
    Ping { message: Option<Vec<u8>> },
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Help,
}
```

2. `murex-server` receives `Command` objects from the protocol decoder, executes the storage operation, and serializes the response.

3. **Dual-Protocol Extension:** If a CLI client or HTTP debug port is added later, a text adapter parser will simply output the same `Command` enum variants without modifying `murex-server`.

---

# 7. Safety & Resource Bounds

* **Max Frame Size:** Default maximum payload length is **64 MB** (`67,108,864` bytes). Any frame declaring a `Payload Length` greater than 64 MB will be rejected immediately with `ERR_INVALID_FRAME` to prevent Out-Of-Memory (OOM) denial-of-service attacks.
* **Stream Buffer Alignment:** Sockets must read full 8-byte headers before allocating payload buffers.

---

# 8. Related Documents

* `README.md`
* `ROADMAP.md`
* `rfcs/RFC-0001-project-vision.md`
