import { Box, Text } from "ink";
import { Spinner } from "../components/Spinner";
import { theme } from "../theme";

// Indeterminate updating view — input blocked like the Rust Scanning/Updating
// guard (only q quits).
export function Updating({ msg }: { msg: string }) {
  return (
    <Box flexDirection="column">
      <Text bold color={theme.accent}>Updating</Text>
      <Text> </Text>
      <Text>
        <Spinner /> {msg}
      </Text>
      <Text> </Text>
      <Text color={theme.dim}>q quit</Text>
    </Box>
  );
}
