import { Badge } from "@/components/ui/badge"
import type { ColumnDef } from "@tanstack/react-table"

export interface VulnerabilityInfo {
  package: string
  severity: string
  title: string
  cve?: string | null
  patched_version?: string
}

export interface VulnerabilityWithToolchain extends VulnerabilityInfo {
  toolchain: string
}

// Severity weights used for the total risk score (higher = more severe)
export const RISK_WEIGHTS: Record<string, number> = {
  critical: 10,
  high: 5,
  moderate: 3,
  low: 1,
}

export const SEVERITY_COLORS: Record<string, string> = {
  critical: "#ef4444",
  high: "#f97316",
  medium: "#d97706",
  moderate: "#d97706",
  low: "#10b981",
  other: "#71717a",
}

export function severityOrder(s: string): number {
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

export function severityColor(s: string): string {
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

/** Flatten every toolchain's vulnerabilities into one list, tagged by toolchain. */
export function collectVulnerabilities(
  results?: Record<string, any>
): VulnerabilityWithToolchain[] {
  if (!results) return []
  const vulns: VulnerabilityWithToolchain[] = []
  for (const [toolchain, data] of Object.entries(results)) {
    for (const v of data?.vulnerabilities ?? []) {
      vulns.push({
        package: v.package,
        severity: v.severity,
        title: v.title,
        cve: v.cve ?? null,
        patched_version: v.patched_version ?? "",
        toolchain,
      })
    }
  }
  return vulns
}

export function countBySeverity(
  vulns: VulnerabilityWithToolchain[]
): Record<string, number> {
  const counts: Record<string, number> = {}
  for (const v of vulns) {
    const key = v.severity.toLowerCase()
    counts[key] = (counts[key] || 0) + 1
  }
  return counts
}

/** Shared vulnerability table columns (dashboard summary + vulnerabilities page). */
export function vulnerabilityColumns(opts: {
  onOpenPackage: (name: string, toolchain: string) => void
  showPatchedVersion?: boolean
}): ColumnDef<VulnerabilityWithToolchain, unknown>[] {
  return [
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
            opts.onOpenPackage(row.original.package, row.original.toolchain)
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
    ...(opts.showPatchedVersion
      ? [
          {
            accessorKey: "patched_version",
            header: "Patched Version",
            cell: ({
              row,
            }: {
              row: { original: VulnerabilityWithToolchain }
            }) => (
              <span className="font-mono text-xs text-muted-foreground">
                {row.original.patched_version || "-"}
              </span>
            ),
          },
        ]
      : []),
  ]
}
