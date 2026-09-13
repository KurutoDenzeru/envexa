// Scan bridge — the TS counterpart of internal/scanner: run every
// toolchains/*.sh concurrently, timeout each, merge ScanResult JSON, flatten
// the outdated lists for the outdated view.
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { configDir } from "./config";
import type { OutdatedItem, PackageInfo, Report, ScanResult } from "./types";

const defaultTimeout = 30_000;

// Dir mirrors scanner.Dir(): ENVEXA_TOOLCHAINS_DIR, ./toolchains (repo/dev
// layout), then the installed share dir.
export function scannerDir(): string {
  const env = process.env.ENVEXA_TOOLCHAINS_DIR;
  if (env) return env;
  if (existsSync(path.join("toolchains", "lib", "scan.sh"))) return "toolchains";
  return path.join(configDir(), "toolchains");
}

// mergeResults applies the same error policy as the Go bridge: a scanner that
// fails, times out, or emits bad JSON becomes an "error" result — a missing
// CLI tool must never crash the report. tool:"" is only legitimate for
// skipped results. keyed in script (glob) order.
export function mergeResults(names: string[], results: ScanResult[]): Record<string, ScanResult> {
  const map: Record<string, ScanResult> = {};
  for (let i = 0; i < names.length; i++) {
    let r = results[i];
    if (!r || (r.tool === "" && r.status !== "skipped")) {
      r = { tool: names[i], status: "error", issues: ["scanner failed"] };
    }
    map[r.tool !== "" ? r.tool : names[i]] = r;
  }
  return map;
}

// flattenOutdated mirrors the Go bridge: outdated_global, formulae, casks,
// then the plain per-project array, per scanner in map key order.
export function flattenOutdated(results: Record<string, ScanResult>): OutdatedItem[] {
  const out: OutdatedItem[] = [];
  const push = (source: string, pkgs: PackageInfo[] | undefined) => {
    for (const p of pkgs ?? []) {
      out.push({ source, name: p.name, current: p.current, latest: p.latest, size: "" });
    }
  };
  for (const [name, r] of Object.entries(results)) {
    push(name, r.outdated_global);
    push(name, r.outdated_formulae);
    push(name, r.outdated_casks);
    push(name, r.outdated);
  }
  return out;
}

// runScanners shells out to every *.sh directly under the toolchains dir
// (lib/ excluded), concurrently, each with a 30s timeout.
export async function runScanners(dir = scannerDir()): Promise<Report> {
  let scripts: string[] = [];
  try {
    scripts = readdirSync(dir).filter((f) => f.endsWith(".sh")).sort();
  } catch {
    scripts = [];
  }

  const names = scripts.map((s) => s.replace(/\.sh$/, ""));
  const results = await Promise.all(scripts.map((s) => runOne(path.join(dir, s))));

  const merged = mergeResults(names, results);
  return {
    timestamp: new Date().toISOString().replace(/\.\d+Z$/, "Z"),
    results: merged,
    outdated: flattenOutdated(merged),
  };
}

async function runOne(script: string): Promise<ScanResult> {
  try {
    const proc = Bun.spawn(["bash", script], {
      stdout: "pipe",
      stderr: "ignore",
      signal: AbortSignal.timeout(defaultTimeout),
    });
    const out = await new Response(proc.stdout).text();
    const code = await proc.exited;
    if (code !== 0) throw new Error(`exit ${code}`);
    return JSON.parse(out) as ScanResult;
  } catch {
    // Merged into an error result by mergeResults.
    return { tool: "", status: "", issues: ["scanner failed"] };
  }
}
