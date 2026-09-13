import { Box, Text } from "ink";
import { Gauge } from "../components/Gauge";
import { Footer } from "../components/Header";
import { PieChart } from "../components/PieChart";
import { Panel } from "../components/Panel";
import { Spans } from "../components/Spans";
import { ToolchainPanel } from "../components/ToolchainPanel";
import { dim, displayVersion, pad, truncate, type Span } from "../lib/format";
import type { Report, VulnerabilityInfo } from "../lib/types";
import { statusColor, statusLabel, theme } from "../theme";

// Dashboard groups mirror the ui.rs sections.
const toolGroups = [
  { title: "System & Runtime", tools: ["brew", "cargo", "docker", "pip", "gem"] },
  { title: "Web Development", tools: ["npm", "pnpm", "yarn", "bun", "deno"] },
  { title: "Project Tooling", tools: ["project", "security", "audit", "ci"] },
];

export type DashTab = 0 | 1 | 2;

export function Dashboard({
  report,
  width,
  height,
  tab,
  sel,
}: {
  report: Report;
  width: number;
  height: number;
  tab: DashTab;
  sel: number;
}) {
  // Shared row cursor across the grouped panels.
  const dashRows = toolGroups.reduce(
    (n, g) => n + g.tools.filter((t) => report.results[t]).length, 0);

  let body: React.ReactNode;
  if (tab === 1) {
    const vulns = report.results.security?.vulnerabilities ?? [];
    body = (
      <>
        <Panel title="Vulnerabilities" width={width - 2}>
          <VulnTable vulns={vulns} sel={sel} />
        </Panel>
        {report.results.security?.issues?.length ? (
          <Text color={theme.dim}>{report.results.security.issues.join(" · ")}</Text>
        ) : null}
      </>
    );
  } else if (tab === 2) {
    body = (
      <Panel title="All toolchains" width={width - 2}>
        <AllToolchainsTable report={report} sel={sel} />
      </Panel>
    );
  } else {
    body = <OverviewBody report={report} width={width} height={height} sel={sel} dashRows={dashRows} />;
  }

  return (
    <Box flexDirection="column">
      {body}
      <Footer width={width} />
    </Box>
  );
}

