import json
import subprocess
import shutil
from pathlib import Path

from mcp.server.fastmcp import FastMCP

from . import scanner, mismatches, unused

mcp = FastMCP("Envexa")

REPORT_FILE = Path(__file__).resolve().parent.parent / "report.json"


# ── Core tools ──────────────────────────────────────────────────────────

@mcp.tool(
    description="Envexa — scan dev environment toolchains (brew, node, python, ruby, cargo, docker). chain: all|brew|npm|pip|gem|cargo|docker"
)
def scan(chain: str = "all") -> str:
    report = scanner.run_scan(chain)
    if "error" in report:
        return report["error"]
    return scanner.format_report(report)


@mcp.tool(
    description="Envexa — check for outdated packages. chain: all|brew|npm|pip|gem|cargo|docker"
)
def check_outdated(chain: str = "all") -> str:
    report = scanner.run_scan(chain)
    results = report["results"]

    lines = ["# Outdated Packages\n"]
    has_anything = False

    for tool, res in results.items():
        items = []
        for key in ("outdated_formulae", "outdated_casks", "outdated_global", "outdated"):
            if key in res and res[key]:
                items.extend(res[key])

        if items:
            has_anything = True
            lines.append(f"## {tool.title()} ({len(items)} outdated)\n")
            for item in items:
                lines.append(
                    f"- **{item['name']}**: {item.get('current', '?')} -> {item.get('latest', '?')}"
                )
            lines.append("")

    if not has_anything:
        return "All packages are up to date!"

    return "\n".join(lines)


@mcp.tool(
    description="Envexa — detect version mismatches of the same package across different projects"
)
def check_mismatches(projects: list[str] = None) -> str:
    result = mismatches.scan_projects(projects)
    if not result:
        return "No version mismatches found across projects."
    lines = ["# Version Mismatches\n"]
    for pkg, versions in result.items():
        lines.append(f"## {pkg}")
        for proj, ver in versions.items():
            lines.append(f"- **{proj}**: {ver}")
        lines.append("")
    return "\n".join(lines)


@mcp.tool(description="Envexa — find unused dependencies in a project directory")
def find_unused(project: str) -> str:
    result = unused.check_unused(project)
    if not result:
        return "No unused dependency analysis available. Make sure the project has a package.json."
    lines = ["# Unused Dependencies\n"]
    for tool, data in result.items():
        lines.append(f"## {tool}")
        deps = data.get("unused", []) + data.get("devUnused", [])
        lines.extend(f"- {d}" for d in deps) if deps else lines.append("No unused dependencies found.")
        lines.append("")
    return "\n".join(lines)


@mcp.tool(description="Envexa — get the latest dev environment health report")
def get_report() -> str:
    if not REPORT_FILE.exists():
        return "No report available. Run `scan` first."
    report = json.loads(REPORT_FILE.read_text())
    return scanner.format_report(report)


# ── Slash-command relay ─────────────────────────────────────────────────

_CMD_HELP = """Available commands:

  /scan [chain]       — Full health scan (chain: all|brew|npm|pip|gem|cargo|docker)
  /outdated [chain]   — Check outdated packages only
  /status             — Quick dashboard summary
  /upgrade <tool>     — Upgrade a toolchain (pip currently supported)
  /report             — Show the latest cached report
  /help               — Show this message

Examples:
  /scan brew          — Scan only Homebrew
  /upgrade pip        — Upgrade pip to latest
  /status             — One-line health check
"""


@mcp.tool(
    description="Envexa — execute a preset slash command. Usage: /scan, /outdated brew, /status, /upgrade pip, /help"
)
def cmd(command: str) -> str:
    parts = command.strip().split()
    if not parts:
        return _CMD_HELP

    cmd_name = parts[0].lower()
    args = parts[1:]

    if cmd_name in ("/help", "help", "--help", "-h"):
        return _CMD_HELP

    if cmd_name in ("/scan", "scan"):
        chain = args[0] if args else "all"
        return scan(chain)

    if cmd_name in ("/outdated", "outdated"):
        chain = args[0] if args else "all"
        return check_outdated(chain)

    if cmd_name in ("/status", "status"):
        report = scanner.run_scan("all")
        results = report["results"]
        rows = []
        for tool in ("brew", "npm", "pip", "gem", "cargo", "docker"):
            if tool not in results:
                continue
            res = results[tool]
            label = {"ok": "PASS", "warning": "WARN", "error": "FAIL", "skipped": "SKIP"}.get(res["status"], "?")
            n = len(scanner._extract_outdated(res))
            detail = f"({n})" if n else ""
            rows.append(f"| {tool.title():8} | {label:<6} {detail:<8} |")
        lines = [
            "# Envexa Status",
            "",
            f"| {'Tool':8} | {'Status':<16} |",
            f"|{'-'*10}|{'-'*18}|",
            *rows,
            "",
            "Run `/scan` for full report or `/outdated` for details.",
        ]
        return "\n".join(lines)

    if cmd_name in ("/report", "report"):
        return get_report()

    if cmd_name in ("/upgrade", "upgrade"):
        if not args:
            return "Specify what to upgrade: `/upgrade pip`"
        target = args[0].lower()
        if target == "pip":
            if not shutil.which("pip3"):
                return "pip3 not found."
            result = subprocess.run(
                ["pip3", "install", "--upgrade", "pip"],
                capture_output=True, text=True, timeout=60,
            )
            if result.returncode == 0:
                return f"pip upgraded successfully.\n```\n{result.stdout.strip()}\n```"
            return f"pip upgrade failed.\n```\n{result.stderr.strip()}\n```"
        return f"Upgrade not implemented for `{target}`. Supported: pip"

    return f"Unknown command: `{command}`\n\n{_CMD_HELP}"


