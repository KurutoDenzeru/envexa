import json
from pathlib import Path

from mcp.server.fastmcp import FastMCP

from . import scanner, mismatches, unused

mcp = FastMCP("Envexa")

REPORT_FILE = Path(__file__).resolve().parent.parent / "report.json"


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
                    f"- **{item['name']}**: {item.get('current', '?')} → {item.get('latest', '?')}"
                )
            lines.append("")

    if not has_anything:
        return "All packages are up to date! 🎉"

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
        unused_deps = data.get("unused", []) + data.get("devUnused", [])
        if unused_deps:
            for dep in unused_deps:
                lines.append(f"- {dep}")
        else:
            lines.append("No unused dependencies found.")
        lines.append("")

    return "\n".join(lines)


@mcp.tool(description="Envexa — get the latest dev environment health report")
def get_report() -> str:
    if not REPORT_FILE.exists():
        return "No report available. Run `scan` first."

    report = json.loads(REPORT_FILE.read_text())
    return scanner.format_report(report)


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
