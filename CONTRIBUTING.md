# Contributing to MurexDB

Thank you for your interest in contributing to MurexDB! 

MurexDB is built incrementally from first principles to serve as production-quality AI agent storage infrastructure. We welcome bug reports, documentation improvements, and architectural RFC discussions.

---

## 1. Development Principles

Before submitting code:
* **RFC First for Architectural Changes:** Any major change to protocols, storage engines, or server interfaces requires an accepted RFC in `rfcs/` before implementation.
* **Build for Today, Design for Tomorrow:** Avoid over-engineering features outside the active milestone scope defined in [`ROADMAP.md`](ROADMAP.md).
* **Strict Quality Standards:** All PRs must pass `cargo check`, `cargo test`, `cargo fmt`, and `cargo clippy`.

---

## 2. Local Development Workflow

```bash
# Clone the repository
git clone https://github.com/your-username/murexDB.git
cd murexDB

# Type check all crates
cargo check --workspace

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Run Clippy lints
cargo clippy --workspace -- -D warnings
```

---

## 3. Pull Request Guidelines

1. Create a feature branch off `main`.
2. Keep commits atomic and use conventional commit messages (e.g. `feat:`, `fix:`, `docs:`, `rfcs:`).
3. Ensure CI passes on your PR.
