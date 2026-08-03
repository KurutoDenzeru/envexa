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

import { CATEGORIES, displayName } from "@/lib/toolchains"
import { siGithub, siInstagram } from "simple-icons"

export const Route = createFileRoute("/settings")({ component: SettingsPage })

// Scanner toggles mirror the toolchain dashboard catalog (ids + display names)
const SCANNER_CATEGORIES = CATEGORIES.map((c) => ({
  name: c.name,
  scanners: c.tools.map((id) => ({ id, label: displayName(id) })),
}))

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
  const [appVersion, setAppVersion] = useState("")

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
      if (cfg.theme && ["dark", "light", "system"].includes(cfg.theme))
        setTheme(cfg.theme as "dark" | "light" | "system")
    } catch (e) {
      console.error("Failed to load config:", e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadConfig()
    fetch("/api/version")
      .then((r) => r.json())
      .then((d) => setAppVersion(d.version))
      .catch(() => {})
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

  const [updateInfo, setUpdateInfo] = useState<{
    checking: boolean
    currentVersion: string
    latestVersion: string
    updateAvailable: boolean
    releaseBody: string
  }>({
    checking: false,
    currentVersion: "",
    latestVersion: "",
    updateAvailable: false,
    releaseBody: "",
  })

  const handleCheckUpdates = async () => {
    setUpdateInfo((prev) => ({ ...prev, checking: true }))
    const id = toast.loading("Checking for updates...")
    try {
      const res = await fetch("/api/update/check")
      const data = await res.json()
      if (data.update_available) {
        toast.success("Update available!", {
          id,
          description: `Envexa v${data.latest_version} is available (you're on v${data.current_version}).`,
          duration: 8000,
        })
      } else {
        toast.success("You're up to date", {
          id,
          description: `Envexa v${data.current_version} is the latest version.`,
        })
      }
      setUpdateInfo({
        checking: false,
        currentVersion: data.current_version,
        latestVersion: data.latest_version,
        updateAvailable: data.update_available,
        releaseBody: data.release_body,
      })
    } catch {
      toast.error("Failed to check for updates", {
        id,
        description: "Could not reach GitHub release API.",
      })
      setUpdateInfo((prev) => ({ ...prev, checking: false }))
    }
  }
  const handleClearCache = () => {
    setClearCacheOpen(false)
    const id = toast.loading("Clearing caches...")
    setTimeout(() => {
      toast.success("Caches cleared", {
        id,
        description: "All cached scan data and logs removed.",
      })
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
    toast.success("Settings reset", {
      description: "All settings restored to factory defaults.",
    })
  }

  if (loading) {
    return (
      <div className="mx-auto flex max-w-7xl animate-in flex-col gap-6 duration-700 fade-in">
        <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
          <div>
            <Skeleton className="h-10 w-48 bg-muted" />
            <Skeleton className="mt-3 h-4 w-72 bg-muted" />
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
    <div className="mx-auto flex max-w-7xl flex-col gap-6">
      <div className="flex flex-col justify-between gap-4 border-b border-border pb-6 md:flex-row md:items-end">
        <div>
          <h1 className="flex items-center gap-3 text-3xl font-bold tracking-tight text-foreground">
            <SettingsIcon className="h-8 w-8 text-foreground" />
            Settings
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Configure Envexa scanner behavior.
          </p>
        </div>
        <Button
          onClick={handleSave}
          disabled={saving}
          className="gap-2"
          size="lg"
        >
          <Save className="h-4 w-4" />
          {saving ? "Saving..." : "Save Changes"}
        </Button>
      </div>

      <Tabs defaultValue="general" className="space-y-6">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="scanners">Scanners</TabsTrigger>
          <TabsTrigger value="about">About</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="animate-in space-y-6 fade-in">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Sliders className="h-5 w-5" />
                General
              </CardTitle>
              <CardDescription>
                Core scanner behavior and defaults.
              </CardDescription>
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
                  onValueChange={(v) =>
                    setSettings((p) => ({
                      ...p,
                      scanTimeout: v ?? p.scanTimeout,
                    }))
                  }
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
                  onValueChange={(v) =>
                    setSettings((p) => ({
                      ...p,
                      daemonInterval: v ?? p.daemonInterval,
                    }))
                  }
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
                  onValueChange={(v) =>
                    setSettings((p) => ({ ...p, cacheTtl: v ?? p.cacheTtl }))
                  }
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
                  onValueChange={(v) =>
                    setSettings((p) => ({
                      ...p,
                      exportFormat: v ?? p.exportFormat,
                    }))
                  }
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

              <FieldRow label="Theme" description="Application color theme.">
                <Tabs
                  value={theme}
                  onValueChange={(v) => {
                    if (v) setTheme(v as "dark" | "light" | "system")
                  }}
                >
                  <TabsList className="h-9">
                    <TabsTrigger
                      value="system"
                      className="h-7 w-7 p-0"
                      title="System"
                    >
                      <Monitor className="h-4 w-4" />
                    </TabsTrigger>
                    <TabsTrigger
                      value="light"
                      className="h-7 w-7 p-0"
                      title="Light"
                    >
                      <Sun className="h-4 w-4" />
                    </TabsTrigger>
                    <TabsTrigger
                      value="dark"
                      className="h-7 w-7 p-0"
                      title="Dark"
                    >
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
                <Database className="h-5 w-5" />
                Logging
              </CardTitle>
              <CardDescription>
                Verbosity and retention settings.
              </CardDescription>
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
                  onValueChange={(v) =>
                    setSettings((p) => ({
                      ...p,
                      logRetention: v ?? p.logRetention,
                    }))
                  }
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
        </TabsContent>

        <TabsContent value="scanners" className="animate-in space-y-6 fade-in">
          <Card>
            <CardHeader>
              <div className="flex items-start justify-between gap-4">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Boxes className="h-5 w-5" />
                    Enabled Scanners
                  </CardTitle>
                  <CardDescription className="mt-1">
                    {settings.enabledScanners.length} of {ALL_SCANNERS.length}{" "}
                    scanners enabled.
                  </CardDescription>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={enableAllScanners}
                  >
                    Enable All
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={disableAllScanners}
                  >
                    Disable All
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-6">
              {SCANNER_CATEGORIES.map((category) => (
                <div key={category.name}>
                  <h4 className="mb-3 text-xs font-semibold tracking-wider text-muted-foreground uppercase">
                    {category.name}
                  </h4>
                  <div className="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
                    {category.scanners.map((scanner) => (
                      <label
                        key={scanner.id}
                        className="flex cursor-pointer items-center gap-3 rounded-lg border border-border/50 bg-muted/50 p-3 transition-colors hover:bg-muted"
                      >
                        <Checkbox
                          checked={settings.enabledScanners.includes(
                            scanner.id
                          )}
                          onCheckedChange={() => toggleScanner(scanner.id)}
                        />
                        <span className="text-sm text-foreground/90">
                          {scanner.label}
                        </span>
                      </label>
                    ))}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="about" className="animate-in space-y-6 fade-in">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="h-5 w-5" />
                About Envexa
              </CardTitle>
              <CardDescription>Version information and links.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-4 rounded-lg border border-border/50 bg-muted/50 p-4">
                <Boxes className="h-12 w-12 text-muted-foreground/50" />
                <div>
                  <h3 className="font-semibold text-foreground">Envexa</h3>
                  <p className="text-sm text-muted-foreground">
                    Blazing-fast Rust TUI, scriptable CLI, and Web Dashboard for
                    monitoring local developer tooling health.
                  </p>
                  <p className="mt-1 font-mono text-sm text-muted-foreground">
                    v{appVersion || "?.?.?"}
                  </p>
                </div>
              </div>
              <div className="space-y-4 rounded-lg border border-border/50 bg-muted/50 p-4">
                <p className="mb-3 text-sm text-muted-foreground/80">
                  Configuration file location:
                </p>
                <code className="block rounded bg-muted px-2 py-1 font-mono text-xs break-all text-muted-foreground">
                  ~/.config/envexa/config.json
                </code>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <ExternalLink className="h-5 w-5" />
                Actions
              </CardTitle>
              <CardDescription>
                Maintenance actions and utilities.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {updateInfo.updateAvailable ? (
                <div className="space-y-3 rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4">
                  <div className="flex items-center gap-2">
                    <div className="h-2 w-2 animate-pulse rounded-full bg-emerald-500" />
                    <span className="text-sm font-medium text-emerald-500">
                      Update available: Envexa v{updateInfo.latestVersion}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    You&apos;re currently on v{updateInfo.currentVersion}.{" "}
                    <a
                      href="https://github.com/KurutoDenzeru/envexa/releases/latest"
                      target="_blank"
                      rel="noreferrer"
                      className="underline hover:text-foreground"
                    >
                      Download the latest release
                    </a>{" "}
                    and restart the server to update.
                  </p>
                  {updateInfo.releaseBody && (
                    <details className="text-xs text-muted-foreground">
                      <summary className="cursor-pointer hover:text-foreground">
                        Release notes
                      </summary>
                      <pre className="mt-2 max-h-40 overflow-y-auto rounded bg-black/10 p-2 font-mono text-[11px] leading-relaxed whitespace-pre-wrap dark:bg-white/5">
                        {updateInfo.releaseBody}
                      </pre>
                    </details>
                  )}
                </div>
              ) : null}
              <Button
                variant="outline"
                onClick={handleCheckUpdates}
                disabled={updateInfo.checking}
                className="h-auto w-full justify-start gap-4 py-4"
              >
                <Info className="h-5 w-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">
                    {updateInfo.checking ? "Checking..." : "Check for Updates"}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {updateInfo.latestVersion && !updateInfo.checking
                      ? `Latest: v${updateInfo.latestVersion} — Current: v${updateInfo.currentVersion}`
                      : "Check if a new version of Envexa is available"}
                  </div>
                </div>
              </Button>
              <Button
                variant="outline"
                onClick={() => setClearCacheOpen(true)}
                className="h-auto w-full justify-start gap-4 py-4 text-destructive hover:bg-destructive/10 hover:text-destructive"
              >
                <Database className="h-5 w-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">Clear All Caches</div>
                  <div className="text-xs text-muted-foreground">
                    Remove all cached scan data and logs
                  </div>
                </div>
              </Button>
              <Button
                variant="outline"
                onClick={() => setResetDefaultsOpen(true)}
                className="h-auto w-full justify-start gap-4 py-4 text-destructive hover:bg-destructive/10 hover:text-destructive"
              >
                <ExternalLink className="h-5 w-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium">Reset to Defaults</div>
                  <div className="text-xs text-muted-foreground">
                    Reset all settings to factory defaults
                  </div>
                </div>
              </Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
      <div className="space-y-4 border-t border-border/50 py-6 text-center">
        <div className="flex items-center justify-center gap-4">
          <a
            href="https://github.com/KurutoDenzeru"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground/60 transition-colors hover:text-foreground"
            title="GitHub"
          >
            <svg
              role="img"
              viewBox="0 0 24 24"
              className="h-5 w-5"
              fill="currentColor"
            >
              <path d={siGithub.path} />
            </svg>
          </a>
          <a
            href="https://linkedin.com/in/kurtcalacday"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground/60 transition-colors hover:text-foreground"
            title="LinkedIn"
          >
            <svg
              role="img"
              viewBox="0 0 24 24"
              className="h-5 w-5"
              fill="currentColor"
            >
              <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 01-2.063-2.065 2.064 2.064 0 112.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" />
            </svg>
          </a>
          <a
            href="https://instagram.com/krtclcdy"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground/60 transition-colors hover:text-foreground"
            title="Instagram"
          >
            <svg
              role="img"
              viewBox="0 0 24 24"
              className="h-5 w-5"
              fill="currentColor"
            >
              <path d={siInstagram.path} />
            </svg>
          </a>
        </div>
        <p className="text-sm text-muted-foreground/60">
          © 2026 All rights reserved by{" "}
          <span className="text-muted-foreground/80">KurutoDenzeru</span>
        </p>
      </div>
      <AlertDialog open={clearCacheOpen} onOpenChange={setClearCacheOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              Clear All Caches?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This will remove all cached scan data and logs. You'll need to run
              a new scan to rebuild the cache.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleClearCache}
              className="text-destructive-foreground bg-destructive hover:bg-destructive/90"
            >
              Clear Caches
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={resetDefaultsOpen} onOpenChange={setResetDefaultsOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              Reset to Defaults?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This will reset all settings to factory defaults, including theme,
              scanner toggles, and all preferences. This cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleResetDefaults}
              className="text-destructive-foreground bg-destructive hover:bg-destructive/90"
            >
              Reset Settings
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
