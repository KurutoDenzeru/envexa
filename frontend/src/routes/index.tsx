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
import { Progress } from "@/components/ui/progress"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ShieldAlert, RefreshCw, Box, CheckCircle, Activity } from "lucide-react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationNext,
  PaginationPrevious,
  PaginationLink,
} from "@/components/ui/pagination"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

export const Route = createFileRoute("/")({ component: App })

export default function App() {
  const [report, setReport] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [vulnPage, setVulnPage] = useState(1)
  const [outdatedPage, setOutdatedPage] = useState(1)
  const [itemsPerPage, setItemsPerPage] = useState(5)

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

  const chartData = useMemo(() => {
    if (!report?.results) return []
    return Object.entries(report.results).map(([toolchain, data]: [string, any]) => ({
      name: toolchain.charAt(0).toUpperCase() + toolchain.slice(1),
      vulnerabilities: data.vulnerabilities?.length || 0,
      outdated: data.outdated?.length || 0,
    }))
  }, [report])

  const allVulnerabilities = useMemo(() => {
    if (!report?.results) return []
    const vulns: any[] = []
    Object.entries(report.results).forEach(([toolchain, data]: [string, any]) => {
      if (data.vulnerabilities) {
        data.vulnerabilities.forEach((v: any) => {
          vulns.push({ ...v, toolchain })
        })
      }
    })
    return vulns
  }, [report])

  const allOutdated = useMemo(() => {
    if (!report?.results) return []
    const out: any[] = []
    Object.entries(report.results).forEach(([toolchain, data]: [string, any]) => {
      if (data.outdated) {
        data.outdated.forEach((o: any) => {
          out.push({ ...o, toolchain })
        })
      }
    })
    return out
  }, [report])

  const vulnCount = allVulnerabilities.length
  const outCount = allOutdated.length
  const healthScore = Math.max(0, 100 - (vulnCount * 10) - (outCount * 2))

  const totalVulnPages = Math.ceil(vulnCount / itemsPerPage)
  const paginatedVulns = allVulnerabilities.slice((vulnPage - 1) * itemsPerPage, vulnPage * itemsPerPage)

  const totalOutdatedPages = Math.ceil(outCount / itemsPerPage)
  const paginatedOutdated = allOutdated.slice((outdatedPage - 1) * itemsPerPage, outdatedPage * itemsPerPage)

  const renderPageNumbers = (currentPage: number, totalPages: number, setPage: (p: number) => void) => {
    const pages = []
    for (let i = 1; i <= totalPages; i++) {
      if (i === 1 || i === totalPages || (i >= currentPage - 1 && i <= currentPage + 1)) {
        pages.push(
          <PaginationItem key={i}>
            <PaginationLink 
              onClick={() => setPage(i)}
              isActive={currentPage === i}
              className={currentPage === i ? "bg-muted" : "cursor-pointer hover:bg-muted/50"}
            >
              {i}
            </PaginationLink>
          </PaginationItem>
        )
      } else if (i === currentPage - 2 || i === currentPage + 2) {
        pages.push(
          <PaginationItem key={i}>
            <span className="px-2 text-muted-foreground/60">...</span>
          </PaginationItem>
        )
      }
    }
    // Remove duplicates from ...
    return pages.filter((item, index, self) => 
      item.key !== null && self.findIndex(t => t.key === item.key) === index
    )
  }

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in duration-700">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-64 bg-muted" />
            <Skeleton className="h-4 w-96 mt-3 bg-muted" />
          </div>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <Skeleton className="h-32 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-32 w-full rounded-xl bg-muted/50" />
          <Skeleton className="h-32 w-full rounded-xl bg-muted/50" />
        </div>

        <Card className="bg-card/50 border-border backdrop-blur-xl">
          <CardHeader>
            <Skeleton className="h-6 w-48 bg-muted" />
            <Skeleton className="h-4 w-64 mt-2 bg-muted" />
          </CardHeader>
          <CardContent>
            <div className="flex flex-col items-center justify-center py-20 gap-4">
              <RefreshCw className="w-10 h-10 animate-spin text-muted-foreground/50" />
              <Skeleton className="h-4 w-48 bg-muted" />
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (!report) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] gap-4">
        <ShieldAlert className="w-12 h-12 text-muted-foreground" />
        <h2 className="text-xl font-medium tracking-tight">Failed to load environment report</h2>
        <button onClick={fetchReport} className="text-sm bg-muted hover:bg-primary/20 px-4 py-2 rounded-md transition-colors">
          Retry Scan
        </button>
      </div>
    )
  }

  return (
    <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">
            Workspace Overview
          </h1>
          <p className="text-muted-foreground mt-2 flex items-center gap-2">
            <CheckCircle className="w-4 h-4 text-muted-foreground" />
            Project scanned at {new Date().toLocaleTimeString()}
          </p>
        </div>
        <button
          onClick={fetchReport}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-foreground px-4 py-2 rounded-md transition-all font-medium text-sm shadow-xs"
        >
          <RefreshCw className="w-4 h-4" />
          Rescan Now
        </button>
      </div>

      {/* Top Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card className="bg-card border-border shadow-xs hover:border-border/80 transition-all">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">System Health</CardTitle>
            <Activity className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground mb-2">{healthScore}%</div>
            <Progress value={healthScore} className="h-1.5" />
            <p className="text-xs text-muted-foreground/60 mt-3">{healthScore > 80 ? "Optimal configuration" : "Needs attention"}</p>
          </CardContent>
        </Card>

        <Card className="bg-card border-border shadow-xs hover:border-border/80 transition-all">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Vulnerabilities</CardTitle>
            <ShieldAlert className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">{vulnCount}</div>
            <p className="text-xs text-muted-foreground/60 mt-1">Detected across toolchains</p>
          </CardContent>
        </Card>

        <Card className="bg-card border-border shadow-xs hover:border-border/80 transition-all">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Outdated Packages</CardTitle>
            <Box className="w-4 h-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-foreground">{outCount}</div>
            <p className="text-xs text-muted-foreground/60 mt-1">Updates available</p>
          </CardContent>
        </Card>
      </div>

      {/* Main Tabs Area */}
      <Tabs defaultValue="analytics" className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-popover/50 p-1 rounded-lg border border-border/50">
          <TabsTrigger value="analytics">Analytics</TabsTrigger>
          <TabsTrigger value="vulnerabilities">
            Vulnerabilities {vulnCount > 0 && <Badge variant="destructive" className="ml-2 bg-red-500/20 text-red-400 border-0">{vulnCount}</Badge>}
          </TabsTrigger>
          <TabsTrigger value="updates">
            Updates {outCount > 0 && <Badge variant="secondary" className="ml-2 bg-blue-500/20 text-blue-400 border-0">{outCount}</Badge>}
          </TabsTrigger>
        </TabsList>
        
        {/* Analytics Tab */}
        <TabsContent value="analytics" className="mt-6">
          <Card className="bg-card/50 border-border backdrop-blur-xl">
            <CardHeader>
              <CardTitle>Issues by Toolchain</CardTitle>
              <CardDescription>Distribution of security and maintenance warnings</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="h-[300px] w-full mt-4">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#333" vertical={false} />
                    <XAxis dataKey="name" stroke="#888" tick={{ fill: '#888' }} axisLine={false} tickLine={false} />
                    <YAxis stroke="#888" tick={{ fill: '#888' }} axisLine={false} tickLine={false} />
                    <Tooltip 
                      cursor={{ fill: 'rgba(255,255,255,0.05)' }}
                      contentStyle={{ backgroundColor: '#171717', border: '1px solid rgba(255,255,255,0.1)', borderRadius: '8px' }}
                    />
                    <Bar dataKey="vulnerabilities" name="Vulnerabilities" fill="#f87171" radius={[4, 4, 0, 0]} maxBarSize={50} />
                    <Bar dataKey="outdated" name="Outdated" fill="#60a5fa" radius={[4, 4, 0, 0]} maxBarSize={50} />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Vulnerabilities Tab */}
        <TabsContent value="vulnerabilities" className="mt-6">
          <Card className="bg-card/50 border-border backdrop-blur-xl">
            <CardHeader>
              <CardTitle>Security Issues</CardTitle>
              <CardDescription>Critical security flaws requiring immediate action</CardDescription>
            </CardHeader>
            <CardContent>
              {allVulnerabilities.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
                  <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
                  <p>No vulnerabilities detected! Great job.</p>
                </div>
              ) : (
                <>
                  <div className="rounded-md border border-border">
                    <Table>
                      <TableHeader className="bg-muted/50">
                        <TableRow className="border-border hover:bg-transparent">
                          <TableHead>Toolchain</TableHead>
                          <TableHead>Package</TableHead>
                          <TableHead>Severity</TableHead>
                          <TableHead>Description</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {paginatedVulns.map((v, idx) => (
                          <TableRow key={idx} className="border-border hover:bg-muted/50">
                            <TableCell className="font-medium capitalize">{v.toolchain}</TableCell>
                            <TableCell>{v.name || v.package || "Unknown"}</TableCell>
                            <TableCell>
                              <Badge variant="destructive" className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none">
                                {v.severity || "High"}
                              </Badge>
                            </TableCell>
                            <TableCell className="text-muted-foreground">{v.description || "Security vulnerability found"}</TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                  
                  {totalVulnPages > 1 && (
                    <div className="mt-4 pt-4 border-t border-border/50 flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-sm text-muted-foreground">Rows per page</span>
                        <Select value={itemsPerPage.toString()} onValueChange={(v) => setItemsPerPage(Number(v))}>
                          <SelectTrigger className="w-[70px] h-8 bg-popover border-border text-xs">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent className="bg-popover border-border">
                            <SelectItem value="5">5</SelectItem>
                            <SelectItem value="10">10</SelectItem>
                            <SelectItem value="20">20</SelectItem>
                            <SelectItem value="50">50</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <Pagination className="mx-0 w-auto">
                        <PaginationContent>
                          <PaginationItem>
                            <PaginationPrevious 
                              onClick={() => setVulnPage(p => Math.max(1, p - 1))}
                              className={vulnPage === 1 ? "pointer-events-none opacity-50" : "cursor-pointer"} 
                            />
                          </PaginationItem>
                          {renderPageNumbers(vulnPage, totalVulnPages, setVulnPage)}
                          <PaginationItem>
                            <PaginationNext 
                              onClick={() => setVulnPage(p => Math.min(totalVulnPages, p + 1))}
                              className={vulnPage === totalVulnPages ? "pointer-events-none opacity-50" : "cursor-pointer"}
                            />
                          </PaginationItem>
                        </PaginationContent>
                      </Pagination>
                    </div>
                  )}
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Updates Tab */}
        <TabsContent value="updates" className="mt-6">
          <Card className="bg-card/50 border-border backdrop-blur-xl">
            <CardHeader>
              <CardTitle>Available Updates</CardTitle>
              <CardDescription>Packages with newer versions available</CardDescription>
            </CardHeader>
            <CardContent>
              {allOutdated.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/60">
                  <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
                  <p>All dependencies are up to date.</p>
                </div>
              ) : (
                <>
                  <div className="rounded-md border border-border">
                    <Table>
                      <TableHeader className="bg-muted/50">
                        <TableRow className="border-border hover:bg-transparent">
                          <TableHead>Toolchain</TableHead>
                          <TableHead>Package</TableHead>
                          <TableHead>Current</TableHead>
                          <TableHead>Latest</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {paginatedOutdated.map((o, idx) => (
                          <TableRow key={idx} className="border-border hover:bg-muted/50">
                            <TableCell className="font-medium capitalize">{o.toolchain}</TableCell>
                            <TableCell>{o.name}</TableCell>
                            <TableCell className="text-muted-foreground">{o.version}</TableCell>
                            <TableCell>
                              <Badge variant="outline" className="border-blue-500/30 text-blue-400 bg-blue-500/10">
                                {o.latest}
                              </Badge>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                  
                  {totalOutdatedPages > 1 && (
                    <div className="mt-4 pt-4 border-t border-border/50 flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-sm text-muted-foreground">Rows per page</span>
                        <Select value={itemsPerPage.toString()} onValueChange={(v) => setItemsPerPage(Number(v))}>
                          <SelectTrigger className="w-[70px] h-8 bg-popover border-border text-xs">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent className="bg-popover border-border">
                            <SelectItem value="5">5</SelectItem>
                            <SelectItem value="10">10</SelectItem>
                            <SelectItem value="20">20</SelectItem>
                            <SelectItem value="50">50</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <Pagination className="mx-0 w-auto">
                        <PaginationContent>
                          <PaginationItem>
                            <PaginationPrevious 
                              onClick={() => setOutdatedPage(p => Math.max(1, p - 1))}
                              className={outdatedPage === 1 ? "pointer-events-none opacity-50" : "cursor-pointer"} 
                            />
                          </PaginationItem>
                          {renderPageNumbers(outdatedPage, totalOutdatedPages, setOutdatedPage)}
                          <PaginationItem>
                            <PaginationNext 
                              onClick={() => setOutdatedPage(p => Math.min(totalOutdatedPages, p + 1))}
                              className={outdatedPage === totalOutdatedPages ? "pointer-events-none opacity-50" : "cursor-pointer"}
                            />
                          </PaginationItem>
                        </PaginationContent>
                      </Pagination>
                    </div>
                  )}
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

      </Tabs>
    </div>
  )
}
