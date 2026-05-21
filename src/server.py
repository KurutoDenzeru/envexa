import json
import subprocess
import shutil
from pathlib import Path

from mcp.server.fastmcp import FastMCP

from . import scanner, mismatches, unused
from .scanner import _DISPLAY_NAMES

mcp = FastMCP("Envexa")

REPORT_FILE = Path(__file__).resolve().parent.parent / "report.json"


# ── Core tools ──────────────────────────────────────────────────────────

@mcp.tool(
    description="Envexa — scan dev environment toolchains. chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker"
)
def scan(chain: str | None = "all") -> str:
    chain = (chain or "").strip() or "all"
    report = scanner.run_scan(chain)
    if "error" in report:
        return report["error"]
    return scanner.format_report(report)


@mcp.tool(
    description="Envexa — check for outdated packages. chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker"
)
def check_outdated(chain: str | None = "all") -> str:
    chain = (chain or "").strip() or "all"
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
            display = _DISPLAY_NAMES.get(tool, tool.title())
            lines.append(f"## {display} ({len(items)} outdated)\n")
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

_CMD_HELP = """Envexa slash commands — type these in chat or pass to the cmd tool:

  /scan [chain]       — Full health scan (chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker)
  /outdated [chain]   — Check outdated packages only
  /status             — Quick dashboard summary
  /upgrade <tool>     — Upgrade a toolchain (pip currently supported)
  /report             — Show the latest cached report
  /help               — Show this message

Examples:
  /scan brew          — Scan only Homebrew
  /scan pnpm          — Scan only pnpm
  /upgrade pip        — Upgrade pip to latest
  /status             — One-line health check
"""


@mcp.tool(
    description="Envexa — execute a preset slash command. Use this when the user types /scan, /outdated, /status, /upgrade, /report, or /help in chat. Usage example: cmd(\"/scan brew\"), cmd(\"/status\")"
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
        chain = args[0].strip() if args else "all"
        return scan(chain)

    if cmd_name in ("/outdated", "outdated"):
        chain = args[0].strip() if args else "all"
        return check_outdated(chain)

    if cmd_name in ("/status", "status"):
        report = scanner.run_scan("all")
        results = report["results"]
        rows = []
        for tool in ("brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker"):
            if tool not in results:
                continue
            res = results[tool]
            label = {"ok": "PASS", "warning": "WARN", "error": "FAIL", "skipped": "SKIP"}.get(res["status"], "?")
            n = len(scanner._extract_outdated(res))
            detail = f"({n})" if n else ""
            display = _DISPLAY_NAMES.get(tool, tool.title())
            rows.append(f"| {display:8} | {label:<6} {detail:<8} |")
        lines = [
            "# Envexa Status",
            "",
            f"| {'Tool':8} | {'Status':<16} |",
            f"|{'-'*10}|{'-'*18}|",
            *rows,
            "",
            "Run `/envexa:scan` for full report or `/envexa:outdated` for details.",
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

@mcp.prompt(name="envexa:scan", description="Envexa — full health scan of dev environment toolchains")
def prompt_scan() -> list[dict]:
    report = scanner.run_scan("all")
    if "error" in report:
        return [{"role": "assistant", "content": report["error"]}]
    return [{"role": "assistant", "content": scanner.format_report(report)}]


@mcp.prompt(name="envexa:status", description="Envexa — full health report of all toolchains")
def prompt_status() -> list[dict]:
    report = scanner.run_scan("all")
    return [{"role": "assistant", "content": scanner.format_report(report)}]


@mcp.prompt(name="envexa:outdated", description="Envexa — list outdated packages across all toolchains")
def prompt_outdated() -> list[dict]:
    report = scanner.run_scan("all")
    results = report["results"]
    lines = ["# Envexa Outdated Packages\n"]
    has_anything = False
    for tool, res in results.items():
        items = []
        for key in ("outdated_formulae", "outdated_casks", "outdated_global", "outdated"):
            if key in res and res[key]:
                items.extend(res[key])
        if items:
            has_anything = True
            display = _DISPLAY_NAMES.get(tool, tool.title())
            lines.append(f"## {display} ({len(items)} outdated)\n")
            for item in items:
                lines.append(f"- **{item['name']}**: {item.get('current', '?')} -> {item.get('latest', '?')}")
            lines.append("")
    if not has_anything:
        lines = ["# Envexa Outdated Packages\n\nAll packages are up to date!"]
    return [{"role": "assistant", "content": "\n".join(lines)}]


# ── Quick-access single-chain tools ─────────────────────────────────────


@mcp.tool(description="Envexa — scan only Homebrew (formulae + casks)")
def brew_status() -> str:
    return scan("brew")


@mcp.tool(description="Envexa — scan only npm/Node.js")
def npm_status() -> str:
    return scan("npm")


@mcp.tool(description="Envexa — scan only pnpm")
def pnpm_status() -> str:
    return scan("pnpm")


@mcp.tool(description="Envexa — scan only Yarn")
def yarn_status() -> str:
    return scan("yarn")


@mcp.tool(description="Envexa — scan only Bun")
def bun_status() -> str:
    return scan("bun")


@mcp.tool(description="Envexa — scan only Deno")
def deno_status() -> str:
    return scan("deno")


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
