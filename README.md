# Envexa

**DevEnv Health Scanner** — interactive TUI that scans your dev environment and surfaces outdated packages. Just run `envexa`.

---

## Quick Start

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

## Usage

```
envexa            Launch interactive TUI
envexa scan       Full scan (CLI output, for scripting)
envexa update     Self-update to latest release
```

### TUI Keybindings

| Key | Action |
|-----|--------|
| `S` | Scan all toolchains |
| `O` | Show outdated packages |
| `/` | Enter search/filter mode |
| `←` `→` / `J` `L` | Switch between tabs |
| `↑` `↓` / `N` `P` | Navigate rows |
| `Q` | Quit (works even during search) |
| `Esc` / `H` | Back to Dashboard |

### TUI Search

Press `/` to enter search mode — the bottom bar becomes a search prompt. Type to filter the current view:

- **Dashboard** — matches toolchain names
- **Outdated** — matches package name, toolchain name, or source type (formula/cask/global/pkg)

Press `Esc` to clear filter & exit, `Enter` to keep the filter active.

### TUI Columns

**Dashboard:** Indicator (▸), Toolchain, Status, Version, Outdated (#), Issues  
**Outdated:** Indicator (▸), Toolchain, Source, Package, Current, Latest

---

## Cache

Scan results are cached to `~/.envexa/cache.json` (TTL: 7 days). The TUI loads cached data on launch — press `S` to refresh.

---

## Self-Update

```bash
envexa update
```

Downloads the latest prebuilt binary from GitHub Releases and atomically replaces the current binary. Works on macOS, Linux, and Windows.

---

## Performance

All 10 toolchains run concurrently via `tokio::join!`. Full scan completes in ~3-4 seconds. Release binary is 3.2MB — no Python, no Node, no runtime dependencies.

## Toolchains

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

---

## Project Structure

```
envexa/
├── Cargo.toml
├── src/
│   ├── main.rs        # Entry — no args = TUI, args = CLI
│   ├── app.rs         # App state, event loop, scan dispatch
│   ├── ui.rs          # ratatui render functions (Dashboard, Outdated, Scanning)
│   ├── cli.rs         # CLI subcommands (scan, update)
│   ├── config.rs      # File-backed cache (~/.envexa/cache.json)
│   ├── scanner.rs     # Scan orchestration + ASCII/table formatting
│   └── toolchains/    # One scanner per toolchain
│       ├── mod.rs
│       ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
│       ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
├── scripts/
│   └── install.sh
├── AGENTS.md
```

---

## Contributing

Contributions are always welcome. Check out [Contributing.md](Contributing) to get started.

## License

MIT — see the [LICENSE](LICENSE) file.
