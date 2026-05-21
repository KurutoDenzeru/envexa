# Envexa

DevEnv Health Monitor — snapshots your entire dev environment (brew, node, python, ruby, cargo, docker) and alerts on outdated tools, version mismatches, and unused packages.

## MCP Tools

| Tool | Description |
|---|---|
| `scan` | Full health snapshot across all or a single toolchain |
| `check_outdated` | List outdated packages across toolchains |
| `check_mismatches` | Detect version conflicts across projects |
| `find_unused` | Find unused dependencies in a project |
| `get_report` | Retrieve the latest scan report |

## MCP Resources

| URI | Description |
|---|---|
| `envexa://report` | Latest report as markdown |
| `envexa://report/raw` | Latest report as JSON |

## Quick Start

```json
// ~/.config/opencode/opencode.json
{
  "mcp": {
    "envexa": {
      "command": [
        "uv", "run", "--directory", "/path/to/envexa",
        "python", "-m", "src.server"
      ],
      "type": "local"
    }
  }
}
```

Requires Python 3.12+ and [uv](https://docs.astral.sh/uv/).

## Toolchains

- **Homebrew** — outdated formulae/casks, installed count
- **Node/npm** — version, global outdated packages
- **Python/pip** — version, outdated packages
- **Ruby/gem** — version, outdated gems
- **Rust/cargo** — version, cargo-outdated availability
- **Docker** — daemon status, disk usage, dangling images
