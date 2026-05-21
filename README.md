# 🔍 Envexa

**DevEnv Health Monitor** — an MCP server that snapshots your dev environment and surfaces outdated packages, version mismatches, unused deps, and runtime issues.

---

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
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
- Version conflicts for the same dependency across multiple projects
- Unused dependencies in your npm projects

All results are cached to `report.json` so you can read them offline or compare over time.

---

## Quick Start

**Prerequisites:** [Rust](https://rustup.rs/) toolchain

```bash
# Build
cargo build --release

# Test the MCP server (sends initialize + scan, prints report)
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"brew"}}}\n' | ./target/release/envexa
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

These are **Envexa-specific** commands — use the `envexa:` prefix to distinguish from other agents (codex, claude, gemini, opencode) that also expose `/status` and similar commands:

| Command | What it does |
|---------|--------------|
| `/envexa:scan` | Full health scan (default: all toolchains) |
| `/envexa:outdated` | Outdated packages only |
| `/envexa:status` | Quick dashboard summary (Envexa-specific) |

---

## MCP Tools

| Tool | Description |
|------|-------------|
| `envexa_scan` | Full health scan of all or one toolchain |
| `envexa_check_outdated` | Outdated packages only |
| `envexa_check_mismatches` | Version conflicts across projects (Python port pending) |
| `envexa_find_unused` | Unused deps in a project (Python port pending) |
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

These appear in opencode's `/` menu — the `envexa:` prefix ensures they don't conflict with other agents:

| Menu entry | What it does |
|------------|-------------|
| `/envexa:envexa_scan` | Full health report |
| `/envexa:envexa_status` | Quick dashboard summary |
| `/envexa:envexa_outdated` | Outdated packages across all toolchains |

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
| Toolchain | Status | Version |
|-----------|--------|---------|
| 🍺 Brew    | ⚠️ WARN (6)         | 5.1.12  |
|  npm     | ⚠️ WARN (3)         | 11.14.1 |
|  pnpm    | ✅ PASS             | v24.15.0 |
|  Yarn    | ⏭️ SKIP             |         |
|  Bun     | ✅ PASS             | 1.3.14  |
|  Deno    | ✅ PASS             | 2.5.4   |
|  pip     | ✅ PASS             | Python 3.14.3 |
|  Gem     | ⚠️ WARN (100)       | ruby 3.2.2 |
| 🦀 Cargo   | ✅ PASS             | rustc 1.93.0 |
| 🐳 Docker  | ✅ PASS             | 29.4.0  |
```

---

## Project Structure

```
envexa/
├── Cargo.toml               # Dependencies + metadata
├── src/
│   ├── main.rs              # Entry point + tokio stdin/stdout loop
│   ├── transport.rs         # JSON-RPC MCP protocol (hand-rolled, no SDK)
│   ├── server.rs            # Tool/prompt/resource registration + dispatch
│   ├── scanner.rs           # Scan orchestration + markdown formatting + cache
│   └── toolchains/          # One scanner per toolchain
│       ├── mod.rs           # ScanResult type + scan_all() concurrent dispatcher
│       ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
│       ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
├── AGENTS.md                # Instructions for AI assistants
├── report.json              # Cached scan results (gitignored)
```

---

## Development

```bash
# Build + run (debug)
cargo run

# Build optimized
cargo build --release

# Test a scan directly
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"brew"}}}\n' | cargo run

# Pipe output through Python to extract just the report text
printf '...' | cargo run | python3 -c "
import sys, json
for line in sys.stdin:
    data = json.loads(line)
    if 'result' in data and isinstance(data['result'], dict) and 'content' in data['result']:
        print(data['result']['content'][0]['text'])
"

---

## License

MIT
