// envexa-tui — the Ink (React for terminals) replacement for the Bubbletea
// port. Key map parity with src/tui/app.rs and internal/tui: s=scan,
// o=outdated, l=logs, c=settings, h/Esc=home, q=quit, ←→ switch views,
// ↑↓ navigate, /=filter, Enter=detail, y=update.
import { Box, Text, render, useApp, useInput, useStdin, useWindowSize } from "ink";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { Hints, Header } from "./components/Header";
import { Spinner } from "./components/Spinner";
import { loadConfig, readLogs, saveConfig, settingsFields, type LogPair } from "./lib/config";
import { filterIndices } from "./lib/format";
import { runScanners } from "./lib/scan";
import type { Detail, Report, UserConfig } from "./lib/types";
import { runUpdate, updateArgs } from "./lib/update";
import { theme } from "./theme";
import { Dashboard, type DashTab } from "./views/Dashboard";
import { Logs } from "./views/Logs";
import { Outdated } from "./views/Outdated";
import { PackageDetail } from "./views/PackageDetail";
import { Settings } from "./views/Settings";
import { Updating } from "./views/Updating";

type View = "dashboard" | "outdated" | "packageDetail" | "updating" | "logs" | "settings";
const viewOrder: View[] = ["dashboard", "outdated", "logs", "settings"];

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}

function dashRowsFor(report: Report | null): number {
  if (!report) return 0;
  const groups = [
    ["brew", "cargo", "docker", "pip", "gem"],
    ["npm", "pnpm", "yarn", "bun", "deno"],
    ["project", "security", "audit", "ci"],
  ];
  return groups.reduce((n, g) => n + g.filter((t) => report.results[t]).length, 0);
}

