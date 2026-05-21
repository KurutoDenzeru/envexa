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

**Prerequisites:** Python 3.12+, [uv](https://docs.astral.sh/uv/)

```bash
# Install
uv sync

# Test a full scan (no MCP server needed)
uv run python -c "from src.scanner import run_scan, format_report; print(format_report(run_scan('all')))"
```

### Register with opencode

Add to `~/.config/opencode/opencode.json`:

```json
{
  "mcp": {
    "envexa": {
      "command": [
        "uv", "run", "--directory", "/absolute/path/to/envexa",
        "python", "-m", "src.server"
      ],
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
| `envexa_check_mismatches` | Version conflicts across projects |
| `envexa_find_unused` | Unused deps in an npm project |
| `envexa_get_report` | Latest cached report as markdown |
| `envexa_brew_status` | Homebrew only |
| `envexa_npm_status` | npm/Node only |
| `envexa_pip_status` | Python/pip only |
| `envexa_pip_upgrade` | Upgrade pip |
| `envexa_cmd` | Slash-command relay (see above) |

---

## MCP Resources

| URI | Type | What you get |
|-----|------|-------------|
| `envexa://report` | `text/markdown` | Formatted health report |
| `envexa://report/raw` | `application/json` | Raw JSON for scripting |

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
| Toolchain  | Status               | Version |
|------------|----------------------|---------|
| 🍺 Brew    | ⚠️ WARN (5)         | 5.1.12  |
|  npm     | ✅ PASS             | v24.15.0 |
|  pnpm    | ✅ PASS             | v11.1.2 |
|  Yarn    | ⏭️ SKIP             |         |
|  Bun     | ✅ PASS             | 1.3.14  |
|  Deno    | ✅ PASS             | 2.5.4   |
|  pip     | ✅ PASS             | Python 3.14.3 |
|  Gem     | ⚠️ WARN (100)       | ruby 3.2.2 |
| 🦀 Cargo   | ✅ PASS             | rustc 1.93.0 |
| 🐳 Docker  | ❌ FAIL             | daemon not running |
```

---

## Project Structure

```
envexa/
├── pyproject.toml           # Dependencies + metadata
├── src/
│   ├── server.py            # MCP server entry point (tools, prompts, resources)
│   ├── scanner.py           # Scan orchestration + report formatting
│   ├── mismatches.py        # Cross-project version conflict detection
│   ├── unused.py            # Unused dependency analysis (via depcheck)
│   └── toolchains/          # One scanner per toolchain
│       ├── brew.py / npm.py / pip.py / gem.py / cargo.py / docker.py
│       ├── pnpm.py / yarn.py / bun.py / deno.py
├── AGENTS.md                # Instructions for AI assistants
├── report.json              # Cached scan results (gitignored)
├── test_output.txt          # Verification test results (gitignored)
```

---

## Development

```bash
# Install
uv sync

# Run a scan locally
uv run python -c "from src.scanner import run_scan, format_report; print(format_report(run_scan('all')))"

# Start MCP server (stdio)
uv run python -m src.server
```

---

## License

MIT
