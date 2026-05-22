# Envexa

**DevEnv Health Scanner** — snapshots your dev environment and surfaces outdated packages, version mismatches, and runtime issues.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Usage](#usage)
- [Commands](#commands)
- [Cache](#cache)
- [Self-Update](#self-update)
- [Uninstall](#uninstall)
- [Performance](#performance)
- [Toolchains](#toolchains)
- [Sample Output](#sample-output)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Quick Start

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash
```

Detects OS/arch, downloads the latest release binary, installs to `~/.local/bin/envexa`.

### Build from source

```bash
cargo build --release
./target/release/envexa --help
```

---

## Usage

```bash
envexa scan brew          Scan Homebrew only
envexa scan               Full scan of all toolchains
envexa status             Dashboard summary
envexa outdated           All outdated packages
envexa report             Show cached report
envexa upgrade pip        Upgrade pip
envexa update             Self-update
envexa info               Version/system info
envexa uninstall          Remove cache + binary
```

Results are cached to `~/.envexa/cache.json` (TTL: 7 days by default). Use `envexa report` to re-read the last scan without re-scanning.

---

## Commands

| Command | Description |
|---------|-------------|
| `scan [chain]` | Full health scan (chain: all, brew, npm, pnpm, yarn, bun, deno, pip, gem, cargo, docker) |
| `status` | Quick dashboard — one row per toolchain |
| `outdated [chain]` | Table of outdated packages only |
| `report` | Latest cached report |
| `upgrade <tool>` | Upgrade a toolchain (pip currently supported) |
| `update` | Self-update to latest GitHub release |
| `info` | Binary path, cache status, version |
| `uninstall` | Remove cache + config + binary |

---

## Cache

Stored at `~/.envexa/cache.json`. Use `--ttl` to override:

```bash
envexa scan --ttl 1     # expire after 1 day
envexa scan --ttl 30    # 30-day cache
```

Cache is auto-checked by `status` and `report` commands — expired caches trigger a fresh scan automatically.

---

## Self-Update

```bash
envexa update
```

Downloads the latest prebuilt binary from GitHub Releases and atomically replaces the current binary. Works on macOS, Linux, and Windows.

---

## Uninstall

```bash
envexa uninstall
```

Removes `~/.envexa/cache.json`, deletes the binary, and cleans up. Prompts for confirmation first.

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

## Sample Output

```
$ envexa status

# Envexa Status
**Generated:** 2026-05-22T08:57:36

+-----------+--------+-------+
| Toolchain | Status | Count |
+-----------+--------+-------+
| Brew      | WARN   | 6     |
| npm       | WARN   | 3     |
| pnpm      | PASS   | -     |
| Yarn      | SKIP   | -     |
| Bun       | PASS   | -     |
| Deno      | PASS   | -     |
| pip       | PASS   | -     |
| Gem       | WARN   | 100   |
| Cargo     | PASS   | -     |
| Docker    | PASS   | -     |
+-----------+--------+-------+
```

---

## Project Structure

```
envexa/
├── Cargo.toml               # Dependencies + metadata
├── src/
│   ├── main.rs              # Entry point — launches CLI
│   ├── cli.rs               # Clap CLI parser + command implementations
│   ├── config.rs            # File-backed cache (~/.envexa/cache.json) + TTL
│   ├── scanner.rs           # Scan orchestration + ASCII table formatting
│   └── toolchains/          # One scanner per toolchain
│       ├── mod.rs           # ScanResult type + scan_all() concurrent dispatcher
│       ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
│       ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
├── scripts/
│   └── install.sh           # Curl-based install script
├── AGENTS.md                # Instructions for AI assistants
```

---

## Contributing

Contributions are always welcome, whether you're fixing bugs, improving docs, or shipping new features that make the project better for everyone.

Check out [Contributing.md](Contributing) to learn how to get started and follow the recommended workflow.

## License

This project is released under the MIT License, giving you the freedom to use, modify, and distribute the code with minimal restrictions.

For the full legal text, see the [MIT](LICENSE) file.
