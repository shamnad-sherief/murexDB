# Changelog

All notable changes to MurexDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
