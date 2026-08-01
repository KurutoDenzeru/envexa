import type { ReactNode } from "react"
import { cn } from "@/lib/utils"
import { useTheme } from "@/components/theme-provider"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  siBun,
  siDeno,
  siDocker,
  siGithub,
  siHomebrew,
  siNpm,
  siPnpm,
  siPython,
  siRubygems,
  siRust,
  siYarn,
} from "simple-icons"
import { Boxes, FileSearch, Folder, Shield } from "lucide-react"

export interface PackageInfo {
  name: string
  current: string
  latest: string
}

export interface VulnerabilityInfo {
  package: string
  severity: string
  title: string
  cve?: string | null
  patched_version?: string
}

export interface ToolchainResult {
  tool: string
  status: string
  version?: string
  node_version?: string
  python_version?: string
  ruby_version?: string
  rustc_version?: string
  cargo_version?: string
  pnpm_version?: string
  bun_version?: string
  deno_version?: string
  installed_count?: number
  outdated_formulae?: PackageInfo[]
  outdated_casks?: PackageInfo[]
  outdated?: PackageInfo[]
  outdated_global?: PackageInfo[]
  issues?: string[]
  project_type?: string
  vulnerabilities?: VulnerabilityInfo[]
  supply_chain_risks?: Array<{
    package: string
    risk_type: string
    description: string
  }>
  audit_items?: Array<{
    name: string
    current: string
    note: string
  }>
}

function ToolIcon({
  icon,
  className = "w-5 h-5",
  color,
  invertInDark = false,
}: {
  icon: { path: string; hex: string }
  className?: string
  color?: string
  invertInDark?: boolean
}) {
  const { theme } = useTheme()
  const isDark =
    invertInDark &&
    (theme === "dark" ||
      (theme === "system" &&
        typeof window !== "undefined" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches))

  return (
    <svg
      role="img"
      viewBox="0 0 24 24"
      className={cn(className, isDark && "brightness-200 invert")}
      fill={color || `#${icon.hex}`}
    >
      <path d={icon.path} />
    </svg>
  )
}

// Map tool names to simple-icons
const TOOL_ICONS: Record<
  string,
  {
    icon: { path: string; hex: string }
    fallback?: ReactNode
    invertInDark?: boolean
  }
> = {
  brew: { icon: siHomebrew },
  cargo: { icon: siRust, invertInDark: true },
  docker: { icon: siDocker },
  pip: { icon: siPython },
  gem: { icon: siRubygems },
  npm: { icon: siNpm },
  pnpm: { icon: siPnpm },
  yarn: { icon: siYarn },
  bun: { icon: siBun, invertInDark: true },
  deno: { icon: siDeno, invertInDark: true },
  project: {
    icon: {
      path: "M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z",
      hex: "71717a",
    },
    fallback: <Folder className="h-5 w-5 text-muted-foreground" />,
  },
  security: {
    icon: {
      path: "M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z",
      hex: "71717a",
    },
    fallback: <Shield className="h-5 w-5 text-muted-foreground" />,
  },
  supply_chain: {
    icon: {
      path: "M20 8h-3V4H3c-1.1 0-2 .9-2 2v11h2c0 1.66 1.34 3 3 3s3-1.34 3-3h6c0 1.66 1.34 3 3 3s3-1.34 3-3h2v-5l-3-4zM6 18.5c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm13.5-9l1.96 2.5H17V9.5h2.5zm-1.5 9c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5z",
      hex: "71717a",
    },
    fallback: <Boxes className="h-5 w-5 text-muted-foreground" />,
  },
  audit: {
    icon: {
      path: "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z",
      hex: "71717a",
    },
    fallback: <FileSearch className="h-5 w-5 text-muted-foreground" />,
  },
  ci: { icon: siGithub, invertInDark: true },
}

export function getToolIcon(tool: string) {
  const entry = TOOL_ICONS[tool]
  if (!entry) return <Boxes className="h-5 w-5 text-muted-foreground" />
  if (entry.fallback) return entry.fallback
  return (
    <ToolIcon
      icon={entry.icon}
      className="h-5 w-5"
      invertInDark={entry.invertInDark}
    />
  )
}

