# 🚧 Envexa

**DevEnv Health Monitor** — snapshots your dev environment, surfaces outdated packages, version mismatches, unused deps, and runtime issues. Runs as either an **MCP server** (for AI agents) or a **CLI tool** (direct terminal use).

---

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [CLI Usage](#cli-usage)
- [Slash Commands](#slash-commands)
- [MCP Tools](#mcp-tools)
- [MCP Resources](#mcp-resources)
- [MCP Prompts](#mcp-prompts)
- [Performance](#performance)
- [Toolchains](#toolchains)
- [Sample Output](#sample-output)
- [Project Structure](#project-structure)
- [Development](#development)

---

## Overview

Ever wonder what's rotting in your dev environment? Run one scan and Envexa tells you:

- Which Homebrew formulae and casks are outdated
- Which npm/pip/gem packages need updating
- If your Docker daemon is running and how much disk it's using

Results are cached to `~/.envexa/cache.json` so you can re-read them across terminal sessions without re-scanning.

Envexa auto-detects its mode:
- **Args present** or **stdin is a terminal** → CLI mode
- **stdin is piped** → MCP server mode (for AI agents)

---

## Quick Start

### Build from source

```bash
cargo build --release
```

### One-line install (curl)

```bash
curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash
```

Detects OS/arch, downloads the latest release binary from GitHub, installs to `~/.local/bin/envexa`.

### Self-update

```bash
envexa update
```

Downloads the latest release from GitHub and atomically swaps the binary. Works on macOS, Linux, and Windows.

---

## CLI Usage

```bash
envexa scan [chain]       Full health scan (chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker)
envexa status             Quick dashboard summary
envexa outdated [chain]   Outdated packages only
envexa report             Show the latest cached report
envexa upgrade pip        Upgrade pip to latest
envexa update             Self-update to latest release
envexa info               Show version and system info
envexa uninstall          Remove cache and config
envexa --help             Show help
envexa --version          Show version
```

### Register with opencode

Add to `~/.config/opencode/opencode.json`:

```json
{
  "mcp": {
    "envexa": {
      "command": ["/absolute/path/to/envexa/target/release/envexa"],
      "type": "local"
    }
  }
}
```

Then restart opencode or reload MCP servers.

---

## Slash Commands

These are **Envexa-specific** commands — use inside an MCP host that supports slash commands, or pass via `envexa_cmd`:

| Command | What it does |
|---------|--------------|
| `/scan` | Full health scan (default: all toolchains) |
| `/outdated` | Outdated packages only |
| `/status` | Quick dashboard summary |
| `/report` | Latest cached report |
| `/upgrade` | Upgrade a toolchain (pip supported) |
| `/help` | Show available commands |

---

## MCP Tools

| Tool | Description |
|------|-------------|
| `envexa_scan` | Full health scan of all or one toolchain |
| `envexa_check_outdated` | Outdated packages only |
| `envexa_check_mismatches` | Version conflicts across projects (not yet implemented) |
| `envexa_find_unused` | Unused deps in a project (not yet implemented) |
| `envexa_get_report` | Latest cached report as markdown |
| `envexa_brew_status` | Homebrew only |
| `envexa_npm_status` | npm/Node only |
| `envexa_pip_status` | Python/pip only |
| `envexa_pip_upgrade` | Upgrade pip |
| `envexa_cmd` | Slash-command relay (see above) |

---

## Performance

All 10 toolchains run concurrently via `tokio::join!`. Full scan completes in ~3-4 seconds wall-clock vs ~6-7 seconds sequential (Python version). The release binary is 3.2MB — no Python, no uv, no virtualenv needed.

## MCP Resources

| URI | Type | What you get |
|-----|------|-------------|
| `envexa://report` | `text/markdown` | Formatted health report (from cache) |
| `envexa://report/raw` | `application/json` | Raw JSON for scripting (from cache) |

---

## MCP Prompts

| Menu entry | What it does |
|------------|-------------|
| `envexa_scan` | Full health report |
| `envexa_status` | Quick dashboard summary |
| `envexa_outdated` | Outdated packages across all toolchains |

---

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
| Toolchain | Status         | Version     |
|-----------|----------------|-------------|
| Brew      | WARN (6)       | 5.1.12      |
| npm       | WARN (3)       | 11.14.1     |
| pnpm      | PASS           | v24.15.0    |
| Yarn      | SKIP           |             |
| Bun       | PASS           | 1.3.14      |
| Deno      | PASS           | 2.5.4       |
| pip       | PASS           | Python 3.14.3 |
| Gem       | WARN (100)     | ruby 3.2.2  |
| Cargo     | PASS           | rustc 1.93.0 |
| Docker    | PASS           | 29.4.0      |
```

---

## Project Structure

```
envexa/
├── Cargo.toml               # Dependencies + metadata
├── src/
│   ├── main.rs              # Entry point — auto-detects CLI vs MCP mode
│   ├── cli.rs               # Clap CLI parser + slash-command executor
│   ├── config.rs            # File-backed cache (~/.envexa/cache.json) + TTL
│   ├── transport.rs         # JSON-RPC MCP protocol (hand-rolled, no SDK)
│   ├── server.rs            # Tool/prompt/resource registration + dispatch
│   ├── scanner.rs           # Scan orchestration + ASCII table formatting + cache
│   └── toolchains/          # One scanner per toolchain
│       ├── mod.rs           # ScanResult type + scan_all() concurrent dispatcher
│       ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
│       ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
├── scripts/
│   └── install.sh           # Curl-based install script (detects OS/arch)
├── .github/
│   └── workflows/           # CI + release build matrix
├── AGENTS.md                # Instructions for AI assistants
```

---

## Contributing

Contributions are always welcome, whether you're fixing bugs, improving docs, or shipping new features that make the project better for everyone.

Check out [Contributing.md](Contributing) to learn how to get started and follow the recommended workflow.

<!-- Please adhere to this project's Code of Conduct. -->

## License

This project is released under the MIT License, giving you the freedom to use, modify, and distribute the code with minimal restrictions.

For the full legal text, see the [MIT](LICENSE) file.
