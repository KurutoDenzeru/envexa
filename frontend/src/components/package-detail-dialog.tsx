"use client"

import { useState, useMemo, useEffect } from "react"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
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
import { Package, Copy, ExternalLink, Shield, CheckCircle } from "lucide-react"
import { useScanData } from "@/components/scan-data-context"
import { enrichOutdated } from "@/lib/outdated"
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
      return <Shield className="h-3 w-3 text-red-500" />
    case "high":
      return <Shield className="h-3 w-3 text-orange-500" />
    case "medium":
    case "moderate":
      return <Shield className="h-3 w-3 text-amber-500" />
    case "low":
      return <Shield className="h-3 w-3 text-emerald-500" />
    default:
      return <Shield className="h-3 w-3 text-muted-foreground" />
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

function getUpdateCommand(
  toolchain: string,
  pkg: string,
  version: string
): string {
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
  return (
    commands[toolchain.toLowerCase()] ||
    `${toolchain} install ${pkg}@${version}`
  )
}

type DetailTab = "security" | "updates" | "audit" | "supply-chain"

export function PackageDetailDialog({
  packageName,
  toolchain,
  open,
  onClose,
}: PackageDetailDialogProps) {
  const { report } = useScanData()

  const scanResult = useMemo((): ScanResult | undefined => {
    if (!report?.results) return undefined
    return report.results[toolchain] as ScanResult | undefined
  }, [report, toolchain])

  const packageInfo = useMemo((): PackageInfo | undefined => {
    if (!report?.results) return undefined
    return enrichOutdated(report.results).find(
      (o) => o.toolchain === toolchain && o.name === packageName
    )
  }, [report, toolchain, packageName])

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
    return scanResult.supply_chain_risks.filter(
      (r) => r.package === packageName
    )
  }, [scanResult, packageName])

  const vulnCount = vulnerabilities.length
  const auditCount = auditItems.length
  const riskCount = supplyChainRisks.length

  const [activeTab, setActiveTab] = useState<DetailTab>(
    vulnCount > 0 ? "security" : packageInfo ? "updates" : "audit"
  )

  // Pick the most relevant tab each time the dialog opens (data may not have
  // been loaded when the component first mounted).
  useEffect(() => {
    if (open) {
      setActiveTab(
        vulnCount > 0 ? "security" : packageInfo ? "updates" : "audit"
      )
    }
  }, [open, vulnCount, packageInfo])

  const updateCommand = packageInfo
    ? getUpdateCommand(toolchain, packageName, packageInfo.latest)
    : ""

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (!open) return null

  const isUpToDate = packageInfo && packageInfo.current === packageInfo.latest

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="flex max-h-[90vh] flex-col border border-border bg-card p-6 shadow-xl sm:max-w-2xl">
        <DialogHeader className="border-b border-border/50 pb-4">
          <div className="flex items-center gap-3">
            <div className="rounded-lg border border-border/60 bg-muted/60 p-2">
              <Package className="h-5 w-5" />
            </div>
            <div>
              <DialogTitle className="font-mono text-2xl font-bold text-foreground">
                {packageName}
              </DialogTitle>
              <DialogDescription className="mt-0.5 text-muted-foreground">
                Security, audit, and update status for this package.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="flex flex-1 flex-col gap-6 overflow-y-auto py-4">
          {/* Top Stats */}
          <div className="grid grid-cols-4 gap-4">
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Current
              </span>
              <span className="font-mono text-xl font-bold text-foreground">
                {packageInfo?.current ?? "-"}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Latest
              </span>
              <span
                className={`font-mono text-xl font-bold ${
                  !packageInfo || isUpToDate
                    ? "text-muted-foreground"
                    : "text-blue-400"
                }`}
              >
                {packageInfo?.latest ?? "-"}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Vulnerabilities
              </span>
              <span
                className={`font-mono text-xl font-bold ${
                  vulnCount > 0 ? "text-red-400" : "text-muted-foreground"
                }`}
              >
                {vulnCount}
              </span>
            </div>
            <div className="rounded-xl border border-border/40 bg-muted/30 p-3.5 text-center">
              <span className="mb-1 block text-xs text-muted-foreground">
                Status
              </span>
              <div className="flex justify-center">
                {!packageInfo ? (
                  <span className="font-mono text-xl font-bold text-muted-foreground">
                    -
                  </span>
                ) : isUpToDate ? (
                  <Badge
                    variant="outline"
                    className="border-green-500/30 bg-green-500/10 text-green-500 shadow-none"
                  >
                    Up to date
                  </Badge>
                ) : (
                  <Badge
                    variant="outline"
                    className="border-blue-500/30 bg-blue-500/10 text-blue-400 shadow-none"
                  >
                    Update available
                  </Badge>
                )}
              </div>
            </div>
          </div>

          {/* Segmented Tab Buttons */}
          <div className="flex gap-1 rounded-lg border border-border/40 bg-muted/30 p-1">
            {(
              [
                { key: "security", label: `Security (${vulnCount})` },
                { key: "updates", label: "Updates" },
                { key: "audit", label: `Audit (${auditCount})` },
                { key: "supply-chain", label: `Supply Chain (${riskCount})` },
              ] as const
            ).map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`flex-1 cursor-pointer rounded-md px-3 py-2 text-xs font-medium transition-all ${
                  activeTab === tab.key
                    ? "border border-border/50 bg-background text-foreground shadow-xs"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tab Content */}
          {activeTab === "security" && (
            <div>
              {vulnCount === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    No Known Vulnerabilities
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This package has no known active security alerts.
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {vulnerabilities.map((vuln, index) => (
                    <div
                      key={index}
                      className="rounded-lg border bg-card p-4 transition-colors hover:bg-muted/30"
                    >
                      <div className="flex items-start gap-3">
                        <div
                          className={cn(
                            "flex-shrink-0 rounded p-1",
                            severityColor(vuln.severity)
                          )}
                        >
                          {severityIcon(vuln.severity)}
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-start justify-between gap-2">
                            <h5 className="text-sm font-medium">
                              {vuln.title}
                            </h5>
                            <Badge
                              variant="outline"
                              className={cn(
                                "shrink-0 text-xs",
                                severityColor(vuln.severity)
                              )}
                            >
                              {vuln.severity.toUpperCase()}
                            </Badge>
                          </div>
                          {vuln.cve && (
                            <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                              <ExternalLink className="h-3 w-3" />
                              <a
                                href={`https://cve.mitre.org/cgi-bin/cvename.cgi?name=${vuln.cve}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="font-mono hover:underline"
                              >
                                {vuln.cve}
                              </a>
                              {vuln.patched_version && (
                                <>
                                  <span>→</span>
                                  <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                                    Fixed in {vuln.patched_version}
                                  </code>
                                </>
                              )}
                            </div>
                          )}
                          {vuln.dependency_path && (
                            <div className="mt-2 text-xs text-muted-foreground">
                              <code className="rounded bg-muted px-1.5 py-0.5 font-mono break-all">
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
            </div>
          )}

          {activeTab === "updates" && (
            <div>
              {!packageInfo ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    No Update Information
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This package has no outdated-package entry for {toolchain}.
                  </p>
                </div>
              ) : isUpToDate ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    Package is Up to Date
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This package uses the latest available release.
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="flex items-center justify-between gap-2 rounded-xl border border-border/40 bg-muted/10 p-4">
                    <div>
                      <span className="text-xs font-semibold tracking-wider text-muted-foreground/60 uppercase">
                        Update Available
                      </span>
                      <p className="mt-1 font-mono text-sm text-foreground">
                        {packageInfo.current} → {packageInfo.latest}
                      </p>
                    </div>
                    <Badge
                      variant="outline"
                      className="border-blue-500/30 bg-blue-500/10 text-blue-400 shadow-none"
                    >
                      {packageInfo.latest}
                    </Badge>
                  </div>
                  <div className="space-y-2 rounded-lg border bg-muted/30 p-4">
                    <h4 className="text-sm font-medium text-muted-foreground">
                      Update Command
                    </h4>
                    <div className="flex items-center gap-2">
                      <code className="flex-1 overflow-x-auto rounded bg-muted px-3 py-2 font-mono text-xs">
                        {updateCommand}
                      </code>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => copyToClipboard(updateCommand)}
                        className="shrink-0"
                      >
                        <Copy className="mr-1 h-4 w-4" />
                        Copy
                      </Button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === "audit" && (
            <div>
              {auditCount === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    No Audit Findings
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    This package passed all audit checks.
                  </p>
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
                        <TableCell className="font-mono text-sm font-medium">
                          {item.name}
                        </TableCell>
                        <TableCell className="font-mono text-sm text-muted-foreground">
                          {item.current}
                        </TableCell>
                        <TableCell className="text-sm">{item.note}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </div>
          )}

          {activeTab === "supply-chain" && (
            <div>
              {riskCount === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center">
                  <CheckCircle className="mb-3 h-10 w-10 text-green-500/60" />
                  <h4 className="text-sm font-semibold text-foreground">
                    No Supply Chain Risks
                  </h4>
                  <p className="mt-1 max-w-sm text-xs text-muted-foreground">
                    No supply chain risks identified for this package.
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {supplyChainRisks.map((risk, index) => (
                    <div
                      key={index}
                      className="rounded-lg border bg-card p-4 transition-colors hover:bg-muted/30"
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex-1">
                          <div className="mb-2 flex items-center gap-2">
                            <h5 className="text-sm font-medium">
                              {risk.package}
                            </h5>
                            <Badge
                              variant="outline"
                              className={cn(
                                "text-xs",
                                riskTypeColor(risk.risk_type)
                              )}
                            >
                              {risk.risk_type}
                            </Badge>
                          </div>
                          <p className="text-sm text-muted-foreground">
                            {risk.description}
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
