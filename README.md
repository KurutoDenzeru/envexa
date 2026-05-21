# Envexa

**DevEnv Health Monitor** — an MCP server that snapshots your entire development environment across six toolchains and surfaces outdated packages, version mismatches, unused dependencies, and runtime issues.

---

## Features

- **Six toolchains** — Homebrew, Node/npm, Python/pip, Ruby/gem, Rust/cargo, Docker
- **Outdated detection** — per-toolchain lists with current vs. latest versions
- **Cross-project mismatch scanning** — finds version conflicts across `package.json`, `Cargo.toml`, and `pyproject.toml`
- **Unused dependency analysis** — powered by `depcheck` for npm projects
- **Dashboard + tree report** — compact summary table followed by a hierarchical outdated tree and per-toolchain detail tables
- **Slash commands** — preset quick commands via the `cmd` tool (`/scan`, `/outdated`, `/status`, `/upgrade pip`, `/help`)
- **Persistent state** — latest scan is cached to `report.json` for offline reads and resource access

---

## Quick start

### Prerequisites

- Python 3.12+
- [uv](https://docs.astral.sh/uv/)

### Install

```bash
uv sync
```

### Register with opencode

Add to your `~/.config/opencode/opencode.json`:

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

Then restart opencode or reload its MCP servers.

---

## MCP tools

| Tool | Description |
|---|---|
| `scan` | Full health scan of all (or a single) toolchain |
| `check_outdated` | Outdated packages only, by toolchain |
| `check_mismatches` | Version conflicts across project directories |
| `find_unused` | Unused deps in a project (npm via depcheck) |
| `get_report` | Latest cached report as markdown |
| `brew_status` | Quick Homebrew scan |
| `npm_status` | Quick npm/Node scan |
| `pip_status` | Quick Python/pip scan |
| `pip_upgrade` | Upgrade pip to latest |
| `cmd` | Slash-command relay (`/scan`, `/outdated brew`, `/status`, `/upgrade pip`, `/help`) |

## MCP resources

| URI | MIME | Description |
|---|---|---|
| `envexa://report` | `text/markdown` | Latest full report |
| `envexa://report/raw` | `application/json` | Latest report as raw JSON |

---

## Toolchains

| Toolchain | What's checked |
|---|---|
| **Homebrew** | Outdated formulae and casks (with versions), installed formula count |
| **Node/npm** | Runtime version, global package outdated list |
| **Python/pip** | Runtime version, global pip outdated list |
| **Ruby/gem** | Runtime version, outdated gem list |
| **Rust/cargo** | Runtime version, `cargo-outdated` tool availability |
| **Docker** | Daemon connectivity, disk usage by type, dangling image count |

---

## Sample output

```
| Toolchain | Status       | Version                        |
|-----------|-------------|--------------------------------|
| Brew      | WARN (5)    | 5.1.12                        |
| Npm       | PASS        | v24.15.0                      |
| Pip       | PASS        | Python 3.14.3                 |
| Gem       | WARN (100)  | ruby 3.2.2                    |
| Cargo     | PASS        | rustc 1.93.0                  |
| Docker    | FAIL        | daemon not running            |
```

---

## Project structure

```
envexa/
├── pyproject.toml        # Python project config
├── src/
│   ├── server.py         # MCP server entry (tools + resources)
│   ├── scanner.py        # Scan orchestration + report formatting
│   ├── mismatches.py     # Cross-project version conflict detection
│   ├── unused.py         # Unused dependency finder
│   └── toolchains/       # One module per toolchain
│       ├── brew.py
│       ├── npm.py
│       ├── pip.py
│       ├── gem.py
│       ├── cargo.py
│       └── docker.py
└── report.json           # Cached scan results
```

---

## Development

```bash
# Install dependencies
uv sync

# Test a full scan locally
uv run python -c "from src.scanner import run_scan, format_report; print(format_report(run_scan('all')))"

# Start the MCP server (stdio)
uv run python -m src.server
```

---

## License

MIT — see [LICENSE](LICENSE).
