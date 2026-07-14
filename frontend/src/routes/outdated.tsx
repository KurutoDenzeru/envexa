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
export const Route = createFileRoute("/outdated")({
  component: Outdated,
})

interface PackageInfo {
  name: string
  current: string
  latest: string
}

interface OutdatedPackage extends PackageInfo {
  toolchain: string
  source: string
  updateType: "major" | "minor" | "patch" | "unknown"
}

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

function sourceLabel(source: string): string {
  const labels: Record<string, string> = {
    formulae: "formulae",
    casks: "casks",
    global: "global",
    default: "default",
  }
  return labels[source] || source
}

function sourceColor(source: string): string {
  const colors: Record<string, string> = {
    formulae: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    casks: "bg-purple-500/10 text-purple-600 border-purple-500/20 dark:bg-purple-500/15 dark:text-purple-400 dark:border-purple-500/25",
    global: "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    default: "bg-emerald-500/10 text-emerald-600 border-emerald-500/20 dark:bg-emerald-500/15 dark:text-emerald-400 dark:border-emerald-500/25",
    npm: "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25",
    pnpm: "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    yarn: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    bun: "bg-amber-500/10 text-amber-600 border-amber-500/20 dark:bg-amber-500/15 dark:text-amber-400 dark:border-amber-500/25",
    pip: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    cargo: "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    gem: "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25",
    docker: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
  }
  return colors[source] || "bg-muted text-muted-foreground border-border"
}

function parseVersion(version: string): { major: number; minor: number; patch: number } | null {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return null
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
  }
}

function getUpdateType(current: string, latest: string): "major" | "minor" | "patch" | "unknown" {
  const curr = parseVersion(current)
  const lat = parseVersion(latest)
  if (!curr || !lat) return "unknown"
  if (lat.major > curr.major) return "major"
  if (lat.minor > curr.minor) return "minor"
  if (lat.patch > curr.patch) return "patch"
  return "unknown"
}

function updateTypeColor(type: "major" | "minor" | "patch" | "unknown"): string {
  switch (type) {
    case "major":
      return "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25"
    case "minor":
      return "bg-amber-500/10 text-amber-600 border-amber-500/20 dark:bg-amber-500/15 dark:text-amber-400 dark:border-amber-500/25"
    case "patch":
      return "bg-emerald-500/10 text-emerald-600 border-emerald-500/20 dark:bg-emerald-500/15 dark:text-emerald-400 dark:border-emerald-500/25"
    default:
      return "bg-muted text-muted-foreground border-border"
  }
}

function updateTypeLabel(type: "major" | "minor" | "patch" | "unknown"): string {
  switch (type) {
    case "major":
      return "Major"
    case "minor":
      return "Minor"
    case "patch":
      return "Patch"
    default:
      return "—"
  }
}


