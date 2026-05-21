import json
from datetime import datetime
from pathlib import Path
from typing import Any

from .toolchains import brew, npm, pip, gem, cargo, docker, pnpm, yarn, bun, deno

REPORT_FILE = Path(__file__).resolve().parent.parent / "report.json"

SCANNERS = {
    "brew": brew.scan,
    "npm": npm.scan,
    "pnpm": pnpm.scan,
    "yarn": yarn.scan,
    "bun": bun.scan,
    "deno": deno.scan,
    "pip": pip.scan,
    "gem": gem.scan,
    "cargo": cargo.scan,
    "docker": docker.scan,
}

_ICONS = {
    "brew": "🍺",
    "npm": "",
    "pnpm": "",
    "yarn": "",
    "bun": "",
    "deno": "",
    "pip": "",
    "gem": "",
    "cargo": "🦀",
    "docker": "🐳",
}

_STATUS_EMOJI = {"ok": "✅", "warning": "⚠️", "error": "❌", "skipped": "⏭️"}
_LABELS = {"ok": "PASS", "warning": "WARN", "error": "FAIL", "skipped": "SKIP"}
_DISPLAY_NAMES = {
    "brew": "Brew", "npm": "npm", "pnpm": "pnpm", "yarn": "Yarn",
    "bun": "Bun", "deno": "Deno", "pip": "pip", "gem": "Gem",
    "cargo": "Cargo", "docker": "Docker",
}


def run_scan(chain: str = "all"):
    chain = (chain or "").strip() or "all"
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
        icon = _ICONS.get(tool, "")
        outdated_items = _extract_outdated(res)
        if outdated_items:
            outdated_all[tool] = outdated_items

        status_emoji = _STATUS_EMOJI.get(res["status"], "")
        status_text = {"ok": "PASS", "warning": f"WARN ({len(outdated_items)})", "error": "FAIL", "skipped": "SKIP"}.get(res["status"], "?")
        version_keys = {"version": "Version", "node_version": "Node", "python_version": "Python", "ruby_version": "Ruby", "rustc_version": "Rust", "pnpm_version": "pnpm", "yarn_version": "yarn", "bun_version": "bun", "deno_version": "deno"}
        ver_str = ""
        for k in version_keys:
            if k in res and res[k]:
                ver_str = res[k]
                break
        display = _DISPLAY_NAMES.get(tool, tool.title())
        dashboard_rows.append(f"| {icon} {display:7} | {status_emoji} {status_text:<16} | {ver_str} |")

    lines.append("## Dashboard")
    lines.append(f"| {'Toolchain':10} | {'Status':<20} | {'Version'} |")
    lines.append(f"|{'-'*12}|{'-'*22}|{'-'*20}|")
    for row in dashboard_rows:
        lines.append(row)
    lines.append("")

    if outdated_all:
        lines.append("## Outdated Packages")
        lines.append("")
        lines.append("| Toolchain | Package | Current | Latest |")
        lines.append("|-----------|---------|---------|--------|")
        for tool in ("brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker"):
            items = outdated_all.get(tool)
            if not items:
                continue
            display = _DISPLAY_NAMES.get(tool, tool.title())
            icon = _ICONS.get(tool, "")
            for item in items:
                lines.append(f"| {icon} {display} | {item['name']} | {item.get('current', '?')} | {item.get('latest', '?')} |")
        lines.append("")

    lines.append("## Per-Toolchain Details")
    lines.append("")
    for tool, res in results.items():
        icon = _ICONS.get(tool, "")
        label = _LABELS.get(res["status"], "?")
        display = _DISPLAY_NAMES.get(tool, tool.title())
        lines.append(f"### {icon} [{label}] {display}")

        if res["status"] == "skipped":
            lines.append(f"> {res['issues'][0] if res['issues'] else 'Skipped'}")
            lines.append("")
            continue

        version_labels = {"version": "Version", "node_version": "Node", "python_version": "Python", "ruby_version": "Ruby", "rustc_version": "Rust", "cargo_version": "Cargo", "pnpm_version": "pnpm", "yarn_version": "yarn", "bun_version": "bun", "deno_version": "deno"}
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


def format_status(report: dict = None) -> str:
    if report is None:
        if REPORT_FILE.exists():
            report = json.loads(REPORT_FILE.read_text())
        else:
            return "No report available. Run `scan` first."

    results = report["results"]
    lines = []
    lines.append("# Envexa Status")
    lines.append(f"**Generated:** {report['timestamp']}")
    lines.append("")
    lines.append("| Toolchain | Status | Count |")
    lines.append("|-----------|--------|-------|")
    for tool in ("brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker"):
        if tool not in results:
            continue
        res = results[tool]
        icon = _ICONS.get(tool, "")
        label = _LABELS.get(res["status"], "?")
        emoji = _STATUS_EMOJI.get(res["status"], "")
        outdated = _extract_outdated(res)
        n = len(outdated)
        display = _DISPLAY_NAMES.get(tool, tool.title())
        count = f"({n})" if n else ""
        lines.append(f"| {icon} {display} | {emoji} {label} | {count} |")
    lines.append("")
    lines.append("Run `/envexa:scan` for full report or `/envexa:outdated` for details.")
    return "\n".join(lines)