function App() {
  const { exit } = useApp();
  const { columns: width, rows: height } = useWindowSize();
  const [view, setView] = useState<View>("dashboard");
  const [dashTab, setDashTab] = useState<DashTab>(0);
  const [report, setReport] = useState<Report | null>(null);
  const [scanning, setScanning] = useState(true);
  const [detail, setDetail] = useState<Detail | null>(null);
  const [updating, setUpdating] = useState(false);
  const [updateMsg, setUpdateMsg] = useState("");
  const [dashSel, setDashSel] = useState(0);
  const [outSel, setOutSel] = useState(0);
  const [fpos, setFpos] = useState(0);
  const [query, setQuery] = useState("");
  const [searchActive, setSearchActive] = useState(false);
  const [setSel, setSetSel] = useState(0);
  const [editCfg, setEditCfg] = useState<UserConfig | null>(null);
  const [logs, setLogs] = useState<LogPair[]>([]);
  const [logsOffset, setLogsOffset] = useState(0);
  const [projectPath, setProjectPath] = useState("—");
  const scanId = useRef(0);

  const refreshProjectPath = useCallback(() => {
    const cfg = loadConfig();
    setProjectPath(cfg.project_path || process.cwd());
  }, []);
  useEffect(() => {
    refreshProjectPath();
  }, [refreshProjectPath]);

  // Piped stdin (or a closed pty) must quit the app instead of leaking a
  // live process — bubbletea handled EOF, Ink needs this explicitly.
  const { stdin } = useStdin();
  useEffect(() => {
    const onClose = () => exit();
    stdin.on("close", onClose);
    return () => {
      stdin.off("close", onClose);
    };
  }, [stdin, exit]);

  const doScan = useCallback(async () => {
    const id = ++scanId.current;
    setScanning(true);
    const rep = await runScanners();
    if (id !== scanId.current) return;
    setReport(rep);
    setScanning(false);
  }, []);
  useEffect(() => {
    void doScan();
  }, [doScan]);

  const startUpdate = async (d: Detail) => {
    try {
      updateArgs(d.source, d.name);
    } catch (e) {
      setUpdateMsg((e as Error).message);
      return;
    }
    setUpdating(true);
    setView("updating");
    setUpdateMsg(
      `Updating ${d.name} (${d.current} → ${d.latest}) via ${d.source}…`,
    );
    const err = await runUpdate(d.source, d.name);
    setUpdating(false);
    if (err) {
      setUpdateMsg(err);
      setView("packageDetail");
      return;
    }
    setUpdateMsg("");
    setView("outdated");
    void doScan();
  };

  const openLogs = useCallback(() => {
    setLogs(readLogs());
    setLogsOffset(0);
    setView("logs");
  }, []);

  const openSettings = useCallback(() => {
    setEditCfg(loadConfig());
    setSetSel(0);
    setView("settings");
  }, []);

  const openDetail = () => {
    if (!report) return;
    const rows = searchActive ? filterIndices(report.outdated, query) : report.outdated.map((_, i) => i);
    const cursor = searchActive ? fpos : outSel;
    if (rows.length === 0 || cursor >= rows.length) return;
    const o = report.outdated[rows[cursor]];
    setDetail({ source: o.source, name: o.name, current: o.current, latest: o.latest });
    setView("packageDetail");
  };

  const adjustSettings = (dir: number): boolean => {
    if (!editCfg || setSel >= settingsFields.length) return false;
    const f = settingsFields[setSel];
    if (f.adjustable) {
      const next = f.adjust(editCfg, dir);
      setEditCfg(next);
      saveConfig(next);
      refreshProjectPath();
    }
    return true;
  };

  const selectRecent = (): boolean => {
    if (!editCfg) return false;
    const idx = setSel - settingsFields.length;
    const recents = editCfg.recent_project_paths;
    if (idx < 0 || idx >= recents.length) return false;
    const next = { ...editCfg, project_path: recents[idx] };
    setEditCfg(next);
    const ok = saveConfig(next);
    refreshProjectPath();
    return ok;
  };

  useInput((input, key) => {
    // Updating blocks input like the Rust Scanning/Updating guard.
    if (view === "updating") {
      if (input === "q") exit();
      return;
    }
    if (searchActive && view === "outdated") {
      if (key.escape) {
        if (query === "") setSearchActive(false);
        else setQuery("");
        setFpos(0);
        return;
      }
      if (key.return) {
        openDetail();
        return;
      }
      if (key.backspace || key.delete) {
        setQuery((q) => q.slice(0, -1));
        setFpos(0);
        return;
      }
      if (key.ctrl || key.meta || key.tab || key.upArrow || key.downArrow || key.leftArrow || key.rightArrow) return;
      if (input) {
        setQuery((q) => q + input);
        setFpos(0);
      }
      return;
    }

    // Global quick nav — works from every view, settings included.
    if (input === "q") {
      exit();
      return;
    }
    switch (input) {
      case "s":
        void doScan();
        return;
      case "o":
        setView("outdated");
        return;
      case "l":
        openLogs();
        return;
      case "c":
        openSettings();
        return;
      case "h":
        setView("dashboard");
        return;
      case "/":
        if (view === "outdated") {
          setSearchActive(true);
          setQuery("");
        }
        return;
      case "y":
        if (view === "packageDetail" && detail && !updating) void startUpdate(detail);
        return;
    }

    if (view === "settings") {
      const recents = editCfg?.recent_project_paths ?? [];
      if (key.upArrow) setSetSel((s) => Math.max(0, s - 1));
      else if (key.downArrow) {
        setSetSel((s) => Math.min(settingsFields.length + recents.length - 1, s + 1));
      } else if (key.leftArrow) adjustSettings(-1);
      else if (key.rightArrow) adjustSettings(1);
      else if (key.return) {
        if (!adjustSettings(1)) selectRecent();
      } else if (key.tab) {
        // Tab cycles views from settings too — arrows stay field edits.
        const next = viewOrder[(viewOrder.indexOf("settings") + 1) % viewOrder.length];
        if (next === "logs") openLogs();
        else if (next === "settings") openSettings();
        else setView(next);
      }
      return;
    }

    if (key.escape) {
      setView(view === "packageDetail" ? "outdated" : "dashboard");
      return;
    }
    if (key.tab && view === "dashboard") {
      setDashTab((t) => ((t + 1) % 3) as DashTab);
      return;
    }
    if (key.leftArrow || key.rightArrow) {
      let idx = viewOrder.indexOf(view);
      if (idx < 0) idx = 0;
      idx = key.rightArrow ? (idx + 1) % viewOrder.length : (idx + viewOrder.length - 1) % viewOrder.length;
      const next = viewOrder[idx];
      setView(next);
      if (next === "logs") openLogs();
      if (next === "settings") openSettings();
      return;
    }
    if (key.upArrow || key.downArrow) {
      const d = key.upArrow ? -1 : 1;
      if (view === "outdated") {
        // Arrows are caret keys for the filter input — ignored mid-search,
        // like the Go textinput branch.
        if (searchActive) return;
        setOutSel((s) => clamp(s + d, 0, Math.max(0, (report?.outdated.length ?? 1) - 1)));
        return;
      }
      if (view === "logs") {
        setLogsOffset((o) => Math.max(0, o + d));
        return;
      }
      if (view === "dashboard") {
        setDashSel((s) => clamp(s + d, 0, Math.max(0, dashRowsFor(report) - 1)));
        return;
      }
    }
    if (key.return && view === "outdated") openDetail();
  });

  if (view === "updating") {
    return <Updating msg={updateMsg} />;
  }
  if (view === "packageDetail") {
    return detail ? <PackageDetail detail={detail} updateMsg={updateMsg} /> : null;
  }
  if (scanning) {
    return (
      <Box flexDirection="column">
        <Text bold color={theme.accent}>
          envexa{width >= 60 ? " — dependency environment scanner" : ""}
        </Text>
        <Text> </Text>
        <Text>
          <Spinner /> Scanning toolchains…
        </Text>
        <Text> </Text>
        <Text color={theme.dim}>s rescan · h home · q quit</Text>
      </Box>
    );
  }

  let body: ReactNode;
  const small = width < 40 || height < 12;
  if (small) {
    body = <Text color={theme.dim}>terminal too small — enlarge window</Text>;
  } else {
    switch (view) {
      case "dashboard":
        body = report
          ? <Dashboard report={report} width={width} height={height} tab={dashTab} sel={dashSel} />
          : <Text color={theme.dim}>no report yet — press s to scan</Text>;
        break;
      case "outdated":
        body = report
          ? (
            <Outdated
              report={report}
              rows={searchActive ? filterIndices(report.outdated, query) : report.outdated.map((_, i) => i)}
              cursor={searchActive ? fpos : outSel}
              searchActive={searchActive}
              query={query}
              width={width}
              height={height}
            />
          )
          : <Text color={theme.dim}>no report yet — press s to scan</Text>;
        break;
      case "logs":
        // Viewport must fit under the header: logo block (6+1 lines) renders
        // only on wide terminals; +2 for the hints line and a spare row.
        body = (
          <Logs
            logs={logs}
            height={height - (width >= 100 && height >= 22 ? 12 : 6)}
            offset={logsOffset}
          />
        );
        break;
      case "settings":
        body = editCfg ? <Settings cfg={editCfg} sel={setSel} /> : null;
        break;
      default:
        body = null;
    }
  }

  return (
    <Box flexDirection="column">
      <Header
        view={view}
        width={width}
        height={height}
        report={report}
        projectPath={projectPath}
      />
      {body}
      <Hints width={width} />
    </Box>
  );
}

if (!process.stdout.isTTY) {
  console.error("envexa-tui requires an interactive terminal");
  process.exit(1);
}

const instance = render(<App />, { alternateScreen: true });
await instance.waitUntilExit();
