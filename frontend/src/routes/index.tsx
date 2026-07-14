import { createFileRoute } from "@tanstack/react-router"
import { useState, useMemo } from "react"
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
  ShieldAlert,
  RefreshCw,
  Box,
  CheckCircle,
  Boxes,
  Clock,
  LayoutGrid,
  Table as TableIcon,
  Gauge,
} from "lucide-react"
import {
  Bar,
  BarChart,
  CartesianGrid,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import { Button } from "@/components/ui/button"
import { Toggle } from "@/components/ui/toggle"
import { ScanProgress } from "@/components/scan-progress"
import { PackageDetailDialog } from "@/components/package-detail-dialog"
import { SimpleChartTooltip } from "@/components/ui/chart"

export const Route = createFileRoute("/")({ component: App })
interface PackageInfo {
  name: string
  current: string
  latest: string
}

interface VulnerabilityInfo {
  package: string
  severity: string
  title: string
  cve?: string | null
  patched_version?: string
}




interface ToolCategory {
  name: string
  tools: string[]
}

const CATEGORIES: ToolCategory[] = [
  { name: "System & Runtime", tools: ["brew", "cargo", "docker", "pip", "gem"] },
  { name: "Web Development", tools: ["npm", "pnpm", "yarn", "bun", "deno"] },
  {
    name: "Project Tooling",
    tools: ["project", "security", "supply_chain", "audit", "ci"],
  },
]

function displayName(tool: string): string {
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

function statusBadge(status: string) {
  const s = status.toLowerCase()
  if (s.includes("fail") || s.includes("error")) {
    return (
      <Badge
        variant="destructive"
        className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none text-xs"
      >
        FAIL
      </Badge>
    )
  }
  if (s.includes("warn")) {
    return (
      <Badge
        variant="outline"
        className="border-yellow-500/30 text-yellow-500 bg-yellow-500/10 shadow-none text-xs"
      >
        WARN
      </Badge>
    )
  }
  if (s.includes("skip") || s.includes("not found")) {
    return (
      <Badge variant="outline" className="border-border text-muted-foreground shadow-none text-xs">
        SKIP
      </Badge>
    )
  }
  return (
    <Badge
      variant="outline"
      className="border-green-500/30 text-green-500 bg-green-500/10 shadow-none text-xs"
    >
      PASS
    </Badge>
  )
}

function severityColor(s: string): string {
  switch (s.toLowerCase()) {
    case "critical":
      return "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25"
    case "high":
      return "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25"
    case "medium":
    case "moderate":
      return "bg-amber-500/10 text-amber-600 border-amber-500/20 dark:bg-amber-500/15 dark:text-amber-400 dark:border-amber-500/25"
    case "low":
      return "bg-emerald-500/10 text-emerald-600 border-emerald-500/20 dark:bg-emerald-500/15 dark:text-emerald-400 dark:border-emerald-500/25"
    default:
      return "bg-muted text-muted-foreground border-border"
  }
}

function severityOrder(s: string): number {
  switch (s.toLowerCase()) {
    case "critical":
      return 0
    case "high":
      return 1
    case "medium":
      return 2
    case "low":
      return 3
    default:
      return 4
  }
}

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const diffHours = Math.floor(diffMins / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffSecs < 60) return "just now"
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`
  return `${Math.floor(diffDays / 365)}y ago`
}

function App() {
  const { report, loading, refetch: fetchReport } = useScanData()
  const [compactView, setCompactView] = useState(false)
  const [selectedPackage, setSelectedPackage] = useState<{ name: string; toolchain: string } | null>(null)
  const allVulnerabilities = useMemo(() => {
    if (!report?.results) return []
    const vulns: Array<VulnerabilityInfo & { toolchain: string }> = []
    Object.entries(report.results).forEach(([toolchain, data]: [string, any]) => {
      if (data.vulnerabilities) {
        data.vulnerabilities.forEach((v: VulnerabilityInfo) => {
          vulns.push({ ...v, toolchain })
        })
      }
    })
    return vulns.sort(
      (a, b) => severityOrder(a.severity) - severityOrder(b.severity),
    )
  }, [report])

  // Aggregate all outdated
  const allOutdated = useMemo(() => {
    if (!report?.results) return []
    const out: Array<PackageInfo & { toolchain: string }> = []
    Object.entries(report.results).forEach(([toolchain, data]: [string, any]) => {
      if (data.outdated) {
        data.outdated.forEach((o: PackageInfo) => {
          out.push({ ...o, toolchain })
        })
      }
    })
    return out
  }, [report])

  // Count active toolchains
  const activeToolchains = useMemo(() => {
    if (!report?.results) return { active: 0, total: 15 }
    const active = Object.values(report.results).filter(
      (r) => r.status && !r.status.toLowerCase().includes("skip"),
    ).length
    return { active, total: 15 }
  }, [report])

  // Severity breakdown
  const severityCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      const key = v.severity.toLowerCase()
      counts[key] = (counts[key] || 0) + 1
    }
    return counts
  }, [allVulnerabilities])

  const vulnCount = allVulnerabilities.length
  const outCount = allOutdated.length
  const outdatedColumns: ColumnDef<PackageInfo & { toolchain: string }, unknown>[] = [
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="font-medium capitalize text-muted-foreground/80 text-sm">
          {row.original.toolchain}
        </span>
      ),
    },
    {
      accessorKey: "name",
      header: "Package",
      cell: ({ row }) => (
        <span
          className="font-mono text-sm text-primary underline underline-offset-2 hover:text-primary/80 cursor-pointer"
          onClick={() => setSelectedPackage({ name: row.original.name, toolchain: row.original.toolchain })}
        >
          {row.original.name}
        </span>
      ),
    },
    {
      accessorKey: "current",
      header: "Current",
      cell: ({ row }) => (
        <span className="font-mono text-sm text-muted-foreground">
          {row.original.current}
        </span>
      ),
    },
    {
      accessorKey: "latest",
      header: "Latest",
      cell: ({ row }) => (
        <span className="font-mono text-sm text-green-500 font-medium">
          {row.original.latest}
        </span>
      ),
    },
  ]

  const topVulnColumns: ColumnDef<VulnerabilityInfo & { toolchain: string }, unknown>[] = [
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="font-medium capitalize text-muted-foreground/80 text-sm">
          {row.original.toolchain}
        </span>
      ),
    },
    {
      accessorKey: "package",
      header: "Package",
      cell: ({ row }) => (
        <span
          className="font-mono text-sm text-primary underline underline-offset-2 hover:text-primary/80 cursor-pointer"
          onClick={() => setSelectedPackage({ name: row.original.package, toolchain: row.original.toolchain })}
        >
          {row.original.package || "Unknown"}
        </span>
      ),
    },
    {
      accessorKey: "severity",
      header: "Severity",
      sortingFn: (a, b) => severityOrder(a.original.severity) - severityOrder(b.original.severity),
      cell: ({ row }) => (
        <Badge
          variant="outline"
          className={`shadow-none ${severityColor(row.original.severity)}`}
        >
          {row.original.severity}
        </Badge>
      ),
    },
    {
      accessorKey: "title",
      header: "Description",
      cell: ({ row }) => (
        <span className="text-muted-foreground text-sm">
          {row.original.title || "Security vulnerability found"}
        </span>
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
  ]

  // Project tooling signals
  const projectTooling = useMemo(() => {
    if (!report?.results) {
      return {
        projectStatus: "skipped",
        securityStatus: "skipped",
        auditStatus: "skipped",
        supplyStatus: "skipped",
        projectOutdated: 0,
        vulnCount: 0,
        auditCount: 0,
        riskCount: 0,
      }
    }
    const project = report.results["project"]
    const security = report.results["security"]
    const audit = report.results["audit"]
    const supply = report.results["supply_chain"]

    return {
      projectStatus: project?.status || "skipped",
      securityStatus: security?.status || "skipped",
      auditStatus: audit?.status || "skipped",
      supplyStatus: supply?.status || "skipped",
      projectOutdated: project?.outdated?.length || 0,
      vulnCount: security?.vulnerabilities?.length || 0,
      auditCount: audit?.audit_items?.length || 0,
      riskCount: supply?.supply_chain_risks?.length || 0,
    }
  }, [report])
  const auditCount = projectTooling.auditCount
  const riskCount = projectTooling.riskCount
  const healthScore = Math.max(0, 100 - vulnCount * 10 - outCount * 5 - auditCount * 3 - riskCount * 5)

  // Signal distribution for bar chart
  const signalData = useMemo(() => {
    return [
      {
        name: "Outdated",
        value: projectTooling.projectOutdated,
        fill: "#60a5fa",
      },
      {
        name: "Critical",
        value: severityCounts["critical"] || 0,
        fill: "#f87171",
      },
      {
        name: "High",
        value: severityCounts["high"] || 0,
        fill: "#fb923c",
      },
      {
        name: "Medium",
        value: severityCounts["medium"] || 0,
        fill: "#facc15",
      },
      {
        name: "Audit",
        value: projectTooling.auditCount,
        fill: "#a78bfa",
      },
    ]
  }, [severityCounts, projectTooling])

  // Toolchain status data grouped by category
  const toolchainTableData = useMemo(() => {
    if (!report?.results) return []
    return CATEGORIES.map((cat) => ({
      category: cat.name,
      tools: cat.tools
        .map((tool) => {
          const data = report.results?.[tool]
          if (!data) return null
          return {
            tool,
            status: data.status,
            version:
              data.version ||
              data.node_version ||
              data.python_version ||
              data.rustc_version ||
              data.cargo_version ||
              "-",
            vulns: data.vulnerabilities?.length || 0,
            outdated: data.outdated?.length || 0,
            issues: data.issues?.length || 0,
          }
        })
        .filter(
          (t): t is NonNullable<typeof t> => t !== null,
        ),
    }))
  }, [report])


  if (loading) {
    return (
      <div className="max-w-7xl mx-auto animate-in fade-in duration-700">
        <ScanProgress loading={true} onRetry={() => fetchReport(true)} />
      </div>
    )
  }

  if (!report) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] gap-4">
        <ShieldAlert className="w-12 h-12 text-muted-foreground" />
        <h2 className="text-xl font-medium tracking-tight">
          Failed to load environment report
        </h2>
        <Button variant="outline" onClick={() => fetchReport(true)} className="gap-2">
          <RefreshCw className="w-4 h-4" />
          Retry Scan
        </Button>
      </div>
    )
  }

  return (
    <div className="max-w-7xl mx-auto flex flex-col gap-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">
            Workspace Overview
          </h1>
          <p className="text-sm text-muted-foreground mt-2 flex items-center gap-2">
            <CheckCircle className="w-4 h-4 text-muted-foreground" />
            Scanned at{" "}
            {report.timestamp
              ? new Date(report.timestamp).toLocaleString()
              : new Date().toLocaleTimeString()}
          </p>
        </div>
        <div className="flex flex-col md:flex-row items-start md:items-end gap-3 md:gap-4">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Clock className="w-4 h-4" />
            <span>Last scanned {report.timestamp ? formatRelativeTime(report.timestamp) : "just now"}</span>
          </div>
          <Button
            variant="outline"
            className="gap-2 shadow-xs"
            onClick={() => fetchReport(true)}
          >
            <RefreshCw className="w-4 h-4" />
            Rescan Now
          </Button>
        </div>
      </div>

      {/* Top Metric Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {/* Risk Score Card */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Risk Score
            </CardTitle>
            <Gauge className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent className="flex flex-col items-center gap-3 py-2">
            <div className="relative w-24 h-24">
              <svg className="w-full h-full transform -rotate-90">
                <circle
                  cx="48"
                  cy="48"
                  r="42"
                  stroke="hsl(var(--muted))"
                  strokeWidth="6"
                  fill="none"
                />
                <circle
                  cx="48"
                  cy="48"
                  r="42"
                  stroke={
                    healthScore > 70
                      ? "hsl(var(--green-500))"
                      : healthScore > 40
                      ? "hsl(var(--yellow-500))"
                      : "hsl(var(--red-500))"
                  }
                  strokeWidth="6"
                  fill="none"
                  strokeDasharray={264}
                  strokeDashoffset={264 - (healthScore / 100) * 264}
                  strokeLinecap="round"
                  className="transition-all duration-500 ease-out"
                />
              </svg>
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-3xl font-bold text-foreground">
                  {healthScore}
                </span>
              </div>
            </div>
            <p className="text-xs text-muted-foreground/60 text-center">
              {healthScore > 70
                ? "Healthy"
                : healthScore > 40
                ? "Needs attention"
                : "Critical"}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Vulnerabilities
            </CardTitle>
            <ShieldAlert className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${vulnCount > 0 ? "text-red-500" : "text-foreground"}`}
            >
              {vulnCount}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Across toolchains
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Outdated
            </CardTitle>
            <Box className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${outCount > 0 ? "text-blue-500" : "text-foreground"}`}
            >
              {outCount}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Updates available
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Active Tools
            </CardTitle>
            <Boxes className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">
              {activeToolchains.active}
              <span className="text-lg text-muted-foreground">
                /{activeToolchains.total}
              </span>
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Toolchains detected
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Project Tooling Readiness */}
      <Card>
        <CardHeader>
          <CardTitle>Project Tooling Readiness</CardTitle>
          <CardDescription>
            Signal distribution across project tooling subsystems.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-6">
          {/* Per-signal status */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="rounded-lg border border-border/50 bg-muted/50 p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground/60 uppercase tracking-wider font-semibold">
                Project
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.projectStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.projectOutdated} outdated
                </span>
              </div>
            </div>
            <div className="rounded-lg border border-border/50 bg-muted/50 p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground/60 uppercase tracking-wider font-semibold">
                Security
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.securityStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.vulnCount} vulns
                </span>
              </div>
            </div>
            <div className="rounded-lg border border-border/50 bg-muted/50 p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground/60 uppercase tracking-wider font-semibold">
                Audit
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.auditStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.auditCount} flagged
                </span>
              </div>
            </div>
            <div className="rounded-lg border border-border/50 bg-muted/50 p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground/60 uppercase tracking-wider font-semibold">
                Supply
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.supplyStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.riskCount} risks
                </span>
              </div>
            </div>
          </div>

          {/* Signal Distribution Bar Chart */}
          <div>
            <h3 className="text-sm font-medium text-muted-foreground mb-3">
              Signal Distribution
            </h3>
            <div className="h-[200px] w-full">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart
                  data={signalData}
                  margin={{ top: 10, right: 10, left: -20, bottom: 0 }}
                >
                  <CartesianGrid
                    strokeDasharray="3 3"
                    stroke="hsl(var(--border))"
                    vertical={false}
                  />
                  <XAxis
                    dataKey="name"
                    stroke="#a1a1aa"
                    tick={{ fill: "#a1a1aa", fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                  />
                  <YAxis
                    stroke="#a1a1aa"
                    tick={{ fill: "#a1a1aa", fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                  />

                  <Tooltip
                    content={<SimpleChartTooltip />}
                    cursor={{ fill: "rgba(255,255,255,0.05)" }}
                  />
                  <Bar
                    dataKey="value"
                    name="Count"
                    radius={[4, 4, 0, 0]}
                    maxBarSize={50}
                  >
                    {signalData.map((entry, index) => (
                      <rect key={index} fill={entry.fill} />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Toolchain Status */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div className="flex items-center gap-2">
            <CardTitle>Toolchain Status</CardTitle>
            <CardDescription className="ml-2 text-xs text-muted-foreground hidden sm:inline">
              Per-tool status, versions, and issue counts.
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Toggle
              variant="outline"
              size="sm"
              pressed={compactView}
              onPressedChange={setCompactView}
              aria-label="Toggle compact view"
              className="gap-1.5"
            >
              <TableIcon className="w-4 h-4" />
              <LayoutGrid className="w-4 h-4" />
              <span className="hidden sm:inline text-xs font-medium">
                {compactView ? "Table" : "Cards"}
              </span>
            </Toggle>
          </div>
        </CardHeader>
        <CardContent>
          {compactView ? (
            <div className="rounded-md border border-border overflow-hidden">
              <Table>
                <TableHeader className="bg-muted/50">
                  <TableRow className="border-border hover:bg-transparent">
                    <TableHead className="w-[150px]">Tool</TableHead>
                    <TableHead className="w-[80px]">Status</TableHead>
                    <TableHead className="w-[120px]">Version</TableHead>
                    <TableHead className="w-[80px] text-center">Vulns</TableHead>
                    <TableHead className="w-[80px] text-center">Outdated</TableHead>
                    <TableHead className="w-[80px] text-center">Issues</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {toolchainTableData.map((cat) => (
                    <>
                      <TableRow
                        key={`header-${cat.category}`}
                        className="border-border bg-muted/30 hover:bg-muted/30"
                      >
                        <TableCell
                          colSpan={6}
                          className="font-semibold text-xs text-muted-foreground uppercase tracking-wider"
                        >
                          {cat.category}
                        </TableCell>
                      </TableRow>
                      {cat.tools.map((t) => (
                        <TableRow
                          key={t.tool}
                          className="border-border hover:bg-muted/50"
                        >
                          <TableCell className="font-medium text-sm capitalize">
                            {displayName(t.tool)}
                          </TableCell>
                          <TableCell>{statusBadge(t.status)}</TableCell>
                          <TableCell className="font-mono text-xs text-muted-foreground">
                            {t.version}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.vulns > 0 ? (
                              <span className="text-red-500 font-semibold text-sm">
                                {t.vulns}
                              </span>
                            ) : (
                              <span className="text-muted-foreground text-sm">0</span>
                            )}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.outdated > 0 ? (
                              <span className="text-blue-500 font-semibold text-sm">
                                {t.outdated}
                              </span>
                            ) : (
                              <span className="text-muted-foreground text-sm">0</span>
                            )}
                          </TableCell>
                          <TableCell className="text-center">
                            {t.issues > 0 ? (
                              <span className="text-yellow-500 font-semibold text-sm">
                                {t.issues}
                              </span>
                            ) : (
                              <span className="text-muted-foreground text-sm">0</span>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : (
            <div className="space-y-4">
              {toolchainTableData.map((cat) => (
                <div key={cat.category}>
                  <h4 className="font-semibold text-xs text-muted-foreground uppercase tracking-wider mb-3">
                    {cat.category}
                  </h4>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {cat.tools.map((t) => (
                      <Card key={t.tool} className="p-4 flex flex-col gap-3">
                        <div className="flex items-start justify-between gap-2">
                          <div>
                            <span className="font-medium text-sm capitalize">
                              {displayName(t.tool)}
                            </span>
                            <span className="font-mono text-xs text-muted-foreground ml-2">
                              {t.version}
                            </span>
                          </div>
                          {statusBadge(t.status)}
                        </div>
                        <div className="grid grid-cols-3 gap-2 text-center pt-2 border-t border-border/50">
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.vulns > 0 ? "text-red-500" : "text-muted-foreground"
                              }`}
                            >
                              {t.vulns}
                            </div>
                            <div className="text-xs text-muted-foreground">Vulns</div>
                          </div>
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.outdated > 0 ? "text-blue-500" : "text-muted-foreground"
                              }`}
                            >
                              {t.outdated}
                            </div>
                            <div className="text-xs text-muted-foreground">Outdated</div>
                          </div>
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.issues > 0 ? "text-yellow-500" : "text-muted-foreground"
                              }`}
                            >
                              {t.issues}
                            </div>
                            <div className="text-xs text-muted-foreground">Issues</div>
                          </div>
                        </div>
                      </Card>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Outdated Packages */}
      <Card>
        <CardHeader>
          <CardTitle>Outdated Packages</CardTitle>
          <CardDescription>
            Packages with available updates across all toolchains.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {allOutdated.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
              <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
              <p>All packages are up to date!</p>
            </div>
          ) : (
            <DataTable
              columns={outdatedColumns}
              data={allOutdated}
              defaultPageSize={8}
              pageSizeOptions={[5, 8, 15, 50]}
            />
          )}
        </CardContent>
      </Card>

      {/* Severity Breakdown */}
      <Card>
        <CardHeader>
          <CardTitle>Severity Breakdown</CardTitle>
          <CardDescription>
            Top vulnerabilities requiring attention.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {/* Severity badges */}
          <div className="flex flex-wrap gap-3">
            {Object.entries(severityCounts).map(([sev, count]) => (
              <Badge
                key={sev}
                variant="outline"
                className={`shadow-none ${severityColor(sev)}`}
              >
                {sev.charAt(0).toUpperCase() + sev.slice(1)}: {count}
              </Badge>
            ))}
            {vulnCount === 0 && (
              <Badge
                variant="outline"
                className="border-green-500/30 text-green-500 bg-green-500/10 shadow-none"
              >
                No vulnerabilities detected
              </Badge>
            )}
          </div>

          {/* Top vulns table */}
          {vulnCount > 0 && (
            <DataTable
              columns={topVulnColumns}
              data={allVulnerabilities}
              defaultPageSize={5}
              pageSizeOptions={[5, 10, 20]}
            />
          )}
        </CardContent>
      </Card>

      {selectedPackage && (
        <PackageDetailDialog
          packageName={selectedPackage.name}
          toolchain={selectedPackage.toolchain}
          open={!!selectedPackage}
          onClose={() => setSelectedPackage(null)}
        />
      )}
    </div>
  )
}
