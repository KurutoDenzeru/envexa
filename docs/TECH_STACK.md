# 🏗️ Envexa Technical Stack

This document details the ground-up technical stack and architectural decisions that power Envexa, ensuring it remains a blazing-fast, responsive, and robust monitoring tool.

## 📚 Table of Contents

- [Core Language](#core-language)
- [Terminal User Interface (TUI)](#terminal-user-interface-tui)
- [Async Runtime & Concurrency](#async-runtime--concurrency)
- [Serialization & Data](#serialization--data)
- [Architecture & State Management](#architecture--state-management)

---

## Core Language

**[Rust (Edition 2021)](https://www.rust-lang.org/)**
Envexa is built entirely in Rust. Rust was chosen for its strict memory safety guarantees, fearless concurrency, and bare-metal performance. By compiling down to a single, portable binary, Envexa avoids runtime dependencies, ensuring it can run reliably across different CI/CD environments and developer machines.

---

## Terminal User Interface (TUI)

**[Ratatui](https://ratatui.rs/)**
Ratatui is the premier Rust library for building terminal user interfaces. It provides immediate-mode rendering for high-performance terminal updates.

We utilize several core Ratatui widgets to build the dashboard:
- `Table` and `List` for the dashboard rows and package queues.
- `Tabs` for top-level navigation.
- `Paragraph` and `Block` for the detailed log panes and layout borders.
- `Gauge` and `LineGauge` for visual readiness scores.

### 🎨 3rd-Party UI Widgets
To enhance the visual fidelity of the TUI beyond standard text grids, we integrate specialized third-party widgets:
- **`tui-piechart`**: Used heavily on the Dashboard view to render a graphical pie chart of the toolchain statuses (Pass/Warning/Error/Skipped).
- **`throbber-widgets-tui`**: Used to render animated braille spinners (e.g., `Checking for updates...` or `Scanning...`). This gives the user immediate visual feedback during blocking IO operations or background tasks.

### ⌨️ Terminal Backend
**[Crossterm](https://github.com/crossterm-rs/crossterm)**
Crossterm acts as the terminal backend for Ratatui. It handles cross-platform terminal manipulation, raw mode execution, and keyboard/mouse event listening.

---

## Async Runtime & Concurrency

**[Tokio](https://tokio.rs/)**
Tokio is the asynchronous runtime backing Envexa.
- The core scanner engine relies on `tokio::join!` to spawn and resolve 14+ independent toolchain checks concurrently.
- `tokio::process::Command` is used exclusively over `std::process::Command` to prevent the UI thread from blocking while waiting for external CLI binaries (like `npm` or `cargo`) to yield output.
- `tokio::time::timeout` ensures that unresponsive or hanging commands do not stall the entire application.

---

## Serialization & Data

**[Serde](https://serde.rs/) & [Serde JSON](https://docs.rs/serde_json/)**
Serde provides a powerful serialization/deserialization framework.
- It translates raw JSON output from toolchains (like `cargo outdated --format json` or `npm outdated --json`) directly into strongly-typed Rust structs.
- It also manages reading/writing the `~/.envexa/cache.json` state and the application settings `~/.envexa/config.json`.

---

## Architecture & State Management

The application is strictly separated into core logic and rendering pipelines:

1. **`app.rs`**: The brain of the TUI. It holds the mutable `App` struct, the application `View` state, navigation indexes, and handles all incoming keyboard events from Crossterm.
2. **`ui.rs`**: The immediate-mode renderer. It takes an immutable reference to `App` and pure layout structs (`Rect`) to draw the frames. No state modification occurs here.
3. **`toolchains/`**: Highly isolated concurrent workers. Each module (e.g., `npm.rs`, `docker.rs`) exposes a single `pub async fn scan() -> ScanResult` interface, meaning new scanners can be plugged into the engine with zero friction.
