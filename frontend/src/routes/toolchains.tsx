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
import { Boxes, PackageOpen, AlertTriangle } from "lucide-react"
import { Progress } from "@/components/ui/progress"
import { Skeleton } from "@/components/ui/skeleton"

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
            
            return (
              <Card key={idx} className="bg-card border-border shadow-md hover:border-border/80 transition-all duration-300">
                <CardHeader className="border-b border-border/50 pb-4 bg-muted/30">
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
                <CardContent className="space-y-6">
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
                </CardContent>
              </Card>
            )
          })
        )}
      </div>
    </div>
  )
}
