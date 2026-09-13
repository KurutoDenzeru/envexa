// Bun tests for the pure lib logic — the smallest checks that fail if the
// bridge/config/filter/update mapping regress (parity with the Go tests they
// replaced).
import { describe, expect, test } from "bun:test";
import { flattenOutdated, mergeResults } from "./scan";
import { parseLogLine, settingsFields } from "./config";
import { displayVersion, filterIndices, pad, sliceWindow, truncate } from "./format";
import { updateArgs } from "./update";
import type { ScanResult } from "./types";

describe("mergeResults", () => {
  test("keeps good results keyed by tool", () => {
    const map = mergeResults(["npm"], [{ tool: "npm", status: "ok", version: "1.2.3" }]);
    expect(map.npm.status).toBe("ok");
  });

  test("errors become scanner-failed results keyed by script name", () => {
    const map = mergeResults(["cargo"], [{ tool: "", status: "", issues: ["scanner failed"] }]);
    expect(map.cargo).toEqual({ tool: "cargo", status: "error", issues: ["scanner failed"] });
  });

  test("skipped results keep the script name when tool is empty", () => {
    const map = mergeResults(["yarn"], [{ tool: "", status: "skipped" }]);
    expect(map.yarn.status).toBe("skipped");
    expect(map.yarn.tool).toBe("");
  });
});

describe("flattenOutdated", () => {
  test("flattens global, formulae, casks, then project arrays per scanner", () => {
    const results = {
      brew: {
        tool: "brew", status: "ok",
        outdated_global: [{ name: "ripgrep", current: "1", latest: "2" }],
        outdated_casks: [{ name: "iterm2", current: "3", latest: "4" }],
      },
      npm: {
        tool: "npm", status: "ok",
        outdated: [{ name: "typescript", current: "5", latest: "6" }],
      },
    } as unknown as Record<string, ScanResult>;
    const flat = flattenOutdated(results);
    expect(flat.map((o) => `${o.source}:${o.name}`)).toEqual([
      "brew:ripgrep", "brew:iterm2", "npm:typescript",
    ]);
  });
});

describe("format helpers", () => {
  test("pad and truncate", () => {
    expect(pad("ab", 4)).toBe("ab  ");
    expect(pad("abcdef", 4)).toBe("abcdef");
    expect(truncate("abcdef", 5)).toBe("abcd…");
    expect(truncate("abcdef", 1)).toBe("…");
  });

  test("displayVersion picks the first present field", () => {
    expect(displayVersion({ tool: "n", status: "ok", node_version: "22", version: "11" } as ScanResult)).toBe("11");
    expect(displayVersion({ tool: "n", status: "ok", bun_version: "1.2" } as ScanResult)).toBe("1.2");
    expect(displayVersion({ tool: "n", status: "ok" } as ScanResult)).toBe("");
  });

  test("filterIndices matches name or source case-insensitively", () => {
    const items = [
      { source: "npm", name: "typescript" },
      { source: "brew", name: "ripgrep" },
    ];
    expect(filterIndices(items, "TYPE")).toEqual([0]);
    expect(filterIndices(items, "brew")).toEqual([1]);
    expect(filterIndices(items, "")).toEqual([0, 1]);
  });

  test("sliceWindow keeps the cursor in view", () => {
    expect(sliceWindow(5, 0, 8)).toEqual([0, 5]);
    expect(sliceWindow(12, 0, 8)).toEqual([0, 8]);
    expect(sliceWindow(12, 9, 8)).toEqual([2, 10]);
    expect(sliceWindow(12, 11, 8)).toEqual([4, 12]);
  });
});

describe("config", () => {
  test("parseLogLine splits level and trailing [source]", () => {
    const e = parseLogLine({ time: new Date(), message: "WARN: low disk [project]" });
    expect(e.level).toBe("WARN");
    expect(e.message).toBe("low disk");
    expect(e.source).toBe("project");
  });

  test("parseLogLine defaults", () => {
    const e = parseLogLine({ time: new Date(), message: "plain message" });
    expect(e.level).toBe("INFO");
    expect(e.source).toBe("system");
  });

  test("settings adjust cycles and clamps like the Go editor", () => {
    // Defaults inline — loadConfig must not be hit (ambient user config).
    const cfg = {
      theme: "default", scan_timeout_secs: 30, daemon_interval_secs: 14400,
      export_format: "markdown", log_retention_days: 7, project_path: null,
    } as unknown as import("./types").UserConfig;
    const themed = settingsFields[0].adjust(cfg, 1);
    expect(themed.theme).toBe("dark");
    const wrapped = settingsFields[0].adjust({ ...cfg, theme: "light" }, 1);
    expect(wrapped.theme).toBe("default");

    const clampedHi = settingsFields[1].adjust({ ...cfg, scan_timeout_secs: 118 }, 1);
    expect(clampedHi.scan_timeout_secs).toBe(120);
    const clampedLo = settingsFields[1].adjust({ ...cfg, scan_timeout_secs: 10 }, -1);
    expect(clampedLo.scan_timeout_secs).toBe(10);

    expect(settingsFields[5].adjustable).toBe(false); // project_path
  });
});

describe("updateArgs", () => {
  test("maps scanner sources to upgrade commands", () => {
    expect(updateArgs("brew", "ripgrep")).toEqual(["brew", ["upgrade", "ripgrep"]]);
    expect(updateArgs("npm", "typescript")).toEqual(["npm", ["install", "-g", "typescript@latest"]]);
    expect(updateArgs("pip", "requests")).toEqual(["pip3", ["install", "--upgrade", "requests"]]);
  });

  test("rejects unsupported sources", () => {
    expect(() => updateArgs("yarn", "left-pad")).toThrow("no update runner for \"yarn\"");
  });
});
