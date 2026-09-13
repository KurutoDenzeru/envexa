import { Panel } from "./Panel";
import { Spans } from "./Spans";
import { dim, displayVersion, pad, rowLabel, statusSpan, truncate, type Span } from "../lib/format";
import type { ScanResult } from "../lib/types";

// One grouped toolchain table (System & Runtime / Web Development / Project
// Tooling) — the Go groupPanels() counterpart. selected is the shared
// dashboard cursor resolved to a global row index (baseOffset + local row).
export function ToolchainPanel({
  title,
  tools,
  results,
  width,
  baseOffset,
  selected,
  flexGrow,
}: {
  title: string;
  tools: string[];
  results: Record<string, ScanResult>;
  width: number;
  baseOffset: number;
  selected: number;
  flexGrow?: number;
}) {
  const rows: Span[][] = [
    [
      dim(pad("Toolchain", 12)),
      dim(pad("Status", 9)),
      dim(pad("Version", 16)),
      dim(pad("Outd", 5)),
      dim("Issues"),
    ],
  ];

  for (const tool of tools) {
    const r = results[tool];
    if (!r) continue;
    const outd = (r.outdated?.length ?? 0) + (r.outdated_global?.length ?? 0) +
      (r.outdated_formulae?.length ?? 0) + (r.outdated_casks?.length ?? 0);
    rows.push([
      { text: pad(rowLabel(tool), 12) },
      statusSpan(r.status, 9),
      { text: pad(truncate(displayVersion(r), 16), 16) },
      { text: pad(truncate(outd > 0 ? String(outd) : "", 5), 5) },
      { text: truncate((r.issues ?? []).join("; "), Math.max(8, width - 44)) },
    ]);
  }

  return (
    <Panel title={title} width={width} flexGrow={flexGrow}>
      {rows.map((spans, i) => (
        // i=0 is the column header — never selectable; the -1 sentinel
        // (no selection) would otherwise alias to it when baseOffset=0.
        <Spans key={i} spans={spans} selected={i > 0 && baseOffset + i - 1 === selected} />
      ))}
    </Panel>
  );
}
