"use client"

import { useState, useEffect, useMemo } from "react"
import { CheckCircle, Loader2, XCircle, MinusCircle, RefreshCw } from "lucide-react"
import { cn } from "@/lib/utils"

export type ToolchainName =
  | "brew"
  | "npm"
  | "pnpm"
  | "yarn"
  | "bun"
  | "deno"
  | "pip"
  | "gem"
  | "cargo"
  | "docker"
  | "project"
  | "security"
  | "audit"
  | "ci"
  | "supply_chain"

export type ToolchainStatus = "pending" | "scanning" | "done" | "error" | "skipped"

export interface ToolchainProgress {
  name: ToolchainName | "__category__"
  label: string
  status: ToolchainStatus
  icon?: React.ReactNode
}

const TOOLCHAIN_LABELS: Record<ToolchainName, string> = {
  brew: "Homebrew",
  npm: "npm",
  pnpm: "pnpm",
  yarn: "Yarn",
  bun: "Bun",
  deno: "Deno",
  pip: "pip",
  gem: "RubyGems",
  cargo: "Cargo",
  docker: "Docker",
  project: "Project Config",
  security: "Security Scan",
  audit: "Dependency Audit",
  ci: "CI Config",
  supply_chain: "Supply Chain",
}

const TOOLCHAIN_CATEGORIES: Record<string, ToolchainName[]> = {
  "Package Managers": ["brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker"],
  "Project Analysis": ["project", "security", "audit", "ci", "supply_chain"],
}

const STATUS_ICONS: Record<ToolchainStatus, React.ReactNode> = {
  pending: <MinusCircle className="w-4 h-4 text-muted-foreground/40" />,
  scanning: <Loader2 className="w-4 h-4 text-blue-400 animate-spin" />,
  done: <CheckCircle className="w-4 h-4 text-emerald-400" />,
  error: <XCircle className="w-4 h-4 text-red-400" />,
  skipped: <MinusCircle className="w-4 h-4 text-muted-foreground/40" />,
}

const STATUS_LABELS: Record<ToolchainStatus, string> = {
  pending: "Pending",
  scanning: "Scanning...",
  done: "Done",
  error: "Error",
  skipped: "Skipped",
}

interface ScanProgressProps {
  className?: string
  loading: boolean
  onRetry?: () => void
}

const INITIAL_TOOLCHAINS: ToolchainProgress[] = Object.entries(TOOLCHAIN_CATEGORIES).flatMap(
  ([category, tools]) => [
    { name: "__category__", label: category, status: "pending" as ToolchainStatus } as ToolchainProgress,
    ...tools.map((name) => ({
      name,
      label: TOOLCHAIN_LABELS[name],
      status: "pending" as ToolchainStatus,
    })),
  ]
)

