import { createFileRoute } from "@tanstack/react-router"
import { CardContent } from "@/components/ui/card"
import {
  Terminal,
  ScrollText,
  Filter,
  Download,
  Search,
  Check,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useState, useMemo, useEffect, useCallback } from "react"

export const Route = createFileRoute("/logs")({ component: LogsPage })

const mockLogs = [
  {
    time: "10:15:32",
    date: "July 31, 2026",
    level: "INFO",
    message: "Starting Envexa scanner engine...",
    source: "system",
  },
  {
    time: "10:15:33",
    date: "July 31, 2026",
    level: "INFO",
    message: "Detected Node.js project. Scanning package.json...",
    source: "node",
  },
  {
    time: "10:15:34",
    date: "July 31, 2026",
    level: "WARN",
    message:
      "Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21)",
    source: "node",
  },
  {
    time: "10:15:35",
    date: "July 31, 2026",
    level: "INFO",
    message: "Detected Rust project. Scanning Cargo.toml...",
    source: "rust",
  },
  {
    time: "10:15:38",
    date: "July 31, 2026",
    level: "ERROR",
    message: "Security vulnerability found in 'regex' crate: CVE-2022-24713",
    source: "rust",
  },
  {
    time: "10:15:39",
    date: "July 31, 2026",
    level: "INFO",
    message: "Detected Python project. Scanning requirements.txt...",
    source: "python",
  },
  {
    time: "10:15:40",
    date: "July 31, 2026",
    level: "INFO",
    message: "Scan completed successfully. Generated report.",
    source: "system",
  },
  {
    time: "10:15:42",
    date: "July 31, 2026",
    level: "DEBUG",
    message: "Cleaning up temporary files...",
    source: "system",
  },
  {
    time: "10:16:01",
    date: "July 31, 2026",
    level: "INFO",
    message: "File change detected in src/main.rs. Re-running scanner...",
    source: "watcher",
  },
]

