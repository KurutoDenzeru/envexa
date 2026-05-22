# Envexa

**Scan your dev environment. Surface outdated packages, vulnerabilities, audit issues, and cleanup opportunities.** Envexa is a fast, interactive TUI that checks **14** toolchains at once and shows you exactly what needs attention.

---

## 🚀 Quick Start

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash
```

Installs to `~/.local/bin/envexa`.

### Build from source

```bash
cargo build --release
./target/release/envexa
```

---

## 🛠 Dev (hot-reload, like `npm run dev`)

```bash
# One-time setup
cargo install cargo-watch

# Run with auto-reload on save
cargo watch -x run
```

That's it. Save any `.rs` file and the TUI restarts instantly — no browser, no port, no localhost. Just your terminal.

For a faster feedback loop, use `cargo watch -x check` to type-check only:

```bash
cargo watch -x check
```

> **Tip:** `cargo watch` recompiles and relaunches on every file change under `src/`. The first rebuild after a save takes ~1–2s; subsequent incremental builds are <1s.

---

## 📖 Usage

```
envexa            Launch interactive TUI
envexa scan       Full scan (CLI output, for scripting)
envexa update     Self-update to latest release
```

### ⌨️ TUI Keybindings

| Key | Action |
|-----|--------|
| `S` | Scan all 14 toolchains |
| `O` | Show outdated packages |
| `Enter` | Open detail view for selected toolchain row |
| `/` | Enter search/filter mode |
| `←` `→` / `J` `L` | Switch between tabs |
| `↑` `↓` / `N` `P` | Navigate rows |
| `Space` | Toggle checkbox selection |
| `Y` | Update selected packages (in PackageDetail view) |
| `U` | Update all checked packages (in Outdated view) |
| `Ctrl+C` / `Q` | Quit |
| `Esc` / `H` | Back to Dashboard |

### 🔍 TUI Search

Press `/` to enter search mode — the bottom bar becomes a search prompt. Type to filter the current view:

- **Dashboard** — matches toolchain names
- **Outdated** — matches package name, toolchain name, or source type (formula/cask/global/pkg)

Press `Esc` to clear filter & exit, `Enter` to keep the filter active.

### 📊 TUI Columns

**Dashboard:** ▸ Checkbox, Toolchain, Status, Version, Outdated (#), Issues  
**Outdated:** ▸ Checkbox, Toolchain, Source, Package, Current, Latest

---

## 💾 Cache

Scan results are cached to `~/.envexa/cache.json` (TTL: 7 days). The TUI loads cached data on launch — press `S` to refresh.

---

## 🔄 Self-Update

```bash
envexa update
```

Downloads the latest prebuilt binary from GitHub Releases and atomically replaces the current binary. Works on macOS, Linux, and Windows.

> **Development builds** skip the release check — run `cargo build --release` first.

---

## ⚡ Performance

All 14 toolchains run concurrently via `tokio::join!`. Full scan completes in ~3-4 seconds. Release binary is 3.8MB — no Python, no Node, no runtime dependencies.

## 🔧 Toolchains

### System & Language Runtimes

| Toolchain | What's checked |
|-----------|----------------|
| **Homebrew** | Outdated formulae + casks, install count |
| **npm** | Runtime version, outdated global packages |
| **pnpm** | Runtime version, outdated global packages |
| **Yarn** | Availability check (if installed) |
| **Bun** | Runtime version, outdated global packages |
| **Deno** | Runtime version, outdated global packages |
| **pip** | Runtime version, outdated packages |
| **Gem** | Runtime version, outdated gem list |
| **Cargo** | Runtime version, cargo-outdated tool check |
| **Docker** | Daemon connectivity, disk usage, dangling images |

### Project, Security, Audit & Cleanup

| Toolchain | What's checked |
|-----------|----------------|
| **Project** | Detects CWD lockfile → runs outdated for detected package manager |
| **Security** | Runs `npm audit`, `pnpm audit`, `bun audit`, `cargo-audit`, `pip-audit` |
| **Audit** | System tool version sanity checks (node↔npm, python↔pip, brew age, rustc↔cargo) |
| **Cleanup** | Reclaimable disk: brew cache, npm cache, Cargo registry, bun cache, pip cache, Docker |

> **Note:** Project scanning uses the directory where `envexa` is launched. To scan a different project, set `project_path` in `~/.envexa/config.json`.

---

## 📁 Project Structure

```
envexa/
├── Cargo.toml            # Dependencies & metadata
├── src/
│   ├── main.rs           # Entry point — no args = TUI, args = CLI
│   ├── cli.rs            # CLI subcommands (scan, update)
│   ├── config.rs         # File-backed cache (~/.envexa/cache.json)
│   │
│   ├── scanner/          # Scan orchestration & report types
│   │   └── mod.rs        # Report, OutdatedItem, extract, format helpers
│   │
│   ├── tui/              # Terminal UI (ratatui)
│   │   ├── mod.rs        # Module declarations
│   │   ├── app.rs        # App state, event loop, scan dispatch
│   │   └── ui.rs         # Render functions (Dashboard, Outdated, PackageDetail)
│   │
│   └── toolchains/       # One scanner per toolchain (14 total)
│       ├── mod.rs        # ScanResult, types, scan_all()
│       ├── brew.rs / npm.rs / pip.rs / gem.rs
│       ├── cargo.rs / docker.rs
│       ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
│       └── project.rs / security.rs / audit.rs / cleanup.rs
│
├── scripts/
│   └── install.sh        # One-line installer
├── AGENTS.md             # Agent coding conventions
└── README.md
```

### Module dependencies

```
main.rs
 ├── cli.rs  ──→ config, scanner, toolchains
 ├── config.rs  ──→ scanner
 ├── scanner/  ──→ toolchains
 ├── tui/
 │   ├── app.rs  ──→ config, scanner, toolchains
 │   └── ui.rs  ──→ scanner
 └── toolchains/  (independent)
```

---

## 🤝🏻 Contributing

Contributions are always welcome, whether you're fixing bugs, improving docs, or shipping new features that make the project better for everyone.

Check out [Contributing.md](Contributing) to learn how to get started and follow the recommended workflow.

---

## ⚖️ License

This project is released under the MIT License, giving you the freedom to use, modify, and distribute the code with minimal restrictions.

For the full legal text, see the [MIT](LICENSE) file.
