import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

export interface DonutSlice {
  name: string
  value: number
  fill: string
}

export const CHART_PALETTE = [
  "#6366f1",
  "#8b5cf6",
  "#ec4899",
  "#ef4444",
  "#f97316",
  "#eab308",
  "#22c55e",
  "#06b6d4",
  "#3b82f6",
  "#71717a",
]

export function DonutChart({ data }: { data: DonutSlice[] }) {
  return (
    <div className="h-[260px] w-full">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={100}
            paddingAngle={3}
            dataKey="value"
            stroke="hsl(var(--background))"
            strokeWidth={2}
          >
            {data.map((entry, index) => (
              <Cell
                key={index}
                fill={entry.fill}
                className="cursor-pointer transition-opacity outline-none hover:opacity-80"
              />
            ))}
          </Pie>
          <Tooltip
            content={({ active, payload }) => {
              if (!active || !payload?.length) return null
              const data = payload[0].payload
              return (
                <div className="rounded-lg border border-border bg-card px-3 py-2 text-xs text-foreground shadow-xs">
                  <div className="flex items-center gap-2">
                    <span
                      className="inline-block h-2 w-2 shrink-0 rounded-full"
                      style={{ backgroundColor: data.fill }}
                    />
                    <span className="font-medium">{data.name}</span>
                    <span className="ml-1 text-muted-foreground">
                      : {data.value}
                    </span>
                  </div>
                </div>
              )
            }}
            cursor={false}
          />
          <Legend
            verticalAlign="bottom"
            height={36}
            formatter={(value: string) => (
              <span className="text-xs text-muted-foreground">{value}</span>
            )}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  )
}

/** Chart card wrapper used by every dashboard page. */
export function DonutCard({
  title,
  description,
  data,
}: {
  title: string
  description: string
  data: DonutSlice[]
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <DonutChart data={data} />
      </CardContent>
    </Card>
  )
}

/** Top-N slice builder for per-key counts (toolchain/package/source donuts). */
export function keyCountPie(
  counts: Record<string, number>,
  options: {
    offset?: number
    limit?: number
    label?: (key: string) => string
  } = {}
): DonutSlice[] {
  const { offset = 0, limit, label = (key: string) => key } = options
  return Object.entries(counts)
    .filter(([, value]) => value > 0)
    .map(([name, value], i) => ({
      name: label(name),
      value,
      fill: CHART_PALETTE[(i + offset) % CHART_PALETTE.length],
    }))
    .sort((a, b) => b.value - a.value)
    .slice(0, limit)
}
