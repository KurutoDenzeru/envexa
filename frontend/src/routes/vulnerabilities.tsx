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
import { RefreshCw, ShieldAlert, CheckCircle, Search } from "lucide-react"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Input } from "@/components/ui/input"
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

export const Route = createFileRoute("/vulnerabilities")({ component: Vulnerabilities })

function Vulnerabilities() {
  const [report, setReport] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState("")
  const [page, setPage] = useState(1)
  const [itemsPerPage, setItemsPerPage] = useState(8)

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

  const filteredVulnerabilities = allVulnerabilities.filter(v => 
    v.package?.toLowerCase().includes(search.toLowerCase()) || 
    v.name?.toLowerCase().includes(search.toLowerCase()) ||
    v.toolchain?.toLowerCase().includes(search.toLowerCase())
  )

  useEffect(() => {
    setPage(1)
  }, [search])

  const totalPages = Math.ceil(filteredVulnerabilities.length / itemsPerPage)
  const paginatedVulnerabilities = filteredVulnerabilities.slice(
    (page - 1) * itemsPerPage,
    page * itemsPerPage
  )

  const renderPageNumbers = (currentPage: number, totalPages: number, setPage: (p: number) => void) => {
    const pages = []
    for (let i = 1; i <= totalPages; i++) {
      if (i === 1 || i === totalPages || (i >= currentPage - 1 && i <= currentPage + 1)) {
        pages.push(
          <PaginationItem key={i}>
            <PaginationLink 
              onClick={() => setPage(i)}
              isActive={currentPage === i}
              className={currentPage === i ? "bg-white/10" : "cursor-pointer hover:bg-white/5"}
            >
              {i}
            </PaginationLink>
          </PaginationItem>
        )
      } else if (i === currentPage - 2 || i === currentPage + 2) {
        pages.push(
          <PaginationItem key={i}>
            <span className="px-2 text-neutral-500">...</span>
          </PaginationItem>
        )
      }
    }
    return pages.filter((item, index, self) => 
      item.key !== null && self.findIndex(t => t.key === item.key) === index
    )
  }

  if (loading) {
    return (
      <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in duration-700">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-white/10 pb-6">
          <div>
            <Skeleton className="h-10 w-64 bg-white/10" />
            <Skeleton className="h-4 w-96 mt-3 bg-white/10" />
          </div>
        </div>
        <Card className="bg-neutral-950/50 border-white/10 backdrop-blur-xl">
          <CardHeader>
            <Skeleton className="h-6 w-48 bg-white/10" />
            <Skeleton className="h-4 w-64 mt-2 bg-white/10" />
          </CardHeader>
          <CardContent>
            <div className="flex flex-col items-center justify-center py-20 gap-4">
              <RefreshCw className="w-10 h-10 animate-spin text-red-500/50" />
              <Skeleton className="h-4 w-48 bg-white/10" />
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="max-w-6xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-white/10 pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-red-400 to-red-600 bg-clip-text text-transparent flex items-center gap-3">
            <ShieldAlert className="w-8 h-8 text-red-500" />
            Security Vulnerabilities
          </h1>
          <p className="text-neutral-400 mt-2">
            Detailed view of all security flaws found in your dependencies.
          </p>
        </div>
      </div>

      <Card className="bg-neutral-950/50 border-white/10 backdrop-blur-xl">
        <CardHeader className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <CardTitle>Identified Flaws</CardTitle>
            <CardDescription>Review and address these issues to secure your workspace.</CardDescription>
          </div>
          <div className="relative w-full md:w-64">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-neutral-500" />
            <Input 
              type="text" 
              placeholder="Search packages..." 
              className="pl-9 bg-black/50 border-white/10 focus-visible:ring-red-500" 
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </CardHeader>
        <CardContent>
          {filteredVulnerabilities.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-neutral-500">
              <CheckCircle className="w-12 h-12 mb-4 text-green-500/50" />
              <p>{search ? "No vulnerabilities match your search." : "No vulnerabilities detected! Your project is secure."}</p>
            </div>
          ) : (
            <>
              <div className="rounded-md border border-white/10 overflow-hidden">
                <Table>
                <TableHeader className="bg-white/5">
                  <TableRow className="border-white/10 hover:bg-transparent">
                    <TableHead className="w-[100px]">Toolchain</TableHead>
                    <TableHead className="w-[200px]">Package</TableHead>
                    <TableHead className="w-[100px]">Severity</TableHead>
                    <TableHead>Description</TableHead>
                    <TableHead className="text-right">Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {paginatedVulnerabilities.map((v, idx) => (
                    <TableRow key={idx} className="border-white/10 hover:bg-white/5">
                      <TableCell className="font-medium capitalize text-neutral-300">{v.toolchain}</TableCell>
                      <TableCell className="font-mono text-sm text-neutral-100">{v.name || v.package || "Unknown"}</TableCell>
                      <TableCell>
                        <Badge variant="destructive" className="bg-red-500/10 text-red-500 border-red-500/20 shadow-none">
                          {v.severity || "High"}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-neutral-400">{v.description || "Security vulnerability found"}</TableCell>
                      <TableCell className="text-right">
                        <button className="text-sm text-blue-400 hover:text-blue-300 underline underline-offset-4 transition-colors">
                          Remediate
                        </button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
            
            {totalPages > 1 && (
              <div className="mt-4 pt-4 border-t border-white/5 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-sm text-neutral-400">Rows per page</span>
                  <Select value={itemsPerPage.toString()} onValueChange={(v) => { setItemsPerPage(Number(v)); setPage(1); }}>
                    <SelectTrigger className="w-[70px] h-8 bg-neutral-900 border-white/10 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent className="bg-neutral-900 border-white/10">
                      <SelectItem value="5">5</SelectItem>
                      <SelectItem value="8">8</SelectItem>
                      <SelectItem value="15">15</SelectItem>
                      <SelectItem value="50">50</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <Pagination className="mx-0 w-auto">
                  <PaginationContent>
                    <PaginationItem>
                      <PaginationPrevious 
                        onClick={() => setPage(p => Math.max(1, p - 1))}
                        className={page === 1 ? "pointer-events-none opacity-50" : "cursor-pointer"} 
                      />
                    </PaginationItem>
                    {renderPageNumbers(page, totalPages, setPage)}
                    <PaginationItem>
                      <PaginationNext 
                        onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                        className={page === totalPages ? "pointer-events-none opacity-50" : "cursor-pointer"}
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
    </div>
  )
}
