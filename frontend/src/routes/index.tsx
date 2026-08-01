import { createFileRoute, useNavigate } from "@tanstack/react-router"
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
  PackageMinus,
  Search,
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
import { Input } from "@/components/ui/input"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScanProgress } from "@/components/scan-progress"
import { PackageDetailDialog } from "@/components/package-detail-dialog"
import {
  enrichOutdated,
  sourceColor,
  sourceLabel,
  updateTypeColor,
  updateTypeLabel,
  type OutdatedPackage,
} from "@/lib/outdated"
import { SimpleChartTooltip } from "@/components/ui/chart"
import {
  siDocker,
  siNpm,
  siPnpm,
  siYarn,
  siBun,
  siDeno,
  siPython,
  siRubygems,
  siRust,
  siHomebrew,
  siApple,
  siGithub,
} from "simple-icons"
import { cn, formatRelativeTime } from "@/lib/utils"

export const Route = createFileRoute("/")({ component: App })

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
  const isDark =
    invertInDark &&
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches

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

const TOOL_ICONS: Record<
  string,
  {
    icon: { path: string; hex: string }
    fallback?: React.ReactNode
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
  project: { icon: siApple, invertInDark: true },
  security: {
    icon: {
      path: "M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z",
      hex: "71717a",
    },
    fallback: <Boxes className="h-5 w-5 text-muted-foreground" />,
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
    fallback: <Box className="h-5 w-5 text-muted-foreground" />,
  },
  ci: { icon: siGithub, invertInDark: true },
}

