import { createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Skeleton } from "@/components/ui/skeleton"
import {
  Settings as SettingsIcon,
  Sliders,
  Boxes,
  Database,
  Save,
  Sun,
  Moon,
  Monitor,
  Info,
  ExternalLink,
} from "lucide-react"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useTheme } from "@/components/theme-provider"
import { toast } from "sonner"

export const Route = createFileRoute("/settings")({ component: SettingsPage })

const ALL_SCANNERS = [
  { id: "brew", label: "Brew" },
  { id: "npm", label: "npm" },
  { id: "pnpm", label: "pnpm" },
  { id: "yarn", label: "Yarn" },
  { id: "bun", label: "Bun" },
  { id: "deno", label: "Deno" },
  { id: "pip", label: "pip" },
  { id: "gem", label: "Gem" },
  { id: "cargo", label: "Cargo" },
  { id: "docker", label: "Docker" },
  { id: "project", label: "Project" },
  { id: "security", label: "Security" },
  { id: "audit", label: "Audit" },
  { id: "ci", label: "CI/CD" },
]

interface SettingsState {
  autoScan: boolean
  scanTimeout: string
  daemonInterval: string
  cacheTtl: string
  enabledScanners: string[]
  exportFormat: string
  verboseLogs: boolean
  logRetention: string
}

function FieldRow({
  label,
  description,
  children,
}: {
  label: string
  description: string
  children: React.ReactNode
}) {
  return (
    <div className="flex items-center justify-between gap-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
      <div className="space-y-0.5">
        <Label className="text-base text-foreground/90">{label}</Label>
        <p className="text-sm text-muted-foreground/60">{description}</p>
      </div>
      {children}
    </div>
  )
}

