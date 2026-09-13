import { Box, Text } from "ink";
import { pad } from "../lib/format";
import type { Detail } from "../lib/types";
import { theme } from "../theme";

// Selected outdated package; `y` confirms the update, Esc returns to the list.
export function PackageDetail({ detail, updateMsg }: { detail: Detail; updateMsg: string }) {
  const rows: [string, string][] = [
    ["package", detail.name],
    ["source", detail.source],
    ["current", detail.current],
    ["latest", detail.latest],
  ];
  return (
    <Box flexDirection="column">
      <Text bold color={theme.accent}>Package detail</Text>
      <Text> </Text>
      {rows.map(([k, v]) => (
        <Text key={k}>
          <Text color={theme.ok}>{pad(k, 10)}</Text>
          {v}
        </Text>
      ))}
      {updateMsg && <Text> </Text>}
      {updateMsg && <Text color={theme.error}>{updateMsg}</Text>}
      <Text> </Text>
      <Text color={theme.dim}>y update · esc back</Text>
    </Box>
  );
}
