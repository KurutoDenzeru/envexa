import { createFileRoute } from "@tanstack/react-router"
import {
  Card,
  CardContent,
} from "@/components/ui/card"
import { Terminal, ScrollText, Filter, Download, Search, Check, Circle, Trash2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useState, useMemo } from "react"

export const Route = createFileRoute("/logs")({ component: LogsPage })

const mockLogs = [
  { time: "10:15:32", level: "INFO", message: "Starting Envexa scanner engine...", source: "system" },
  { time: "10:15:33", level: "INFO", message: "Detected Node.js project. Scanning package.json...", source: "node" },
  { time: "10:15:34", level: "WARN", message: "Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21)", source: "node" },
  { time: "10:15:35", level: "INFO", message: "Detected Rust project. Scanning Cargo.toml...", source: "rust" },
  { time: "10:15:38", level: "ERROR", message: "Security vulnerability found in 'regex' crate: CVE-2022-24713", source: "rust" },
  { time: "10:15:39", level: "INFO", message: "Detected Python project. Scanning requirements.txt...", source: "python" },
  { time: "10:15:40", level: "INFO", message: "Scan completed successfully. Generated report.", source: "system" },
  { time: "10:15:42", level: "DEBUG", message: "Cleaning up temporary files...", source: "system" },
  { time: "10:16:01", level: "INFO", message: "File change detected in src/main.rs. Re-running scanner...", source: "watcher" },
]

function LogsPage() {
  const [filterLevel, setFilterLevel] = useState<string>("ALL")
  const [search, setSearch] = useState("")

  const filteredLogs = useMemo(() => {
    return mockLogs.filter(log => {
      const matchesLevel = filterLevel === "ALL" || log.level === filterLevel
      const matchesSearch = log.message.toLowerCase().includes(search.toLowerCase()) || 
                            log.source.toLowerCase().includes(search.toLowerCase())
      return matchesLevel && matchesSearch
    })
  }, [filterLevel, search])
  return (
    <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <ScrollText className="w-8 h-8 text-foreground" />
            System Logs
          </h1>
          <p className="text-muted-foreground mt-2">
            Real-time event logs and diagnostic output from the scanning engine.
          </p>
        </div>
        <div className="flex gap-3 w-full md:w-auto">
          <div className="relative w-full md:w-64">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground/60" />
            <Input 
              type="text" 
              placeholder="Search logs..." 
              className="pl-9 bg-background/50 border-border focus-visible:ring-blue-500" 
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger className="inline-flex h-9 px-4 py-2 items-center justify-between rounded-md border border-border bg-popover text-sm font-medium text-foreground transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 w-32 gap-2">
              <Filter className="w-4 h-4 text-muted-foreground" /> 
              {filterLevel === "ALL" ? "All Levels" : filterLevel}
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-40 bg-popover border-border">
              {["ALL", "INFO", "WARN", "ERROR", "DEBUG"].map((level) => (
                <DropdownMenuItem key={level} onClick={() => setFilterLevel(level)} className="justify-between cursor-pointer focus:bg-muted/50">
                  {level === "ALL" ? "All Levels" : level}
                  {filterLevel === level && <Check className="w-4 h-4 text-blue-500" />}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Button variant="outline" size="icon" className="bg-popover text-foreground border-border hover:bg-muted/50 hover:text-red-400" title="Clear Logs">
            <Trash2 className="w-4 h-4" />
          </Button>
          <Button variant="outline" size="icon" className="bg-popover text-foreground border-border hover:bg-muted/50" title="Export Logs">
            <Download className="w-4 h-4" />
          </Button>
        </div>
      </div>

      <Card className="bg-card border-border shadow-xs overflow-hidden backdrop-blur-xl">
        {/* Sleek Terminal Header */}
        <div className="flex items-center px-4 py-3 bg-muted/30 border-b border-border">
          <div className="flex items-center gap-2 text-muted-foreground">
            <Terminal className="w-4 h-4" />
            <span className="text-xs font-mono font-medium tracking-wider uppercase">System Console</span>
          </div>
        </div>
        <CardContent className="p-0">
          <div className="font-mono text-[13px] leading-relaxed p-4 h-[600px] overflow-y-auto">
            {filteredLogs.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
                <Filter className="w-10 h-10 mb-4 opacity-20" />
                <p>No logs match the current filters.</p>
              </div>
            ) : (
              <>
                {filteredLogs.map((log, i) => (
                  <div key={i} className="flex gap-4 py-1.5 hover:bg-muted/50 px-2 rounded-md transition-colors group">
                    <span className="text-muted-foreground w-20 shrink-0 select-none group-hover:text-foreground transition-colors">{log.time}</span>
                    <span className={`w-14 shrink-0 font-bold select-none ${
                      log.level === 'INFO' ? 'text-blue-500' : 
                      log.level === 'WARN' ? 'text-yellow-500' : 
                      log.level === 'DEBUG' ? 'text-purple-500' :
                      'text-red-500'
                    }`}>
                      {log.level.padEnd(5)}
                    </span>
                    <span className="text-muted-foreground w-20 shrink-0 select-none hidden sm:block truncate">[{log.source}]</span>
                    <span className={`whitespace-pre-wrap break-words ${
                      log.level === 'ERROR' ? 'text-red-500' :
                      log.level === 'WARN' ? 'text-yellow-600' :
                      'text-foreground'
                    }`}>
                      {log.message}
                    </span>
                  </div>
                ))}
                <div className="flex gap-4 py-3 px-2 mt-2 border-t border-border">
                  <span className="text-muted-foreground w-20 shrink-0">...</span>
                  <div className="flex items-center gap-2 text-green-500 font-medium">
                    <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
                    Watching for incoming events
                  </div>
                </div>
              </>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
