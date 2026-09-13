// Color convention mirrors src/tui/theme.rs: ok=green, warning=yellow,
// error=red, skipped=darkgray.
export const theme = {
  ok: "green",
  warn: "yellow",
  error: "red",
  skipped: "gray",
  accent: "#5fffd7",
  dim: "gray",
  selectedBg: "cyan",
  selectedFg: "black",
} as const;

export function statusColor(status: string): string {
  switch (status) {
    case "ok":
      return theme.ok;
    case "warn":
    case "warning":
      return theme.warn;
    case "error":
      return theme.error;
    default:
      return theme.skipped;
  }
}

export function statusLabel(status: string): string {
  switch (status) {
    case "ok":
      return "pass";
    case "warn":
    case "warning":
      return "warn";
    case "error":
      return "error";
    default:
      return "skip";
  }
}
