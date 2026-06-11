import { createFileRoute } from "@tanstack/react-router"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Terminal, ScrollText, Filter, Download } from "lucide-react"
import { Button } from "@/components/ui/button"

export const Route = createFileRoute("/logs")({ component: LogsPage })

const mockLogs = [
  { time: "10:15:32", level: "INFO", message: "Starting Envexa scanner engine...", source: "system" },
  { time: "10:15:33", level: "INFO", message: "Detected Node.js project. Scanning package.json...", source: "node" },
  { time: "10:15:34", level: "WARN", message: "Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21)", source: "node" },
  { time: "10:15:35", level: "INFO", message: "Detected Rust project. Scanning Cargo.toml...", source: "rust" },
  { time: "10:15:38", level: "ERROR", message: "Security vulnerability found in 'regex' crate: CVE-2022-24713", source: "rust" },
  { time: "10:15:39", level: "INFO", message: "Detected Python project. Scanning requirements.txt...", source: "python" },
  { time: "10:15:40", level: "INFO", message: "Scan completed successfully. Generated report.", source: "system" },
]

function LogsPage() {
  return (
    <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-blue-400 to-indigo-500 bg-clip-text text-transparent flex items-center gap-3">
            <ScrollText className="w-8 h-8 text-blue-500" />
            System Logs
          </h1>
          <p className="text-muted-foreground mt-2">
            Real-time event logs and diagnostic output from the scanning engine.
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2 bg-popover text-foreground border-border hover:bg-muted/50">
            <Filter className="w-4 h-4" /> Filter
          </Button>
          <Button variant="outline" className="gap-2 bg-popover text-foreground border-border hover:bg-muted/50">
            <Download className="w-4 h-4" /> Export
          </Button>
        </div>
      </div>

      <Card className="bg-card/50 border-border backdrop-blur-xl">
        <CardHeader className="flex flex-row items-center gap-2 border-b border-border/50 pb-4">
          <Terminal className="w-5 h-5 text-muted-foreground/80" />
          <CardTitle className="text-lg">Scanner Output</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="bg-[#0D0D0D] font-mono text-sm p-4 h-[600px] overflow-y-auto rounded-b-lg">
            {mockLogs.map((log, i) => (
              <div key={i} className="flex gap-4 py-1 hover:bg-white/5 px-2 rounded">
                <span className="text-muted-foreground/60 w-20 shrink-0">{log.time}</span>
                <span className={`w-16 shrink-0 font-semibold ${
                  log.level === 'INFO' ? 'text-blue-400' : 
                  log.level === 'WARN' ? 'text-yellow-400' : 
                  'text-red-400'
                }`}>
                  [{log.level}]
                </span>
                <span className="text-purple-400 w-16 shrink-0">[{log.source}]</span>
                <span className="text-neutral-300 whitespace-pre-wrap break-all">{log.message}</span>
              </div>
            ))}
            <div className="flex gap-4 py-1 px-2">
              <span className="text-muted-foreground/60 w-20 shrink-0">...</span>
              <span className="text-green-400 font-semibold animate-pulse">Waiting for new events...</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
