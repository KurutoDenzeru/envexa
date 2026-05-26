# 🚧 Envexa

**Envexa is a Rust TUI for checking local developer tooling health.** It scans system runtimes, package managers, project dependencies, security advisories, version alignment, and cleanup opportunities from one terminal dashboard.

Envexa is built for fast local checks before work, before commits, or when a machine starts to drift. Run it with no args for the interactive TUI, or use `envexa scan` for scriptable Markdown output.

## 📚 Table of Contents

- [Highlights](#-highlights)
- [Install](#-install)
- [Usage](#-usage)
- [TUI](#-tui)
- [Project Tooling Sector](#-project-tooling-sector)
- [Toolchains](#-toolchains)
- [CLI Report](#-cli-report)
- [Cache](#-cache)
- [Development](#-development)
- [Architecture](#-architecture)
- [Design Notes](#-design-notes)
- [Release](#-release)
- [Contributing](#-contributing)
- [License](#-license)

---

## ✨ Highlights

- Concurrent scanner for 14 toolchains using `tokio::join!`.
- Interactive `ratatui` dashboard with status pie chart, readiness gauges, barcharts, tables, tabs, and animated scan/update states.
- Dedicated Project Tooling sector for Project, Security, Audit, and Cleanup signals.
- CLI report mode for automation, logs, or issue comments.
- Self-update command for macOS release binaries.
- File-backed cache at `~/.envexa/cache.json` so the TUI can open with recent data instantly.

---

## 📦 Install

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash
```

The installer places `envexa` in `~/.local/bin`.

### Build from source

```bash
git clone https://github.com/KurutoDenzeru/envexa.git
cd envexa
cargo build --release
./target/release/envexa
```

---

## 🚀 Usage

```bash
envexa             # launch interactive TUI
envexa scan        # print full health report to stdout
envexa update      # self-update from latest GitHub release
envexa --help      # print CLI help
```

Run Envexa from the project directory you want to inspect. To scan another path, set `project_path` in `~/.envexa/config.json`.

---

## 🖥️ TUI

The dashboard follows a dev-monitoring layout: summary visuals on the left, actionable tables on the right, and keyboard shortcuts in the footer.

| Area | Purpose |
|------|---------|
| Overview | Pass/warn/fail/skip distribution pie chart |
| Project Tooling | Project readiness gauge, risk score, vulnerability/audit/package barchart |
| Health line | Overall health ratio, outdated count, cache age |
| Category tables | System, Web Development, and Project Tooling scanner rows |
| Outdated tab | Package update queue with checkbox selection |
| Detail views | Per-toolchain outdated, vulnerability, audit, and cleanup records |

### Keybindings

| Key | Action |
|-----|--------|
| `S` | Scan all toolchains |
| `O` | Open Outdated tab |
| `Enter` | Open selected dashboard row detail |
| `/` | Search/filter current view |
| `Left` / `Right` | Switch tabs |
| `Up` / `Down` | Navigate rows |
| `Space` | Toggle package selection where updates are supported |
| `Y` | Update selected packages in a package detail view |
| `U` | Update checked packages from Outdated tab |
| `Esc` / `H` | Return to dashboard |
| `Q` / `Ctrl+C` | Quit |

### Search

Press `/` to filter:

- Dashboard: toolchain names.
- Outdated: toolchain, package name, or source type.

Press `Esc` to clear the filter. Press `Enter` to keep the filter active.

---

## 🧰 Project Tooling Sector

Project Tooling is the local project lens inside Envexa.

| Scanner | What it checks |
|---------|----------------|
| ✨ Project | Detects lockfiles and runs the matching package manager outdated check |
| 🔐 Security | Runs available security audit tools across JavaScript, Rust, and Python ecosystems |
| 🧪 Audit | Checks runtime/tool version alignment such as Node/npm, Python/pip, and rustc/Cargo |
| 🧹 Cleanup | Finds reclaimable package-manager and Docker cache space |

The TUI summarizes this sector with:

- Readiness gauge based on dependency drift, vulnerability severity, and audit findings.
- Signal barchart for outdated project packages, critical/high/moderate/other vulnerabilities, and audit items.
- Focus labels that show the strongest next action instead of generic issue text.

---

## 🛠️ Toolchains

### 🖥️ System & Runtime

| Toolchain | Checks |
|-----------|--------|
| Homebrew | Version, installed formula count, outdated formulae/casks |
| pip | Python/pip version and outdated packages |
| Gem | Ruby version and outdated gems |
| Cargo | rustc/Cargo versions and optional `cargo-outdated` results |
| Docker | CLI/daemon availability, disk usage, image/container cleanup signals |

### 🌐 Web Development

| Toolchain | Checks |
|-----------|--------|
| npm | Node/npm version and global package drift |
| pnpm | Node/pnpm version and global package drift |
| Yarn | Availability and version signal |
| Bun | Bun version and global package drift |
| Deno | Deno version and outdated package signal |

---

## 📄 CLI Report

```bash
envexa scan
```

`scan` prints a Markdown report with:

- Dashboard status table.
- Outdated package table.
- Vulnerability table.
- Audit findings.
- Cleanup recommendations.
- Per-toolchain details.

Use it for logs, CI notes, or PR comments when you need a plain text artifact.

---

## 🗄️ Cache

Envexa caches scan data at:

```text
~/.envexa/cache.json
```

Default cache TTL is 7 days. The TUI reads cached results on launch and refreshes when you press `S`.

---

## 🧑‍💻 Development

```bash
cargo build
cargo run
cargo run -- scan
cargo run -- --help
```

Optional fast loop:

```bash
cargo install cargo-watch
cargo watch -x check
```

Before submitting changes:

```bash
cargo build
cargo clippy -- -D warnings
cargo fmt --check
```

For TUI work, also launch `cargo run` in a real terminal and verify scan, navigation, search, detail views, and quit behavior.

---

## 🏗️ Architecture

```text
envexa/
├── Cargo.toml
├── src/
│   ├── main.rs             # no args = TUI, args = CLI
│   ├── cli.rs              # scan/update/help dispatch
│   ├── config.rs           # cache/config persistence
│   ├── scanner/
│   │   └── mod.rs          # report formatting and extraction helpers
│   ├── tui/
│   │   ├── app.rs          # app state, events, scan/update actions
│   │   ├── mod.rs
│   │   └── ui.rs           # ratatui render functions and widgets
│   └── toolchains/
│       ├── mod.rs          # ScanResult, protocol types, scan_all()
│       ├── brew.rs
│       ├── npm.rs / pnpm.rs / yarn.rs / bun.rs / deno.rs
│       ├── pip.rs / gem.rs / cargo.rs / docker.rs
│       └── project.rs / security.rs / audit.rs / cleanup.rs
├── scripts/
│   ├── install.sh
│   └── build-and-upload.sh
└── .github/
    └── workflows/
```

Scanner modules are intentionally small: one toolchain, one `pub async fn scan() -> ScanResult`, graceful skip on missing tools, and structured output for CLI/TUI reuse.

---

## 🎛️ Design Notes

Envexa uses built-in `ratatui` widgets where they fit cleanly: `Table`, `Tabs`, `Gauge`, `LineGauge`, and `BarChart`. It also uses focused third-party widgets where they add clear value: `tui-piechart` for the overview chart and `throbber-widgets-tui` for scan/update activity.

New TUI widgets should improve scan readability or action priority. Avoid decorative widgets that do not help users decide what to update, audit, or clean.

---

## 🚢 Release

Maintainers publish macOS binaries through GitHub Releases. For details on release notes formatting, templates, and logs, see [.github/ISSUE_TEMPLATE/RELEASES.md](.github/ISSUE_TEMPLATE/RELEASES.md).

```bash
cargo clean
# bump Cargo.toml version, commit
git tag vX.Y.Z && git push origin vX.Y.Z
gh release create vX.Y.Z --title "vX.Y.Z" --notes "..."
scripts/build-and-upload.sh vX.Y.Z
gh release view vX.Y.Z --json assets --jq '.assets[].name'
cargo clean
```

---

## 🤝 Contributing

Contributions are always welcome, whether you're fixing bugs, improving docs, or shipping new features that make the project better for everyone.

Check out [Contributing.md](Contributing) to learn how to get started and follow the recommended workflow.

---

## ⚖️ License

This project is released under the MIT License, giving you the freedom to use, modify, and distribute the code with minimal restrictions.

For the full legal text, see the [MIT](LICENSE) file.
