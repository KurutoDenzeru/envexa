import { createFileRoute } from "@tanstack/react-router"
import { useState, useMemo } from "react"
import { useScanData } from "@/components/scan-data-context"
import { PackageDetailDialog } from "@/components/package-detail-dialog"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ShieldAlert, CheckCircle, Search } from "lucide-react"
import { Input } from "@/components/ui/input"
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import { DonutChart, CHART_PALETTE } from "@/components/donut-chart"
import { ScanProgress } from "@/components/scan-progress"
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

const SEVERITY_COLORS: Record<string, string> = {
  critical: "#ef4444",
  high: "#f97316",
  medium: "#d97706",
  moderate: "#d97706",
  low: "#10b981",
  other: "#71717a",
}

function Vulnerabilities() {
  const { report, loading, refetch } = useScanData()
  const [search, setSearch] = useState("")
  const [detailPkg, setDetailPkg] = useState<{
    name: string
    toolchain: string
  } | null>(null)

  const openDetail = (name: string, toolchain: string) => {
    setDetailPkg({ name, toolchain })
  }

  const closeDetail = () => setDetailPkg(null)

  const allVulnerabilities = useMemo((): VulnEntry[] => {
    if (!report?.results) return []
    const vulns: VulnEntry[] = []
    Object.entries(report.results).forEach(
      ([toolchain, data]: [string, any]) => {
        if (data.vulnerabilities) {
          data.vulnerabilities.forEach((v: any) => {
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
      }
    )
    return vulns
  }, [report])

  const filteredVulnerabilities = useMemo(() => {
    return allVulnerabilities
      .filter((v) => {
        const matchesSearch =
          v.package.toLowerCase().includes(search.toLowerCase()) ||
          v.title.toLowerCase().includes(search.toLowerCase()) ||
          v.toolchain.toLowerCase().includes(search.toLowerCase())
        return matchesSearch
      })
      .sort((a, b) => severityOrder(a.severity) - severityOrder(b.severity))
  }, [allVulnerabilities, search])

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

  // Pie chart data: vulns per toolchain
  const toolchainPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      counts[v.toolchain] = (counts[v.toolchain] || 0) + 1
    }
    return Object.entries(counts)
      .map(([name, count], i) => ({
        name,
        value: count,
        fill: CHART_PALETTE[i % CHART_PALETTE.length],
      }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 10)
  }, [allVulnerabilities])

  // Pie chart data: vulns per package
  const packagePieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      counts[v.package] = (counts[v.package] || 0) + 1
    }
    return Object.entries(counts)
      .map(([name, count], i) => ({
        name,
        value: count,
        fill: CHART_PALETTE[(i + 3) % CHART_PALETTE.length],
      }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 10)
  }, [allVulnerabilities])

  const vulnColumns: ColumnDef<VulnEntry, unknown>[] = [
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="font-medium text-muted-foreground/80 capitalize">
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
            openDetail(row.original.package, row.original.toolchain)
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
    {
      accessorKey: "patched_version",
      header: "Patched Version",
      cell: ({ row }) => (
        <span className="font-mono text-xs text-muted-foreground">
          {row.original.patched_version || "-"}
        </span>
      ),
    },
  ]
  if (loading) {
    return (
      <div className="mx-auto max-w-7xl animate-in duration-700 fade-in">
        <ScanProgress loading={true} onRetry={refetch} />
      </div>
    )
  }

  const total = allVulnerabilities.length
  const critical = severityCounts["critical"] || 0
  const high = severityCounts["high"] || 0
  const medium = severityCounts["medium"] || 0

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <ShieldAlert className="h-8 w-8 text-foreground" />
            Security Vulnerabilities
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            All security flaws across your dependencies.
          </p>
        </div>
      </div>

      {/* Severity Summary Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total
            </CardTitle>
            <ShieldAlert className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">{total}</div>
            <p className="mt-1 text-xs text-muted-foreground/60">
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
            <p className="mt-1 text-xs text-muted-foreground/60">
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
            <p className="mt-1 text-xs text-muted-foreground/60">
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
            <p className="mt-1 text-xs text-muted-foreground/60">
              Schedule for review
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Charts Section */}
      {total > 0 && (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* Severity Distribution Donut */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Severity Distribution</CardTitle>
              <CardDescription>
                Breakdown of vulnerabilities by severity level.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <DonutChart data={pieData} />
            </CardContent>
          </Card>

          {/* Vulnerabilities by Toolchain */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">By Toolchain</CardTitle>
              <CardDescription>
                Top toolchains with the most vulnerabilities.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <DonutChart data={toolchainPieData} />
            </CardContent>
          </Card>
          {/* Vulnerabilities by Package */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">By Package</CardTitle>
              <CardDescription>
                Top packages with the most vulnerabilities.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <DonutChart data={packagePieData} />
            </CardContent>
          </Card>
        </div>
      )}
      {/* Vulnerability Table */}
      <Card>
        <CardHeader className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <CardTitle>Identified Flaws</CardTitle>
            <CardDescription>
              Review and address these issues to secure your workspace.
            </CardDescription>
          </div>
          <div className="flex w-full items-center gap-3 md:w-auto">
            <div className="relative w-full md:w-64">
              <Search className="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground/60" />
              <Input
                type="text"
                placeholder="Search packages..."
                className="border-border bg-background/50 pl-9"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
          </div>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {total > 0 && (
            <div className="flex flex-wrap gap-3">
              {Object.entries(severityCounts)
                .filter(([, count]) => count > 0)
                .sort(([a], [b]) => severityOrder(a) - severityOrder(b))
                .map(([sev, count]) => (
                  <Badge
                    key={sev}
                    variant="outline"
                    className={`shadow-none ${severityColor(sev)}`}
                  >
                    {sev.charAt(0).toUpperCase() + sev.slice(1)}: {count}
                  </Badge>
                ))}
            </div>
          )}
          {filteredVulnerabilities.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
              <CheckCircle className="mb-4 h-12 w-12 text-green-500/50" />
              <p>
                {search
                  ? "No vulnerabilities match your search."
                  : "No vulnerabilities detected. Your project is secure."}
              </p>
            </div>
          ) : (
            <DataTable
              columns={vulnColumns}
              data={filteredVulnerabilities}
              defaultPageSize={8}
              pageSizeOptions={[5, 8, 15, 50]}
            />
          )}
        </CardContent>
      </Card>
      <PackageDetailDialog
        packageName={detailPkg?.name ?? ""}
        toolchain={detailPkg?.toolchain ?? ""}
        open={!!detailPkg}
        onClose={closeDetail}
      />
    </div>
  )
}
