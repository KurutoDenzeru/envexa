export interface OutdatedPackage {
  name: string
  current: string
  latest: string
  toolchain: string
  source: string
  updateType: "major" | "minor" | "patch" | "unknown"
}

export function parseVersion(
  version: string
): { major: number; minor: number; patch: number } | null {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return null
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
  }
}

export function getUpdateType(
  current: string,
  latest: string
): "major" | "minor" | "patch" | "unknown" {
  const curr = parseVersion(current)
  const lat = parseVersion(latest)
  if (!curr || !lat) return "unknown"
  if (lat.major > curr.major) return "major"
  if (lat.minor > curr.minor) return "minor"
  if (lat.patch > curr.patch) return "patch"
  return "unknown"
}

export function updateTypeColor(
  type: "major" | "minor" | "patch" | "unknown"
): string {
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

export function updateTypeLabel(
  type: "major" | "minor" | "patch" | "unknown"
): string {
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

export function sourceLabel(source: string): string {
  const labels: Record<string, string> = {
    formulae: "formulae",
    casks: "casks",
    global: "global",
    default: "default",
  }
  return labels[source] || source
}

export function sourceColor(source: string): string {
  const colors: Record<string, string> = {
    formulae:
      "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    casks:
      "bg-purple-500/10 text-purple-600 border-purple-500/20 dark:bg-purple-500/15 dark:text-purple-400 dark:border-purple-500/25",
    global:
      "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    default:
      "bg-emerald-500/10 text-emerald-600 border-emerald-500/20 dark:bg-emerald-500/15 dark:text-emerald-400 dark:border-emerald-500/25",
    npm: "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25",
    pnpm: "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    yarn: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    bun: "bg-amber-500/10 text-amber-600 border-amber-500/20 dark:bg-amber-500/15 dark:text-amber-400 dark:border-amber-500/25",
    pip: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
    cargo:
      "bg-orange-500/10 text-orange-600 border-orange-500/20 dark:bg-orange-500/15 dark:text-orange-400 dark:border-orange-500/25",
    gem: "bg-red-500/10 text-red-600 border-red-500/20 dark:bg-red-500/15 dark:text-red-400 dark:border-red-500/25",
    docker:
      "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:bg-blue-500/15 dark:text-blue-400 dark:border-blue-500/25",
  }
  return colors[source] || "bg-muted text-muted-foreground border-border"
}

/**
 * Flattens every toolchain result into enriched outdated entries,
 * covering Homebrew formulae/casks, global packages, and generic outdated lists.
 */
export function enrichOutdated(
  results?: Record<string, any>
): OutdatedPackage[] {
  if (!results) return []
  const outdated: OutdatedPackage[] = []

  Object.entries(results).forEach(([toolchain, data]: [string, any]) => {
    const push = (
      list: { name: string; current: string; latest: string }[],
      source: string
    ) => {
      list.forEach((o) => {
        outdated.push({
          ...o,
          toolchain,
          source,
          updateType: getUpdateType(o.current, o.latest),
        })
      })
    }

    if (data.outdated_formulae) push(data.outdated_formulae, "formulae")
    if (data.outdated_casks) push(data.outdated_casks, "casks")
    if (data.outdated_global) push(data.outdated_global, "global")
    if (data.outdated) push(data.outdated, toolchain)
  })

  return outdated
}