function SettingsPage() {
  const { theme, setTheme } = useTheme()
  const [loading] = useState(false)
  const [settings, setSettings] = useState<SettingsState>({
    autoScan: false,
    scanTimeout: "30",
    daemonInterval: "14400",
    cacheTtl: "30",
    enabledScanners: ALL_SCANNERS.map((s) => s.id),
    exportFormat: "markdown",
    verboseLogs: false,
    logRetention: "7",
  })

  const toggleScanner = (id: string) => {
    setSettings((prev) => ({
      ...prev,
      enabledScanners: prev.enabledScanners.includes(id)
        ? prev.enabledScanners.filter((s) => s !== id)
        : [...prev.enabledScanners, id],
    }))
  }
  const handleSave = () => {
    toast.success("Settings saved", {
      description: "Configuration updated. Backend persistence coming soon.",
    })
  }

  const handleCheckUpdates = () => {
    const id = toast.loading("Checking for updates...")
    setTimeout(() => {
      toast.success("You're up to date", {
        id,
        description: "Envexa v2.11.0 is the latest version.",
      })
    }, 1500)
  }

  if (loading) {
    return (
      <div className="max-w-7xl mx-auto flex flex-col gap-6 animate-in fade-in duration-700">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
          <div>
            <Skeleton className="h-10 w-48 bg-muted" />
            <Skeleton className="h-4 w-72 mt-3 bg-muted" />
          </div>
          <Skeleton className="h-9 w-36 bg-muted" />
        </div>
        <Skeleton className="h-40 w-full rounded-xl bg-muted/50" />
        <Skeleton className="h-48 w-full rounded-xl bg-muted/50" />
        <Skeleton className="h-32 w-full rounded-xl bg-muted/50" />
        <Skeleton className="h-48 w-full rounded-xl bg-muted/50" />
        <Skeleton className="h-24 w-full rounded-xl bg-muted/50" />
      </div>
    )
  }

  return (
    <div className="max-w-7xl mx-auto flex flex-col gap-6">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground flex items-center gap-3">
            <SettingsIcon className="w-8 h-8 text-foreground" />
            Settings
          </h1>
          <p className="text-sm text-muted-foreground mt-2">
            Configure Envexa scanner behavior.
          </p>
        </div>
        <Button
          variant="outline"
          className="gap-2 shadow-xs"
          onClick={handleSave}
        >
          <Save className="w-4 h-4" /> Save Changes
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Sun className="w-5 h-5 text-muted-foreground" />
            <CardTitle>Appearance</CardTitle>
          </div>
          <CardDescription>Customize the look and feel of Envexa.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
            <div className="space-y-0.5">
              <Label className="text-base text-foreground/90">Theme</Label>
              <p className="text-sm text-muted-foreground/60">
                Select your preferred color theme.
              </p>
            </div>
            <Tabs
              value={theme}
              onValueChange={(v) => setTheme((v ?? "system") as "light" | "dark" | "system")}
              className="w-[200px]"
            >
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="light" title="Light Theme">
                  <Sun className="h-4 w-4" />
                </TabsTrigger>
                <TabsTrigger value="dark" title="Dark Theme">
                  <Moon className="h-4 w-4" />
                </TabsTrigger>
                <TabsTrigger value="auto" title="System Theme">
                  <Monitor className="h-4 w-4" />
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        </CardContent>
      </Card>

      {/* Scanner Configuration */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Sliders className="w-5 h-5 text-muted-foreground" />
            <CardTitle>Scanner Configuration</CardTitle>
          </div>
          <CardDescription>
            Control scan behavior, timeouts, and caching.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <FieldRow
            label="Auto-scan on Startup"
            description="Automatically run a full scan when Envexa starts."
          >
            <Switch
              checked={settings.autoScan}
              onCheckedChange={(checked) =>
                setSettings((prev) => ({ ...prev, autoScan: checked }))
              }
            />
          </FieldRow>

          <FieldRow
            label="Scan Timeout"
            description="Maximum time allowed per toolchain scan."
          >
            <Select
              value={settings.scanTimeout}
              onValueChange={(v) =>
                setSettings((prev) => ({ ...prev, scanTimeout: v ?? prev.scanTimeout }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="15">15 seconds</SelectItem>
                <SelectItem value="30">30 seconds</SelectItem>
                <SelectItem value="60">60 seconds</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>

          <FieldRow
            label="Daemon Interval"
            description="How often the background scanner runs."
          >
            <Select
              value={settings.daemonInterval}
              onValueChange={(v) =>
                setSettings((prev) => ({ ...prev, daemonInterval: v ?? prev.daemonInterval }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="3600">Every 1 hour</SelectItem>
                <SelectItem value="14400">Every 4 hours</SelectItem>
                <SelectItem value="28800">Every 8 hours</SelectItem>
                <SelectItem value="86400">Every 24 hours</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>

          <FieldRow
            label="Cache TTL"
            description="How long scan results are cached before re-fetching."
          >
            <Select
              value={settings.cacheTtl}
              onValueChange={(v) =>
                setSettings((prev) => ({ ...prev, cacheTtl: v ?? prev.cacheTtl }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="15">15 minutes</SelectItem>
                <SelectItem value="30">30 minutes</SelectItem>
                <SelectItem value="60">60 minutes</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>
        </CardContent>
      </Card>

      {/* Enabled Toolchains */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Boxes className="w-5 h-5 text-muted-foreground" />
            <CardTitle>Enabled Toolchains</CardTitle>
          </div>
          <CardDescription>
            Select which package managers Envexa should scan.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            {ALL_SCANNERS.map((scanner) => (
              <label
                key={scanner.id}
                className="flex items-center gap-3 rounded-lg border border-border/50 bg-muted/50 p-3 transition-colors hover:bg-muted cursor-pointer"
              >
                <Checkbox
                  checked={settings.enabledScanners.includes(scanner.id)}
                  onCheckedChange={() => toggleScanner(scanner.id)}
                />
                <span className="text-sm text-foreground/90">{scanner.label}</span>
              </label>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Data & Logging */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-muted-foreground" />
            <CardTitle>Data & Logging</CardTitle>
          </div>
          <CardDescription>
            Export format, log verbosity, and retention settings.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <FieldRow
            label="Export Format"
            description="Default format for scan report exports."
          >
            <Select
              value={settings.exportFormat}
              onValueChange={(v) =>
                setSettings((prev) => ({ ...prev, exportFormat: v ?? prev.exportFormat }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="markdown">Markdown</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>

          <FieldRow
            label="Verbose Logs"
            description="Include detailed output in scan logs."
          >
            <Switch
              checked={settings.verboseLogs}
              onCheckedChange={(checked) =>
                setSettings((prev) => ({ ...prev, verboseLogs: checked }))
              }
            />
          </FieldRow>

          <FieldRow
            label="Log Retention"
            description="How long to keep historical scan logs."
          >
            <Select
              value={settings.logRetention}
              onValueChange={(v) =>
                setSettings((prev) => ({ ...prev, logRetention: v ?? prev.logRetention }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1 day</SelectItem>
                <SelectItem value="7">7 days</SelectItem>
                <SelectItem value="30">30 days</SelectItem>
                <SelectItem value="90">90 days</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Info className="w-5 h-5 text-muted-foreground" />
            <CardTitle>About</CardTitle>
          </div>
          <CardDescription>Version information and updates.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between gap-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
            <div className="space-y-0.5">
              <Label className="text-base text-foreground/90">Version</Label>
              <p className="text-sm text-muted-foreground/60">v2.11.0</p>
            </div>
            <Button variant="outline" size="sm" className="gap-2" onClick={handleCheckUpdates}>
              <ExternalLink className="w-3.5 h-3.5" />
              Check for Updates
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
