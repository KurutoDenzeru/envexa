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
import { CheckCircle, Search, Package, ArrowUpRight } from "lucide-react"
import { Input } from "@/components/ui/input"
import { Checkbox } from "@/components/ui/checkbox"
import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { DataTable } from "@/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import { PackageDetailDialog } from "@/components/package-detail-dialog"
import {
  enrichOutdated,
  sourceColor,
  sourceLabel,
  updateTypeColor,
  updateTypeLabel,
  UPDATE_TYPE_COLORS,
  type OutdatedPackage,
} from "@/lib/outdated"
import { DonutCard, keyCountPie } from "@/components/donut-chart"
import { StatCard } from "@/components/stat-card"
import { displayName } from "@/lib/toolchains"
export const Route = createFileRoute("/outdated")({
  component: Outdated,
})

function Outdated() {
  const { report, loading } = useScanData()
  const [search, setSearch] = useState("")
  const [selectedPackages, setSelectedPackages] = useState<Set<string>>(
    new Set()
  )
  const [selectAll, setSelectAll] = useState(false)
  const [currentPageSize, setCurrentPageSize] = useState(10)
  const [currentPage, setCurrentPage] = useState(0)
  const [selectedPackage, setSelectedPackage] = useState<{
    name: string
    toolchain: string
  } | null>(null)

  const allOutdated = useMemo((): OutdatedPackage[] => {
    return enrichOutdated(report?.results)
  }, [report])

  const filteredOutdated = useMemo(() => {
    return allOutdated
      .filter((o) => {
        const matchesSearch =
          o.name.toLowerCase().includes(search.toLowerCase()) ||
          o.toolchain.toLowerCase().includes(search.toLowerCase()) ||
          o.source.toLowerCase().includes(search.toLowerCase())
        return matchesSearch
      })
      .sort((a, b) => {
        // Sort by update type priority: major > minor > patch > unknown
        const typeOrder = { major: 0, minor: 1, patch: 2, unknown: 3 }
        const typeDiff = typeOrder[a.updateType] - typeOrder[b.updateType]
        if (typeDiff !== 0) return typeDiff
        // Then by toolchain
        const toolchainDiff = a.toolchain.localeCompare(b.toolchain)
        if (toolchainDiff !== 0) return toolchainDiff
        // Then by package name
        return a.name.localeCompare(b.name)
      })
  }, [allOutdated, search])

  const updateTypeCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const o of allOutdated) {
      counts[o.updateType] = (counts[o.updateType] || 0) + 1
    }
    return counts
  }, [allOutdated])

  // Compute visible rows on current page for select-all
  const getVisibleRows = () => {
    const start = currentPage * currentPageSize
    return filteredOutdated.slice(start, start + currentPageSize)
  }

  const handleSelectAll = (checked: boolean) => {
    setSelectAll(checked)
    const visibleRows = getVisibleRows()
    const newSet = new Set(selectedPackages)
    if (checked) {
      visibleRows.forEach((o) =>
        newSet.add(`${o.toolchain}:${o.source}:${o.name}`)
      )
    } else {
      visibleRows.forEach((o) =>
        newSet.delete(`${o.toolchain}:${o.source}:${o.name}`)
      )
    }
    setSelectedPackages(newSet)
  }

  const handleSelectPackage = (pkg: OutdatedPackage, checked: boolean) => {
    const key = `${pkg.toolchain}:${pkg.source}:${pkg.name}`
    const newSet = new Set(selectedPackages)
    if (checked) {
      newSet.add(key)
    } else {
      newSet.delete(key)
    }
    setSelectedPackages(newSet)
    setSelectAll(
      getVisibleRows().every((o) => {
        const k = `${o.toolchain}:${o.source}:${o.name}`
        return newSet.has(k)
      })
    )
  }

  const updateTypePieData = useMemo(() => {
    return (["major", "minor", "patch", "unknown"] as const)
      .filter((t) => (updateTypeCounts[t] || 0) > 0)
      .map((t) => ({
        name: t === "unknown" ? "Unknown" : updateTypeLabel(t),
        value: updateTypeCounts[t],
        fill: UPDATE_TYPE_COLORS[t],
      }))
  }, [updateTypeCounts])

  const toolchainPieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const o of allOutdated) {
      counts[o.toolchain] = (counts[o.toolchain] || 0) + 1
    }
    return keyCountPie(counts, { label: displayName })
  }, [allOutdated])

  const sourcePieData = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const o of allOutdated) {
      counts[o.source] = (counts[o.source] || 0) + 1
    }
    return keyCountPie(counts, { label: sourceLabel, offset: 5 })
  }, [allOutdated])

  const handleUpdateSelected = () => {
    const selected = Array.from(selectedPackages)
      .map((key) => {
        const [toolchain, source, name] = key.split(":")
        const pkg = allOutdated.find(
          (o) =>
            o.toolchain === toolchain && o.source === source && o.name === name
        )
        return pkg
      })
      .filter(Boolean)

    console.log(
      "Update Selected:",
      selected.map((p) => ({
        toolchain: p!.toolchain,
        source: p!.source,
        package: p!.name,
        current: p!.current,
        latest: p!.latest,
      }))
    )
  }
  const outdatedColumns: ColumnDef<OutdatedPackage, unknown>[] = [
    {
      id: "select",
      header: () => (
        <Checkbox
          checked={selectAll}
          onCheckedChange={handleSelectAll}
          aria-label="Select all packages on this page"
        />
      ),
      cell: ({ row }) => {
        const key = `${row.original.toolchain}:${row.original.source}:${row.original.name}`
        return (
          <Checkbox
            checked={selectedPackages.has(key)}
            onCheckedChange={(checked) =>
              handleSelectPackage(row.original, checked as boolean)
            }
            aria-label={`Select ${row.original.name}`}
          />
        )
      },
      enableSorting: false,
      size: 48,
    },
    {
      accessorKey: "toolchain",
      header: "Toolchain",
      cell: ({ row }) => (
        <span className="font-medium text-muted-foreground/80 capitalize">
          {displayName(row.original.toolchain)}
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
          className="h-5 border-blue-500/30 bg-blue-500/10 px-1.5 text-[11px] text-blue-400 shadow-none"
        >
          {row.original.latest}
        </Badge>
      ),
    },
    {
      accessorKey: "updateType",
      header: "Update Type",
      sortingFn: (a, b) => {
        const order = { major: 0, minor: 1, patch: 2, unknown: 3 }
        return (
          (order[a.original.updateType] ?? 3) -
          (order[b.original.updateType] ?? 3)
        )
      },
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

  if (loading) {
    return (
      <div className="mx-auto flex max-w-7xl flex-col gap-6">
        <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
          <div>
            <Skeleton className="h-10 w-48 bg-muted" />
            <Skeleton className="mt-3 h-4 w-96 bg-muted" />
          </div>
        </div>
        <Card>
          <CardContent className="pt-6">
            <div className="space-y-4">
              {[...Array(5)].map((_, i) => (
                <Skeleton key={i} className="h-12 w-full bg-muted" />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  const total = allOutdated.length
  const major = updateTypeCounts.major || 0
  const minor = updateTypeCounts.minor || 0
  const patch = updateTypeCounts.patch || 0

  return (
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <Package className="h-8 w-8 text-foreground" />
            Outdated Packages
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            All outdated packages across your toolchains with update severity
            classification.
          </p>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-5">
        <StatCard
          title="Total"
          icon={<Package className="h-4 w-4 text-muted-foreground" />}
          value={total}
          subtext="Across all toolchains"
        />

        <StatCard
          title="Major"
          icon={<ArrowUpRight className="h-4 w-4 text-muted-foreground" />}
          value={major}
          valueClassName={major > 0 ? "text-red-500" : "text-foreground"}
          subtext="Breaking changes"
        />

        <StatCard
          title="Minor"
          value={minor}
          valueClassName={minor > 0 ? "text-amber-500" : "text-foreground"}
          subtext="New features"
        />

        <StatCard
          title="Patch"
          value={patch}
          valueClassName={patch > 0 ? "text-emerald-500" : "text-foreground"}
          subtext="Bug fixes"
        />

        <StatCard
          title="Unknown"
          value={updateTypeCounts.unknown || 0}
          valueClassName="text-muted-foreground"
          subtext="Non-semver versions"
        />
      </div>

      {/* Charts Section */}
      {total > 0 && (
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          <DonutCard
            title="Update Type Distribution"
            description="Breakdown of outdated packages by update type."
            data={updateTypePieData}
          />

          <DonutCard
            title="By Toolchain"
            description="Toolchains with the most outdated packages."
            data={toolchainPieData}
          />

          <DonutCard
            title="By Source"
            description="Sources with the most outdated packages."
            data={sourcePieData}
          />
        </div>
      )}

      {/* Outdated Packages Table */}
      <Card>
        <CardHeader className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <CardTitle>Identified Updates</CardTitle>
            <CardDescription>
              Review and select packages to update. Major updates may contain
              breaking changes.
            </CardDescription>
          </div>
          <div className="flex w-full flex-wrap items-center gap-3 md:w-auto">
            <div className="relative w-full min-w-[200px] flex-1 md:w-72">
              <Search className="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground/60" />
              <Input
                type="text"
                placeholder="Search packages, toolchains, sources..."
                className="border-border bg-background/50 pl-9"
                value={search}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                  setSearch(e.target.value)
                }
              />
            </div>
          </div>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {total > 0 && (
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
                {search
                  ? "No packages match your search."
                  : "All packages are up to date!"}
              </p>
            </div>
          ) : (
            <>
              <DataTable
                columns={outdatedColumns}
                data={filteredOutdated}
                defaultPageSize={10}
                pageSizeOptions={[10, 25, 50, 100]}
                onPageChange={(page, pageSize) => {
                  setCurrentPage(page)
                  setCurrentPageSize(pageSize)
                  setSelectedPackages(new Set())
                  setSelectAll(false)
                }}
              />
              {selectedPackages.size > 0 && (
                <div className="mt-4 flex items-center justify-between border-t border-border/50 pt-4">
                  <div className="flex items-center gap-3">
                    <span className="text-sm text-muted-foreground">
                      {selectedPackages.size} package
                      {selectedPackages.size !== 1 ? "s" : ""} selected
                    </span>
                  </div>
                  <Button
                    onClick={handleUpdateSelected}
                    className="gap-2"
                    disabled={selectedPackages.size === 0}
                  >
                    <ArrowUpRight className="h-4 w-4" />
                    Update Selected
                  </Button>
                </div>
              )}
            </>
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
