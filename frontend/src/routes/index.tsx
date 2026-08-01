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
  Gauge,
  PackageMinus,
  Search,
  Home,
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
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ScanProgress } from "@/components/scan-progress"
import { PackageDetailDialog } from "@/components/package-detail-dialog"
import {
  enrichOutdated,
  sourceColor,
  sourceLabel,
  updateTypeColor,
  updateTypeLabel,
} from "@/lib/outdated"
import type { OutdatedPackage } from "@/lib/outdated"
import { SimpleChartTooltip } from "@/components/ui/chart"
import { ToolchainDetailDialog } from "@/components/toolchain-card"
import {
  ToolchainStatusView,
  useToolchains,
} from "@/components/toolchain-status"
import { StatCard } from "@/components/stat-card"
import { CATEGORIES, statusBadge } from "@/lib/toolchains"
import {
  collectVulnerabilities,
  countBySeverity,
  severityColor,
  severityOrder,
  vulnerabilityColumns,
} from "@/lib/vulnerabilities"
import type { VulnerabilityWithToolchain } from "@/lib/vulnerabilities"

export const Route = createFileRoute("/")({ component: App })

function App() {
  const { report, loading, refetch: fetchReport } = useScanData()
  const { groups } = useToolchains()
  const [compactView, setCompactView] = useState(false)
  const [openDialog, setOpenDialog] = useState<string | null>(null)
  const [selectedPackage, setSelectedPackage] = useState<{
    name: string
    toolchain: string
  } | null>(null)
  const [outdatedSearch, setOutdatedSearch] = useState("")
  const [vulnSearch, setVulnSearch] = useState("")
  const allVulnerabilities = useMemo(() => {
    return collectVulnerabilities(report?.results).sort(
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
    if (!report?.results) return { active: 0, total: 0 }
    const total = CATEGORIES.reduce((sum, cat) => sum + cat.tools.length, 0)
    const active = Object.values(report.results).filter(
      (r) => r.status && !r.status.toLowerCase().includes("skip")
    ).length
    return { active, total }
  }, [report])

  // Severity breakdown
  const severityCounts = useMemo(
    () => countBySeverity(allVulnerabilities),
    [allVulnerabilities]
  )

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

  const topVulnColumns: ColumnDef<VulnerabilityWithToolchain, unknown>[] =
    vulnerabilityColumns({
      onOpenPackage: (name, toolchain) =>
        setSelectedPackage({ name, toolchain }),
      showPatchedVersion: false,
    })

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
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <Home className="h-8 w-8 text-foreground" />
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
      </div>

      {/* Top Metric Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          title="Risk Score"
          icon={<Gauge className="h-4 w-4 text-muted-foreground" />}
          value={healthScore}
          valueClassName={
            healthScore > 70
              ? "text-green-500"
              : healthScore > 40
                ? "text-yellow-500"
                : "text-red-500"
          }
          subtext={
            healthScore > 70
              ? "Healthy"
              : healthScore > 40
                ? "Needs attention"
                : "Critical"
          }
        />

        <StatCard
          title="Vulnerabilities"
          icon={<ShieldAlert className="h-4 w-4 text-muted-foreground" />}
          value={vulnCount}
          valueClassName={vulnCount > 0 ? "text-red-500" : "text-foreground"}
          subtext="Across toolchains"
        />

        <StatCard
          title="Outdated"
          icon={<Box className="h-4 w-4 text-muted-foreground" />}
          value={outCount}
          valueClassName={outCount > 0 ? "text-blue-500" : "text-foreground"}
          subtext="Updates available"
        />

        <StatCard
          title="Active Tools"
          icon={<Boxes className="h-4 w-4 text-muted-foreground" />}
          value={
            <>
              {activeToolchains.active}
              <span className="text-lg text-muted-foreground">
                /{activeToolchains.total}
              </span>
            </>
          }
          subtext="Toolchains detected"
        />
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
      <ToolchainStatusView
        compactView={compactView}
        onCompactViewChange={setCompactView}
        onSelectTool={setOpenDialog}
      />

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

      {/* Page-level toolchain dialogs so cards and table rows open in place */}
      {groups
        .flatMap((cat) => cat.tools)
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
