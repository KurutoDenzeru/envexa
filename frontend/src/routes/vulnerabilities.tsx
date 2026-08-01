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
import { DonutCard, keyCountPie } from "@/components/donut-chart"
import { StatCard } from "@/components/stat-card"
import { ScanProgress } from "@/components/scan-progress"
import {
  SEVERITY_COLORS,
  collectVulnerabilities,
  countBySeverity,
  severityColor,
  severityOrder,
  vulnerabilityColumns,
} from "@/lib/vulnerabilities"
export const Route = createFileRoute("/vulnerabilities")({
  component: Vulnerabilities,
})

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

  const allVulnerabilities = useMemo(
    () => collectVulnerabilities(report?.results),
    [report]
  )

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

  const severityCounts = useMemo(
    () => countBySeverity(allVulnerabilities),
    [allVulnerabilities]
  )

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
  const toolchainCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      counts[v.toolchain] = (counts[v.toolchain] || 0) + 1
    }
    return counts
  }, [allVulnerabilities])
  const toolchainPieData = useMemo(
    () => keyCountPie(toolchainCounts, { limit: 10 }),
    [toolchainCounts]
  )

  // Pie chart data: vulns per package
  const packageCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of allVulnerabilities) {
      counts[v.package] = (counts[v.package] || 0) + 1
    }
    return counts
  }, [allVulnerabilities])
  const packagePieData = useMemo(
    () => keyCountPie(packageCounts, { offset: 3, limit: 10 }),
    [packageCounts]
  )

  const vulnColumns = vulnerabilityColumns({
    onOpenPackage: openDetail,
    showPatchedVersion: true,
  })
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
        <StatCard
          title="Total"
          icon={<ShieldAlert className="h-4 w-4 text-muted-foreground" />}
          value={total}
          subtext="Across all toolchains"
        />

        <StatCard
          title="Critical"
          value={critical}
          valueClassName={critical > 0 ? "text-red-500" : "text-foreground"}
          subtext="Immediate action required"
        />

        <StatCard
          title="High"
          value={high}
          valueClassName={high > 0 ? "text-orange-500" : "text-foreground"}
          subtext="Should be addressed soon"
        />

        <StatCard
          title="Medium"
          value={medium}
          valueClassName={medium > 0 ? "text-yellow-500" : "text-foreground"}
          subtext="Schedule for review"
        />
      </div>

      {/* Charts Section */}
      {total > 0 && (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          <DonutCard
            title="Severity Distribution"
            description="Breakdown of vulnerabilities by severity level."
            data={pieData}
          />

          <DonutCard
            title="By Toolchain"
            description="Top toolchains with the most vulnerabilities."
            data={toolchainPieData}
          />

          <DonutCard
            title="By Package"
            description="Top packages with the most vulnerabilities."
            data={packagePieData}
          />
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