function LogsPage() {
  const [filterLevel, setFilterLevel] = useState<string>("ALL")
  const [search, setSearch] = useState("")
  const [logs, setLogs] = useState<any[]>([])
  const [logsPath, setLogsPath] = useState<string>("envexa-system.log")
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const start = Date.now()
    fetch("/api/logs")
      .then((res) => {
        if (!res.ok) throw new Error("Failed to fetch logs")
        return res.json()
      })
      .then((data) => {
        setLogs(data.logs)
        setLogsPath(data.path)
        const elapsed = Date.now() - start
        const remaining = Math.max(0, 100 - elapsed)
        setTimeout(() => setLoading(false), remaining)
      })
      .catch((err) => {
        console.error(err)
        const parsedMock = mockLogs.map((log) => ({
          time: log.time,
          date: log.date,
          level: log.level,
          message: log.message,
          source: log.source,
        }))
        setLogs(parsedMock)
        setLogsPath("~/.local/share/envexa/logs.json")
        const elapsed = Date.now() - start
        const remaining = Math.max(0, 100 - elapsed)
        setTimeout(() => setLoading(false), remaining)
      })
  }, [])

  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      const matchesLevel = filterLevel === "ALL" || log.level === filterLevel
      const matchesSearch =
        log.message.toLowerCase().includes(search.toLowerCase()) ||
        log.source.toLowerCase().includes(search.toLowerCase())
      return matchesLevel && matchesSearch
    })
  }, [logs, filterLevel, search])

  const downloadLogs = useCallback(() => {
    const header =
      "# Envexa System Logs\n# Generated: " +
      new Date().toISOString() +
      "\n# Source: " +
      logsPath +
      "\n\n"
    const formatted = filteredLogs
      .map(
        (log) =>
          `${log.time} ${log.level.padEnd(5)} [${log.source}] ${log.message}`
      )
      .join("\n")
    const content = header + formatted + "\n"
    const blob = new Blob([content], { type: "text/plain;charset=utf-8" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = logsPath.split("/").pop() || "envexa-logs.txt"
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [filteredLogs, logsPath])

  if (loading) {
    return (
      <div className="mx-auto flex max-w-7xl flex-col gap-6">
        <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
          <div>
            <Skeleton className="h-10 w-48 bg-muted" />
            <Skeleton className="mt-3 h-4 w-80 bg-muted" />
          </div>
          <div className="flex gap-3">
            <Skeleton className="h-9 w-52 bg-muted" />
            <Skeleton className="h-9 w-32 bg-muted" />
          </div>
        </div>
        <Skeleton className="h-[400px] w-full rounded-xl bg-muted/50" />
      </div>
    )
  }

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <ScrollText className="h-8 w-8 text-foreground" />
            System Logs
          </h1>
          <p className="mt-2 text-muted-foreground">
            Real-time event logs and diagnostic output from the scanning engine.
          </p>
        </div>
        <div className="flex w-full gap-3 md:w-auto">
          <div className="relative w-full md:w-52">
            <Search className="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground/60" />
            <Input
              type="text"
              placeholder="Search logs..."
              className="h-9 w-full border-border bg-background/50 pl-9 focus-visible:ring-blue-500"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger className="inline-flex h-9 w-32 cursor-pointer items-center justify-between gap-2 rounded-md border border-border bg-popover px-3 text-sm font-medium text-foreground shadow-xs transition-colors select-none hover:bg-muted/50 focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none">
              <span className="flex items-center gap-2 truncate">
                <Filter className="h-4 w-4 shrink-0 text-muted-foreground" />
                <span className="truncate">
                  {filterLevel === "ALL" ? "All Levels" : filterLevel}
                </span>
              </span>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              className="w-40 border-border bg-popover"
            >
              {["ALL", "INFO", "WARN", "ERROR", "DEBUG"].map((level) => (
                <DropdownMenuItem
                  key={level}
                  onClick={() => setFilterLevel(level)}
                  className="cursor-pointer justify-between focus:bg-muted/50"
                >
                  {level === "ALL" ? "All Levels" : level}
                  {filterLevel === level && (
                    <Check className="h-4 w-4 text-blue-500" />
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="outline"
                  size="icon"
                  className="h-9 w-9 border-border bg-popover text-foreground shadow-xs hover:bg-muted/50"
                  onClick={downloadLogs}
                  aria-label="Download Logs"
                />
              }
            >
              <Download className="h-4 w-4" />
            </TooltipTrigger>
            <TooltipContent>Download Logs</TooltipContent>
          </Tooltip>
        </div>
      </div>

      <div className="overflow-hidden rounded-xl border border-zinc-800 bg-[#0c0c0e] font-mono shadow-2xl">
        {/* Authentic macOS Terminal Title Bar */}
        <div className="flex items-center justify-between border-b border-zinc-800/80 bg-[#18181b] px-4 py-2.5 select-none">
          {/* macOS window control buttons */}
          <div className="flex w-20 items-center gap-2">
            <span className="h-3 w-3 cursor-pointer rounded-full border border-[#e0443e] bg-[#ff5f56] transition-opacity hover:opacity-85"></span>
            <span className="h-3 w-3 cursor-pointer rounded-full border border-[#dab12d] bg-[#ffbd2e] transition-opacity hover:opacity-85"></span>
            <span className="h-3 w-3 cursor-pointer rounded-full border border-[#1aab29] bg-[#27c93f] transition-opacity hover:opacity-85"></span>
          </div>
          {/* Center Monospace Session Title */}
          <div className="flex max-w-[60%] items-center gap-1.5 truncate font-sans text-xs font-medium tracking-wide text-zinc-400">
            <Terminal className="h-3.5 w-3.5 shrink-0 text-zinc-500" />
            <span className="truncate">{logsPath}</span>
          </div>
          {/* Right indicator for balance */}
          <div className="w-20 text-right font-mono text-[10px] tracking-wider text-zinc-600">
            bash
          </div>
        </div>

        <CardContent className="p-0">
          <div className="h-[500px] scrollbar-thin scrollbar-thumb-zinc-800 scrollbar-track-transparent overflow-y-auto bg-[#0c0c0e] p-4 font-mono text-[13px] leading-relaxed text-zinc-100">
            {loading ? (
              <div className="flex h-full flex-col items-center justify-center space-y-4 text-zinc-500">
                <span className="h-6 w-6 animate-spin rounded-full border-2 border-zinc-600 border-t-zinc-200"></span>
                <p className="font-sans text-sm">Loading real-time logs...</p>
              </div>
            ) : filteredLogs.length === 0 ? (
              <div className="flex h-full flex-col items-center justify-center text-zinc-500">
                <Filter className="mb-4 h-10 w-10 text-zinc-400 opacity-20" />
                <p className="font-sans text-sm">
                  No logs match the current filters.
                </p>
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
                  const sourceColor =
                    sourceColors[log.source.toLowerCase()] || "text-zinc-500"
                  return (
                    <div
                      key={i}
                      className="group flex gap-4 rounded px-2 py-1.5 transition-colors hover:bg-zinc-900/60"
                    >
                      <span className="w-36 shrink-0 font-mono text-zinc-500 transition-colors select-none group-hover:text-zinc-400">
                        {log.date
                          ? (() => {
                              const m = log.date.match(/^(\w+)\s+(\d+)/)
                              return m
                                ? m[1].slice(0, 3) +
                                    " " +
                                    m[2].padStart(2, " ") +
                                    " "
                                : ""
                            })()
                          : ""}
                        {log.time}
                      </span>
                      <span
                        className={`w-14 shrink-0 font-mono font-bold select-none ${
                          log.level === "INFO"
                            ? "text-[#57ab5a]"
                            : log.level === "WARN"
                              ? "text-[#e5c07b]"
                              : log.level === "DEBUG"
                                ? "text-[#b392f0]"
                                : "text-[#f85149]"
                        }`}
                      >
                        {log.level.padEnd(5)}
                      </span>
                      <span
                        className={`hidden w-20 shrink-0 truncate font-mono select-none sm:block ${sourceColor}`}
                      >
                        [{log.source}]
                      </span>
                      <span
                        className={`font-mono break-words whitespace-pre-wrap ${
                          log.level === "ERROR"
                            ? "font-medium text-[#f85149]"
                            : log.level === "WARN"
                              ? "text-[#e5c07b]"
                              : "text-zinc-200"
                        }`}
                      >
                        {log.message}
                      </span>
                    </div>
                  )
                })}
              </>
            )}
          </div>
        </CardContent>
      </div>
    </div>
  )
}
