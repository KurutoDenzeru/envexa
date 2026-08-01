import { useState } from "react"
import type { ReactNode } from "react"
import { cn } from "@/lib/utils"
import { useTheme } from "@/components/theme-provider"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Table, TableBody, TableCell, TableRow } from "@/components/ui/table"
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
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
import { Boxes, CheckCircle, FileSearch, Folder, Shield } from "lucide-react"

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

export function getVersionFields(
  tc: ToolchainResult
): Array<{ label: string; value: string }> {
  const fields: Array<{ label: string; value: string }> = []
  if (tc.version) fields.push({ label: "Version", value: tc.version })
  if (tc.node_version)
    fields.push({ label: "Node Version", value: tc.node_version })
  if (tc.python_version)
    fields.push({ label: "Python Version", value: tc.python_version })
  if (tc.ruby_version)
    fields.push({ label: "Ruby Version", value: tc.ruby_version })
  if (tc.rustc_version)
    fields.push({ label: "Rustc Version", value: tc.rustc_version })
  if (tc.cargo_version)
    fields.push({ label: "Cargo Version", value: tc.cargo_version })
  if (tc.pnpm_version)
    fields.push({ label: "pnpm Version", value: tc.pnpm_version })
  if (tc.bun_version)
    fields.push({ label: "Bun Version", value: tc.bun_version })
  if (tc.deno_version)
    fields.push({ label: "Deno Version", value: tc.deno_version })
  if (tc.installed_count !== undefined)
    fields.push({ label: "Installed Count", value: String(tc.installed_count) })
  fields.push({ label: "Scanner Status", value: tc.status || "Ok" })
  if (tc.project_type)
    fields.push({ label: "Project Type", value: tc.project_type })
  return fields
}

const vulnColumns: ColumnDef<VulnerabilityInfo, unknown>[] = [
  {
    accessorKey: "package",
    header: "Package",
    cell: ({ row }) => (
      <span className="font-mono text-xs font-medium">
        {row.original.package}
      </span>
    ),
  },
  {
    accessorKey: "title",
    header: "Title",
    cell: ({ row }) => (
      <div className="text-xs font-semibold text-foreground">
        {row.original.title}
      </div>
    ),
  },
  {
    accessorKey: "cve",
    header: "CVE",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-muted-foreground">
        {row.original.cve || "-"}
      </span>
    ),
  },
  {
    accessorKey: "severity",
    header: "Severity",
    cell: ({ row }) => (
      <Badge
        variant="destructive"
        className="h-5 border-red-500/20 bg-red-500/10 px-1.5 text-[10px] text-red-500 shadow-none"
      >
        {row.original.severity}
      </Badge>
    ),
  },
  {
    accessorKey: "patched_version",
    header: "Patched",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-muted-foreground">
        {row.original.patched_version || "-"}
      </span>
    ),
  },
]

const outdatedColumns: ColumnDef<PackageInfo, unknown>[] = [
  {
    accessorKey: "name",
    header: "Package",
    cell: ({ row }) => (
      <span className="font-mono text-xs font-medium">{row.original.name}</span>
    ),
  },
  {
    accessorKey: "current",
    header: "Current",
    cell: ({ row }) => (
      <span className="font-mono text-xs text-muted-foreground">
        {row.original.current}
      </span>
    ),
  },
  {
    accessorKey: "latest",
    header: "Latest",
    cell: ({ row }) => (
      <Badge
        variant="outline"
        className="h-5 border-blue-500/30 bg-blue-500/10 px-1.5 text-[10px] text-blue-400 shadow-none"
      >
        {row.original.latest}
      </Badge>
    ),
  },
]

export function ToolchainDetailDialog({
  tc,
  open,
  onOpenChange,
}: {
  tc: ToolchainResult
  open?: boolean
  onOpenChange?: (open: boolean) => void
}) {
  const vulnCount = tc.vulnerabilities?.length || 0
  const outdatedCount = tc.outdated?.length || 0
  const issuesCount = tc.issues?.length || 0
  const [activeTab, setActiveTab] = useState<"security" | "updates" | "specs">(
    vulnCount > 0 ? "security" : outdatedCount > 0 ? "updates" : "specs"
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[90vh] flex-col border border-border bg-card p-6 shadow-xl sm:max-w-2xl">
        <DialogHeader className="border-b border-border/50 pb-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg border border-border/60 bg-muted/60 p-2">
              {getToolIcon(tc.tool)}
            </div>
            <div>
              <DialogTitle className="text-2xl font-bold text-foreground capitalize">
                {displayName(tc.tool)}
              </DialogTitle>
              <DialogDescription className="mt-0.5 text-muted-foreground">
                Vulnerability audit and package state.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="flex flex-1 flex-col gap-6 overflow-y-auto py-4">
          {/* Top Stats */}
          <div className="grid grid-cols-4 gap-4">
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Vulnerabilities
              </span>
              <span
                className={`font-mono text-xl font-bold ${vulnCount > 0 ? "text-red-400" : "text-muted-foreground"}`}
              >
                {vulnCount}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Outdated
              </span>
              <span
                className={`font-mono text-xl font-bold ${outdatedCount > 0 ? "text-blue-400" : "text-muted-foreground"}`}
              >
                {outdatedCount}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Issues
              </span>
              <span
                className={`font-mono text-xl font-bold ${issuesCount > 0 ? "text-yellow-400" : "text-muted-foreground"}`}
              >
                {issuesCount}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Status
              </span>
              <div className="flex justify-center">
                {statusBadge(tc.status)}
              </div>
            </div>
          </div>

          {/* Segmented Tab Buttons */}
          <div className="flex gap-1 rounded-lg border border-border/40 bg-muted/30 p-1">
            {(
              [
                { key: "security", label: `Security (${vulnCount})` },
                { key: "updates", label: `Updates (${outdatedCount})` },
                { key: "specs", label: "Specs" },
              ] as const
            ).map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`flex-1 cursor-pointer rounded-md px-3 py-2 text-xs font-medium transition-all ${
                  activeTab === tab.key
                    ? "border border-border/50 bg-background text-foreground shadow-xs"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tab Content */}
          {activeTab === "security" && (
            <div>
              {vulnCount === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    No Security Flaws Detected
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This toolchain has no known active security alerts.
                  </p>
                </div>
              ) : (
                <DataTable
                  columns={vulnColumns}
                  data={tc.vulnerabilities || []}
                  defaultPageSize={5}
                  pageSizeOptions={[5, 10, 25]}
                />
              )}
            </div>
          )}

          {activeTab === "updates" && (
            <div>
              {outdatedCount === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    All Dependencies Up to Date
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This toolchain uses the latest available package releases.
                  </p>
                </div>
              ) : (
                <DataTable
                  columns={outdatedColumns}
                  data={tc.outdated || []}
                  defaultPageSize={5}
                  pageSizeOptions={[5, 10, 25]}
                />
              )}
            </div>
          )}

          {activeTab === "specs" && (
            <div className="overflow-hidden rounded-lg border border-border/40 bg-muted/10">
              <Table>
                <TableBody>
                  {getVersionFields(tc).map((v, vIdx) => (
                    <TableRow
                      key={vIdx}
                      className="border-border hover:bg-muted/30"
                    >
                      <TableCell className="text-xs font-medium text-muted-foreground">
                        {v.label}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs text-foreground">
                        {v.value}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
