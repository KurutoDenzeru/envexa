import { createFileRoute } from "@tanstack/react-router"
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
import { Settings as SettingsIcon, Bell, ShieldCheck, Database, Save, Sun, Moon, Monitor, Paintbrush } from "lucide-react"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useTheme } from "@/components/theme-provider"

export const Route = createFileRoute("/settings")({ component: SettingsPage })

function SettingsPage() {
  const { theme, setTheme } = useTheme()
  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-border pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-neutral-200 to-neutral-500 bg-clip-text text-transparent flex items-center gap-3">
            <SettingsIcon className="w-8 h-8 text-muted-foreground" />
            Scanner Settings
          </h1>
          <p className="text-muted-foreground mt-2">
            Configure how Envexa scans and reports on your environments.
          </p>
        </div>
        <Button className="gap-2 bg-primary text-primary-foreground hover:bg-neutral-200 shadow-[0_0_20px_-5px_rgba(255,255,255,0.3)]">
          <Save className="w-4 h-4" /> Save Changes
        </Button>
      </div>

      <div className="grid gap-6">
        <Card className="bg-card/50 border-border backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-green-500" />
              <CardTitle>Security & Auditing</CardTitle>
            </div>
            <CardDescription>Configure vulnerability thresholds and auto-auditing behaviors.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">Strict Mode</Label>
                <p className="text-sm text-muted-foreground/60">Fail builds immediately if critical vulnerabilities are detected.</p>
              </div>
              <Switch id="strict-mode" defaultChecked />
            </div>
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">Include Dev Dependencies</Label>
                <p className="text-sm text-muted-foreground/60">Scan toolchains for issues in development dependencies.</p>
              </div>
              <Switch id="dev-deps" />
            </div>
          </CardContent>
        </Card>

        <Card className="bg-card/50 border-border backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Database className="w-5 h-5 text-blue-500" />
              <CardTitle>Toolchain Ignorance</CardTitle>
            </div>
            <CardDescription>Select which package managers Envexa should skip scanning.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">Skip Python (pip)</Label>
                <p className="text-sm text-muted-foreground/60">Ignore requirements.txt and pip envs entirely.</p>
              </div>
              <Switch id="skip-pip" />
            </div>
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">Skip Node (npm/bun)</Label>
                <p className="text-sm text-muted-foreground/60">Ignore package.json files.</p>
              </div>
              <Switch id="skip-npm" />
            </div>
          </CardContent>
        </Card>

        <Card className="bg-card/50 border-border backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Paintbrush className="w-5 h-5 text-purple-500" />
              <CardTitle>Appearance</CardTitle>
            </div>
            <CardDescription>Customize the look and feel of Envexa.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">Theme Preference</Label>
                <p className="text-sm text-muted-foreground/60">Select your preferred color theme.</p>
              </div>
              <Tabs value={theme} onValueChange={(v) => setTheme(v as any)} className="w-[200px]">
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

        <Card className="bg-card/50 border-border backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Bell className="w-5 h-5 text-yellow-500" />
              <CardTitle>Notifications</CardTitle>
            </div>
            <CardDescription>How you want to be alerted about scanning results.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-border/50 bg-muted/50 p-4 transition-colors hover:bg-muted">
              <div className="space-y-0.5">
                <Label className="text-base text-foreground/90">System Notifications</Label>
                <p className="text-sm text-muted-foreground/60">Show OS level notifications when background scans complete.</p>
              </div>
              <Switch id="sys-notifs" defaultChecked />
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
