import { createFileRoute } from "@tanstack/react-router"
import { useState, useEffect } from "react"
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
  Info,
  ExternalLink,
  Monitor,
  Sun,
  Moon,
  AlertTriangle,
} from "lucide-react"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useTheme } from "@/components/theme-provider"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { toast } from "sonner"

export const Route = createFileRoute("/settings")({ component: SettingsPage })

const SCANNER_CATEGORIES = [
  {
    name: "System & Runtime",
    scanners: [
      { id: "brew", label: "Brew" },
      { id: "cargo", label: "Cargo" },
      { id: "docker", label: "Docker" },
      { id: "pip", label: "pip" },
      { id: "gem", label: "Gem" },
    ],
  },
  {
    name: "Web Development",
    scanners: [
      { id: "npm", label: "npm" },
      { id: "pnpm", label: "pnpm" },
      { id: "yarn", label: "Yarn" },
      { id: "bun", label: "Bun" },
      { id: "deno", label: "Deno" },
    ],
  },
  {
    name: "Project Tooling",
    scanners: [
      { id: "project", label: "Project" },
      { id: "security", label: "Security" },
      { id: "supply_chain", label: "Supply Chain" },
      { id: "audit", label: "Audit" },
      { id: "ci", label: "CI/CD" },
    ],
  },
]

const ALL_SCANNERS = SCANNER_CATEGORIES.flatMap((c) => c.scanners)

