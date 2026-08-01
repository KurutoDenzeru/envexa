import { Fragment, useMemo } from "react"
import { useScanData } from "@/components/scan-data-context"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Boxes, LayoutGrid, Table as TableIcon } from "lucide-react"
import { ToolchainCard } from "@/components/toolchain-card"
import {
  CATEGORIES,
  displayName,
  getPrimaryVersion,
  statusBadge,
} from "@/lib/toolchains"
import type { ToolchainResult } from "@/lib/toolchains"

/** Grouped toolchain data (full objects) for the current report. */
export function useToolchains() {
  const { report } = useScanData()
  const groups = useMemo(() => {
    if (!report?.results) return []
    return CATEGORIES.map((cat) => ({
      category: cat.name,
      tools: cat.tools
        .map((tool) => {
          const data = report.results[tool]
          return data ? ({ ...data, tool } as ToolchainResult) : null
        })
        .filter((t): t is ToolchainResult => t !== null),
    }))
  }, [report])

  const flat = useMemo(() => groups.flatMap((g) => g.tools), [groups])
  const totalTools = flat.length
  return { groups, flat, totalTools }
}

/** Shared Toolchain Status section: grid/list toggle, cards and compact table. */
export function ToolchainStatusView({
  compactView,
  onCompactViewChange,
  onSelectTool,
}: {
  compactView: boolean
  onCompactViewChange: (value: boolean) => void
  onSelectTool: (tool: string) => void
}) {
  const { groups } = useToolchains()

  const tableData = useMemo(
    () =>
      groups.map((cat) => ({
        category: cat.category,
        tools: cat.tools.map((tc) => ({
          tool: tc.tool,
          status: tc.status,
          version: getPrimaryVersion(tc),
          vulns: tc.vulnerabilities?.length || 0,
          outdated: tc.outdated?.length || 0,
          issues: tc.issues?.length || 0,
        })),
      })),
    [groups]
  )

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Boxes className="h-4 w-4 shrink-0 text-muted-foreground" />
              Toolchain Status
            </CardTitle>
            <CardDescription className="mt-1">
              Per-tool status, versions, and issue counts.
            </CardDescription>
          </div>
          <Tabs
            value={compactView ? "table" : "cards"}
            onValueChange={(v) => onCompactViewChange(v === "table")}
          >
            <TabsList className="h-9">
              <TabsTrigger
                value="table"
                className="h-7 w-7 p-0"
                title="Table view"
              >
                <TableIcon className="h-4 w-4" />
              </TabsTrigger>
              <TabsTrigger
                value="cards"
                className="h-7 w-7 p-0"
                title="Cards view"
              >
                <LayoutGrid className="h-4 w-4" />
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </CardHeader>
      <CardContent>
        {compactView ? (
          <div className="overflow-hidden rounded-md border border-border">
            <Table>
              <TableHeader className="bg-muted/50">
                <TableRow className="border-border hover:bg-transparent">
                  <TableHead className="w-[150px]">Tool</TableHead>
                  <TableHead className="w-[80px]">Status</TableHead>
                  <TableHead className="w-[120px]">Version</TableHead>
                  <TableHead className="w-[80px] text-center">Vulns</TableHead>
                  <TableHead className="w-[80px] text-center">
                    Outdated
                  </TableHead>
                  <TableHead className="w-[80px] text-center">Issues</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {tableData.map((cat) => (
                  <Fragment key={cat.category}>
                    <TableRow className="border-border bg-muted/30 hover:bg-muted/30">
                      <TableCell
                        colSpan={6}
                        className="text-xs font-semibold tracking-wider text-muted-foreground uppercase"
                      >
                        {cat.category}
                      </TableCell>
                    </TableRow>
                    {cat.tools.map((t) => (
                      <TableRow
                        key={t.tool}
                        className="cursor-pointer border-border hover:bg-muted/50"
                        onClick={() => onSelectTool(t.tool)}
                      >
                        <TableCell className="text-sm font-medium capitalize">
                          {displayName(t.tool)}
                        </TableCell>
                        <TableCell>{statusBadge(t.status)}</TableCell>
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {t.version}
                        </TableCell>
                        <TableCell className="text-center">
                          {t.vulns > 0 ? (
                            <span className="text-sm font-semibold text-red-500">
                              {t.vulns}
                            </span>
                          ) : (
                            <span className="text-sm text-muted-foreground">
                              0
                            </span>
                          )}
                        </TableCell>
                        <TableCell className="text-center">
                          {t.outdated > 0 ? (
                            <span className="text-sm font-semibold text-blue-500">
                              {t.outdated}
                            </span>
                          ) : (
                            <span className="text-sm text-muted-foreground">
                              0
                            </span>
                          )}
                        </TableCell>
                        <TableCell className="text-center">
                          {t.issues > 0 ? (
                            <span className="text-sm font-semibold text-yellow-500">
                              {t.issues}
                            </span>
                          ) : (
                            <span className="text-sm text-muted-foreground">
                              0
                            </span>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </Fragment>
                ))}
              </TableBody>
            </Table>
          </div>
        ) : (
          <div className="space-y-4">
            {groups.map((cat) => (
              <div key={cat.category}>
                <h4 className="mb-3 text-xs font-semibold tracking-wider text-muted-foreground uppercase">
                  {cat.category}
                </h4>
                <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {cat.tools.map((tc) => (
                    <ToolchainCard
                      key={tc.tool}
                      tc={tc}
                      onClick={() => onSelectTool(tc.tool)}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
