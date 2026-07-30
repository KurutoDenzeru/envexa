"use client"

import { useState, useMemo } from "react"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { AlertTriangle, Shield, Package, Copy, ExternalLink, Download } from "lucide-react"
import { useScanData } from "@/components/scan-data-context"
import { cn } from "@/lib/utils"
interface PackageDetailDialogProps {
  packageName: string
  toolchain: string
  open: boolean
  onClose: () => void
}

interface VulnerabilityInfo {
  package: string
  severity: string
  title: string
  cve?: string | null
  patched_version?: string
  dependency_path?: string
}

interface AuditItem {
  name: string
  current: string
  note: string
}

interface SupplyChainRisk {
  package: string
  risk_type: string
  description: string
}

interface PackageInfo {
  name: string
  current: string
  latest: string
}

interface ScanResult {
  vulnerabilities?: VulnerabilityInfo[]
  audit_items?: AuditItem[]
  supply_chain_risks?: SupplyChainRisk[]
  outdated?: PackageInfo[]
  issues?: string[]
}

function severityColor(severity: string): string {
  switch (severity.toLowerCase()) {
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

function severityIcon(severity: string) {
  switch (severity.toLowerCase()) {
    case "critical":
      return <Shield className="w-3 h-3 text-red-500" />
    case "high":
      return <Shield className="w-3 h-3 text-orange-500" />
    case "medium":
    case "moderate":
      return <Shield className="w-3 h-3 text-amber-500" />
    case "low":
      return <Shield className="w-3 h-3 text-emerald-500" />
    default:
      return <Badge className="w-3 h-3 text-muted-foreground" />
  }
}

function riskTypeColor(riskType: string): string {
  switch (riskType.toLowerCase()) {
    case "malicious":
    case "malware":
      return "bg-red-500/10 text-red-600 border-red-500/20"
    case "deprecated":
    case "unmaintained":
      return "bg-amber-500/10 text-amber-600 border-amber-500/20"
    case "license":
      return "bg-blue-500/10 text-blue-600 border-blue-500/20"
    case "supply chain":
    case "supply-chain":
      return "bg-purple-500/10 text-purple-600 border-purple-500/20"
    default:
      return "bg-muted text-muted-foreground border-border"
  }
}

function getUpdateCommand(toolchain: string, pkg: string, version: string): string {
  const commands: Record<string, string> = {
    npm: `npm install ${pkg}@${version}`,
    pnpm: `pnpm add ${pkg}@${version}`,
    yarn: `yarn add ${pkg}@${version}`,
    bun: `bun add ${pkg}@${version}`,
    cargo: `cargo add ${pkg}@${version}`,
    pip: `pip install ${pkg}==${version}`,
    pip3: `pip3 install ${pkg}==${version}`,
    pipx: `pipx install ${pkg}==${version}`,
    poetry: `poetry add ${pkg}@${version}`,
    uv: `uv add ${pkg}==${version}`,
    brew: `brew upgrade ${pkg}`,
    gem: `gem install ${pkg} -v ${version}`,
    bundle: `bundle update ${pkg}`,
    docker: `docker pull ${pkg}:${version}`,
    deno: `deno add ${pkg}@${version}`,
  }
  return commands[toolchain.toLowerCase()] || `${toolchain} install ${pkg}@${version}`
}

export function PackageDetailDialog({
  packageName,
  toolchain,
  open,
  onClose,
}: PackageDetailDialogProps) {
  const { report } = useScanData()
  const [activeTab, setActiveTab] = useState("overview")

  const scanResult = useMemo((): ScanResult | undefined => {
    if (!report?.results) return undefined
    return report.results[toolchain] as ScanResult | undefined
  }, [report, toolchain])

  const packageInfo = useMemo((): PackageInfo | undefined => {
    if (!scanResult?.outdated) return undefined
    return scanResult.outdated.find((p) => p.name === packageName)
  }, [scanResult, packageName])

  const vulnerabilities = useMemo((): VulnerabilityInfo[] => {
    if (!scanResult?.vulnerabilities) return []
    return scanResult.vulnerabilities.filter((v) => v.package === packageName)
  }, [scanResult, packageName])

  const auditItems = useMemo((): AuditItem[] => {
    if (!scanResult?.audit_items) return []
    return scanResult.audit_items.filter((a) => a.name === packageName)
  }, [scanResult, packageName])

  const supplyChainRisks = useMemo((): SupplyChainRisk[] => {
    if (!scanResult?.supply_chain_risks) return []
    return scanResult.supply_chain_risks.filter((r) => r.package === packageName)
  }, [scanResult, packageName])

  const severityCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const v of vulnerabilities) {
      const key = v.severity.toLowerCase()
      counts[key] = (counts[key] || 0) + 1
    }
    return counts
  }, [vulnerabilities])

  const totalVulns = vulnerabilities.length
  const criticalCount = severityCounts["critical"] || 0
  const highCount = severityCounts["high"] || 0
  const mediumCount = severityCounts["medium"] || 0
  const lowCount = severityCounts["low"] || 0

  const updateCommand = packageInfo
    ? getUpdateCommand(toolchain, packageName, packageInfo.latest)
    : ""

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (!open) return null

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden">
        <DialogHeader className="border-b pb-4">
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1 min-w-0">
              <DialogTitle className="text-xl font-semibold truncate">
                <Package className="w-5 h-5 inline-block mr-2 text-muted-foreground" />
                {packageName}
              </DialogTitle>
              <div className="flex items-center gap-3 mt-1 text-sm text-muted-foreground">
                <Badge variant="outline" className="text-xs">
                  {toolchain}
                </Badge>
                {packageInfo && (
                  <>
                    <Badge variant="secondary" className="text-xs">
                      Current: {packageInfo.current}
                    </Badge>
                    <Badge variant="outline" className="text-xs">
                      Latest: {packageInfo.latest}
                    </Badge>
                  </>
                )}
              </div>
            </div>
            <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0" onClick={onClose}>
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </Button>
          </div>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 overflow-hidden flex flex-col">
          <TabsList className="border-b bg-muted/30">
            <TabsTrigger value="overview" className="data-[state=active]:bg-background">
              Overview
              {packageInfo && (
                <Badge variant="secondary" className="ml-1.5 h-4 px-1.5 text-[10px]">
                  {packageInfo.current} → {packageInfo.latest}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="vulnerabilities" className="data-[state=active]:bg-background">
              Vulnerabilities
              {totalVulns > 0 && (
                <Badge
                  variant={
                    criticalCount > 0
                      ? "destructive"
                      : highCount > 0
                      ? "default"
                      : mediumCount > 0
                      ? "secondary"
                      : "outline"
                  }
                  className="ml-1.5 h-4 px-1.5 text-[10px]"
                >
                  {totalVulns}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="audit" className="data-[state=active]:bg-background">
              Audit
              {auditItems.length > 0 && (
                <Badge variant="outline" className="ml-1.5 h-4 px-1.5 text-[10px]">
                  {auditItems.length}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="supply-chain" className="data-[state=active]:bg-background">
              Supply Chain
              {supplyChainRisks.length > 0 && (
                <Badge variant="outline" className="ml-1.5 h-4 px-1.5 text-[10px]">
                  {supplyChainRisks.length}
                </Badge>
              )}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="flex-1 overflow-auto p-4 space-y-6">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-muted-foreground">Package</h4>
                <p className="text-lg font-mono font-medium">{packageName}</p>
              </div>
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-muted-foreground">Toolchain</h4>
                <Badge variant="outline">{toolchain}</Badge>
              </div>
            </div>

            {packageInfo && (
              <div className="grid gap-4 md:grid-cols-3">
                <div className="space-y-2 p-4 bg-muted/30 rounded-lg border">
                  <h4 className="text-sm font-medium text-muted-foreground">Current Version</h4>
                  <p className="text-xl font-mono font-medium">{packageInfo.current}</p>
                </div>
                <div className="space-y-2 p-4 bg-muted/30 rounded-lg border">
                  <h4 className="text-sm font-medium text-muted-foreground">Latest Version</h4>
                  <p className="text-xl font-mono font-medium text-green-600 dark:text-green-400">
                    {packageInfo.latest}
                  </p>
                </div>
                <div className="space-y-2 p-4 bg-muted/30 rounded-lg border">
                  <h4 className="text-sm font-medium text-muted-foreground">Status</h4>
                  <Badge
                    variant={packageInfo.current === packageInfo.latest ? "default" : "destructive"}
                    className="text-sm"
                  >
                    {packageInfo.current === packageInfo.latest ? "Up to date" : "Update available"}
                  </Badge>
                </div>
              </div>
            )}

            {packageInfo && packageInfo.current !== packageInfo.latest && (
              <div className="space-y-2 p-4 bg-muted/30 rounded-lg border">
                <h4 className="text-sm font-medium text-muted-foreground">Update Command</h4>
                <div className="flex items-center gap-2">
                  <code className="flex-1 text-sm bg-muted px-3 py-2 rounded font-mono text-xs overflow-x-auto">
                    {updateCommand}
                  </code>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => copyToClipboard(updateCommand)}
                    className="shrink-0"
                  >
                    <Copy className="w-4 h-4 mr-1" />
                    Copy
                  </Button>
                </div>
              </div>
            )}

            {vulnerabilities.length > 0 && (
              <div className="space-y-3 pt-4 border-t">
                <h4 className="text-sm font-medium text-muted-foreground">Vulnerability Summary</h4>
                <div className="flex flex-wrap gap-2">
                  {criticalCount > 0 && (
                    <Badge variant="destructive" className="gap-1">
                      <Shield className="w-3 h-3" />
                      Critical: {criticalCount}
                    </Badge>
                  )}
                  {highCount > 0 && (
                    <Badge variant="default" className="gap-1">
                      <Shield className="w-3 h-3" />
                      High: {highCount}
                    </Badge>
                  )}
                  {mediumCount > 0 && (
                    <Badge variant="secondary" className="gap-1">
                      <Shield className="w-3 h-3" />
                      Medium: {mediumCount}
                    </Badge>
                  )}
                  {lowCount > 0 && (
                    <Badge variant="outline" className="gap-1">
                      <Shield className="w-3 h-3" />
                      Low: {lowCount}
                    </Badge>
                  )}
                </div>
              </div>
            )}

            {auditItems.length > 0 && (
              <div className="space-y-3 pt-4 border-t">
                <h4 className="text-sm font-medium text-muted-foreground">Audit Items</h4>
                <div className="flex flex-wrap gap-2">
                  {auditItems.map((item) => (
                    <Badge key={item.name} variant="outline">
                      {item.name}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {supplyChainRisks.length > 0 && (
              <div className="space-y-3 pt-4 border-t">
                <h4 className="text-sm font-medium text-muted-foreground">Supply Chain Risks</h4>
                <div className="flex flex-wrap gap-2">
                  {supplyChainRisks.map((risk, i) => (
                    <Badge
                      key={i}
                      variant="outline"
                      className={riskTypeColor(risk.risk_type)}
                    >
                      {risk.risk_type}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {(!packageInfo || packageInfo.current === packageInfo.latest) &&
              vulnerabilities.length === 0 &&
              auditItems.length === 0 &&
              supplyChainRisks.length === 0 && (
                <div className="text-center py-12 text-muted-foreground">
                  <Package className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p className="text-sm">No issues found for this package</p>
                </div>
              )}
          </TabsContent>

          <TabsContent value="vulnerabilities" className="flex-1 overflow-auto p-4">
            {vulnerabilities.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                <Shield className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No vulnerabilities found for this package</p>
              </div>
            ) : (
              <div className="space-y-3">
                {vulnerabilities.map((vuln, index) => (
                  <div
                    key={index}
                    className="border rounded-lg p-4 bg-card hover:bg-muted/30 transition-colors"
                  >
                    <div className="flex items-start gap-3">
                      <div className={cn("flex-shrink-0 p-1 rounded", severityColor(vuln.severity))}>
                        {severityIcon(vuln.severity)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-2">
                          <h5 className="font-medium text-sm">{vuln.title}</h5>
                          <Badge
                            variant="outline"
                            className={cn("text-xs shrink-0", severityColor(vuln.severity))}
                          >
                            {vuln.severity.toUpperCase()}
                          </Badge>
                        </div>
                        {vuln.cve && (
                          <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                            <ExternalLink className="w-3 h-3" />
                            <a
                              href={`https://cve.mitre.org/cgi-bin/cvename.cgi?name=${vuln.cve}`}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="hover:underline font-mono"
                            >
                              {vuln.cve}
                            </a>
                            {vuln.patched_version && (
                              <>
                                <span>→</span>
                                <code className="bg-muted px-1.5 py-0.5 rounded text-xs">
                                  Fixed in {vuln.patched_version}
                                </code>
                              </>
                            )}
                          </div>
                        )}
                        {vuln.dependency_path && (
                          <div className="mt-2 text-xs text-muted-foreground">
                            <code className="bg-muted px-1.5 py-0.5 rounded font-mono break-all">
                              {vuln.dependency_path}
                            </code>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </TabsContent>

          <TabsContent value="audit" className="flex-1 overflow-auto p-4">
            {auditItems.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                <AlertTriangle className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No audit findings for this package</p>
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-1/4">Rule</TableHead>
                    <TableHead className="w-1/4">Current State</TableHead>
                    <TableHead>Recommendation</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {auditItems.map((item, index) => (
                    <TableRow key={index}>
                      <TableCell className="font-mono text-sm font-medium">{item.name}</TableCell>
                      <TableCell className="font-mono text-sm text-muted-foreground">
                        {item.current}
                      </TableCell>
                      <TableCell className="text-sm">{item.note}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </TabsContent>

          <TabsContent value="supply-chain" className="flex-1 overflow-auto p-4">
            {supplyChainRisks.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                <Package className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No supply chain risks identified for this package</p>
              </div>
            ) : (
              <div className="space-y-3">
                {supplyChainRisks.map((risk, index) => (
                  <div
                    key={index}
                    className="border rounded-lg p-4 bg-card hover:bg-muted/30 transition-colors"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-2">
                          <h5 className="font-medium text-sm">{risk.package}</h5>
                          <Badge
                            variant="outline"
                            className={cn("text-xs", riskTypeColor(risk.risk_type))}
                          >
                            {risk.risk_type}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">{risk.description}</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>

        <div className="flex items-center justify-end gap-2 border-t pt-4">
          <Button variant="outline" size="sm" onClick={onClose}>
            Close
          </Button>
          <Button variant="outline" size="sm" onClick={() => {}}>
            <Download className="w-4 h-4 mr-1" />
            Export Report
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}