export function displayName(tool: string): string {
  const names: Record<string, string> = {
    brew: "Brew",
    npm: "npm",
    pnpm: "pnpm",
    yarn: "Yarn",
    bun: "Bun",
    deno: "Deno",
    pip: "pip",
    gem: "Gem",
    cargo: "Cargo",
    docker: "Docker",
    project: "Project",
    security: "Security",
    supply_chain: "Supply Chain",
    audit: "Audit",
    ci: "CI/CD",
  }
  return names[tool] || tool
}

export function statusBadge(status: string) {
  const s = status.toLowerCase()
  if (s.includes("fail") || s.includes("error")) {
    return (
      <Badge
        variant="destructive"
        className="border-red-500/20 bg-red-500/10 text-red-500 shadow-none"
      >
        FAIL
      </Badge>
    )
  }
  if (s.includes("warn")) {
    return (
      <Badge
        variant="outline"
        className="border-yellow-500/30 bg-yellow-500/10 text-yellow-500 shadow-none"
      >
        WARN
      </Badge>
    )
  }
  if (s.includes("skip") || s.includes("not found")) {
    return (
      <Badge
        variant="outline"
        className="border-border text-muted-foreground shadow-none"
      >
        SKIP
      </Badge>
    )
  }
  return (
    <Badge
      variant="outline"
      className="border-green-500/30 bg-green-500/10 text-green-500 shadow-none"
    >
      PASS
    </Badge>
  )
}

export function getPrimaryVersion(tc: ToolchainResult): string {
  return (
    tc.version ||
    tc.node_version ||
    tc.python_version ||
    tc.ruby_version ||
    tc.rustc_version ||
    tc.cargo_version ||
    tc.pnpm_version ||
    tc.bun_version ||
    tc.deno_version ||
    "-"
  )
}

export function ToolchainCard({
  tc,
  onViewDetails,
}: {
  tc: ToolchainResult
  onViewDetails: () => void
}) {
  const vulnCount = tc.vulnerabilities?.length || 0
  const outdatedCount = tc.outdated?.length || 0
  const issuesCount = tc.issues?.length || 0

  return (
    <Card className="flex flex-col justify-between border-border bg-card shadow-xs transition-all duration-300 hover:border-border/80">
      <CardHeader className="border-b border-border/50 pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-lg text-foreground capitalize">
            {getToolIcon(tc.tool)}
            {displayName(tc.tool)}
          </CardTitle>
          {statusBadge(tc.status)}
        </div>
      </CardHeader>

      <CardContent className="flex flex-1 flex-col gap-4 pt-4">
        <div className="flex min-w-0 items-center gap-2 rounded-md border border-border/40 bg-muted/30 p-2.5 font-mono text-xs text-muted-foreground/80">
          <span
            className="min-w-0 truncate text-foreground"
            title={getPrimaryVersion(tc)}
          >
            {getPrimaryVersion(tc)}
          </span>
        </div>

        <div className="grid grid-cols-3 gap-3">
          <div className="rounded-lg border border-border/50 bg-muted/50 p-3">
            <div className="mb-1 text-[10px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
              Vulns
            </div>
            <div
              className={`text-xl font-bold ${vulnCount > 0 ? "text-red-400" : "text-foreground/90"}`}
            >
              {vulnCount}
            </div>
          </div>
          <div className="rounded-lg border border-border/50 bg-muted/50 p-3">
            <div className="mb-1 text-[10px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
              Outdated
            </div>
            <div
              className={`text-xl font-bold ${outdatedCount > 0 ? "text-blue-400" : "text-foreground/90"}`}
            >
              {outdatedCount}
            </div>
          </div>
          <div className="rounded-lg border border-border/50 bg-muted/50 p-3">
            <div className="mb-1 text-[10px] font-semibold tracking-wider text-muted-foreground/60 uppercase">
              Issues
            </div>
            <div
              className={`text-xl font-bold ${issuesCount > 0 ? "text-yellow-400" : "text-foreground/90"}`}
            >
              {issuesCount}
            </div>
          </div>
        </div>

        <div className="flex justify-end border-t border-border/30 pt-2">
          <button
            onClick={onViewDetails}
            className="inline-flex h-9 cursor-pointer items-center justify-center gap-1.5 rounded-md border border-border bg-popover px-4 text-xs font-semibold text-foreground shadow-xs transition-all select-none hover:bg-muted/50 hover:text-foreground"
          >
            View Details
          </button>
        </div>
      </CardContent>
    </Card>
  )
}
