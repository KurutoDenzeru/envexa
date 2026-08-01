import { createFileRoute } from "@tanstack/react-router"
import { Fragment, useState, useMemo, useEffect } from "react"
import { useScanData } from "@/components/scan-data-context"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import {
  PackageOpen,
  CheckCircle,
  ShieldAlert,
  Table as TableIcon,
  Boxes,
  LayoutGrid,
} from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import { ScanProgress } from "@/components/scan-progress"
import { DonutChart, CHART_PALETTE } from "@/components/donut-chart"
import {
  ToolchainCard,
  displayName,
  getPrimaryVersion,
  getToolIcon,
  statusBadge,
} from "@/components/toolchain-card"
import type {
  ToolchainResult,
  VulnerabilityInfo,
  PackageInfo,
} from "@/components/toolchain-card"

export const Route = createFileRoute("/toolchains")({
  validateSearch: (search: Record<string, unknown>) => ({
    open: (search.open as string) || undefined,
  }),
  component: Toolchains,
})

interface ToolCategory {
  name: string
  tools: string[]
}

const CATEGORIES: ToolCategory[] = [
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

// Severity weights used for the total risk score (higher = more severe)
const RISK_WEIGHTS: Record<string, number> = {
  critical: 10,
  high: 5,
  moderate: 3,
  low: 1,
}

function statusKey(status: string): "pass" | "warn" | "fail" | "skip" {
  const s = status.toLowerCase()
  if (s.includes("fail") || s.includes("error")) return "fail"
  if (s.includes("warn")) return "warn"
  if (s.includes("skip") || s.includes("not found")) return "skip"
  return "pass"
}

function getVersionFields(
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

function ToolchainDetailDialog({
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

function Toolchains() {
  const { report, loading, refetch: fetchReport } = useScanData()
  const searchParams = Route.useSearch()
  const [openDialog, setOpenDialog] = useState<string | null>(
    searchParams.open ?? null
  )
  // Toggle between grid (cards) and compact table view
  const [compactView, setCompactView] = useState(false)

  useEffect(() => {
    if (searchParams.open) {
      setOpenDialog(searchParams.open)
      window.history.replaceState({}, "", "/toolchains")
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const toolchainMap = useMemo(() => {
    if (!report?.results) return new Map<string, ToolchainResult>()
    const map = new Map<string, ToolchainResult>()
    for (const [name, data] of Object.entries(report.results)) {
      map.set(name, { ...data, tool: name })
    }
    return map
  }, [report])

  const groupedCategories = useMemo(() => {
    return CATEGORIES.map((cat) => ({
      ...cat,
      items: cat.tools
        .map((tool) => toolchainMap.get(tool))
        .filter((tc): tc is ToolchainResult => tc !== undefined),
    }))
  }, [toolchainMap])

  // Compact per-tool rows for the table view
  const toolchainTableData = useMemo(() => {
    return groupedCategories.map((cat) => ({
      category: cat.name,
      tools: cat.items.map((tc) => ({
        tool: tc.tool,
        status: tc.status,
        version: getPrimaryVersion(tc),
        vulns: tc.vulnerabilities?.length || 0,
        outdated: tc.outdated?.length || 0,
        issues: tc.issues?.length || 0,
      })),
    }))
  }, [groupedCategories])

  const totalTools = groupedCategories.reduce(
    (sum, cat) => sum + cat.items.length,
    0
  )

  // Donut chart data: toolchain status distribution
  const STATUS_COLORS: Record<string, string> = {
    pass: "#22c55e",
    warn: "#eab308",
    fail: "#ef4444",
    skip: "#71717a",
  }
  const statusPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const cat of groupedCategories) {
      for (const tc of cat.items) {
        const key = statusKey(tc.status)
        counts[key] = (counts[key] || 0) + 1
      }
    }
    return Object.entries(counts)
      .filter(([, count]) => count > 0)
      .map(([name, value]) => ({
        name: name.charAt(0).toUpperCase() + name.slice(1),
        value,
        fill: STATUS_COLORS[name],
      }))
  }, [groupedCategories])

  // Donut chart data: vulnerabilities per toolchain
  const vulnPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const cat of groupedCategories) {
      for (const tc of cat.items) {
        const count = tc.vulnerabilities?.length || 0
        if (count > 0) counts[tc.tool] = (counts[tc.tool] || 0) + count
      }
    }
    return Object.entries(counts)
      .map(([name, value], i) => ({
        name: displayName(name),
        value,
        fill: CHART_PALETTE[i % CHART_PALETTE.length],
      }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 10)
  }, [groupedCategories])

  // Donut chart data: outdated packages per toolchain
  const outdatedPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const cat of groupedCategories) {
      for (const tc of cat.items) {
        const count = tc.outdated?.length || 0
        if (count > 0) counts[tc.tool] = (counts[tc.tool] || 0) + count
      }
    }
    return Object.entries(counts)
      .map(([name, value], i) => ({
        name: displayName(name),
        value,
        fill: CHART_PALETTE[(i + 3) % CHART_PALETTE.length],
      }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 10)
  }, [groupedCategories])

  const totalVulns = vulnPieData.reduce((sum, d) => sum + d.value, 0)
  const totalOutdated = outdatedPieData.reduce((sum, d) => sum + d.value, 0)

  // Aggregated vulnerability stats for the summary cards + severity breakdown
  const severityCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const cat of groupedCategories) {
      for (const tc of cat.items) {
        for (const v of tc.vulnerabilities || []) {
          const sev = v.severity.toLowerCase()
          counts[sev] = (counts[sev] || 0) + 1
        }
      }
    }
    return counts
  }, [groupedCategories])

  // All vulnerabilities across toolchains
  const vulnStats = useMemo(() => {
    const total = Object.values(severityCounts).reduce((sum, n) => sum + n, 0)
    return {
      total,
      critical: severityCounts["critical"] || 0,
      high: severityCounts["high"] || 0,
      riskScore: Object.entries(severityCounts).reduce(
        (sum, [sev, count]) => sum + (RISK_WEIGHTS[sev] || 1) * count,
        0
      ),
    }
  }, [severityCounts])

  if (loading) {
    return (
      <div className="mx-auto max-w-7xl animate-in duration-700 fade-in">
        <ScanProgress loading={true} onRetry={() => fetchReport(true)} />
      </div>
    )
  }

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <Boxes className="h-8 w-8 text-foreground" />
            Environment Toolchains
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Package managers and runtimes detected in your environment.
          </p>
        </div>
        {totalTools > 0 && (
          <div className="flex items-end">
            <Tabs
              value={compactView ? "table" : "cards"}
              onValueChange={(v) => setCompactView(v === "table")}
            >
              <TabsList className="h-9">
                <TabsTrigger
                  value="table"
                  className="h-7 w-7 p-0"
                  title="Table view"
                >
                  <TableIcon className="h-4 w-4" />
                </TabsTrigger>
                <TabsTrigger
                  value="cards"
                  className="h-7 w-7 p-0"
                  title="Cards view"
                >
                  <LayoutGrid className="h-4 w-4" />
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        )}
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Toolchains
            </CardTitle>
            <Boxes className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">
              {totalTools}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Detected in your environment
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Risk Score
            </CardTitle>
            <ShieldAlert className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${
                vulnStats.critical > 0
                  ? "text-red-500"
                  : vulnStats.high > 0
                    ? "text-orange-500"
                    : "text-foreground"
              }`}
            >
              {vulnStats.riskScore}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Severity-weighted vulnerabilities
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Vulnerabilities
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${vulnStats.total > 0 ? "text-red-400" : "text-foreground"}`}
            >
              {vulnStats.total}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Across all toolchains
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Outdated
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${totalOutdated > 0 ? "text-blue-400" : "text-foreground"}`}
            >
              {totalOutdated}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Packages behind latest
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Charts Section */}
      {totalTools > 0 && (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Status Distribution</CardTitle>
              <CardDescription>
                Toolchain status across your environment.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <DonutChart data={statusPieData} />
            </CardContent>
          </Card>

          {totalVulns > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">
                  Vulnerabilities by Toolchain
                </CardTitle>
                <CardDescription>
                  Toolchains with the most vulnerabilities.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <DonutChart data={vulnPieData} />
              </CardContent>
            </Card>
          )}

          {totalOutdated > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">
                  Outdated by Toolchain
                </CardTitle>
                <CardDescription>
                  Toolchains with the most outdated packages.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <DonutChart data={outdatedPieData} />
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {totalTools === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-muted/50 py-12">
          <PackageOpen className="mb-4 h-12 w-12 text-neutral-600" />
          <p className="text-muted-foreground">No toolchains detected.</p>
        </div>
      ) : compactView ? (
        <Card>
          <CardHeader>
            <div className="flex items-start justify-between gap-4">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <Boxes className="h-4 w-4 shrink-0 text-muted-foreground" />
                  Toolchain Status
                </CardTitle>
                <CardDescription className="mt-1">
                  Per-tool status, versions, and issue counts.
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="overflow-hidden rounded-md border border-border">
              <Table>
                <TableHeader className="bg-muted/50">
                  <TableRow className="border-border hover:bg-transparent">
                    <TableHead className="w-[150px]">Tool</TableHead>
                    <TableHead className="w-[80px]">Status</TableHead>
                    <TableHead className="w-[120px]">Version</TableHead>
                    <TableHead className="w-[80px] text-center">
                      Vulns
                    </TableHead>
                    <TableHead className="w-[80px] text-center">
                      Outdated
                    </TableHead>
                    <TableHead className="w-[80px] text-center">
                      Issues
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {toolchainTableData.map((cat) => (
                    <Fragment key={cat.category}>
                      <TableRow className="border-border bg-muted/30 hover:bg-muted/30">
                        <TableCell
                          colSpan={6}
                          className="text-xs font-semibold tracking-wider text-muted-foreground uppercase"
                        >
                          {cat.category}
                        </TableCell>
                      </TableRow>
                      {cat.tools.map((t) => (
                        <TableRow
                          key={t.tool}
                          className="cursor-pointer border-border hover:bg-muted/50"
                          onClick={() => setOpenDialog(t.tool)}
                        >
                          <TableCell className="text-sm font-medium capitalize">
                            {displayName(t.tool)}
                          </TableCell>
                          <TableCell>{statusBadge(t.status)}</TableCell>
                          <TableCell className="font-mono text-xs text-muted-foreground">
                            {t.version}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.vulns > 0 ? (
                              <span className="text-sm font-semibold text-red-500">
                                {t.vulns}
                              </span>
                            ) : (
                              <span className="text-sm text-muted-foreground">
                                0
                              </span>
                            )}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.outdated > 0 ? (
                              <span className="text-sm font-semibold text-blue-500">
                                {t.outdated}
                              </span>
                            ) : (
                              <span className="text-sm text-muted-foreground">
                                0
                              </span>
                            )}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.issues > 0 ? (
                              <span className="text-sm font-semibold text-yellow-500">
                                {t.issues}
                              </span>
                            ) : (
                              <span className="text-sm text-muted-foreground">
                                0
                              </span>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </Fragment>
                  ))}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      ) : (
        groupedCategories.map((cat) => (
          <div key={cat.name} className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold tracking-tight text-foreground">
              {cat.name}
            </h2>
            {cat.items.length === 0 ? (
              <div className="flex items-center justify-center rounded-xl border border-dashed border-border/50 bg-muted/30 py-8">
                <p className="text-sm text-muted-foreground/60">
                  No {cat.name.toLowerCase()} toolchains detected.
                </p>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                {cat.items.map((tc) => (
                  <ToolchainCard
                    key={tc.tool}
                    tc={tc}
                    onViewDetails={() => setOpenDialog(tc.tool)}
                  />
                ))}
              </div>
            )}
          </div>
        ))
      )}

      {/* Page-level dialogs so row clicks work in both views */}
      {groupedCategories
        .flatMap((cat) => cat.items)
        .map((tc) => (
          <ToolchainDetailDialog
            key={tc.tool}
            tc={tc}
            open={openDialog === tc.tool}
            onOpenChange={(isOpen) => setOpenDialog(isOpen ? tc.tool : null)}
          />
        ))}
    </div>
  )
}
