// Shared row-rendering helpers: fixed-width monospace cells like the Go TUI
// (pad/truncate over display width), styled as {text, color} spans.
import { statusColor, statusLabel, theme } from "../theme";
import type { ScanResult } from "./types";

export interface Span {
  text: string;
  color?: string;
  bold?: boolean;
}

export function pad(s: string, n: number): string {
  return s.length >= n ? s : s + " ".repeat(n - s.length);
}

export function truncate(s: string, w: number): string {
  if (s.length <= w) return s;
  if (w <= 1) return "…";
  return s.slice(0, w - 1) + "…";
}

// displayVersion picks the first tool-specific version field, like the Rust
// toolchains view.
export function displayVersion(r: ScanResult): string {
  for (
    const v of [
      r.version, r.node_version, r.python_version, r.ruby_version, r.rustc_version,
      r.cargo_version, r.pnpm_version, r.bun_version, r.deno_version,
    ]
  ) {
    if (v) return v;
  }
  return "";
}

export function installedCount(r: ScanResult): string {
  return r.installed_count === undefined ? "" : String(r.installed_count);
}

export function countOutdated(r: ScanResult): number {
  return (r.outdated?.length ?? 0) + (r.outdated_global?.length ?? 0) +
    (r.outdated_formulae?.length ?? 0) + (r.outdated_casks?.length ?? 0);
}

export function rowLabel(tool: string): string {
  if (tool === "ci") return "CI/CD";
  if (tool === "") return tool;
  return tool[0].toUpperCase() + tool.slice(1);
}

export function statusSpan(status: string, width: number): Span {
  return {
    text: pad(statusLabel(status).toUpperCase(), width),
    color: statusColor(status),
  };
}

export function dim(text: string): Span {
  return { text, color: theme.dim };
}

export function accent(text: string, bold = false): Span {
  return { text, color: theme.accent, bold };
}

// filterIndices returns indices of items matching the query
// (case-insensitive substring over package and source).
export function filterIndices(
  items: { source: string; name: string }[],
  query: string,
): number[] {
  const q = query.toLowerCase();
  const out: number[] = [];
  for (let i = 0; i < items.length; i++) {
    if (q === "" || items[i].name.toLowerCase().includes(q) ||
      items[i].source.toLowerCase().includes(q)
    ) {
      out.push(i);
    }
  }
  return out;
}

// sliceWindow keeps the cursor inside a visible window of max rows — the
// bubbles/table scroll behavior.
export function sliceWindow(len: number, cursor: number, max: number): [number, number] {
  if (len <= max) return [0, len];
  const start = Math.min(Math.max(0, cursor - (max - 1)), len - max);
  return [start, start + max];
}

export function scanAge(timestamp: string): string {
  const ts = new Date(timestamp);
  if (isNaN(ts.getTime())) return "—";
  const mins = (Date.now() - ts.getTime()) / 60_000;
  if (mins < 1) return "just now";
  if (mins < 60) return `${Math.floor(mins)}m ago`;
  return `${Math.floor(mins / 60)}h ago`;
}