function OverviewBody({
  report,
  width,
  height,
  sel,
  dashRows,
}: {
  report: Report;
  width: number;
  height: number;
  sel: number;
  dashRows: number;
}) {
  // Left column ~30% of the terminal, clamped to a readable band; below ~100
  // cols everything stacks and the pie hides.
  const wide = width >= 100;
  const leftW = wide ? Math.max(30, Math.min(40, Math.floor((width * 3) / 10))) : width - 2;
  const rightW = wide ? width - leftW - 2 : width - 2;

  const left = (
    <Box flexDirection="column">
      <OverviewPanel report={report} width={leftW} height={height} />
      <ProjectToolingPanel report={report} width={leftW} />
    </Box>
  );

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
      />
    );
    base += g.tools.filter((t) => report.results[t]).length;
    return panel;
  });

  return (
    <Box flexDirection={wide ? "row" : "column"} gap={1}>
      {left}
      <Box flexDirection="column" gap={1}>{groups}</Box>
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
function OverviewPanel({ report, width, height }: { report: Report; width: number; height: number }) {
  const counts = statusCounts(report);
  const legend: Span[] = [];
  for (const [st, label] of [["ok", "PASS"], ["warn", "WARN"], ["error", "ERROR"], ["skipped", "SKIP"]] as const) {
    if (counts[st] === 0) continue;
    legend.push({ text: `■ ${label} (${counts[st]})`, color: statusColor(st) });
  }
  const showPie = width - 2 >= 22 && height >= 18;
  return (
    <Panel title="Overview" width={width}>
      <LegendWrap legend={legend} width={width - 2} />
      {showPie && (
        <Box marginTop={1}>
          <PieChart counts={counts} />
        </Box>
      )}
    </Panel>
  );
}

// Wrap styled chunks at width with two-space joins (wrapChunks counterpart).
function LegendWrap({ legend, width }: { legend: Span[]; width: number }) {
  const lines: Span[][] = [];
  let cur: Span[] = [];
  let curW = 0;
  for (const s of legend) {
    if (cur.length > 0 && curW + 2 + s.text.length > width) {
      lines.push(cur);
      cur = [];
      curW = 0;
    }
    if (cur.length > 0) cur.push({ text: "  " });
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

// Readiness bar, first-class Project/Security/Audit signals, severity chips.
function ProjectToolingPanel({ report, width }: { report: Report; width: number }) {
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
    <Panel title="Project Tooling" width={width}>
      <Gauge value={readiness} width={Math.max(10, width - 4)} />
      <Text>
        <Text color={theme.accent}>readiness {Math.round(readiness * 100)}%</Text>
        <Text color={theme.dim}> │ risk {risk}/100</Text>
      </Text>
      <Text> </Text>
      <Spans spans={prow("Project", p?.status ?? "skipped",
        `${p?.project_type || "—"} / ${(p?.outdated ?? []).length} outdated`)} />
      <Spans spans={prow("Security", sec?.status ?? "skipped",
        `${(sec?.vulnerabilities ?? []).length} vulns`)} />
      <Spans spans={prow("Audit", audit?.status ?? "skipped",
        `${(audit?.audit_items ?? []).length} checks flagged`)} />
      <Text> </Text>
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

const vulnColumns = ["Package", "Severity", "Fixed", "Title"];
const vulnWidths = [20, 10, 12, 0];

function VulnTable({
  vulns,
  sel,
}: {
  vulns: VulnerabilityInfo[];
  sel: number;
}) {
  const rows = vulns.map((v) => [
    { text: pad(truncate(v.package, 20), 20) },
    { text: pad(v.severity, 10), color: severityColor(v.severity) },
    { text: pad(v.patched_version, 12) },
    { text: v.title + (v.cve ?? "") },
  ]);
  return <WindowedRows rows={rows} widths={vulnWidths} headers={vulnColumns} sel={sel} />;
}

function severityColor(sev: string): string {
  switch (sev) {
    case "CRITICAL":
    case "HIGH": return theme.error;
    case "MEDIUM": return theme.warn;
    default: return theme.dim;
  }
}

const toolColumns = ["Toolchain", "Status", "Version", "Installed"];

function AllToolchainsTable({ report, sel }: { report: Report; sel: number }) {
  const rows: Span[][] = [];
  for (const [name, r] of Object.entries(report.results).sort(([a], [b]) => a.localeCompare(b))) {
    rows.push([
      { text: pad(name, 14) },
      { text: pad(statusLabel(r.status).toUpperCase(), 10), color: statusColor(r.status) },
      { text: pad(truncate(displayVersion(r), 20), 20) },
      { text: r.installed_count === undefined ? "" : String(r.installed_count) },
    ]);
  }
  return <WindowedRows rows={rows} widths={[14, 10, 20, 0]} headers={toolColumns} sel={sel} />;
}

// Simple fixed-window table (8 visible rows, cursor kept in view) — the
// bubbles/table scroll behavior.
function WindowedRows({
  rows,
  widths,
  headers,
  sel,
}: {
  rows: Span[][];
  widths: number[];
  headers: string[];
  sel: number;
}) {
  const maxRows = 8;
  const start = rows.length <= maxRows
    ? 0
    : Math.min(Math.max(0, sel - (maxRows - 1)), rows.length - maxRows);
  const visible = rows.slice(start, start + maxRows);
  return (
    <Box flexDirection="column">
      <Spans spans={headers.map((h, i) => dim(pad(h, widths[i] || h.length)))} />
      {visible.map((spans, i) => (
        <Spans key={start + i} spans={spans} selected={start + i === sel} />
      ))}
    </Box>
  );
}
