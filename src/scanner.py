import json
from datetime import datetime
from pathlib import Path
from typing import Any

from .toolchains import brew, npm, pip, gem, cargo, docker

REPORT_FILE = Path(__file__).resolve().parent.parent / "report.json"

SCANNERS = {
    "brew": brew.scan,
    "npm": npm.scan,
    "pip": pip.scan,
    "gem": gem.scan,
    "cargo": cargo.scan,
    "docker": docker.scan,
}

_LABELS = {"ok": "PASS", "warning": "WARN", "error": "FAIL", "skipped": "SKIP"}


def run_scan(chain: str = "all"):
    if chain == "all":
        results = {name: fn() for name, fn in SCANNERS.items()}
    elif chain in SCANNERS:
        results = {chain: SCANNERS[chain]()}
    else:
        return {
            "error": f"Unknown chain: {chain}. Options: all, {', '.join(SCANNERS.keys())}"
        }

    report = {
        "timestamp": datetime.now().isoformat(),
        "results": results,
    }

    REPORT_FILE.write_text(json.dumps(report, indent=2))
    return report


def _extract_outdated(res: dict) -> list[dict[str, Any]]:
    items = []
    for key in ("outdated_formulae", "outdated_casks", "outdated_global", "outdated"):
        if key in res and res[key]:
            for item in res[key]:
                if isinstance(item, dict) and "name" in item:
                    items.append(item)
    return items


def format_report(report: dict = None) -> str:
    if report is None:
        if REPORT_FILE.exists():
            report = json.loads(REPORT_FILE.read_text())
        else:
            return "No report available. Run `scan` first."

    results = report["results"]
    lines = []
    lines.append("# Envexa Health Report")
    lines.append(f"**Generated:** {report['timestamp']}")
    lines.append("")

    outdated_all: dict[str, list[dict[str, Any]]] = {}
    dashboard_rows = []

    for tool, res in results.items():
        outdated_items = _extract_outdated(res)
        if outdated_items:
            outdated_all[tool] = outdated_items

        status_text = {"ok": "PASS", "warning": f"WARN ({len(outdated_items)})", "error": "FAIL", "skipped": "SKIP"}.get(res["status"], "?")
        version_keys = {"version": "Version", "node_version": "Node", "python_version": "Python", "ruby_version": "Ruby", "rustc_version": "Rust"}
        ver_str = ""
        for k in version_keys:
            if k in res and res[k]:
                ver_str = res[k]
                break
        dashboard_rows.append(f"| {tool.title():8} | {status_text:<18} | {ver_str} |")

    lines.append("## Dashboard")
    lines.append(f"| {'Toolchain':8} | {'Status':<18} | {'Version'} |")
    lines.append(f"|{'-'*10}|{'-'*20}|{'-'*20}|")
    for row in dashboard_rows:
        lines.append(row)
    lines.append("")

    if outdated_all:
        lines.append("## Outdated Packages")
        lines.append("```")
        tool_names = list(outdated_all.keys())
        for i, tool in enumerate(tool_names):
            items = outdated_all[tool]
            is_last_tool = i == len(tool_names) - 1
            tool_prefix = "└── " if is_last_tool else "├── "
            lines.append(f"{tool_prefix}{tool.title()} ({len(items)})")
            indent_prefix = "    " if is_last_tool else "│   "
            for j, item in enumerate(items):
                sub_prefix = "└── " if j == len(items) - 1 else "├── "
                ver = f"{item.get('current', '?')} -> {item.get('latest', '?')}"
                lines.append(f"{indent_prefix}{sub_prefix}{item['name']}: {ver}")
        lines.append("```")
        lines.append("")

    lines.append("## Per-Toolchain Details")
    lines.append("")
    for tool, res in results.items():
        label = _LABELS.get(res["status"], "?")
        lines.append(f"### [{label}] {tool.title()}")

        if res["status"] == "skipped":
            lines.append(f"> {res['issues'][0] if res['issues'] else 'Skipped'}")
            lines.append("")
            continue

        version_labels = {"version": "Version", "node_version": "Node", "python_version": "Python", "ruby_version": "Ruby", "rustc_version": "Rust", "cargo_version": "Cargo"}
        ver_parts = []
        for k, v in version_labels.items():
            if k in res and res[k]:
                ver_parts.append(f"**{v}:** {res[k]}")
        if ver_parts:
            lines.append(" | ".join(ver_parts))

        installed = res.get("installed_count")
        if installed:
            lines.append(f"**Formulae:** {installed}")

        disk = res.get("disk_usage")
        if disk:
            for typ, info in disk.items():
                lines.append(f"- **{typ}:** {info.get('size', '?')} ({info.get('reclaimable', '?')} reclaimable)")

        outdated_items = _extract_outdated(res)
        if outdated_items:
            lines.append("")
            lines.append("| Package | Current | Latest |")
            lines.append("|---------|---------|--------|")
            for item in outdated_items:
                lines.append(f"| {item['name']} | {item.get('current', '?')} | {item.get('latest', '?')} |")

        tool_issues = [i for i in res.get("issues", []) if i]
        if tool_issues:
            lines.append("")
            for i in tool_issues:
                lines.append(f"> {i}")

        lines.append("")

    return "\n".join(lines)