# ── Slash-command prompts (appear in / menu as /envexa:scan:mcp etc.) ───

@mcp.prompt(name="envexa:scan", description="Scan dev environment toolchains")
def prompt_scan(chain: str = "all") -> list[dict]:
    report = scanner.run_scan(chain)
    if "error" in report:
        return [{"role": "user", "content": report["error"]}]
    return [{"role": "user", "content": scanner.format_report(report)}]


@mcp.prompt(name="envexa:status", description="Quick dashboard summary")
def prompt_status() -> list[dict]:
    report = scanner.run_scan("all")
    results = report["results"]
    rows = []
    for tool in ("brew", "npm", "pip", "gem", "cargo", "docker"):
        if tool not in results:
            continue
        res = results[tool]
        label = {"ok": "PASS", "warning": "WARN", "error": "FAIL", "skipped": "SKIP"}.get(res["status"], "?")
        n = len(scanner._extract_outdated(res))
        detail = f"({n})" if n else ""
        rows.append(f"| {tool.title():8} | {label:<6} {detail:<8} |")
    content = "\n".join([
        "# Envexa Status",
        "",
        f"| {'Tool':8} | {'Status':<16} |",
        f"|{'-'*10}|{'-'*18}|",
        *rows,
        "",
        "Run `/envexa:scan` for full report or `/envexa:outdated` for details.",
    ])
    return [{"role": "user", "content": content}]


@mcp.prompt(name="envexa:outdated", description="Check outdated packages")
def prompt_outdated(chain: str = "all") -> list[dict]:
    report = scanner.run_scan(chain)
    results = report["results"]
    lines = ["# Outdated Packages\n"]
    has_anything = False
    for tool, res in results.items():
        items = []
        for key in ("outdated_formulae", "outdated_casks", "outdated_global", "outdated"):
            if key in res and res[key]:
                items.extend(res[key])
        if items:
            has_anything = True
            lines.append(f"## {tool.title()} ({len(items)} outdated)\n")
            for item in items:
                lines.append(f"- **{item['name']}**: {item.get('current', '?')} -> {item.get('latest', '?')}")
            lines.append("")
    if not has_anything:
        lines = ["All packages are up to date!"]
    return [{"role": "user", "content": "\n".join(lines)}]


@mcp.prompt(name="envexa:upgrade", description="Upgrade a toolchain (pip)")
def prompt_upgrade(tool: str = "pip") -> list[dict]:
    if tool.lower() != "pip":
        return [{"role": "user", "content": f"Upgrade not implemented for `{tool}`. Supported: pip"}]
    if not shutil.which("pip3"):
        return [{"role": "user", "content": "pip3 not found."}]
    result = subprocess.run(
        ["pip3", "install", "--upgrade", "pip"],
        capture_output=True, text=True, timeout=60,
    )
    if result.returncode == 0:
        return [{"role": "user", "content": f"pip upgraded successfully.\n```\n{result.stdout.strip()}\n```"}]
    return [{"role": "user", "content": f"pip upgrade failed.\n```\n{result.stderr.strip()}\n```"}]


@mcp.prompt(name="envexa:report", description="Show the latest cached report")
def prompt_report() -> list[dict]:
    if not REPORT_FILE.exists():
        return [{"role": "user", "content": "No report available. Run `/envexa:scan` first."}]
    report = json.loads(REPORT_FILE.read_text())
    return [{"role": "user", "content": scanner.format_report(report)}]


@mcp.prompt(name="envexa:help", description="Show available envexa commands")
def prompt_help() -> list[dict]:
    content = """# Envexa Slash Commands

| Command | Description |
|---|---|
| `/envexa:scan [chain]` | Full health scan (chain: all, brew, npm, pip, gem, cargo, docker) |
| `/envexa:status` | Quick dashboard summary |
| `/envexa:outdated [chain]` | Check outdated packages |
| `/envexa:upgrade [tool]` | Upgrade a toolchain (pip) |
| `/envexa:report` | Show latest cached report |
| `/envexa:help` | Show this message |

**Examples:**
- `/envexa:scan brew` — Scan only Homebrew
- `/envexa:upgrade pip` — Upgrade pip to latest
- `/envexa:status` — One-line health check
"""
    return [{"role": "user", "content": content}]


# ── Quick-access single-chain tools ─────────────────────────────────────


@mcp.tool(description="Envexa — scan only Homebrew (formulae + casks)")
def brew_status() -> str:
    return scan("brew")


@mcp.tool(description="Envexa — scan only npm/Node.js")
def npm_status() -> str:
    return scan("npm")


@mcp.tool(description="Envexa — scan only Python/pip")
def pip_status() -> str:
    return scan("pip")


@mcp.tool(description="Envexa — upgrade pip to the latest version")
def pip_upgrade() -> str:
    return cmd("/upgrade pip")


# ── Resources ───────────────────────────────────────────────────────────

@mcp.resource(
    uri="envexa://report",
    name="Envexa Health Report",
    description="Latest dev environment health report as markdown",
    mime_type="text/markdown",
)
def report_resource() -> str:
    if not REPORT_FILE.exists():
        return "No report available. Run `scan` first."
    report = json.loads(REPORT_FILE.read_text())
    return scanner.format_report(report)


@mcp.resource(
    uri="envexa://report/raw",
    name="Envexa Health Report (Raw)",
    description="Latest dev environment health report as raw JSON",
    mime_type="application/json",
)
def report_raw_resource() -> str:
    if not REPORT_FILE.exists():
        return json.dumps({"error": "No report available"}, indent=2)
    return REPORT_FILE.read_text()


if __name__ == "__main__":
    mcp.run()
