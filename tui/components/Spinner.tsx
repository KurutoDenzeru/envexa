import { useEffect, useState } from "react";
import { Text } from "ink";

const frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// Minimal braille spinner — replaces bubbles/spinner (Meter) for the scan and
// update activity indicators.
export function Spinner() {
  const [i, setI] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setI((i) => (i + 1) % frames.length), 80);
    return () => clearInterval(t);
  }, []);
  return <Text>{frames[i]}</Text>;
}
