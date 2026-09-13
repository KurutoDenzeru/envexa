// On-disk state: user config, settings editor fields, and scan logs — the TS
// counterpart of internal/config. Every settings change persists to
// config.json immediately (same contract as PUT /api/config).
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import type { UserConfig } from "./types";

// Dir mirrors config.Dir(): XDG_DATA_HOME, ~/.local/share/envexa, ~/.envexa.
export function configDir(): string {
  const envData = process.env.ENVEXA_DATA_DIR;
  if (envData) return envData;
  const xdg = process.env.XDG_DATA_HOME;
  if (xdg) return path.join(xdg, "envexa");
  const home = process.env.HOME ?? ".";
  return path.join(home, ".local", "share", "envexa");
}

const DEFAULT: UserConfig = {
  cache_ttl_minutes: 15,
  project_path: null,
  recent_project_paths: [],
  favorite_project_paths: [],
  auto_scan_on_startup: false,
  theme: "default",
  verbose_logs: false,
  scan_timeout_secs: 30,
  daemon_interval_secs: 14400,
  export_format: "markdown",
  enabled_scanners: null,
  log_retention_days: 7,
};

// loadConfig mirrors load_config: any parse failure falls back to defaults.
export function loadConfig(): UserConfig {
  try {
    const raw = readFileSync(path.join(configDir(), "config.json"), "utf8");
    return { ...DEFAULT, ...(JSON.parse(raw) as Partial<UserConfig>) };
  } catch {
    return { ...DEFAULT };
  }
}

export function saveConfig(cfg: UserConfig): boolean {
  try {
    mkdirSync(configDir(), { recursive: true });
    writeFileSync(path.join(configDir(), "config.json"), JSON.stringify(cfg, null, 2));
    return true;
  } catch {
    return false;
  }
}

// Logs — logs.json is [[timestamp, msg]] with RFC3339 timestamps.
export interface LogPair {
  time: Date;
  message: string;
}

export interface LogEntry {
  level: string;
  message: string;
  source: string;
}

export function readLogs(): LogPair[] {
  try {
    const raw = readFileSync(path.join(dataDir(), "logs.json"), "utf8");
    const tuples = JSON.parse(raw) as [string, string][];
    const logs: LogPair[] = [];
    for (const [ts, msg] of tuples) {
      const time = new Date(ts);
      if (isNaN(time.getTime())) continue;
      logs.push({ time, message: msg });
    }
    return logs;
  } catch {
    return [];
  }
}

function dataDir(): string {
  return configDir();
}

// parseLogLine splits a raw "LEVEL: message [source]" entry — the exact
// semantics of parse_log_line in server.rs.
export function parseLogLine(pair: LogPair): LogEntry {
  let level = "INFO";
  let source = "system";
  let message = pair.message;

  for (const p of ["INFO: ", "WARN: ", "ERROR: ", "DEBUG: "]) {
    if (message.startsWith(p)) {
      level = p.slice(0, -2);
      message = message.slice(p.length);
      break;
    }
  }
  const start = message.lastIndexOf("[");
  const end = message.lastIndexOf("]");
  if (start >= 0 && start < end) {
    source = message.slice(start + 1, end);
    message = message.slice(0, start).trim();
  }
  return { level, message, source };
}

// Settings editor fields mirror the web dashboard settings page. adjust
// returns a new config (immutable) so the caller persists it.
export interface SettingsField {
  name: string;
  adjustable: boolean;
  value: (cfg: UserConfig) => string;
  adjust: (cfg: UserConfig, dir: number) => UserConfig;
}

function cycleStr(cur: string, opts: string[], dir: number): string {
  const idx = opts.indexOf(cur);
  return opts[(((idx + dir) % opts.length) + opts.length) % opts.length];
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}

export const settingsFields: SettingsField[] = [
  {
    name: "theme",
    adjustable: true,
    value: (c) => c.theme,
    adjust: (c, d) => ({ ...c, theme: cycleStr(c.theme, ["default", "dark", "light"], d) }),
  },
  {
    name: "scan_timeout_secs",
    adjustable: true,
    value: (c) => `${c.scan_timeout_secs}s`,
    adjust: (c, d) => ({ ...c, scan_timeout_secs: clamp(c.scan_timeout_secs + d * 5, 10, 120) }),
  },
  {
    name: "daemon_interval_secs",
    adjustable: true,
    value: (c) => `${c.daemon_interval_secs}s`,
    adjust: (c, d) =>
      ({ ...c, daemon_interval_secs: clamp(c.daemon_interval_secs + d * 300, 300, 86400) }),
  },
  {
    name: "export_format",
    adjustable: true,
    value: (c) => c.export_format,
    adjust: (c, d) =>
      ({ ...c, export_format: cycleStr(c.export_format, ["markdown", "json", "sarif"], d) }),
  },
  {
    name: "log_retention_days",
    adjustable: true,
    value: (c) => `${c.log_retention_days}`,
    adjust: (c, d) => ({ ...c, log_retention_days: clamp(c.log_retention_days + d, 0, 90) }),
  },
  {
    name: "project_path",
    adjustable: false,
    value: (c) => c.project_path ?? "(current directory)",
    adjust: (c) => c,
  },
];
