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
import { Boxes, PackageOpen, AlertTriangle, CheckCircle, Terminal, Cpu } from "lucide-react"
import { Progress } from "@/components/ui/progress"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { ScrollArea } from "@/components/ui/scroll-area"

export const Route = createFileRoute("/toolchains")({ component: Toolchains })

function Toolchains() {
  const [report, setReport] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  const fetchReport = async () => {
    setLoading(true)
    try {
      const res = await fetch("/api/scan")
      const data = await res.json()
      setReport(data)
    } catch (e) {
      console.error("Failed to fetch report", e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchReport()
  }, [])

  const toolchainList = useMemo(() => {
    if (!report?.results) return []
    return Object.entries(report.results).map(([name, data]: [string, any]) => ({
      name,
      vulnCount: data.vulnerabilities?.length || 0,
      outdatedCount: data.outdated?.length || 0,
      details: data,
    }))
  }, [report])

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in duration-700">
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
    <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <Boxes className="w-8 h-8 text-foreground" />
            Environment Toolchains
          </h1>
          <p className="text-muted-foreground mt-2">
            Inspect the package managers and environments detected in your project.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {toolchainList.length === 0 ? (
          <div className="col-span-1 md:col-span-2 flex flex-col items-center justify-center py-12 bg-muted/50 border border-border rounded-xl">
            <PackageOpen className="w-12 h-12 mb-4 text-neutral-600" />
            <p className="text-muted-foreground">No toolchains detected.</p>
          </div>
        ) : (
          toolchainList.map((tc, idx) => {
            const healthScore = Math.max(0, 100 - (tc.vulnCount * 15) - (tc.outdatedCount * 5))
            
            // Collect version strings
            const versions: { label: string, value: string }[] = []
            if (tc.details.version) versions.push({ label: "Version", value: tc.details.version })
            if (tc.details.node_version) versions.push({ label: "Node Version", value: tc.details.node_version })
            if (tc.details.python_version) versions.push({ label: "Python Version", value: tc.details.python_version })
            if (tc.details.rustc_version) versions.push({ label: "Rustc Version", value: tc.details.rustc_version })
            if (tc.details.cargo_version) versions.push({ label: "Cargo Version", value: tc.details.cargo_version })
            if (tc.details.bun_version) versions.push({ label: "Bun Version", value: tc.details.bun_version })
            if (tc.details.deno_version) versions.push({ label: "Deno Version", value: tc.details.deno_version })

            return (
              <Card key={idx} className="bg-card border-border shadow-xs hover:border-border/80 transition-all duration-300 flex flex-col justify-between">
                <CardHeader className="border-b border-border/50 pb-4">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-2xl capitalize text-foreground flex items-center gap-2">
                      <Boxes className="w-5 h-5 text-muted-foreground" />
                      {tc.name}
                    </CardTitle>
                    {tc.vulnCount > 0 ? (
                      <Badge variant="destructive" className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none gap-1">
                        <AlertTriangle className="w-3 h-3" /> Critical
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="border-green-500/30 text-green-400 bg-green-500/10">Healthy</Badge>
                    )}
                  </div>
                  <CardDescription className="text-muted-foreground/60">Dependencies managed by {tc.name}</CardDescription>
                </CardHeader>
                
                <CardContent className="space-y-6 pt-4 flex-1">
                  <div>
                    <div className="flex justify-between text-sm mb-2">
                      <span className="text-muted-foreground">Subsystem Health</span>
                      <span className="font-mono text-foreground/90">{healthScore}%</span>
                    </div>
                    <Progress value={healthScore} className="h-2" />
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4">
                    <div className="bg-muted/50 rounded-lg p-4 border border-border/50">
                      <div className="text-xs text-muted-foreground/60 mb-1 uppercase tracking-wider font-semibold">Vulnerabilities</div>
                      <div className={`text-2xl font-bold ${tc.vulnCount > 0 ? 'text-red-400' : 'text-foreground/90'}`}>
                        {tc.vulnCount}
                      </div>
                    </div>
                    <div className="bg-muted/50 rounded-lg p-4 border border-border/50">
                      <div className="text-xs text-muted-foreground/60 mb-1 uppercase tracking-wider font-semibold">Updates</div>
                      <div className={`text-2xl font-bold ${tc.outdatedCount > 0 ? 'text-blue-400' : 'text-foreground/90'}`}>
                        {tc.outdatedCount}
                      </div>
                    </div>
                  </div>

                  {versions.length > 0 && (
                    <div className="text-xs font-mono text-muted-foreground/80 flex items-center gap-2 bg-muted/30 p-2.5 rounded-md border border-border/40">
                      <Cpu className="w-3.5 h-3.5 text-muted-foreground" />
                      <span className="font-semibold">{versions[0].label}:</span>
                      <span className="text-foreground">{versions[0].value}</span>
                    </div>
                  )}

                  <div className="flex justify-end pt-2 border-t border-border/30">
                    <Dialog>
                      <DialogTrigger className="inline-flex h-9 items-center justify-center rounded-md border border-border bg-popover px-4 text-xs font-semibold text-foreground transition-all hover:bg-muted/50 hover:text-foreground cursor-pointer select-none shadow-xs gap-1.5">
                        <Terminal className="w-3.5 h-3.5 text-muted-foreground" />
                        View Subsystem Details
                      </DialogTrigger>
                      <DialogContent className="sm:max-w-2xl bg-card border border-border p-6 shadow-xl max-h-[90vh] flex flex-col">
                        <DialogHeader className="pb-4 border-b border-border/50">
                          <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-muted/60 border border-border/60">
                              <Boxes className="w-6 h-6 text-foreground capitalize" />
                            </div>
                            <div>
                              <DialogTitle className="text-2xl capitalize font-bold text-foreground">
                                {tc.name} Subsystem Details
                              </DialogTitle>
                              <DialogDescription className="text-muted-foreground mt-0.5">
                                Comprehensive vulnerability audit & packages state for {tc.name}.
                              </DialogDescription>
                            </div>
                          </div>
                        </DialogHeader>

                        <div className="flex-1 overflow-y-auto py-4 space-y-6">
                          {/* Top Specs Grid */}
                          <div className="grid grid-cols-3 gap-4">
                            <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                              <span className="text-xs text-muted-foreground block mb-1">Health Score</span>
                              <span className={`text-xl font-bold font-mono ${healthScore >= 90 ? 'text-green-400' : healthScore >= 70 ? 'text-yellow-400' : 'text-red-400'}`}>
                                {healthScore}%
                              </span>
                            </div>
                            <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                              <span className="text-xs text-muted-foreground block mb-1">Vulnerabilities</span>
                              <span className={`text-xl font-bold font-mono ${tc.vulnCount > 0 ? 'text-red-400' : 'text-muted-foreground'}`}>
                                {tc.vulnCount}
                              </span>
                            </div>
                            <div className="bg-muted/30 rounded-xl p-3.5 border border-border/40 text-center">
                              <span className="text-xs text-muted-foreground block mb-1">Outdated</span>
                              <span className={`text-xl font-bold font-mono ${tc.outdatedCount > 0 ? 'text-blue-400' : 'text-muted-foreground'}`}>
                                {tc.outdatedCount}
                              </span>
                            </div>
                          </div>

                          <Tabs defaultValue={tc.vulnCount > 0 ? "vulnerabilities" : tc.outdatedCount > 0 ? "updates" : "specs"} className="w-full">
                            <TabsList className="grid w-full grid-cols-3 bg-muted/50 p-1 rounded-lg border border-border/40">
                              <TabsTrigger value="vulnerabilities" className="text-xs py-1.5">
                                Security ({tc.vulnCount})
                              </TabsTrigger>
                              <TabsTrigger value="updates" className="text-xs py-1.5">
                                Updates ({tc.outdatedCount})
                              </TabsTrigger>
                              <TabsTrigger value="specs" className="text-xs py-1.5">
                                Subsystem Specs
                              </TabsTrigger>
                            </TabsList>

                            {/* Vulnerabilities Tab */}
                            <TabsContent value="vulnerabilities" className="mt-4 focus-visible:outline-none">
                              {tc.vulnCount === 0 ? (
                                <div className="flex flex-col items-center justify-center py-12 text-center border border-dashed border-border/40 rounded-xl bg-muted/10">
                                  <CheckCircle className="w-10 h-10 mb-3 text-green-500/60" />
                                  <h4 className="font-semibold text-foreground text-sm">No Security Flaws Detected</h4>
                                  <p className="text-xs text-muted-foreground max-w-sm mt-1">This subsystem's dependency tree has no known active security alerts.</p>
                                </div>
                              ) : (
                                <ScrollArea className="max-h-[300px] border border-border/40 rounded-lg bg-muted/10 overflow-y-auto">
                                  <Table>
                                    <TableHeader className="bg-muted/50 sticky top-0 z-10">
                                      <TableRow className="border-border hover:bg-transparent">
                                        <TableHead className="text-xs h-9">Package</TableHead>
                                        <TableHead className="text-xs h-9">Vulnerability</TableHead>
                                        <TableHead className="text-xs h-9 text-right">Severity</TableHead>
                                      </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                      {tc.details.vulnerabilities.map((v: any, vIdx: number) => (
                                        <TableRow key={vIdx} className="border-border hover:bg-muted/30">
                                          <TableCell className="font-medium text-xs font-mono">{v.package}</TableCell>
                                          <TableCell className="text-xs">
                                            <div className="font-semibold text-foreground">{v.title}</div>
                                            {v.cve && <div className="text-[10px] text-muted-foreground mt-0.5">{v.cve}</div>}
                                          </TableCell>
                                          <TableCell className="text-right">
                                            <Badge variant="destructive" className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none text-[10px] h-5 px-1.5">
                                              {v.severity}
                                            </Badge>
                                          </TableCell>
                                        </TableRow>
                                      ))}
                                    </TableBody>
                                  </Table>
                                </ScrollArea>
                              )}
                            </TabsContent>

                            {/* Updates Tab */}
                            <TabsContent value="updates" className="mt-4 focus-visible:outline-none">
                              {tc.outdatedCount === 0 ? (
                                <div className="flex flex-col items-center justify-center py-12 text-center border border-dashed border-border/40 rounded-xl bg-muted/10">
                                  <CheckCircle className="w-10 h-10 mb-3 text-green-500/60" />
                                  <h4 className="font-semibold text-foreground text-sm">All Dependencies Up to Date</h4>
                                  <p className="text-xs text-muted-foreground max-w-sm mt-1">This subsystem uses the latest available package releases.</p>
                                </div>
                              ) : (
                                <ScrollArea className="max-h-[300px] border border-border/40 rounded-lg bg-muted/10 overflow-y-auto">
                                  <Table>
                                    <TableHeader className="bg-muted/50 sticky top-0 z-10">
                                      <TableRow className="border-border hover:bg-transparent">
                                        <TableHead className="text-xs h-9">Package</TableHead>
                                        <TableHead className="text-xs h-9">Current</TableHead>
                                        <TableHead className="text-xs h-9 text-right">Latest</TableHead>
                                      </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                      {tc.details.outdated.map((o: any, oIdx: number) => (
                                        <TableRow key={oIdx} className="border-border hover:bg-muted/30">
                                          <TableCell className="font-medium text-xs font-mono">{o.name}</TableCell>
                                          <TableCell className="text-xs text-muted-foreground font-mono">{o.version}</TableCell>
                                          <TableCell className="text-right">
                                            <Badge variant="outline" className="border-blue-500/30 text-blue-400 bg-blue-500/10 shadow-none text-[10px] h-5 px-1.5">
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
                            <TabsContent value="specs" className="mt-4 focus-visible:outline-none">
                              <div className="border border-border/40 rounded-lg overflow-hidden bg-muted/10">
                                <Table>
                                  <TableBody>
                                    {versions.map((v, vIdx) => (
                                      <TableRow key={vIdx} className="border-border hover:bg-muted/30">
                                        <TableCell className="font-medium text-xs text-muted-foreground">{v.label}</TableCell>
                                        <TableCell className="text-xs text-right font-mono text-foreground">{v.value}</TableCell>
                                      </TableRow>
                                    ))}
                                    <TableRow className="border-border hover:bg-muted/30">
                                      <TableCell className="font-medium text-xs text-muted-foreground">Scanner Status</TableCell>
                                      <TableCell className="text-xs text-right capitalize text-green-400 font-mono">
                                        {tc.details.status || "Ok"}
                                      </TableCell>
                                    </TableRow>
                                    {tc.details.project_type && (
                                      <TableRow className="border-border hover:bg-transparent">
                                        <TableCell className="font-medium text-xs text-muted-foreground">Project Subtype</TableCell>
                                        <TableCell className="text-xs text-right font-mono text-foreground capitalize">
                                          {tc.details.project_type}
                                        </TableCell>
                                      </TableRow>
                                    )}
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
          })
        )}
      </div>
    </div>
  )
}
