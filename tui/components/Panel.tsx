import { Box, Text } from "ink";
import type { ReactNode } from "react";
import { theme } from "../theme";

// Rounded panel with a title row inside the border — Ink 7 has no border
// titles, so the Go panel() header line becomes the first child.
export function Panel({
  title,
  width,
  children,
}: {
  title: string;
  width: number;
  children: ReactNode;
}) {
  return (
    <Box
      flexDirection="column"
      borderStyle="round"
      borderColor={theme.dim}
      width={width}
    >
      <Text color={theme.dim}>{title}</Text>
      {children}
    </Box>
  );
}
