import { createFileRoute } from "@tanstack/react-router"
import { useState, useMemo, useEffect } from "react"
import { useScanData } from "@/components/scan-data-context"
import { PackageOpen, ShieldAlert, Boxes } from "lucide-react"
import { ScanProgress } from "@/components/scan-progress"
import { DonutCard, keyCountPie } from "@/components/donut-chart"
import { StatCard } from "@/components/stat-card"
import { ToolchainDetailDialog } from "@/components/toolchain-card"
import {
  ToolchainStatusView,
  useToolchains,
} from "@/components/toolchain-status"
import { displayName, statusKey } from "@/lib/toolchains"
import {
  RISK_WEIGHTS,
  collectVulnerabilities,
  countBySeverity,
} from "@/lib/vulnerabilities"

export const Route = createFileRoute("/toolchains")({
  validateSearch: (search: Record<string, unknown>) => ({
    open: (search.open as string) || undefined,
  }),
  component: Toolchains,
})

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

  const { groups, flat, totalTools } = useToolchains()

  // Donut chart data: toolchain status distribution
  const STATUS_COLORS: Record<string, string> = {
    pass: "#22c55e",
    warn: "#eab308",
    fail: "#ef4444",
    skip: "#71717a",
  }
  const statusPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const tc of flat) {
      const key = statusKey(tc.status)
      counts[key] = (counts[key] || 0) + 1
    }
    return Object.entries(counts)
      .filter(([, count]) => count > 0)
      .map(([name, value]) => ({
        name: name.charAt(0).toUpperCase() + name.slice(1),
        value,
        fill: STATUS_COLORS[name],
      }))
  }, [flat])

  // Donut chart data: vulnerabilities per toolchain
  const vulnCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const tc of flat) {
      const count = tc.vulnerabilities?.length || 0
      if (count > 0) counts[tc.tool] = (counts[tc.tool] || 0) + count
    }
    return counts
  }, [flat])
  const vulnPieData = useMemo(
    () => keyCountPie(vulnCounts, { label: displayName, limit: 10 }),
    [vulnCounts]
  )

  // Donut chart data: outdated packages per toolchain
  const outdatedCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const tc of flat) {
      const count = tc.outdated?.length || 0
      if (count > 0) counts[tc.tool] = (counts[tc.tool] || 0) + count
    }
    return counts
  }, [flat])
  const outdatedPieData = useMemo(
    () =>
      keyCountPie(outdatedCounts, { label: displayName, offset: 3, limit: 10 }),
    [outdatedCounts]
  )

  const totalVulns = vulnPieData.reduce((sum, d) => sum + d.value, 0)
  const totalOutdated = outdatedPieData.reduce((sum, d) => sum + d.value, 0)

  // Aggregated vulnerability stats for the summary cards + severity breakdown
  const allVulns = useMemo(
    () => collectVulnerabilities(report?.results),
    [report]
  )
  const severityCounts = useMemo(() => countBySeverity(allVulns), [allVulns])

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
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <StatCard
          title="Toolchains"
          icon={<Boxes className="h-4 w-4 text-muted-foreground" />}
          value={totalTools}
          subtext="Detected in your environment"
        />

        <StatCard
          title="Risk Score"
          icon={<ShieldAlert className="h-4 w-4 text-muted-foreground" />}
          value={vulnStats.riskScore}
          valueClassName={
            vulnStats.critical > 0
              ? "text-red-500"
              : vulnStats.high > 0
                ? "text-orange-500"
                : "text-foreground"
          }
          subtext="Severity-weighted vulnerabilities"
        />

        <StatCard
          title="Vulnerabilities"
          value={vulnStats.total}
          valueClassName={
            vulnStats.total > 0 ? "text-red-400" : "text-foreground"
          }
          subtext="Across all toolchains"
        />

        <StatCard
          title="Outdated"
          value={totalOutdated}
          valueClassName={
            totalOutdated > 0 ? "text-blue-400" : "text-foreground"
          }
          subtext="Packages behind latest"
        />
      </div>

      {/* Charts Section */}
      {totalTools > 0 && (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          <DonutCard
            title="Status Distribution"
            description="Toolchain status across your environment."
            data={statusPieData}
          />

          {totalVulns > 0 && (
            <DonutCard
              title="Vulnerabilities by Toolchain"
              description="Toolchains with the most vulnerabilities."
              data={vulnPieData}
            />
          )}

          {totalOutdated > 0 && (
            <DonutCard
              title="Outdated by Toolchain"
              description="Toolchains with the most outdated packages."
              data={outdatedPieData}
            />
          )}
        </div>
      )}

      {totalTools === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-muted/50 py-12">
          <PackageOpen className="mb-4 h-12 w-12 text-neutral-600" />
          <p className="text-muted-foreground">No toolchains detected.</p>
        </div>
      ) : (
        <ToolchainStatusView
          compactView={compactView}
          onCompactViewChange={setCompactView}
          onSelectTool={setOpenDialog}
        />
      )}

      {/* Page-level dialogs so row clicks work in both views */}
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
