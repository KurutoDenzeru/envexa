import { createFileRoute } from "@tanstack/react-router"
import { useEffect, useState, useMemo } from "react"
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
  Activity,
  Boxes,
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
import { Skeleton } from "@/components/ui/skeleton"
import { Button } from "@/components/ui/button"
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationNext,
  PaginationPrevious,
  PaginationLink,
} from "@/components/ui/pagination"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

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

interface SupplyChainRisk {
  package: string
  risk_type: string
  description: string
}

interface AuditItem {
  name: string
  current: string
  note: string
}

interface ScanResult {
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
  supply_chain_risks?: SupplyChainRisk[]
  audit_items?: AuditItem[]
}

interface ScanReport {
  timestamp?: string
  results?: Record<string, ScanResult>
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
      return "bg-red-500/10 text-red-500 border-red-500/20"
    case "high":
      return "bg-orange-500/10 text-orange-500 border-orange-500/20"
    case "medium":
      return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"
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
  const [report, setReport] = useState<ScanReport | null>(null)
  const [loading, setLoading] = useState(true)
  const [vulnPage, setVulnPage] = useState(1)
  const [itemsPerPage, setItemsPerPage] = useState(5)

  const fetchReport = async () => {
    setLoading(true)
    try {
      const res = await fetch("/api/scan")
      const data: unknown = await res.json()
      setReport(data as ScanReport)
    } catch (e) {
      console.error("Failed to fetch report", e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchReport()
  }, [])

  // Aggregate all vulnerabilities
  const allVulnerabilities = useMemo(() => {
    if (!report?.results) return []
    const vulns: Array<VulnerabilityInfo & { toolchain: string }> = []
    Object.entries(report.results).forEach(([toolchain, data]) => {
      if (data.vulnerabilities) {
        data.vulnerabilities.forEach((v) => {
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
    Object.entries(report.results).forEach(([toolchain, data]) => {
      if (data.outdated) {
        data.outdated.forEach((o) => {
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
  const healthScore = Math.max(0, 100 - vulnCount * 10 - outCount * 2)

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

  // Pagination for top vulns
  const topVulnTotalPages = Math.ceil(vulnCount / itemsPerPage)
  const paginatedVulns = allVulnerabilities.slice(
    (vulnPage - 1) * itemsPerPage,
    vulnPage * itemsPerPage,
  )

  const renderPageNumbers = (
    currentPage: number,
    total: number,
    setP: (p: number) => void,
  ) => {
    const pages = []
    for (let i = 1; i <= total; i++) {
      if (
        i === 1 ||
        i === total ||
        (i >= currentPage - 1 && i <= currentPage + 1)
      ) {
        pages.push(
          <PaginationItem key={i}>
            <PaginationLink
              onClick={() => setP(i)}
              isActive={currentPage === i}
              className={
                currentPage === i
                  ? "bg-muted"
                  : "cursor-pointer hover:bg-muted/50"
              }
            >
              {i}
            </PaginationLink>
          </PaginationItem>,
        )
      } else if (i === currentPage - 2 || i === currentPage + 2) {
        pages.push(
          <PaginationItem key={i}>
            <span className="px-2 text-muted-foreground/60">...</span>
          </PaginationItem>,
        )
      }
    }
    return pages.filter(
      (item, index, self) =>
        item.key !== null &&
        self.findIndex((t) => t.key === item.key) === index,
    )
  }

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto flex flex-col gap-6 animate-in fade-in duration-700">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-64 bg-muted" />
            <Skeleton className="h-4 w-96 mt-3 bg-muted" />
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Skeleton className="h-28 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-28 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-28 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-28 w-full rounded-xl bg-muted/50" />
        </div>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-48 bg-muted" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-[200px] w-full bg-muted/50" />
          </CardContent>
        </Card>
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
        <Button variant="outline" onClick={fetchReport} className="gap-2">
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
        <Button
          variant="outline"
          className="gap-2 shadow-xs"
          onClick={fetchReport}
        >
          <RefreshCw className="w-4 h-4" />
          Rescan Now
        </Button>
      </div>

      {/* Top Metric Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Health
            </CardTitle>
            <Activity className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">{healthScore}%</div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              {healthScore > 80 ? "Optimal" : "Needs attention"}
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
                    cursor={{ fill: "rgba(255,255,255,0.05)" }}
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "8px",
                      color: "hsl(var(--foreground))",
                    }}
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

      {/* Toolchain Status Table */}
      <Card>
        <CardHeader>
          <CardTitle>Toolchain Status</CardTitle>
          <CardDescription>
            Per-tool status, versions, and issue counts.
          </CardDescription>
        </CardHeader>
        <CardContent>
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
            <>
              <div className="rounded-md border border-border overflow-hidden">
                <Table>
                  <TableHeader className="bg-muted/50">
                    <TableRow className="border-border hover:bg-transparent">
                      <TableHead className="w-[120px]">Toolchain</TableHead>
                      <TableHead className="w-[160px]">Package</TableHead>
                      <TableHead className="w-[90px]">Severity</TableHead>
                      <TableHead>Description</TableHead>
                      <TableHead className="w-[120px]">CVE</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {paginatedVulns.map((v, idx) => (
                      <TableRow
                        key={idx}
                        className="border-border hover:bg-muted/50"
                      >
                        <TableCell className="font-medium capitalize text-muted-foreground/80 text-sm">
                          {v.toolchain}
                        </TableCell>
                        <TableCell className="font-mono text-sm text-foreground">
                          {v.package || "Unknown"}
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant="destructive"
                            className={`shadow-none ${severityColor(v.severity)}`}
                          >
                            {v.severity}
                          </Badge>
                        </TableCell>
                        <TableCell className="text-muted-foreground text-sm">
                          {v.title || "Security vulnerability found"}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {v.cve || "-"}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              {topVulnTotalPages > 1 && (
                <div className="pt-4 border-t border-border/50 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      Rows per page
                    </span>
                    <Select
                      value={itemsPerPage.toString()}
                      onValueChange={(v) => {
                        setItemsPerPage(Number(v))
                        setVulnPage(1)
                      }}
                    >
                      <SelectTrigger className="w-[70px] h-8 bg-popover border-border text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-popover border-border">
                        <SelectItem value="5">5</SelectItem>
                        <SelectItem value="10">10</SelectItem>
                        <SelectItem value="20">20</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <Pagination className="mx-0 w-auto">
                    <PaginationContent>
                      <PaginationItem>
                        <PaginationPrevious
                          onClick={() =>
                            setVulnPage((p) => Math.max(1, p - 1))
                          }
                          className={
                            vulnPage === 1
                              ? "pointer-events-none opacity-50"
                              : "cursor-pointer"
                          }
                        />
                      </PaginationItem>
                      {renderPageNumbers(vulnPage, topVulnTotalPages, setVulnPage)}
                      <PaginationItem>
                        <PaginationNext
                          onClick={() =>
                            setVulnPage((p) =>
                              Math.min(topVulnTotalPages, p + 1),
                            )
                          }
                          className={
                            vulnPage === topVulnTotalPages
                              ? "pointer-events-none opacity-50"
                              : "cursor-pointer"
                          }
                        />
                      </PaginationItem>
                    </PaginationContent>
                  </Pagination>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
