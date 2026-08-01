import type { ReactNode } from "react"
import { cn } from "@/lib/utils"
import { useTheme } from "@/components/theme-provider"
import { Badge } from "@/components/ui/badge"
import { Boxes, FileSearch, Folder, Shield } from "lucide-react"
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
import type { VulnerabilityInfo } from "@/lib/vulnerabilities"

export interface PackageInfo {
  name: string
  current: string
  latest: string
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

export interface ToolCategory {
  name: string
  tools: string[]
}

export const CATEGORIES: ToolCategory[] = [
  {
    name: "System & Runtime",
    tools: ["brew", "cargo", "docker", "pip", "gem"],
  },
  { name: "Web Development", tools: ["npm", "pnpm", "yarn", "bun", "deno"] },
  {
    name: "Project Tooling",
    tools: ["project", "security", "supply_chain", "audit", "ci"],
  },
]

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

type ToolIconEntry = {
  icon: { path: string; hex: string }
  fallback?: ReactNode
  invertInDark?: boolean
}

// Map tool names to simple-icons
const TOOL_ICONS: Record<string, ToolIconEntry> = {
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
  const entry = (TOOL_ICONS as Record<string, ToolIconEntry | undefined>)[tool]
  if (!entry) return <Boxes className="h-5 w-5 text-muted-foreground" />
  return (
    entry.fallback ?? (
      <ToolIcon
        icon={entry.icon}
        className="h-5 w-5"
        invertInDark={entry.invertInDark}
      />
    )
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

export function statusKey(status: string): "pass" | "warn" | "fail" | "skip" {
  const s = status.toLowerCase()
  if (s.includes("fail") || s.includes("error")) return "fail"
  if (s.includes("warn")) return "warn"
  if (s.includes("skip") || s.includes("not found")) return "skip"
  return "pass"
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
