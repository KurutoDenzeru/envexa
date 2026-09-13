import { Box, Text } from "ink";
import { Gauge } from "../components/Gauge";
import { Footer } from "../components/Header";
import { PieChart } from "../components/PieChart";
import { Panel } from "../components/Panel";
import { Spans } from "../components/Spans";
import { ToolchainPanel } from "../components/ToolchainPanel";
import { pad, type Span } from "../lib/format";
import type { Report } from "../lib/types";
import { statusColor, statusLabel, theme } from "../theme";

// Dashboard groups mirror the ui.rs sections; exported so the App key
// handler can Tab-cycle the highlight across panels.
export const toolGroups = [
  { title: "System & Runtime", tools: ["brew", "cargo", "docker", "pip", "gem"] },
  { title: "Web Development", tools: ["npm", "pnpm", "yarn", "bun", "deno"] },
  { title: "Project Tooling", tools: ["project", "security", "audit", "ci"] },
];

export function Dashboard({
  report,
  width,
  height,
  sel,
}: {
  report: Report;
  width: number;
  height: number;
  sel: number;
}) {
  const dashRows = toolGroups.reduce(
    (n, g) => n + g.tools.filter((t) => report.results[t]).length, 0);

  // Left column ~30% of the terminal, clamped to a readable band; below ~100
  // cols everything stacks and the pie hides.
  const wide = width >= 100;
  const leftW = wide ? Math.max(30, Math.min(40, Math.floor((width * 3) / 10))) : width - 2;
  const rightW = wide ? width - leftW - 2 : width - 2;

  let base = 0;
  const groups = toolGroups.map((g) => {
    const panel = (
      <ToolchainPanel
        key={g.title}
        title={g.title}
        tools={g.tools}
        results={report.results}
        width={rightW}
        baseOffset={base}
        selected={sel < dashRows ? sel : -1}
        flexGrow={1}
      />
    );
    base += g.tools.filter((t) => report.results[t]).length;
    return panel;
  });

  const left = (
    <Box flexDirection="column" width={leftW} flexShrink={0} gap={1}>
      <OverviewPanel report={report} width={leftW} height={height} flexGrow={3} />
      <ProjectToolingPanel report={report} width={leftW} flexGrow={2} />
    </Box>
  );

  return (
    // flexGrow, not height={height}: this box renders inside the App shell
    // (header + hints already take rows), so claiming the full terminal
    // height overflows and scrolls the alt screen over the logo.
    <Box flexDirection="column" flexGrow={1} flexShrink={1} justifyContent="space-between">
      <Box flexDirection={wide ? "row" : "column"} gap={1}>
        {left}
        <Box flexDirection="column" flexGrow={wide ? 1 : 0} width={wide ? undefined : rightW} gap={1}>
          {groups}
        </Box>
      </Box>
      <Footer width={width} />
    </Box>
  );
}

function statusCounts(report: Report) {
  let ok = 0, warn = 0, error = 0, skipped = 0;
  for (const r of Object.values(report.results)) {
    switch (r.status) {
      case "ok": ok++; break;
      case "warn":
      case "warning": warn++; break;
      case "error": error++; break;
      default: skipped++;
    }
  }
  return { ok, warn, error, skipped };
}

// Legend + dot pie (pie hidden on narrow terminals).
function OverviewPanel({
  report,
  width,
  height,
  flexGrow,
}: {
  report: Report;
  width: number;
  height: number;
  flexGrow?: number;
}) {
  const counts = statusCounts(report);
  const legend = (
    [
      ["ok", "PASS"],
      ["warn", "WARN"],
      ["error", "ERROR"],
      ["skipped", "SKIP"],
    ] as const
  ).filter(([st]) => counts[st] > 0)
    .map(([st, label]) => ({ text: `■ ${label} (${counts[st]})`, color: statusColor(st) }));
  const showPie = width - 2 >= 22 && height >= 18;
  return (
    <Panel title="Overview" width={width} flexGrow={flexGrow}>
      <LegendWrap legend={legend} width={width - 2} />
      {showPie && (
        <Box flexGrow={1} flexDirection="column" justifyContent="center">
          <PieChart counts={counts} />
        </Box>
      )}
    </Panel>
  );
}

