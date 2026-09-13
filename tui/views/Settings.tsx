import { Box, Text } from "ink";
import { Spans } from "../components/Spans";
import { settingsFields } from "../lib/config";
import { pad } from "../lib/format";
import type { UserConfig } from "../lib/types";
import { theme } from "../theme";

// Editable settings — every field mirrors the web dashboard settings page;
// ←→ / enter change values, changes persist to config.json instantly.
export function Settings({ cfg, sel }: { cfg: UserConfig; sel: number }) {
  const recents = cfg.recent_project_paths;
  return (
    <Box flexDirection="column">
      {settingsFields.map((f, i) => {
        let val = f.value(cfg);
        if (f.adjustable) val = `‹ ${val} ›`;
        return (
          <Spans
            key={f.name}
            selected={i === sel}
            spans={[
              { text: pad(f.name, 22), color: theme.ok },
              { text: val },
            ]}
          />
        );
      })}

      {recents.length > 0 && (
        <>
          <Text> </Text>
          <Text bold color={theme.accent}>recent projects — enter to switch</Text>
          {recents.map((p, j) => (
            <Spans
              key={p}
              selected={settingsFields.length + j === sel}
              spans={[{ text: `  ${p}` }]}
            />
          ))}
        </>
      )}

      <Text> </Text>
      <Text color={theme.dim}>↑↓ select · ←→ / enter change · saved to config.json instantly</Text>
    </Box>
  );
}