export function ScanProgress({ className, loading, onRetry }: ScanProgressProps) {
  const [toolchains, setToolchains] = useState<ToolchainProgress[]>(INITIAL_TOOLCHAINS)
  const [globalStatus, setGlobalStatus] = useState<"idle" | "warming" | "scanning" | "done" | "error">("idle")
  const [retryCount, setRetryCount] = useState(0)

  useEffect(() => {
    if (!loading) {
      setGlobalStatus("done")
      setToolchains((prev) =>
        prev.map((t) =>
          t.name === "__category__" ? t : t.status === "pending" ? { ...t, status: "skipped" } : t
        )
      )
      return
    }

    setGlobalStatus("warming")
    setToolchains(INITIAL_TOOLCHAINS)

    const handler = (e: CustomEvent<string>) => {
      const status = e.detail
      if (status === "warming") {
        setGlobalStatus("warming")
      } else if (status === "active") {
        setGlobalStatus("scanning")
        simulateProgress()
      } else if (status === "error") {
        setGlobalStatus("error")
        setToolchains((prev) =>
          prev.map((t) =>
            t.name === "__category__" || t.status === "done"
              ? t
              : { ...t, status: t.status === "scanning" ? "error" : "skipped" }
          )
        )
      }
    }

    window.addEventListener("scanner-status", handler as EventListener)
    return () => window.removeEventListener("scanner-status", handler as EventListener)
  }, [loading])

  const simulateProgress = () => {
    const toolchainNames = INITIAL_TOOLCHAINS.filter((t) => t.name !== "__category__").map((t) => t.name)
    let index = 0

    const interval = setInterval(() => {
      if (index >= toolchainNames.length) {
        clearInterval(interval)
        setGlobalStatus("done")
        return
      }

      const name = toolchainNames[index]
      setToolchains((prev) =>
        prev.map((t) => {
          if (t.name === name) return { ...t, status: "scanning" }
          if (t.name !== "__category__" && t.status === "pending" && prev.find((p) => p.name === name)?.status === "scanning") {
            return { ...t, status: "done" }
          }
          return t
        })
      )

      setTimeout(() => {
        setToolchains((prev) =>
          prev.map((t) => (t.name === name ? { ...t, status: "done" } : t))
        )
        index++
      }, 300 + Math.random() * 400)
    }, 200 + Math.random() * 300)

    return () => clearInterval(interval)
  }

  const categoryGroups = useMemo(() => {
    const groups: Array<{ category: string; tools: ToolchainProgress[] }> = []
    let currentCategory = ""

    for (const tc of toolchains) {
      if (tc.name === "__category__") {
        currentCategory = tc.label
        groups.push({ category: currentCategory, tools: [] })
      } else {
        const lastGroup = groups[groups.length - 1]
        if (lastGroup) lastGroup.tools.push(tc)
      }
    }
    return groups
  }, [toolchains])

  const doneCount = toolchains.filter((t) => t.status === "done").length
  const totalCount = toolchains.filter((t) => t.name !== "__category__").length

  if (!loading && globalStatus === "done") {
    return null
  }

  return (
    <div className={cn("space-y-6 animate-in fade-in duration-500", className)}>
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center gap-3">
          <div
            className={cn(
              "w-2 h-2 rounded-full animate-pulse",
              globalStatus === "warming" && "bg-orange-500",
              globalStatus === "scanning" && "bg-blue-500",
              globalStatus === "done" && "bg-emerald-500 animate-none",
              globalStatus === "error" && "bg-red-500 animate-none"
            )}
          />
          <div>
            <p className="text-sm font-medium text-foreground">
              {globalStatus === "warming" && "Warming up scanner..."}
              {globalStatus === "scanning" && "Scanning toolchains..."}
              {globalStatus === "done" && "Scan complete"}
              {globalStatus === "error" && "Scan failed"}
              {globalStatus === "idle" && "Initializing..."}
            </p>
            <p className="text-xs text-muted-foreground">
              {doneCount} / {totalCount} toolchains completed
            </p>
          </div>
        </div>

        {globalStatus === "error" && onRetry && (
          <button
            onClick={() => {
              onRetry()
              setRetryCount((c) => c + 1)
            }}
            className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw className={cn("w-4 h-4", retryCount > 0 && "animate-spin")} />
            Retry Scan
          </button>
        )}
      </div>

      <div className="space-y-4">
        {categoryGroups.map((group, catIndex) => (
          <div key={group.category} className="space-y-2 animate-in slide-in-from-top duration-300" style={{ animationDelay: `${catIndex * 50}ms` }}>
            <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider px-1">
              {group.category}
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
              {group.tools.map((tc, toolIndex) => (
                <ToolchainRow key={tc.name} toolchain={tc} toolIndex={toolIndex} />
              ))}
            </div>
          </div>
        ))}
      </div>

      {globalStatus === "scanning" && (
        <div className="h-1 w-full bg-muted rounded-full overflow-hidden">
          <div
            className="h-full bg-blue-500 animate-pulse"
            style={{ width: `${(doneCount / totalCount) * 100}%` }}
          />
        </div>
      )}
    </div>
  )
}

function ToolchainRow({ toolchain, toolIndex }: { toolchain: ToolchainProgress; toolIndex: number }) {
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    const timer = setTimeout(() => setMounted(true), toolIndex * 30)
    return () => clearTimeout(timer)
  }, [toolIndex])

  const statusConfig = {
    pending: { bg: "bg-muted/30", border: "border-border/30", text: "text-muted-foreground/60" },
    scanning: { bg: "bg-blue-500/10", border: "border-blue-500/30", text: "text-blue-400" },
    done: { bg: "bg-emerald-500/10", border: "border-emerald-500/30", text: "text-emerald-400" },
    error: { bg: "bg-red-500/10", border: "border-red-500/30", text: "text-red-400" },
    skipped: { bg: "bg-muted/20", border: "border-border/20", text: "text-muted-foreground/40" },
  }

  const config = statusConfig[toolchain.status]

  return (
    <div
      className={cn(
        "flex items-center gap-3 px-3 py-2.5 rounded-lg border transition-all duration-300",
        config.bg,
        config.border,
        !mounted && "opacity-0 translate-y-1"
      )}
      style={{ animationDelay: `${toolIndex * 30}ms` }}
    >
      <div className={cn("flex-shrink-0", config.text)}>{STATUS_ICONS[toolchain.status]}</div>
      <span className={cn("text-sm font-medium truncate", config.text)}>{toolchain.label}</span>
      <span className={cn("ml-auto text-xs font-mono", config.text)}>
        {STATUS_LABELS[toolchain.status]}
      </span>
    </div>
  )
}