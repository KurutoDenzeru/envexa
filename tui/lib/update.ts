// Update runner — the TS counterpart of internal/tui/update_runner.go:
// map a scanner source to its per-package upgrade command, run it, and on
// failure keep only the output tail for the detail-view message.
export function updateArgs(source: string, name: string): [string, string[]] {
  switch (source) {
    case "brew":
      return ["brew", ["upgrade", name]];
    case "npm":
      return ["npm", ["install", "-g", `${name}@latest`]];
    case "gem":
      return ["gem", ["update", name]];
    case "pip":
    case "pip3":
      return ["pip3", ["install", "--upgrade", name]];
    case "cargo":
      return ["cargo", ["install", name]];
    default:
      throw new Error(`no update runner for "${source}"`);
  }
}

// runUpdate resolves to an error message on failure, null on success.
export async function runUpdate(source: string, name: string): Promise<string | null> {
  let cmd: string;
  let args: string[];
  try {
    [cmd, args] = updateArgs(source, name);
  } catch (e) {
    return (e as Error).message;
  }
  try {
    const proc = Bun.spawn([cmd, ...args], { stdout: "pipe", stderr: "pipe" });
    const [stdout, stderr, code] = await Promise.all([
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
      proc.exited,
    ]);
    const out = stdout + stderr;
    if (code !== 0) {
      const lines = out.trim().split("\n").slice(-3);
      const msg = lines.join(" | ") || `exit code ${code}`;
      return `${name} update failed: ${msg}`;
    }
    return null;
  } catch (e) {
    return `${name} update failed: ${(e as Error).message}`;
  }
}
