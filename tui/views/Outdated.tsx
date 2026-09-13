import { Box, Text } from "ink";
import { Spans } from "../components/Spans";
import { dim, pad, truncate, type Span } from "../lib/format";
import type { OutdatedItem, Report } from "../lib/types";
import { theme } from "../theme";

// Outdated packages table with the /-filter search line — the Go
// viewOutdated() counterpart. rows/cursor are filter-aware (App owns the
// filter state); the list windows around the cursor on long lists.
export function Outdated({
  report,
  rows,
  cursor,
  searchActive,
  query,
  width,
  height,
}: {
  report: Report;
  rows: number[]; // indices into report.outdated, in display order
  cursor: number; // position within rows
  searchActive: boolean;
  query: string;
  width: number;
  height: number;
}) {
  // Fill the page like the logs view: subtract the header block (logo+path on
  // wide terminals), the hints line, the column header, the count line, and
  // the search line when active.
  const headerLines = width >= 100 && height >= 22 ? 12 : 6;
  const maxRows = Math.max(4, height - headerLines - 3 - (searchActive ? 1 : 0));
  const start = rows.length <= maxRows
    ? 0
    : Math.min(Math.max(0, cursor - (maxRows - 1)), rows.length - maxRows);

  const header: Span[] = [
    dim(pad("Source", 12)),
    dim(pad("Package", 20)),
    dim(pad("Current", 12)),
    dim(pad("Latest", 12)),
    dim("Size"),
  ];

  return (
    <Box flexDirection="column">
      {searchActive && (
        <Text>
          <Text color={theme.accent}>/ {query}</Text>
          <Text color={theme.dim}>▏  {rows.length}/{report.outdated.length} matches — esc clears</Text>
        </Text>
      )}
      <Box flexDirection="column">
        <Spans spans={header} />
        {rows.slice(start, start + maxRows).map((idx, i) => {
          const o = report.outdated[idx];
          const spans: Span[] = [
            { text: pad(o.source, 12) },
            { text: pad(truncate(o.name, 20), 20) },
            { text: pad(o.current, 12) },
            { text: pad(o.latest, 12) },
            { text: o.size },
          ];
          return <Spans key={idx} spans={spans} selected={start + i === cursor} />;
        })}
      </Box>
      {rows.length === 0 && (
        <Text color={theme.dim}>no report yet — press s to scan</Text>
      )}
      {rows.length > 0 && <Text>{report.outdated.length} outdated packages</Text>}
    </Box>
  );
}
