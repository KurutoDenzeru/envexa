import { Box, Text } from "ink";
import { theme } from "../theme";

// Readiness bar with an optional centered label overlay — the counterpart of
// bubbles/progress with the readiness/risk caption rendered on the bar, like
// the Rust original.
export function Gauge({
  value,
  width,
  label,
}: {
  value: number;
  width: number;
  label?: string;
}) {
  const filled = Math.round(Math.min(1, Math.max(0, value)) * width);
  return (
    <Box width={width} height={1} flexShrink={0}>
      <Text>
        <Text color={theme.ok}>{"█".repeat(filled)}</Text>
        <Text color={theme.dim}>{"█".repeat(width - filled)}</Text>
      </Text>
      {label && (
        <Box position="absolute" top={0} left={0} width={width} justifyContent="center">
          <Text color="black">{label}</Text>
        </Box>
      )}
    </Box>
  );
}
