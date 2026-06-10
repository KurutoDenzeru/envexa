<h1 align="center">🧪 Envexa Test Suite</h1>

<p align="center">
  <strong>Comprehensive testing guidelines and examples for the Envexa developer tooling health monitor.</strong>
</p>

---

Welcome to the Envexa testing directory! This folder houses all our integration tests, parser validations, and mock fixtures to ensure the CLI and TUI are robust and reliable.

## 📚 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [✨ Structure](#-features)
- [🛠️ Writing Tests](#-writing-tests)
- [🧑‍💻 Running Tests](#-development)

---

## 🚀 Quick Start

To run the entire Envexa test suite locally:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_npm_parse_outdated
```

---

## ✨ Structure

- **`parser_tests.rs`**: Unit tests validating that JSON/text outputs from various package managers (npm, cargo, etc.) are parsed correctly into internal structures.
- **`integration_tests.rs`**: An example setup for higher-level application flows and command invocations.
- **`fixtures/`**: A folder containing raw JSON/text outputs from commands for mock testing.

---

## 🛠️ Writing Tests

We encourage adding tests for any new parser, scanner, or TUI logic.

When writing tests:
1. **Mock External Inputs**: Do not rely on active system environments for unit tests. Use raw string literals or file fixtures for output parsing tests.
2. **Use Assertions**: Liberally use `assert_eq!` and `assert!` to strictly validate data shapes.
3. **Follow the Format**: Keep test names descriptive, e.g., `test_<toolchain>_<action>`.

Check out `integration_tests.rs` for a starter template on writing your own test!

---

## 🧑‍💻 Running Tests

Continuous Integration automatically runs `cargo test` on all pull requests. Ensure your local tests pass before submitting changes:

```bash
cargo check && cargo test
```
