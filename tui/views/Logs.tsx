import { LogStream } from "../components/LogStream";
import type { LogPair } from "../lib/config";

// Scan history from the envexa data dir — newest last, live tail by default,
// ↑↓ scrolls (offset counts lines up from the bottom). height arrives
// pre-sized to the space under the header.
export function Logs({ logs, height, offset }: { logs: LogPair[]; height: number; offset: number }) {
  return <LogStream logs={logs} height={Math.max(4, height)} offset={offset} />;
}
