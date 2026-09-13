import { Box } from "ink";
import { parseLogLine, type LogPair } from "../lib/config";
import { pad, type Span } from "../lib/format";
import { theme } from "../theme";
import { Spans } from "./Spans";

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function fmtTime(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${MONTHS[d.getMonth()]} ${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function levelColor(level: string): string {
  switch (level) {
    case "WARN": return theme.warn;
    case "ERROR": return theme.error;
    case "DEBUG": return theme.dim;
    default: return theme.ok;
  }
}

// Colored, deduped log lines — web-dashboard style: level-colored entry,
// source tag, and a ×N marker collapsing consecutive duplicates (the Go
// formatLogLines counterpart).
export function formatLogLines(logs: LogPair[]): Span[][] {
  const out: Span[][] = [];
  let i = 0;
  while (i < logs.length) {
    const e = parseLogLine(logs[i]);
    let j = i + 1;
    while (j < logs.length) {
      const n = parseLogLine(logs[j]);
      if (n.level === e.level && n.message === e.message && n.source === e.source) {
        j++;
        continue;
      }
      break;
    }
    const spans: Span[] = [
      { text: fmtTime(logs[i].time), color: theme.dim },
      { text: "  " },
      { text: pad(e.level, 5), color: levelColor(e.level) },
      { text: "  " },
      { text: e.message },
    ];
    if (e.source !== "system") spans.push({ text: `  [${e.source}]`, color: theme.dim });
    if (j - i > 1) spans.push({ text: `  ×${j - i}`, color: theme.dim });
    out.push(spans);
    i = j;
  }
  return out;
}

// Scrollable log region: offset counts lines up from the bottom (0 = live
// tail), matching the bubbles/viewport GotoBottom behavior.
export function LogStream({
  logs,
  height,
  offset,
}: {
  logs: LogPair[];
  height: number;
  offset: number;
}) {
  let lines = formatLogLines(logs);
  if (lines.length === 0) {
    lines = [[{ text: "no logs yet — run a scan", color: theme.dim }]];
  }
  const start = Math.max(0, lines.length - height - offset);
  const visible = lines.slice(start, start + height);
  return (
    <Box flexDirection="column">
      {visible.map((spans, i) => (
        <Spans key={start + i} spans={spans} />
      ))}
    </Box>
  );
}
