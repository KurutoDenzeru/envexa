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
import { ShieldAlert, CheckCircle, Search } from "lucide-react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
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
import {
  PieChart,
  Pie,
  Cell,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts"

export const Route = createFileRoute("/vulnerabilities")({
  component: Vulnerabilities,
})

interface VulnEntry {
  package: string
  severity: string
  title: string
  cve: string | null
  patched_version: string
  toolchain: string
}

interface ScanResult {
  vulnerabilities?: Array<{
    package: string
    severity: string
    title: string
    cve?: string | null
    patched_version?: string
  }>
}

interface ScanReport {
  results?: Record<string, ScanResult>
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

function severityColor(s: string): string {
  switch (s.toLowerCase()) {
    case "critical":
      return "bg-red-500/10 text-red-500 border-red-500/20"
    case "high":
      return "bg-orange-500/10 text-orange-500 border-orange-500/20"
    case "medium":
      return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"
    case "low":
      return "bg-blue-500/10 text-blue-500 border-blue-500/20"
    default:
      return "bg-muted text-muted-foreground border-border"
  }
}

const SEVERITY_COLORS: Record<string, string> = {
  critical: "#ef4444",
  high: "#f97316",
  medium: "#eab308",
  low: "#3b82f6",
  other: "#71717a",
}

function Vulnerabilities() {
  const [report, setReport] = useState<ScanReport | null>(null)
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState("")
  const [toolchainFilter, setToolchainFilter] = useState<string>("all")
  const [page, setPage] = useState(1)
  const [itemsPerPage, setItemsPerPage] = useState(8)

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

  const allVulnerabilities = useMemo((): VulnEntry[] => {
    if (!report?.results) return []
    const vulns: VulnEntry[] = []
    Object.entries(report.results).forEach(([toolchain, data]) => {
      if (data.vulnerabilities) {
        data.vulnerabilities.forEach((v) => {
          vulns.push({
            package: v.package,
            severity: v.severity,
            title: v.title,
            cve: v.cve ?? null,
            patched_version: v.patched_version ?? "",
            toolchain,
          })
        })
      }
    })
    return vulns
  }, [report])

  const availableToolchains = useMemo(() => {
    const set = new Set(allVulnerabilities.map((v) => v.toolchain))
    return Array.from(set).sort()
  }, [allVulnerabilities])

  const filteredVulnerabilities = useMemo(() => {
    return allVulnerabilities
      .filter((v) => {
        const matchesSearch =
          v.package.toLowerCase().includes(search.toLowerCase()) ||
          v.title.toLowerCase().includes(search.toLowerCase()) ||
          v.toolchain.toLowerCase().includes(search.toLowerCase())
        const matchesToolchain =
          toolchainFilter === "all" || v.toolchain === toolchainFilter
        return matchesSearch && matchesToolchain
      })
      .sort((a, b) => severityOrder(a.severity) - severityOrder(b.severity))
  }, [allVulnerabilities, search, toolchainFilter])

  useEffect(() => {
    setPage(1)
  }, [search, toolchainFilter])

  const severityCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      const key = v.severity.toLowerCase()
      counts[key] = (counts[key] || 0) + 1
    }
    return counts
  }, [allVulnerabilities])

  // Pie chart data
  const pieData = useMemo(() => {
    return Object.entries(severityCounts)
      .filter(([, count]) => count > 0)
      .map(([name, value]) => ({
        name: name.charAt(0).toUpperCase() + name.slice(1),
        value,
        fill: SEVERITY_COLORS[name] || SEVERITY_COLORS.other,
      }))
  }, [severityCounts])

  // Bar chart data: vulns per toolchain
  const toolchainBarData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      counts[v.toolchain] = (counts[v.toolchain] || 0) + 1
    }
    return Object.entries(counts)
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10)
  }, [allVulnerabilities])

  const totalPages = Math.ceil(filteredVulnerabilities.length / itemsPerPage)
  const paginatedVulnerabilities = filteredVulnerabilities.slice(
    (page - 1) * itemsPerPage,
    page * itemsPerPage,
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
      <div className="max-w-7xl mx-auto flex flex-col gap-6">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-64 bg-muted" />
            <Skeleton className="h-4 w-96 mt-3 bg-muted" />
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Skeleton className="h-24 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-24 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-24 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-24 w-full rounded-xl bg-muted/50" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton className="h-[300px] w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-[300px] w-full rounded-xl bg-muted/50" />
        </div>
        <Skeleton className="h-[400px] w-full rounded-xl bg-muted/50" />
      </div>
    )
  }

  const total = allVulnerabilities.length
  const critical = severityCounts["critical"] || 0
  const high = severityCounts["high"] || 0
  const medium = severityCounts["medium"] || 0

  return (
    <div className="max-w-7xl mx-auto flex flex-col gap-6">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <ShieldAlert className="w-8 h-8 text-foreground" />
            Security Vulnerabilities
          </h1>
          <p className="text-sm text-muted-foreground mt-2">
            All security flaws across your dependencies.
          </p>
        </div>
      </div>

      {/* Severity Summary Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total
            </CardTitle>
            <ShieldAlert className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">{total}</div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Across all toolchains
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Critical
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${critical > 0 ? "text-red-500" : "text-foreground"}`}
            >
              {critical}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Immediate action required
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              High
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${high > 0 ? "text-orange-500" : "text-foreground"}`}
            >
              {high}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Should be addressed soon
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Medium
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div
              className={`text-3xl font-bold ${medium > 0 ? "text-yellow-500" : "text-foreground"}`}
            >
              {medium}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Schedule for review
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Charts Section */}
      {total > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Severity Distribution Donut */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Severity Distribution</CardTitle>
              <CardDescription>
                Breakdown of vulnerabilities by severity level.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="h-[260px] w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={pieData}
                      cx="50%"
                      cy="50%"
                      innerRadius={60}
                      outerRadius={100}
                      paddingAngle={3}
                      dataKey="value"
                      stroke="hsl(var(--background))"
                      strokeWidth={2}
                    >
                      {pieData.map((entry, index) => (
                        <Cell key={index} fill={entry.fill} />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={{
                        backgroundColor: "hsl(var(--card))",
                        border: "1px solid hsl(var(--border))",
                        borderRadius: "8px",
                        color: "hsl(var(--foreground))",
                      }}
                    />
                    <Legend
                      verticalAlign="bottom"
                      height={36}
                      formatter={(value: string) => (
                        <span className="text-xs text-muted-foreground">
                          {value}
                        </span>
                      )}
                    />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>

          {/* Vulnerabilities by Toolchain */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                By Toolchain
              </CardTitle>
              <CardDescription>
                Top toolchains with the most vulnerabilities.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="h-[260px] w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart
                    data={toolchainBarData}
                    layout="vertical"
                    margin={{ top: 0, right: 10, left: 0, bottom: 0 }}
                  >
                    <CartesianGrid
                      strokeDasharray="3 3"
                      stroke="hsl(var(--border))"
                      horizontal={false}
                    />
                    <XAxis
                      type="number"
                      stroke="#a1a1aa"
                      tick={{ fill: "#a1a1aa", fontSize: 11 }}
                      axisLine={false}
                      tickLine={false}
                      allowDecimals={false}
                    />
                    <YAxis
                      type="category"
                      dataKey="name"
                      stroke="#a1a1aa"
                      tick={{ fill: "#a1a1aa", fontSize: 11 }}
                      axisLine={false}
                      tickLine={false}
                      width={90}
                      tickFormatter={(v: string) =>
                        v.length > 10 ? v.slice(0, 10) + "..." : v
                      }
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: "hsl(var(--card))",
                        border: "1px solid hsl(var(--border))",
                        borderRadius: "8px",
                        color: "hsl(var(--foreground))",
                      }}
                      cursor={{ fill: "rgba(255,255,255,0.05)" }}
                    />
                    <Bar
                      dataKey="count"
                      name="Vulnerabilities"
                      fill="#ef4444"
                      radius={[0, 4, 4, 0]}
                      maxBarSize={24}
                    />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Vulnerability Table */}
      <Card>
        <CardHeader className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <CardTitle>Identified Flaws</CardTitle>
            <CardDescription>
              Review and address these issues to secure your workspace.
            </CardDescription>
          </div>
          <div className="flex items-center gap-3 w-full md:w-auto">
            <div className="relative w-full md:w-64">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground/60" />
              <Input
                type="text"
                placeholder="Search packages..."
                className="pl-9 bg-background/50 border-border"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            <Select
              value={toolchainFilter}
              onValueChange={(v) => setToolchainFilter(v ?? "all")}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="All toolchains" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Toolchains</SelectItem>
                {availableToolchains.map((tc) => (
                  <SelectItem key={tc} value={tc}>
                    {tc}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          {filteredVulnerabilities.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
              <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
              <p>
                {search || toolchainFilter !== "all"
                  ? "No vulnerabilities match your filters."
                  : "No vulnerabilities detected. Your project is secure."}
              </p>
            </div>
          ) : (
            <>
              <div className="rounded-md border border-border overflow-hidden">
                <Table>
                  <TableHeader className="bg-muted/50">
                    <TableRow className="border-border hover:bg-transparent">
                      <TableHead className="w-[120px]">Toolchain</TableHead>
                      <TableHead className="w-[180px]">Package</TableHead>
                      <TableHead className="w-[100px]">Severity</TableHead>
                      <TableHead>Description</TableHead>
                      <TableHead className="w-[130px]">CVE</TableHead>
                      <TableHead className="w-[120px]">
                        Patched Version
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {paginatedVulnerabilities.map((v, idx) => (
                      <TableRow
                        key={idx}
                        className="border-border hover:bg-muted/50"
                      >
                        <TableCell className="font-medium capitalize text-muted-foreground/80">
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
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {v.patched_version || "-"}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              {totalPages > 1 && (
                <div className="mt-4 pt-4 border-t border-border/50 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      Rows per page
                    </span>
                    <Select
                      value={itemsPerPage.toString()}
                      onValueChange={(v) => {
                        if (v) setItemsPerPage(Number(v))
                        setPage(1)
                      }}
                    >
                      <SelectTrigger className="w-[70px] h-8 bg-popover border-border text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-popover border-border">
                        <SelectItem value="5">5</SelectItem>
                        <SelectItem value="8">8</SelectItem>
                        <SelectItem value="15">15</SelectItem>
                        <SelectItem value="50">50</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <Pagination className="mx-0 w-auto">
                    <PaginationContent>
                      <PaginationItem>
                        <PaginationPrevious
                          onClick={() => setPage((p) => Math.max(1, p - 1))}
                          className={
                            page === 1
                              ? "pointer-events-none opacity-50"
                              : "cursor-pointer"
                          }
                        />
                      </PaginationItem>
                      {renderPageNumbers(page, totalPages, setPage)}
                      <PaginationItem>
                        <PaginationNext
                          onClick={() =>
                            setPage((p) => Math.min(totalPages, p + 1))
                          }
                          className={
                            page === totalPages
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
