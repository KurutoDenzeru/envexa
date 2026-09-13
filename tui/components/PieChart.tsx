import { Box, Text } from "ink";
import { statusColor } from "../theme";

// Status distribution as a colored dot pie — the TS counterpart of the Go
// dot-pie (view.go pieChart): 9x9 cell grid, 0.5 aspect correction, slices in
// ok → warn → error → skipped order. Cells render as "● " to match the Go
// output's density and width.
export function PieChart({
  counts,
}: {
  counts: { ok: number; warn: number; error: number; skipped: number };
}) {
  const total = counts.ok + counts.warn + counts.error + counts.skipped;
  if (total === 0) return <Text> </Text>;

  const order = ["ok", "warn", "error", "skipped"] as const;
  const R = 4;
  const rows: { ch: string; color: string }[][] = [];
  for (let y = -R; y <= R; y++) {
    const row: { ch: string; color: string }[] = [];
    for (let x = -R; x <= R; x++) {
      const dx = x * 0.5;
      const dy = y;
      if (dx * dx + dy * dy > R * R + 0.5) {
        row.push({ ch: " ", color: "white" });
        continue;
      }
      const deg = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360;
      let acc = 0;
      let st: (typeof order)[number] = "skipped";
      for (const s of order) {
        acc += (counts[s] / total) * 360;
        if (deg < acc) {
          st = s;
          break;
        }
      }
      row.push({ ch: "●", color: statusColor(st) });
    }
    rows.push(row);
  }

  return (
    <Box flexDirection="column" alignItems="center">
      {rows.map((row, i) => {
        const trimmed = [...row];
        while (trimmed.length > 0 && trimmed[trimmed.length - 1].ch === " ") trimmed.pop();
        return (
          <Text key={i}>
            {trimmed.map((cell, j) => (
              <Text key={j}>
                <Text color={cell.color}>{cell.ch}</Text>
                {j < trimmed.length - 1 ? " " : ""}
              </Text>
            ))}
          </Text>
        );
      })}
    </Box>
  );
}
