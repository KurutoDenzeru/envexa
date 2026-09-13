import { Text } from "ink";
import type { Span } from "../lib/format";
import { theme } from "../theme";

// Renders styled spans on one line; with selected=true the line re-styles as
// the highlighted row (background cyan, foreground black) — the bubbles/table
// selection look.
export function Spans({ spans, selected = false }: { spans: Span[]; selected?: boolean }) {
  if (selected) {
    return (
      <Text backgroundColor={theme.selectedBg} color={theme.selectedFg}>
        {spans.map((s) => s.text).join("")}
      </Text>
    );
  }
  return (
    <Text>
      {spans.map((s, i) => (
        <Text key={i} color={s.color} bold={s.bold}>
          {s.text}
        </Text>
      ))}
    </Text>
  );
}