// Wrap styled chunks at width with two-space joins (wrapChunks counterpart).
function LegendWrap({ legend, width }: { legend: { text: string; color: string }[]; width: number }) {
  const lines: { text: string; color: string }[][] = [];
  let cur: { text: string; color: string }[] = [];
  let curW = 0;
  for (const s of legend) {
    if (cur.length > 0 && curW + 2 + s.text.length > width) {
      lines.push(cur);
      cur = [];
      curW = 0;
    }
    if (cur.length > 0) cur.push({ text: "  ", color: "white" });
    cur.push(s);
    curW += s.text.length + (cur.length > 1 ? 2 : 0);
  }
  if (cur.length > 0) lines.push(cur);
  return (
    <Box flexDirection="column">
      {lines.map((spans, i) => (
        <Spans key={i} spans={spans} />
      ))}
    </Box>
  );
}

// Readiness bar with the caption on the bar, first-class Project/Security/
// Audit signals, severity chips.
function ProjectToolingPanel({
  report,
  width,
  flexGrow,
}: {
  report: Report;
  width: number;
  flexGrow?: number;
}) {
  const counts = statusCounts(report);
  const total = Object.keys(report.results).length;
  const readiness = total > 0 ? counts.ok / total : 0;
  const risk = Math.min(100, counts.error * 15 + counts.warn * 5);

  const prow = (name: string, st: string, note: string): Span[] => [
    { text: pad(name, 9) },
    { text: pad(statusLabel(st).toUpperCase(), 6), color: statusColor(st), bold: true },
    { text: note },
  ];
  const p = report.results.project;
  const sec = report.results.security;
  const audit = report.results.audit;

  return (
    <Panel title="Project Tooling" width={width} flexGrow={flexGrow}>
      <Gauge
        value={readiness}
        width={Math.max(10, width - 4)}
        label={`readiness ${Math.round(readiness * 100)}% │ risk ${risk}/100`}
      />
      <Text> </Text>
      <Spans spans={prow("Project", p?.status ?? "skipped",
        `${p?.project_type || "—"} / ${(p?.outdated ?? []).length} outdated`)} />
      <Spans spans={prow("Security", sec?.status ?? "skipped",
        `${(sec?.vulnerabilities ?? []).length} vulns`)} />
      <Spans spans={prow("Audit", audit?.status ?? "skipped",
        `${(audit?.audit_items ?? []).length} checks flagged`)} />
      <Box flexGrow={1} />
      <SeverityChips report={report} />
    </Panel>
  );
}

function SeverityChips({ report }: { report: Report }) {
  let crit = 0, high = 0, med = 0, oth = 0;
  for (const v of report.results.security?.vulnerabilities ?? []) {
    switch (v.severity) {
      case "CRITICAL": crit++; break;
      case "HIGH": high++; break;
      case "MEDIUM": med++; break;
      default: oth++;
    }
  }
  const auditN = (report.results.audit?.audit_items ?? []).length;
  const chip = (name: string, n: number, base = "error"): Span => ({
    text: `${name} ${n}`,
    color: n > 0 ? base : theme.skipped,
  });
  const chips: Span[] = [
    { text: `Outd ${report.outdated.length}`, color: theme.warn },
    chip("Crit", crit), chip("High", high), chip("Medi", med), chip("Othe", oth),
    { text: `Audi ${auditN}`, color: auditN > 0 ? theme.warn : theme.skipped },
  ];
  return (
    <Text>
      {chips.map((s, i) => (
        <Text key={i} color={s.color}>{s.text}{i < chips.length - 1 ? "  " : ""}</Text>
      ))}
    </Text>
  );
}
