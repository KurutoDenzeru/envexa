import type { ReactNode } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

/** Summary metric card used across all dashboard pages. */
export function StatCard({
  title,
  icon,
  value,
  valueClassName,
  subtext,
}: {
  title: string
  icon?: ReactNode
  value: ReactNode
  valueClassName?: string
  subtext: string
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">
          {title}
        </CardTitle>
        {icon}
      </CardHeader>
      <CardContent>
        <div
          className={`text-3xl font-bold ${valueClassName ?? "text-foreground"}`}
        >
          {value}
        </div>
        <p className="mt-1 text-xs text-muted-foreground/60">{subtext}</p>
      </CardContent>
    </Card>
  )
}