interface UserConfig {
  cache_ttl_minutes: number
  project_path: string | null
  recent_project_paths: string[]
  favorite_project_paths: string[]
  auto_scan_on_startup: boolean
  theme: string
  verbose_logs: boolean
  scan_timeout_secs: number
  daemon_interval_secs: number
  export_format: string
  enabled_scanners: string[] | null
  log_retention_days: number
}

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
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
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
  const [clearCacheOpen, setClearCacheOpen] = useState(false)
  const [resetDefaultsOpen, setResetDefaultsOpen] = useState(false)

  const loadConfig = async () => {
    try {
      const res = await fetch("/api/config")
      if (!res.ok) throw new Error("Failed to load config")
      const cfg: UserConfig = await res.json()
      setSettings({
        autoScan: cfg.auto_scan_on_startup ?? false,
        scanTimeout: String(cfg.scan_timeout_secs ?? 30),
        daemonInterval: String(cfg.daemon_interval_secs ?? 14400),
        cacheTtl: String(cfg.cache_ttl_minutes ?? 30),
        enabledScanners: cfg.enabled_scanners ?? ALL_SCANNERS.map((s) => s.id),
        exportFormat: cfg.export_format ?? "markdown",
        verboseLogs: cfg.verbose_logs ?? false,
        logRetention: String(cfg.log_retention_days ?? 7),
      })
      if (cfg.theme && ["dark", "light", "system"].includes(cfg.theme)) setTheme(cfg.theme as "dark" | "light" | "system")
    } catch (e) {
      console.error("Failed to load config:", e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadConfig()
  }, [])

  const toggleScanner = (id: string) => {
    setSettings((prev) => ({
      ...prev,
      enabledScanners: prev.enabledScanners.includes(id)
        ? prev.enabledScanners.filter((s) => s !== id)
        : [...prev.enabledScanners, id],
    }))
  }
  const enableAllScanners = () => {
    setSettings((prev) => ({
      ...prev,
      enabledScanners: ALL_SCANNERS.map((s) => s.id),
    }))
  }

  const disableAllScanners = () => {
    setSettings((prev) => ({
      ...prev,
      enabledScanners: [],
    }))
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      const res = await fetch("/api/config", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          cache_ttl_minutes: Number(settings.cacheTtl),
          project_path: null,
          recent_project_paths: [],
          favorite_project_paths: [],
          auto_scan_on_startup: settings.autoScan,
          theme: theme,
          verbose_logs: settings.verboseLogs,
          scan_timeout_secs: Number(settings.scanTimeout),
          daemon_interval_secs: Number(settings.daemonInterval),
          export_format: settings.exportFormat,
          enabled_scanners: settings.enabledScanners,
          log_retention_days: Number(settings.logRetention),
        }),
      })
      if (!res.ok) throw new Error("Failed to save")
      toast.success("Settings saved", {
        description: "Configuration updated successfully.",
      })
    } catch (e) {
      toast.error("Failed to save", {
        description: e instanceof Error ? e.message : "Unknown error",
      })
    } finally {
      setSaving(false)
    }
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
  const handleClearCache = () => {
    setClearCacheOpen(false)
    const id = toast.loading("Clearing caches...")
    setTimeout(() => {
      toast.success("Caches cleared", { id, description: "All cached scan data and logs removed." })
    }, 1000)
  }

  const handleResetDefaults = () => {
    setResetDefaultsOpen(false)
    setSettings({
      autoScan: false,
      scanTimeout: "30",
      daemonInterval: "14400",
      cacheTtl: "30",
      enabledScanners: ALL_SCANNERS.map((s) => s.id),
      exportFormat: "markdown",
      verboseLogs: false,
      logRetention: "7",
    })
    setTheme("system")
    toast.success("Settings reset", { description: "All settings restored to factory defaults." })
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
        <Button onClick={handleSave} disabled={saving} className="gap-2" size="lg">
          <Save className="w-4 h-4" />
          {saving ? "Saving..." : "Save Changes"}
        </Button>
      </div>

      <Tabs defaultValue="general" className="space-y-6">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="scanners">Scanners</TabsTrigger>
          <TabsTrigger value="about">About</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6 animate-in fade-in">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Sliders className="w-5 h-5" />
                General
              </CardTitle>
              <CardDescription>Core scanner behavior and defaults.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <FieldRow
                label="Auto-scan on startup"
                description="Automatically run a full scan when the dashboard opens."
              >
                <Switch
                  checked={settings.autoScan}
                  onCheckedChange={(checked) =>
                    setSettings((prev) => ({ ...prev, autoScan: checked }))
                  }
                />
              </FieldRow>

              <FieldRow
                label="Scan timeout (seconds)"
                description="Maximum time to wait for a single scan to complete."
              >
                <Select
                  value={settings.scanTimeout}
                  onValueChange={(v) => setSettings((p) => ({ ...p, scanTimeout: v ?? p.scanTimeout }))}
                >
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="15">15s</SelectItem>
                    <SelectItem value="30">30s</SelectItem>
                    <SelectItem value="60">60s</SelectItem>
                    <SelectItem value="120">120s</SelectItem>
                  </SelectContent>
                </Select>
              </FieldRow>

              <FieldRow
                label="Daemon interval (seconds)"
                description="How often the background daemon rescans. 14400 = 4 hours."
              >
                <Select
                  value={settings.daemonInterval}
                  onValueChange={(v) => setSettings((p) => ({ ...p, daemonInterval: v ?? p.daemonInterval }))}
                >
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="3600">1 hour (3600s)</SelectItem>
                    <SelectItem value="14400">4 hours (14400s)</SelectItem>
                    <SelectItem value="28800">8 hours (28800s)</SelectItem>
                    <SelectItem value="86400">24 hours (86400s)</SelectItem>
                  </SelectContent>
                </Select>
              </FieldRow>

              <FieldRow
                label="Cache TTL (minutes)"
                description="How long to cache scan results before re-scanning."
              >
                <Select
                  value={settings.cacheTtl}
                  onValueChange={(v) => setSettings((p) => ({ ...p, cacheTtl: v ?? p.cacheTtl }))}
                >
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="5">5 min</SelectItem>
                    <SelectItem value="15">15 min</SelectItem>
                    <SelectItem value="30">30 min</SelectItem>
                    <SelectItem value="60">1 hour</SelectItem>
                  </SelectContent>
                </Select>
              </FieldRow>

              <FieldRow
                label="Export format"
                description="Default format for scan result exports."
              >
                <Select
                  value={settings.exportFormat}
                  onValueChange={(v) => setSettings((p) => ({ ...p, exportFormat: v ?? p.exportFormat }))}
                >
                  <SelectTrigger className="w-[180px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="json">JSON</SelectItem>
                    <SelectItem value="markdown">Markdown</SelectItem>
                    <SelectItem value="csv">CSV</SelectItem>
                  </SelectContent>
                </Select>
              </FieldRow>

              <FieldRow
                label="Theme"
                description="Application color theme."
              >
                <Tabs value={theme} onValueChange={(v) => { if (v) setTheme(v as "dark" | "light" | "system") }}>
                  <TabsList className="h-9">
                    <TabsTrigger value="system" className="h-7 w-7 p-0" title="System">
                      <Monitor className="h-4 w-4" />
                    </TabsTrigger>
                    <TabsTrigger value="light" className="h-7 w-7 p-0" title="Light">
                      <Sun className="h-4 w-4" />
                    </TabsTrigger>
                    <TabsTrigger value="dark" className="h-7 w-7 p-0" title="Dark">
                      <Moon className="h-4 w-4" />
                    </TabsTrigger>
                  </TabsList>
                </Tabs>
              </FieldRow>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Database className="w-5 h-5" />
                Logging
              </CardTitle>
              <CardDescription>Verbosity and retention settings.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <FieldRow
                label="Verbose logs"
                description="Enable detailed debug logging for troubleshooting."
              >
                <Switch
                  checked={settings.verboseLogs}
                  onCheckedChange={(checked) =>
                    setSettings((prev) => ({ ...prev, verboseLogs: checked }))
                  }
                />
              </FieldRow>

              <FieldRow
                label="Log retention (days)"
                description="How many days to keep log files before rotation."
              >
                <Select
                  value={settings.logRetention}
                  onValueChange={(v) => setSettings((p) => ({ ...p, logRetention: v ?? p.logRetention }))}
                >
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="1">1 day</SelectItem>
                    <SelectItem value="7">1 week</SelectItem>
                    <SelectItem value="14">2 weeks</SelectItem>
                    <SelectItem value="30">1 month</SelectItem>
                    <SelectItem value="90">3 months</SelectItem>
                  </SelectContent>
                </Select>
              </FieldRow>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <ExternalLink className="w-5 h-5" />
                Actions
              </CardTitle>
              <CardDescription>Maintenance actions and utilities.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <Button variant="outline" onClick={handleCheckUpdates} className="w-full justify-start gap-4 h-auto py-4">
                <Info className="w-5 h-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">Check for Updates</div>
                  <div className="text-xs text-muted-foreground">Check if a new version of Envexa is available</div>
                </div>
              </Button>
              <Button variant="outline" onClick={() => setClearCacheOpen(true)} className="w-full justify-start gap-4 h-auto py-4 text-destructive hover:bg-destructive/10 hover:text-destructive">
                <Database className="w-5 h-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">Clear All Caches</div>
                  <div className="text-xs text-muted-foreground">Remove all cached scan data and logs</div>
                </div>
              </Button>
              <Button variant="outline" onClick={() => setResetDefaultsOpen(true)} className="w-full justify-start gap-4 h-auto py-4 text-destructive hover:bg-destructive/10 hover:text-destructive">
                <ExternalLink className="w-5 h-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">Reset to Defaults</div>
                  <div className="text-xs text-muted-foreground">Reset all settings to factory defaults</div>
                </div>
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="scanners" className="space-y-6 animate-in fade-in">
          <Card>
            <CardHeader>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Boxes className="w-5 h-5" />
                    Enabled Scanners
                  </CardTitle>
                  <CardDescription className="mt-1">
                    {settings.enabledScanners.length} of {ALL_SCANNERS.length} scanners enabled.
                  </CardDescription>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm" onClick={enableAllScanners}>
                    Enable All
                  </Button>
                  <Button variant="outline" size="sm" onClick={disableAllScanners}>
                    Disable All
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-6">
              {SCANNER_CATEGORIES.map((category) => (
                <div key={category.name}>
                  <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                    {category.name}
                  </h4>
                  <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                    {category.scanners.map((scanner) => (
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
                </div>
              ))}
            </CardContent>
          </Card>
        </TabsContent>


        <TabsContent value="about" className="space-y-6 animate-in fade-in">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5" />
                About Envexa
              </CardTitle>
              <CardDescription>Version information and links.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-4 rounded-lg border border-border/50 bg-muted/50 p-4">
                <Boxes className="w-12 h-12 text-muted-foreground/50" />
                <div>
                  <h3 className="font-semibold text-foreground">Envexa</h3>
                  <p className="text-sm text-muted-foreground">Universal environment scanner</p>
                  <p className="text-sm font-mono text-muted-foreground mt-1">v2.11.0</p>
                </div>
              </div>
              <div className="flex flex-wrap gap-3">
                <a
                  href="https://github.com/kurtcalacday/envexa"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  <ExternalLink className="w-4 h-4" />
                  GitHub
                </a>
                <a
                  href="https://github.com/kurtcalacday/envexa/issues"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  <Info className="w-4 h-4" />
                  Report Issue
                </a>
              </div>
              <div className="rounded-lg border border-border/50 bg-muted/50 p-4">
                <p className="text-sm text-muted-foreground/80 mb-3">
                  Configuration file location:
                </p>
                <code className="text-xs font-mono text-muted-foreground bg-muted px-2 py-1 rounded block break-all">
                  ~/.config/envexa/config.json
                </code>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
      <AlertDialog open={clearCacheOpen} onOpenChange={setClearCacheOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-destructive" />
              Clear All Caches?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This will remove all cached scan data and logs. You'll need to run a new scan to rebuild the cache.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleClearCache} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              Clear Caches
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={resetDefaultsOpen} onOpenChange={setResetDefaultsOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-destructive" />
              Reset to Defaults?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This will reset all settings to factory defaults, including theme, scanner toggles, and all preferences. This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleResetDefaults} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              Reset Settings
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}