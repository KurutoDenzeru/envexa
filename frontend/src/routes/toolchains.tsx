import { createFileRoute } from "@tanstack/react-router"
import { useEffect, useState, useMemo } from "react"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import {
  Boxes,
  PackageOpen,
  CheckCircle,
  Terminal,
  Cpu,
  RefreshCw,
} from "lucide-react"
import { Skeleton } from "@/components/ui/skeleton"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { ScrollArea } from "@/components/ui/scroll-area"

export const Route = createFileRoute("/toolchains")({ component: Toolchains })

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

interface ToolchainResult {
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
  supply_chain_risks?: Array<{
    package: string
    risk_type: string
    description: string
  }>
  audit_items?: Array<{
    name: string
    current: string
    note: string
  }>
}

interface ScanReport {
  results?: Record<string, ToolchainResult>
}

interface ToolCategory {
  name: string
  tools: string[]
}

const CATEGORIES: ToolCategory[] = [
  { name: "System & Runtime", tools: ["brew", "cargo", "docker", "pip", "gem"] },
  {
    name: "Web Development",
    tools: ["npm", "pnpm", "yarn", "bun", "deno"],
  },
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
        className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none"
      >
        FAIL
      </Badge>
    )
  }
  if (s.includes("warn")) {
    return (
      <Badge
        variant="outline"
        className="border-yellow-500/30 text-yellow-500 bg-yellow-500/10 shadow-none"
      >
        WARN
      </Badge>
    )
  }
  if (s.includes("skip") || s.includes("not found")) {
    return (
      <Badge variant="outline" className="border-border text-muted-foreground shadow-none">
        SKIP
      </Badge>
    )
  }
  return (
    <Badge
      variant="outline"
      className="border-green-500/30 text-green-500 bg-green-500/10 shadow-none"
    >
      PASS
    </Badge>
  )
}

function getPrimaryVersion(tc: ToolchainResult): string {
  return (
    tc.version ||
    tc.node_version ||
    tc.python_version ||
    tc.ruby_version ||
    tc.rustc_version ||
    tc.cargo_version ||
    tc.pnpm_version ||
    tc.bun_version ||
    tc.deno_version ||
    "-"
  )
}

function getVersionFields(tc: ToolchainResult): Array<{ label: string; value: string }> {
  const fields: Array<{ label: string; value: string }> = []
  if (tc.version) fields.push({ label: "Version", value: tc.version })
  if (tc.node_version) fields.push({ label: "Node Version", value: tc.node_version })
  if (tc.python_version)
    fields.push({ label: "Python Version", value: tc.python_version })
  if (tc.ruby_version) fields.push({ label: "Ruby Version", value: tc.ruby_version })
  if (tc.rustc_version)
    fields.push({ label: "Rustc Version", value: tc.rustc_version })
  if (tc.cargo_version)
    fields.push({ label: "Cargo Version", value: tc.cargo_version })
  if (tc.pnpm_version)
    fields.push({ label: "pnpm Version", value: tc.pnpm_version })
  if (tc.bun_version) fields.push({ label: "Bun Version", value: tc.bun_version })
  if (tc.deno_version)
    fields.push({ label: "Deno Version", value: tc.deno_version })
  if (tc.installed_count !== undefined)
    fields.push({ label: "Installed Count", value: String(tc.installed_count) })
  fields.push({ label: "Scanner Status", value: tc.status || "Ok" })
  if (tc.project_type)
    fields.push({ label: "Project Type", value: tc.project_type })
  return fields
}

