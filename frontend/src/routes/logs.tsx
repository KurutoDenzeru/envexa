import { createFileRoute } from "@tanstack/react-router"
import {
  CardContent,
} from "@/components/ui/card"
import { Terminal, ScrollText, Filter, Download, Search, Check, Trash2 } from "lucide-react"
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
            <DropdownMenuTrigger className="inline-flex h-9 px-3 items-center justify-between rounded-md border border-border bg-popover text-sm font-medium text-foreground transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring shadow-xs w-32 gap-2 cursor-pointer select-none">
              <span className="flex items-center gap-2 truncate">
                <Filter className="w-4 h-4 text-muted-foreground shrink-0" /> 
                <span className="truncate">{filterLevel === "ALL" ? "All Levels" : filterLevel}</span>
              </span>
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
          <Button variant="outline" size="icon" className="bg-popover text-foreground border-border hover:bg-muted/50 hover:text-red-400 h-9 w-9 shadow-xs" title="Clear Logs">
            <Trash2 className="w-4 h-4" />
          </Button>
          <Button variant="outline" size="icon" className="bg-popover text-foreground border-border hover:bg-muted/50 h-9 w-9 shadow-xs" title="Export Logs">
            <Download className="w-4 h-4" />
          </Button>
        </div>
      </div>

      <div className="bg-[#0c0c0e] border border-zinc-800 shadow-2xl rounded-xl overflow-hidden font-mono">
        {/* Authentic macOS Terminal Title Bar */}
        <div className="flex items-center justify-between px-4 py-2.5 bg-[#18181b] border-b border-zinc-800/80 select-none">
          {/* macOS window control buttons */}
          <div className="flex items-center gap-2 w-20">
            <span className="w-3 h-3 rounded-full bg-[#ff5f56] border border-[#e0443e] cursor-pointer hover:opacity-85 transition-opacity"></span>
            <span className="w-3 h-3 rounded-full bg-[#ffbd2e] border border-[#dab12d] cursor-pointer hover:opacity-85 transition-opacity"></span>
            <span className="w-3 h-3 rounded-full bg-[#27c93f] border border-[#1aab29] cursor-pointer hover:opacity-85 transition-opacity"></span>
          </div>
          {/* Center Monospace Session Title */}
          <div className="flex items-center gap-1.5 text-xs font-sans font-medium text-zinc-400 tracking-wide">
            <Terminal className="w-3.5 h-3.5 text-zinc-500" />
            <span>envexa-system — logs — 80×24</span>
          </div>
          {/* Right indicator for balance */}
          <div className="w-20 text-right text-[10px] text-zinc-600 font-mono tracking-wider">
            bash
          </div>
        </div>

        <CardContent className="p-0">
          <div className="font-mono text-[13px] leading-relaxed p-4 h-[600px] overflow-y-auto bg-[#0c0c0e] text-zinc-100 scrollbar-thin scrollbar-thumb-zinc-800 scrollbar-track-transparent">
            {filteredLogs.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-zinc-500">
                <Filter className="w-10 h-10 mb-4 opacity-20 text-zinc-400" />
                <p className="font-sans text-sm">No logs match the current filters.</p>
              </div>
            ) : (
              <>
                {filteredLogs.map((log, i) => {
                  const sourceColors: Record<string, string> = {
                    rust: "text-[#ff7b72]",
                    node: "text-[#7ee787]",
                    python: "text-[#79c0ff]",
                    system: "text-[#a5d6ff]",
                    watcher: "text-[#d2a8ff]",
                  }
                  const sourceColor = sourceColors[log.source.toLowerCase()] || "text-zinc-500"
                  return (
                    <div key={i} className="flex gap-4 py-1.5 hover:bg-zinc-900/60 px-2 rounded transition-colors group">
                      <span className="text-zinc-500 w-20 shrink-0 select-none group-hover:text-zinc-400 transition-colors font-mono">{log.time}</span>
                      <span className={`w-14 shrink-0 font-bold select-none font-mono ${
                        log.level === 'INFO' ? 'text-[#57ab5a]' : 
                        log.level === 'WARN' ? 'text-[#e5c07b]' : 
                        log.level === 'DEBUG' ? 'text-[#b392f0]' :
                        'text-[#f85149]'
                      }`}>
                        {log.level.padEnd(5)}
                      </span>
                      <span className={`w-20 shrink-0 select-none hidden sm:block truncate font-mono ${sourceColor}`}>[{log.source}]</span>
                      <span className={`whitespace-pre-wrap break-words font-mono ${
                        log.level === 'ERROR' ? 'text-[#f85149] font-medium' :
                        log.level === 'WARN' ? 'text-[#e5c07b]' :
                        'text-zinc-200'
                      }`}>
                        {log.message}
                      </span>
                    </div>
                  )
                })}
                <div className="flex gap-4 py-3 px-2 mt-2 border-t border-zinc-800/80">
                  <span className="text-zinc-500 w-20 shrink-0 font-mono">...</span>
                  <div className="flex items-center gap-2 text-emerald-400 font-medium font-mono">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-ping"></span>
                    <span>envexa-daemon: watching incoming events</span>
                    <span className="inline-block w-1.5 h-4 bg-emerald-400 animate-pulse ml-0.5"></span>
                  </div>
                </div>
              </>
            )}
          </div>
        </CardContent>
      </div>
    </div>
  )
}
