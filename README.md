<p align="center">
  <img src="assets/bulldozer.png" alt="Envexa Logo" width="120" />
</p>

<h1 align="center">🚧 Envexa</h1>

<p align="center">
  <strong>Blazing-fast Rust TUI and CLI for monitoring local developer tooling health</strong>
</p>

<p align="center">
  <img src="assets/envexa-tui.png" alt="Envexa TUI Screenshot" width="800" />
</p>

---

A blazing-fast Rust TUI and scriptable CLI for monitoring local developer tooling health. Instantly track outdated packages and audit security risks across 14+ toolchains.

## 📚 Table of Contents

- [🚀 Quick Start](#-quick-start)
- [✨ Features](#-features)
- [🏗️ Architecture](#-architecture)
- [🧑‍💻 Development](#-development)
- [🤝 Contributing](#-contributing)
- [⚖️ License](#-license)

---

## 🚀 Quick Start

### Install
```bash
# One-line install
curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash

# Or build from source
git clone https://github.com/KurutoDenzeru/envexa.git && cd envexa && cargo install --path .
```

### Usage
```bash
envexa             # Launch the interactive TUI dashboard
envexa scan        # Generate a comprehensive markdown report
envexa update      # Update to the latest release
```

---

## ✨ Features

- **Concurrent Engine**: Scans 14+ toolchains (Homebrew, npm, Cargo, Docker, etc.) in parallel.
- **Interactive TUI**: Features custom pie charts, health gauges, and quick keyboard navigation.
- **Project Tooling Sector**: Deep-dives into local lockfiles, dependency drift, and security audits.
- **CLI Reports**: Generates production-ready Markdown reports instantly for CI/CD or PRs.
- **Smart Cache**: Zero-friction launches utilizing local JSON state (`~/.envexa/cache.json`).

---

## 🧑‍💻 Development

```bash
cargo run           # Launch interactive TUI in terminal
cargo run -- scan   # Run CLI report mode
cargo watch -x run  # Live reloading for TUI
```

Before submitting changes, ensure you run:
```bash
cargo clippy -- -D warnings && cargo fmt --check
```

---

## 🏗️ Architecture

```text
envexa/
├── Cargo.toml
├── src/
│   ├── main.rs             # Application entrypoint (TUI or CLI router)
│   ├── cli.rs              # CLI command parser and runner
│   ├── config.rs           # Persistent configurations and cached state
│   ├── scanner/
│   │   └── mod.rs          # Formatting utilities and diagnostic extraction
│   ├── tui/
│   │   ├── app.rs          # App state management, keyboard events, and scheduler
│   │   ├── mod.rs
│   │   └── ui.rs           # Ratatui rendering pipeline and interface structures
│   └── toolchains/
│       ├── mod.rs          # ScanResult schema, protocols, and multi-thread runners
│       ├── brew.rs
│       ├── npm.rs / pnpm.rs / yarn.rs / bun.rs / deno.rs
│       ├── pip.rs / gem.rs / cargo.rs / docker.rs
│       └── project.rs / security.rs / audit.rs / ci.rs
├── scripts/
│   ├── install.sh
│   └── build-and-upload.sh
└── .github/
    └── workflows/
```

Individual scanner modules are kept highly isolated. Each scanner implements a single `pub async fn scan() -> ScanResult` function, executes in parallel, and handles missing CLI tools gracefully to prevent crashes.

### System Overview

```mermaid
graph TB
    subgraph "Envexa Architecture"
        Main[main.rs<br/>Application Entry]
        
        subgraph "Core Modules"
            CLI[cli.rs<br/>CLI Parser]
            Config[config.rs<br/>Configuration]
            Scanner[scanner/mod.rs<br/>Formatting & Extraction]
        end
        
        subgraph "TUI Layer"
            App[app.rs<br/>State & Events]
            UI[ui.rs<br/>Ratatui Rendering]
        end
        
        subgraph "Toolchain Scanners"
            direction LR
            Brew[brew.rs]
            NPM[npm.rs]
            Pnpm[pnpm.rs]
            Yarn[yarn.rs]
            Bun[bun.rs]
            Deno[deno.rs]
            Pip[pip.rs]
            Gem[gem.rs]
            Cargo[cargo.rs]
            Docker[docker.rs]
            Project[project.rs]
            Security[security.rs]
            Audit[audit.rs]
            CI[ci.rs]
        end
    end
    
    Main --> CLI
    Main --> App
    CLI --> Scanner
    App --> UI
    App --> Scanner
    
    Scanner --> Brew
    Scanner --> NPM
    Scanner --> Pnpm
    Scanner --> Yarn
    Scanner --> Bun
    Scanner --> Deno
    Scanner --> Pip
    Scanner --> Gem
    Scanner --> Cargo
    Scanner --> Docker
    Scanner --> Project
    Scanner --> Security
    Scanner --> Audit
    Scanner --> CI
    
    style Main fill:#e1f5fe
    style CLI fill:#f3e5f5
    style Config fill:#f3e5f5
    style Scanner fill:#fff3e0
    style App fill:#e8f5e9
    style UI fill:#e8f5e9
    style Brew fill:#fce4ec
    style NPM fill:#fce4ec
    style Pnpm fill:#fce4ec
    style Yarn fill:#fce4ec
    style Bun fill:#fce4ec
    style Deno fill:#fce4ec
    style Pip fill:#fce4ec
    style Gem fill:#fce4ec
    style Cargo fill:#fce4ec
    style Docker fill:#fce4ec
    style Project fill:#e0f2f1
    style Security fill:#e0f2f1
    style Audit fill:#e0f2f1
    style CI fill:#e0f2f1
```

```mermaid
graph LR
    subgraph "Scan Pipeline"
        Input[User Trigger] --> Parallel[Parallel Scanner Engine<br/>tokio::join!]
        Parallel --> Results[ScanResult Aggregation]
        Results --> Cache[Cache Layer<br/>~/.envexa/cache.json]
        Cache --> Output{Output Mode}
        Output -->|TUI| Dashboard[Interactive Dashboard<br/>ratatui]
        Output -->|CLI| Report[Markdown Report]
    end
    
    subgraph "Data Flow"
        Results --> Outdated[Outdated Packages]
        Results --> Security[Security Advisories]
        Results --> Audit[Audit Findings]
    end
    
    style Input fill:#e1f5fe
    style Parallel fill:#fff3e0
    style Results fill:#e8f5e9
    style Cache fill:#f3e5f5
    style Dashboard fill:#e0f2f1
    style Report fill:#e0f2f1
    style Outdated fill:#fce4ec
    style Security fill:#fce4ec
    style Audit fill:#fce4ec
```

---

## 🤝 Contributing

Contributions are always welcome, whether you're fixing bugs, improving docs, or shipping new features that make the project better for everyone.

Check out [Contributing.md](Contributing) to learn how to get started and follow the recommended workflow.

---

## ⚖️ License

This project is released under the MIT License, giving you the freedom to use, modify, and distribute the code with minimal restrictions.

For the full legal text, see the [MIT](LICENSE) file.
