import { Text } from "ink";
import { theme } from "../theme";

// Readiness bar — the counterpart of bubbles/progress with the default
// gradient, reduced to a flat filled/empty bar over fixed width.
export function Gauge({ value, width }: { value: number; width: number }) {
  const filled = Math.round(Math.min(1, Math.max(0, value)) * width);
  return (
    <Text>
      <Text color={theme.ok}>{"█".repeat(filled)}</Text>
      <Text color={theme.dim}>{"█".repeat(width - filled)}</Text>
    </Text>
  );
}
