import { Box, Text } from "ink";
import type { ReactNode } from "react";
import { theme } from "../theme";

// Rounded panel with a title row inside the border — Ink 7 has no border
// titles, so the Go panel() header line becomes the first child. flexGrow
// lets dashboard panels stretch to fill the terminal height.
export function Panel({
  title,
  width,
  flexGrow,
  children,
}: {
  title: string;
  width: number;
  flexGrow?: number;
  children: ReactNode;
}) {
  return (
    <Box
      flexDirection="column"
      borderStyle="round"
      borderColor={theme.dim}
      width={width}
      flexGrow={flexGrow}
      flexShrink={flexGrow ? 1 : 0}
    >
      <Text color={theme.dim}>{title}</Text>
      {children}
    </Box>
  );
}