function ToolchainCard({ tc }: { tc: ToolchainResult }) {
  const vulnCount = tc.vulnerabilities?.length || 0
  const outdatedCount = tc.outdated?.length || 0

  return (
    <Card className="bg-card border-border shadow-xs hover:border-border/80 transition-all duration-300 flex flex-col justify-between">
      <CardHeader className="border-b border-border/50 pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg capitalize text-foreground flex items-center gap-2">
            <Boxes className="w-4 h-4 text-muted-foreground" />
            {displayName(tc.tool)}
          </CardTitle>
          {statusBadge(tc.status)}
        </div>
      </CardHeader>

      <CardContent className="pt-4 flex-1 flex flex-col gap-4">
        <div className="flex items-center gap-2 text-xs font-mono text-muted-foreground/80 bg-muted/30 p-2.5 rounded-md border border-border/40">
          <Cpu className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <span className="text-foreground">{getPrimaryVersion(tc)}</span>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="bg-muted/50 rounded-lg p-3 border border-border/50">
            <div className="text-[10px] text-muted-foreground/60 mb-1 uppercase tracking-wider font-semibold">
              Vulns
            </div>
            <div
              className={`text-xl font-bold ${vulnCount > 0 ? "text-red-400" : "text-foreground/90"}`}
            >
              {vulnCount}
            </div>
          </div>
          <div className="bg-muted/50 rounded-lg p-3 border border-border/50">
            <div className="text-[10px] text-muted-foreground/60 mb-1 uppercase tracking-wider font-semibold">
              Outdated
            </div>
            <div
              className={`text-xl font-bold ${outdatedCount > 0 ? "text-blue-400" : "text-foreground/90"}`}
            >
              {outdatedCount}
            </div>
          </div>
        </div>

        <div className="flex justify-end pt-2 border-t border-border/30">
          <Dialog>
            <DialogTrigger className="inline-flex h-9 items-center justify-center rounded-md border border-border bg-popover px-4 text-xs font-semibold text-foreground transition-all hover:bg-muted/50 hover:text-foreground cursor-pointer select-none shadow-xs gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-muted-foreground" />
              View Details
            </DialogTrigger>
            <DialogContent className="sm:max-w-2xl bg-card border border-border p-6 shadow-xl max-h-[90vh] flex flex-col">
              <DialogHeader className="pb-4 border-b border-border/50">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-muted/60 border border-border/60">
                    <Boxes className="w-6 h-6 text-foreground capitalize" />
                  </div>
                  <div>
                    <DialogTitle className="text-2xl capitalize font-bold text-foreground">
                      {displayName(tc.tool)}
                    </DialogTitle>
                    <DialogDescription className="text-muted-foreground mt-0.5">
                      Vulnerability audit and package state.
                    </DialogDescription>
                  </div>
                </div>
              </DialogHeader>

              <div className="flex-1 overflow-y-auto py-4 flex flex-col gap-6">
                {/* Top Stats */}
                <div className="grid grid-cols-3 gap-4">
                  <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                    <span className="text-xs text-muted-foreground block mb-1">
                      Vulnerabilities
                    </span>
                    <span
                      className={`text-xl font-bold font-mono ${vulnCount > 0 ? "text-red-400" : "text-muted-foreground"}`}
                    >
                      {vulnCount}
                    </span>
                  </div>
                  <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                    <span className="text-xs text-muted-foreground block mb-1">
                      Outdated
                    </span>
                    <span
                      className={`text-xl font-bold font-mono ${outdatedCount > 0 ? "text-blue-400" : "text-muted-foreground"}`}
                    >
                      {outdatedCount}
                    </span>
                  </div>
                  <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                    <span className="text-xs text-muted-foreground block mb-1">
                      Status
                    </span>
                    <div className="flex justify-center">{statusBadge(tc.status)}</div>
                  </div>
                </div>

                <Tabs
                  defaultValue={
                    vulnCount > 0
                      ? "security"
                      : outdatedCount > 0
                        ? "updates"
                        : "specs"
                  }
                  className="w-full"
                >
                  <TabsList className="grid w-full grid-cols-3 bg-muted/50 p-1 rounded-lg border border-border/40">
                    <TabsTrigger value="security" className="text-xs py-1.5">
                      Security ({vulnCount})
                    </TabsTrigger>
                    <TabsTrigger value="updates" className="text-xs py-1.5">
                      Updates ({outdatedCount})
                    </TabsTrigger>
                    <TabsTrigger value="specs" className="text-xs py-1.5">
                      Specs
                    </TabsTrigger>
                  </TabsList>

                  {/* Security Tab */}
                  <TabsContent
                    value="security"
                    className="mt-4 focus-visible:outline-none"
                  >
                    {vulnCount === 0 ? (
                      <div className="flex flex-col items-center justify-center py-12 text-center border border-dashed border-border/40 rounded-xl bg-muted/10">
                        <CheckCircle className="w-10 h-10 mb-3 text-green-500/60" />
                        <h4 className="font-semibold text-foreground text-sm">
                          No Security Flaws Detected
                        </h4>
                        <p className="text-xs text-muted-foreground max-w-sm mt-1">
                          This toolchain has no known active security alerts.
                        </p>
                      </div>
                    ) : (
                      <ScrollArea className="max-h-[300px] border border-border/40 rounded-lg bg-muted/10 overflow-y-auto">
                        <Table>
                          <TableHeader className="bg-muted/50 sticky top-0 z-10">
                            <TableRow className="border-border hover:bg-transparent">
                              <TableHead className="text-xs h-9">Package</TableHead>
                              <TableHead className="text-xs h-9">Title</TableHead>
                              <TableHead className="text-xs h-9">CVE</TableHead>
                              <TableHead className="text-xs h-9">Severity</TableHead>
                              <TableHead className="text-xs h-9">Patched</TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {tc.vulnerabilities?.map((v, vIdx) => (
                              <TableRow
                                key={vIdx}
                                className="border-border hover:bg-muted/30"
                              >
                                <TableCell className="font-medium text-xs font-mono">
                                  {v.package}
                                </TableCell>
                                <TableCell className="text-xs">
                                  <div className="font-semibold text-foreground">
                                    {v.title}
                                  </div>
                                </TableCell>
                                <TableCell className="text-xs font-mono text-muted-foreground">
                                  {v.cve || "-"}
                                </TableCell>
                                <TableCell>
                                  <Badge
                                    variant="destructive"
                                    className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none text-[10px] h-5 px-1.5"
                                  >
                                    {v.severity}
                                  </Badge>
                                </TableCell>
                                <TableCell className="text-xs font-mono text-muted-foreground">
                                  {v.patched_version || "-"}
                                </TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      </ScrollArea>
                    )}
                  </TabsContent>

                  {/* Updates Tab */}
                  <TabsContent
                    value="updates"
                    className="mt-4 focus-visible:outline-none"
                  >
                    {outdatedCount === 0 ? (
                      <div className="flex flex-col items-center justify-center py-12 text-center border border-dashed border-border/40 rounded-xl bg-muted/10">
                        <CheckCircle className="w-10 h-10 mb-3 text-green-500/60" />
                        <h4 className="font-semibold text-foreground text-sm">
                          All Dependencies Up to Date
                        </h4>
                        <p className="text-xs text-muted-foreground max-w-sm mt-1">
                          This toolchain uses the latest available package releases.
                        </p>
                      </div>
                    ) : (
                      <ScrollArea className="max-h-[300px] border border-border/40 rounded-lg bg-muted/10 overflow-y-auto">
                        <Table>
                          <TableHeader className="bg-muted/50 sticky top-0 z-10">
                            <TableRow className="border-border hover:bg-transparent">
                              <TableHead className="text-xs h-9">Package</TableHead>
                              <TableHead className="text-xs h-9">Current</TableHead>
                              <TableHead className="text-xs h-9 text-right">
                                Latest
                              </TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {tc.outdated?.map((o, oIdx) => (
                              <TableRow
                                key={oIdx}
                                className="border-border hover:bg-muted/30"
                              >
                                <TableCell className="font-medium text-xs font-mono">
                                  {o.name}
                                </TableCell>
                                <TableCell className="text-xs text-muted-foreground font-mono">
                                  {o.current}
                                </TableCell>
                                <TableCell className="text-right">
                                  <Badge
                                    variant="outline"
                                    className="border-blue-500/30 text-blue-400 bg-blue-500/10 shadow-none text-[10px] h-5 px-1.5"
                                  >
                                    {o.latest}
                                  </Badge>
                                </TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      </ScrollArea>
                    )}
                  </TabsContent>

                  {/* Specs Tab */}
                  <TabsContent
                    value="specs"
                    className="mt-4 focus-visible:outline-none"
                  >
                    <div className="border border-border/40 rounded-lg overflow-hidden bg-muted/10">
                      <Table>
                        <TableBody>
                          {getVersionFields(tc).map((v, vIdx) => (
                            <TableRow
                              key={vIdx}
                              className="border-border hover:bg-muted/30"
                            >
                              <TableCell className="font-medium text-xs text-muted-foreground">
                                {v.label}
                              </TableCell>
                              <TableCell className="text-xs text-right font-mono text-foreground">
                                {v.value}
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  </TabsContent>
                </Tabs>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </CardContent>
    </Card>
  )
}

function Toolchains() {
  const [report, setReport] = useState<ScanReport | null>(null)
  const [loading, setLoading] = useState(true)

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

  const toolchainMap = useMemo(() => {
    if (!report?.results) return new Map<string, ToolchainResult>()
    const map = new Map<string, ToolchainResult>()
    for (const [name, data] of Object.entries(report.results)) {
      map.set(name, { ...data, tool: name })
    }
    return map
  }, [report])

  const groupedCategories = useMemo(() => {
    return CATEGORIES.map((cat) => ({
      ...cat,
      items: cat.tools
        .map((tool) => toolchainMap.get(tool))
        .filter((tc): tc is ToolchainResult => tc !== undefined),
    }))
  }, [toolchainMap])

  const totalTools = groupedCategories.reduce(
    (sum, cat) => sum + cat.items.length,
    0,
  )

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto flex flex-col gap-6 animate-in fade-in duration-700">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-64 bg-muted" />
            <Skeleton className="h-4 w-96 mt-3 bg-muted" />
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton className="h-56 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-56 w-full rounded-xl bg-muted/50" />
        </div>
      </div>
    )
  }

  return (
    <div className="max-w-7xl mx-auto flex flex-col gap-6">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <Boxes className="w-8 h-8 text-foreground" />
            Environment Toolchains
          </h1>
          <p className="text-sm text-muted-foreground mt-2">
            Package managers and runtimes detected in your environment.
          </p>
        </div>
        <Button variant="outline" className="gap-2 shadow-xs" onClick={fetchReport}>
          <RefreshCw className="w-4 h-4" /> Rescan
        </Button>
      </div>

      {totalTools === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 bg-muted/50 border border-border rounded-xl">
          <PackageOpen className="w-12 h-12 mb-4 text-neutral-600" />
          <p className="text-muted-foreground">No toolchains detected.</p>
        </div>
      ) : (
        groupedCategories.map((cat) => (
          <div key={cat.name} className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold text-foreground tracking-tight">
              {cat.name}
            </h2>
            {cat.items.length === 0 ? (
              <div className="flex items-center justify-center py-8 bg-muted/30 border border-dashed border-border/50 rounded-xl">
                <p className="text-sm text-muted-foreground/60">
                  No {cat.name.toLowerCase()} toolchains detected.
                </p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {cat.items.map((tc) => (
                  <ToolchainCard key={tc.tool} tc={tc} />
                ))}
              </div>
            )}
          </div>
        ))
      )}
    </div>
  )
}
