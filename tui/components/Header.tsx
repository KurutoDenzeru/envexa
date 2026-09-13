import { Box, Text } from "ink";
import { accent, dim, scanAge, type Span } from "../lib/format";
import type { Report } from "../lib/types";
import { theme } from "../theme";

const logoArt = `███████╗███╗   ██╗██╗   ██╗███████╗██╗  ██╗ █████╗
██╔════╝████╗  ██║██║   ██║██╔════╝╚██╗██╔╝██╔══██╗
█████╗ ██╔██╗ ██║██║   ██║█████╗   ╚███╔╝ ███████║
██╔══╝ ██║╚██╗██║╚██╗ ██╔╝██╔══╝   ██╔██╗ ██╔══██║
███████╗██║ ╚████║ ╚████╔╝ ███████╗██╔╝ ██╗██║  ██║
╚══════╝╚═╝  ╚═══╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝`;

export type HeaderView = "dashboard" | "outdated" | "logs" | "settings";

const tabs: { v: HeaderView; n: string }[] = [
  { v: "dashboard", n: "Dashboard" },
  { v: "outdated", n: "Outdated" },
  { v: "logs", n: "Logs" },
  { v: "settings", n: "Settings" },
];

// Shared chrome: logo, project path, tab bar, key hints, status line — the Go
// dashboardHeader() counterpart.
export function Header({
  view,
  width,
  height,
  report,
  projectPath,
}: {
  view: HeaderView;
  width: number;
  height: number;
  report: Report | null;
  projectPath: string;
}) {
  return (
    <Box flexDirection="column" marginBottom={0}>
      {width >= 100 && height >= 22 && (
        <Box flexDirection="column" width={width} alignItems="center">
          <Text color={theme.accent}>{logoArt}</Text>
          <Text color={theme.dim}>{projectPath}</Text>
        </Box>
      )}
      <TabsBar view={view} width={width} />
      <Hints width={width} />
      {report && <StatusLine report={report} />}
    </Box>
  );
}

export function TabsBar({ view, width }: { view: HeaderView; width: number }) {
  if (width < 60) {
    return (
      <Text>
        {tabs.map((t, i) => (
          <Text
            key={t.v}
            color={t.v === view ? theme.accent : theme.dim}
            bold={t.v === view}
          >
            {t.n[0]}{i < tabs.length - 1 ? " " : ""}
          </Text>
        ))}
      </Text>
    );
  }
  return (
    <Text>
      {tabs.map((t, i) => (
        <Text key={t.v}>
          <Text color={t.v === view ? theme.accent : theme.dim} bold={t.v === view}>
            {t.n}
          </Text>
          {i < tabs.length - 1 && <Text color={theme.dim}> │ </Text>}
        </Text>
      ))}
    </Text>
  );
}

export function Hints({ width }: { width: number }) {
  if (width < 70) {
    return <Text color={theme.dim}>[S]can [O]utd [L]ogs [C]fg [Q]uit</Text>;
  }
  return (
    <Text>
      <Text color={theme.accent}>[S]</Text>can  <Text color={theme.accent}>[O]</Text>utdated {" "}
      <Text color={theme.accent}>[L]</Text>ogs  <Text color={theme.accent}>[C]</Text>onfig{" "}
      <Text color={theme.dim}>←→ views  ↑↓ nav  </Text>
      <Text color={theme.accent}>[Q]</Text>
      <Text color={theme.dim}>uit</Text>
    </Text>
  );
}

function statusCounts(report: Report) {
  let ok = 0, warn = 0, error = 0, skip = 0;
  for (const r of Object.values(report.results)) {
    switch (r.status) {
      case "ok": ok++; break;
      case "warn":
      case "warning": warn++; break;
      case "error": error++; break;
      default: skip++;
    }
  }
  return { ok, warn, error, skip };
}

// Health percentage, colored status dots, outdated total, relative scan age —
// the "64% ● 9 ● 3 ● 0 ● 2 ● 119 outdated" bar.
function StatusLine({ report }: { report: Report }) {
  const { ok, warn, error, skip } = statusCounts(report);
  const total = ok + warn + error + skip;
  if (total === 0) return null;
  const health = Math.floor(((ok + warn + skip) * 100) / total);

  const dot = (n: number, st: string): Span => ({
    text: `● ${n} `,
    color: st === "warn" ? theme.warn : st === "ok" ? theme.ok : st === "error" ? theme.error : theme.skipped,
  });
  const spans: Span[] = [
    accent(String(health), true),
    { text: "%  " },
    dot(ok, "ok"),
    dot(warn, "warn"),
    dot(error, "error"),
    dot(skip, "skipped"),
    { text: "● ", color: theme.warn },
    { text: `${report.outdated.length} outdated  ` },
    dim(`⏱ ${scanAge(report.timestamp)}`),
  ];
  return (
    <Text>
      {spans.map((s, i) => (
        <Text key={i} color={s.color} bold={s.bold}>{s.text}</Text>
      ))}
    </Text>
  );
}

export function Footer({ width }: { width: number }) {
  const version = process.env.ENVEXA_VERSION ?? "dev";
  return (
    <Box width={Math.max(0, width)} justifyContent="flex-end">
      <Text color={theme.dim}>⚡ Envexa {version}  ·  Crafted by Kuruto Denzeru</Text>
    </Box>
  );
}