function getToolIcon(tool: string) {
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
        className="border-red-500/20 bg-red-500/10 text-xs text-red-500 shadow-none"
      >
        FAIL
      </Badge>
    )
  }
  if (s.includes("warn")) {
    return (
      <Badge
        variant="outline"
        className="border-yellow-500/30 bg-yellow-500/10 text-xs text-yellow-500 shadow-none"
      >
        WARN
      </Badge>
    )
  }
  if (s.includes("skip") || s.includes("not found")) {
    return (
      <Badge
        variant="outline"
        className="border-border text-xs text-muted-foreground shadow-none"
      >
        SKIP
      </Badge>
    )
  }
  return (
    <Badge
      variant="outline"
      className="border-green-500/30 bg-green-500/10 text-xs text-green-500 shadow-none"
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

function App() {
  const { report, loading, refetch: fetchReport } = useScanData()
  const navigate = useNavigate()
  const [compactView, setCompactView] = useState(false)
  const [selectedPackage, setSelectedPackage] = useState<{
    name: string
    toolchain: string
  } | null>(null)
  const [outdatedSearch, setOutdatedSearch] = useState("")
  const [vulnSearch, setVulnSearch] = useState("")
  const allVulnerabilities = useMemo(() => {
    if (!report?.results) return []
    const vulns: Array<VulnerabilityInfo & { toolchain: string }> = []
    Object.entries(report.results).forEach(
      ([toolchain, data]: [string, any]) => {
        if (data.vulnerabilities) {
          data.vulnerabilities.forEach((v: VulnerabilityInfo) => {
            vulns.push({ ...v, toolchain })
          })
        }
      }
    )
    return vulns.sort(
      (a, b) => severityOrder(a.severity) - severityOrder(b.severity)
    )
  }, [report])

  // Aggregate all outdated
  const allOutdated = useMemo((): OutdatedPackage[] => {
    return enrichOutdated(report?.results)
  }, [report])

  const filteredOutdated = useMemo(() => {
    if (!outdatedSearch) return allOutdated
    const q = outdatedSearch.toLowerCase()
    return allOutdated.filter(
      (o) =>
        o.name.toLowerCase().includes(q) ||
        o.toolchain.toLowerCase().includes(q) ||
        o.source.toLowerCase().includes(q)
    )
  }, [allOutdated, outdatedSearch])

  const filteredVulnerabilities = useMemo(() => {
    if (!vulnSearch) return allVulnerabilities
    const q = vulnSearch.toLowerCase()
    return allVulnerabilities.filter(
      (v) =>
        v.package.toLowerCase().includes(q) ||
        (v.title || "").toLowerCase().includes(q) ||
        v.toolchain.toLowerCase().includes(q)
    )
  }, [allVulnerabilities, vulnSearch])

  // Count active toolchains
  const activeToolchains = useMemo(() => {
    if (!report?.results) return { active: 0, total: 15 }
    const active = Object.values(report.results).filter(
      (r) => r.status && !r.status.toLowerCase().includes("skip")
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
  const updateTypeCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const o of allOutdated) {
      counts[o.updateType] = (counts[o.updateType] || 0) + 1
    }
    return counts
  }, [allOutdated])
  const outdatedColumns: ColumnDef<OutdatedPackage, unknown>[] = [
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="text-sm font-medium text-muted-foreground/80 capitalize">
          {row.original.toolchain}
        </span>
      ),
    },
    {
      accessorKey: "name",
      header: "Package",
      cell: ({ row }) => (
        <span
          className="cursor-pointer font-mono text-sm text-primary underline underline-offset-2 hover:text-primary/80"
          onClick={() =>
            setSelectedPackage({
              name: row.original.name,
              toolchain: row.original.toolchain,
            })
          }
        >
          {row.original.name}
        </span>
      ),
    },
    {
      accessorKey: "source",
      header: "Source",
      cell: ({ row }) => (
        <Badge
          variant="outline"
          className={`shadow-none ${sourceColor(row.original.source)}`}
        >
          {sourceLabel(row.original.source)}
        </Badge>
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
        <span className="font-mono text-sm font-medium text-green-500">
          {row.original.latest}
        </span>
      ),
    },
    {
      accessorKey: "updateType",
      header: "Update Type",
      cell: ({ row }) => (
        <Badge
          variant="outline"
          className={`shadow-none ${updateTypeColor(row.original.updateType)}`}
        >
          {updateTypeLabel(row.original.updateType)}
        </Badge>
      ),
    },
  ]

  const topVulnColumns: ColumnDef<
    VulnerabilityInfo & { toolchain: string },
    unknown
  >[] = [
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="text-sm font-medium text-muted-foreground/80 capitalize">
          {row.original.toolchain}
        </span>
      ),
    },
    {
      accessorKey: "package",
      header: "Package",
      cell: ({ row }) => (
        <span
          className="cursor-pointer font-mono text-sm text-primary underline underline-offset-2 hover:text-primary/80"
          onClick={() =>
            setSelectedPackage({
              name: row.original.package,
              toolchain: row.original.toolchain,
            })
          }
        >
          {row.original.package || "Unknown"}
        </span>
      ),
    },
    {
      accessorKey: "severity",
      header: "Severity",
      sortingFn: (a, b) =>
        severityOrder(a.original.severity) - severityOrder(b.original.severity),
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
        <span className="text-sm text-muted-foreground">
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
  const healthScore = Math.max(
    0,
    100 - vulnCount * 10 - outCount * 5 - auditCount * 3 - riskCount * 5
  )

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
        .filter((t): t is NonNullable<typeof t> => t !== null),
    }))
  }, [report])

  if (loading) {
    return (
      <div className="mx-auto max-w-7xl animate-in duration-700 fade-in">
        <ScanProgress loading={true} onRetry={() => fetchReport(true)} />
      </div>
    )
  }

  if (!report) {
    return (
      <div className="flex min-h-[50vh] flex-col items-center justify-center gap-4">
        <ShieldAlert className="h-12 w-12 text-muted-foreground" />
        <h2 className="text-xl font-medium tracking-tight">
          Failed to load environment report
        </h2>
        <Button
          variant="outline"
          onClick={() => fetchReport(true)}
          className="gap-2"
        >
          <RefreshCw className="h-4 w-4" />
          Retry Scan
        </Button>
      </div>
    )
  }

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      {/* Header */}
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">
            Workspace Overview
          </h1>
          <p className="mt-2 flex items-center gap-2 text-sm text-muted-foreground">
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
            Scanned at{" "}
            {report.timestamp
              ? new Date(report.timestamp).toLocaleString()
              : new Date().toLocaleTimeString()}
          </p>
        </div>
        <div className="flex flex-col items-start gap-3 md:flex-row md:items-end md:gap-4">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Clock className="h-4 w-4" />
            <span>
              Last scanned{" "}
              {report.timestamp
                ? formatRelativeTime(report.timestamp)
                : "just now"}
            </span>
          </div>
          <Button
            variant="outline"
            className="gap-2 shadow-xs"
            onClick={() => fetchReport(true)}
          >
            <RefreshCw className="h-4 w-4" />
            Rescan Now
          </Button>
        </div>
      </div>

      {/* Top Metric Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        {/* Risk Score Card */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Risk Score
            </CardTitle>
            <Gauge className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${
                healthScore > 70
                  ? "text-green-500"
                  : healthScore > 40
                    ? "text-yellow-500"
                    : "text-red-500"
              }`}
            >
              {healthScore}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
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
            <ShieldAlert className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${vulnCount > 0 ? "text-red-500" : "text-foreground"}`}
            >
              {vulnCount}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Across toolchains
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Outdated
            </CardTitle>
            <Box className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${outCount > 0 ? "text-blue-500" : "text-foreground"}`}
            >
              {outCount}
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Updates available
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Active Tools
            </CardTitle>
            <Boxes className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">
              {activeToolchains.active}
              <span className="text-lg text-muted-foreground">
                /{activeToolchains.total}
              </span>
            </div>
            <p className="mt-1 text-xs text-muted-foreground/60">
              Toolchains detected
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Project Tooling Readiness */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Gauge className="h-4 w-4 shrink-0 text-muted-foreground" />
            Project Tooling Readiness
          </CardTitle>
          <CardDescription>
            Signal distribution across project tooling subsystems.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-6">
          {/* Per-signal status */}
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <div className="flex flex-col gap-1 rounded-lg border border-border/50 bg-muted/50 p-3">
              <span className="text-xs font-semibold tracking-wider text-muted-foreground/60 uppercase">
                Project
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.projectStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.projectOutdated} outdated
                </span>
              </div>
            </div>
            <div className="flex flex-col gap-1 rounded-lg border border-border/50 bg-muted/50 p-3">
              <span className="text-xs font-semibold tracking-wider text-muted-foreground/60 uppercase">
                Security
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.securityStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.vulnCount} vulns
                </span>
              </div>
            </div>
            <div className="flex flex-col gap-1 rounded-lg border border-border/50 bg-muted/50 p-3">
              <span className="text-xs font-semibold tracking-wider text-muted-foreground/60 uppercase">
                Audit
              </span>
              <div className="flex items-center gap-2">
                {statusBadge(projectTooling.auditStatus)}
                <span className="text-xs text-muted-foreground">
                  {projectTooling.auditCount} flagged
                </span>
              </div>
            </div>
            <div className="flex flex-col gap-1 rounded-lg border border-border/50 bg-muted/50 p-3">
              <span className="text-xs font-semibold tracking-wider text-muted-foreground/60 uppercase">
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
            <h3 className="mb-3 text-sm font-medium text-muted-foreground">
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
        </CardHeader>
        <CardContent>
          {compactView ? (
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
                    <>
                      <TableRow
                        key={`header-${cat.category}`}
                        className="border-border bg-muted/30 hover:bg-muted/30"
                      >
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
                          onClick={() =>
                            navigate({
                              to: "/toolchains",
                              search: { open: t.tool },
                            })
                          }
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
                    </>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : (
            <div className="space-y-4">
              {toolchainTableData.map((cat) => (
                <div key={cat.category}>
                  <h4 className="mb-3 text-xs font-semibold tracking-wider text-muted-foreground uppercase">
                    {cat.category}
                  </h4>
                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    {cat.tools.map((t) => (
                      <Card
                        key={t.tool}
                        className="flex cursor-pointer flex-col gap-3 p-4 transition-colors hover:bg-muted/50"
                        onClick={() =>
                          navigate({
                            to: "/toolchains",
                            search: { open: t.tool },
                          })
                        }
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex items-center gap-2">
                            {getToolIcon(t.tool)}
                            <div>
                              <span className="text-sm font-medium capitalize">
                                {displayName(t.tool)}
                              </span>
                              <span className="ml-2 font-mono text-xs text-muted-foreground">
                                {t.version}
                              </span>
                            </div>
                          </div>
                          {statusBadge(t.status)}
                        </div>
                        <div className="grid grid-cols-3 gap-2 border-t border-border/50 pt-2 text-center">
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.vulns > 0
                                  ? "text-red-500"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {t.vulns}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              Vulns
                            </div>
                          </div>
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.outdated > 0
                                  ? "text-blue-500"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {t.outdated}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              Outdated
                            </div>
                          </div>
                          <div>
                            <div
                              className={`text-lg font-semibold ${
                                t.issues > 0
                                  ? "text-yellow-500"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {t.issues}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              Issues
                            </div>
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
        <CardHeader className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <CardTitle className="flex items-center gap-2">
              <PackageMinus className="h-4 w-4 shrink-0 text-muted-foreground" />
              Outdated Packages
            </CardTitle>
            <CardDescription>
              Packages with available updates across all toolchains.
            </CardDescription>
          </div>
          <div className="relative w-full md:w-72">
            <Search className="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground/60" />
            <Input
              type="text"
              placeholder="Search packages, toolchains, sources..."
              className="border-border bg-background/50 pl-9"
              value={outdatedSearch}
              onChange={(e) => setOutdatedSearch(e.target.value)}
            />
          </div>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {allOutdated.length > 0 && (
            <div className="flex flex-wrap gap-3">
              {(["major", "minor", "patch", "unknown"] as const)
                .filter((t) => (updateTypeCounts[t] || 0) > 0)
                .map((t) => (
                  <Badge
                    key={t}
                    variant="outline"
                    className={`shadow-none ${updateTypeColor(t)}`}
                  >
                    {t === "unknown" ? "Unknown" : updateTypeLabel(t)}:{" "}
                    {updateTypeCounts[t]}
                  </Badge>
                ))}
            </div>
          )}
          {filteredOutdated.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
              <CheckCircle className="mb-4 h-12 w-12 text-green-500/50" />
              <p>
                {outdatedSearch
                  ? "No packages match your search."
                  : "All packages are up to date!"}
              </p>
            </div>
          ) : (
            <DataTable
              columns={outdatedColumns}
              data={filteredOutdated}
              defaultPageSize={8}
              pageSizeOptions={[5, 8, 15, 50]}
            />
          )}
        </CardContent>
      </Card>

      {/* Severity Breakdown */}
      <Card>
        <CardHeader className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <CardTitle className="flex items-center gap-2">
              <ShieldAlert className="h-4 w-4 shrink-0 text-muted-foreground" />
              Severity Breakdown
            </CardTitle>
            <CardDescription>
              Top vulnerabilities requiring attention.
            </CardDescription>
          </div>
          <div className="relative w-full md:w-72">
            <Search className="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground/60" />
            <Input
              type="text"
              placeholder="Search packages..."
              className="border-border bg-background/50 pl-9"
              value={vulnSearch}
              onChange={(e) => setVulnSearch(e.target.value)}
            />
          </div>
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
                className="border-green-500/30 bg-green-500/10 text-green-500 shadow-none"
              >
                No vulnerabilities detected
              </Badge>
            )}
          </div>

          {/* Top vulns table */}
          {vulnCount > 0 &&
            (filteredVulnerabilities.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
                <Search className="mb-4 h-12 w-12 opacity-50" />
                <p>No vulnerabilities match your search.</p>
              </div>
            ) : (
              <DataTable
                columns={topVulnColumns}
                data={filteredVulnerabilities}
                defaultPageSize={5}
                pageSizeOptions={[5, 10, 20]}
              />
            ))}
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