function Outdated() {
  const { report, loading } = useScanData()
  const [search, setSearch] = useState("")
  const [selectedPackages, setSelectedPackages] = useState<Set<string>>(new Set())
  const [selectAll, setSelectAll] = useState(false)
  const [currentPageSize, setCurrentPageSize] = useState(10)
  const [currentPage, setCurrentPage] = useState(0)

  const allOutdated = useMemo((): OutdatedPackage[] => {
    if (!report?.results) return []
    const outdated: OutdatedPackage[] = []

    Object.entries(report.results).forEach(([toolchain, data]: [string, any]) => {
      // Handle outdated_formulae (Homebrew formulae)
      if (data.outdated_formulae) {
        data.outdated_formulae.forEach((o: PackageInfo) => {
          const updateType = getUpdateType(o.current, o.latest)
          outdated.push({
            ...o,
            toolchain,
            source: "formulae",
            updateType,
          })
        })
      }

      // Handle outdated_casks (Homebrew casks)
      if (data.outdated_casks) {
        data.outdated_casks.forEach((o: PackageInfo) => {
          const updateType = getUpdateType(o.current, o.latest)
          outdated.push({
            ...o,
            toolchain,
            source: "casks",
            updateType,
          })
        })
      }

      // Handle outdated_global (global packages)
      if (data.outdated_global) {
        data.outdated_global.forEach((o: PackageInfo) => {
          const updateType = getUpdateType(o.current, o.latest)
          outdated.push({
            ...o,
            toolchain,
            source: "global",
            updateType,
          })
        })
      }

      // Handle generic outdated
      if (data.outdated) {
        data.outdated.forEach((o: PackageInfo) => {
          const updateType = getUpdateType(o.current, o.latest)
          outdated.push({
            ...o,
            toolchain,
            source: toolchain, // Use toolchain name as source for generic outdated
            updateType,
          })
        })
      }
    })

    return outdated
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
      visibleRows.forEach((o) => newSet.add(`${o.toolchain}:${o.source}:${o.name}`))
    } else {
      visibleRows.forEach((o) => newSet.delete(`${o.toolchain}:${o.source}:${o.name}`))
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
    setSelectAll(getVisibleRows().every((o) => {
      const k = `${o.toolchain}:${o.source}:${o.name}`
      return newSet.has(k)
    }))
  }

  const handleUpdateSelected = () => {
    const selected = Array.from(selectedPackages).map((key) => {
      const [toolchain, source, name] = key.split(":")
      const pkg = allOutdated.find(
        (o) => o.toolchain === toolchain && o.source === source && o.name === name,
      )
      return pkg
    }).filter(Boolean)

    console.log("Update Selected:", selected.map((p) => ({
      toolchain: p!.toolchain,
      source: p!.source,
      package: p!.name,
      current: p!.current,
      latest: p!.latest,
    })))
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
            onCheckedChange={(checked) => handleSelectPackage(row.original, checked as boolean)}
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
        <span className="font-medium capitalize text-muted-foreground/80">
          {displayName(row.original.toolchain)}
        </span>
      ),
    },
    {
      accessorKey: "name",
      header: "Package",
      cell: ({ row }) => (
        <span className="font-mono text-sm text-foreground">
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
          className="border-blue-500/30 text-blue-400 bg-blue-500/10 shadow-none text-[11px] h-5 px-1.5"
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
        return (order[a.original.updateType] ?? 3) - (order[b.original.updateType] ?? 3)
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
      <div className="max-w-7xl mx-auto flex flex-col gap-6">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-48 bg-muted" />
            <Skeleton className="h-4 w-96 mt-3 bg-muted" />
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
    <div className="max-w-7xl mx-auto flex flex-col gap-6">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <Package className="w-8 h-8 text-foreground" />
            Outdated Packages
          </h1>
          <p className="text-sm text-muted-foreground mt-2">
            All outdated packages across your toolchains with update severity classification.
          </p>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total
            </CardTitle>
            <Package className="w-4 h-4 text-muted-foreground" />
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
              Major
            </CardTitle>
            <ArrowUpRight className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-3xl font-bold ${major > 0 ? "text-red-500" : "text-foreground"}`}>
              {major}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Breaking changes
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Minor
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-3xl font-bold ${minor > 0 ? "text-amber-500" : "text-foreground"}`}>
              {minor}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              New features
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Patch
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-3xl font-bold ${patch > 0 ? "text-emerald-500" : "text-foreground"}`}>
              {patch}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Bug fixes
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Unknown
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-muted-foreground">
              {updateTypeCounts.unknown || 0}
            </div>
            <p className="text-xs text-muted-foreground/60 mt-1">
              Non-semver versions
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Outdated Packages Table */}
      <Card>
        <CardHeader className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <CardTitle>Identified Updates</CardTitle>
            <CardDescription>
              Review and select packages to update. Major updates may contain breaking changes.
            </CardDescription>
          </div>
          <div className="flex items-center gap-3 w-full md:w-auto flex-wrap">
            <div className="relative w-full md:w-72 flex-1 min-w-[200px]">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground/60" />
              <Input
                type="text"
                placeholder="Search packages, toolchains, sources..."
                className="pl-9 bg-background/50 border-border"
                value={search}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearch(e.target.value)}
              />
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {filteredOutdated.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
              <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
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
                <div className="mt-4 pt-4 border-t border-border/50 flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-sm text-muted-foreground">
                      {selectedPackages.size} package{selectedPackages.size !== 1 ? "s" : ""} selected
                    </span>
                  </div>
                  <Button
                    onClick={handleUpdateSelected}
                    className="gap-2"
                    disabled={selectedPackages.size === 0}
                  >
                    <ArrowUpRight className="w-4 h-4" />
                    Update Selected
                  </Button>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
