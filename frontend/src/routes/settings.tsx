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
import { Settings as SettingsIcon, Bell, ShieldCheck, Database, Save } from "lucide-react"

export const Route = createFileRoute("/settings")({ component: SettingsPage })

function SettingsPage() {
  return (
    <div className="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-white/10 pb-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-neutral-200 to-neutral-500 bg-clip-text text-transparent flex items-center gap-3">
            <SettingsIcon className="w-8 h-8 text-neutral-400" />
            Scanner Settings
          </h1>
          <p className="text-neutral-400 mt-2">
            Configure how Envexa scans and reports on your environments.
          </p>
        </div>
        <Button className="gap-2 bg-white text-black hover:bg-neutral-200 shadow-[0_0_20px_-5px_rgba(255,255,255,0.3)]">
          <Save className="w-4 h-4" /> Save Changes
        </Button>
      </div>

      <div className="grid gap-6">
        <Card className="bg-neutral-950/50 border-white/10 backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-green-500" />
              <CardTitle>Security & Auditing</CardTitle>
            </div>
            <CardDescription>Configure vulnerability thresholds and auto-auditing behaviors.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
              <div className="space-y-0.5">
                <Label className="text-base text-neutral-200">Strict Mode</Label>
                <p className="text-sm text-neutral-500">Fail builds immediately if critical vulnerabilities are detected.</p>
              </div>
              <Switch id="strict-mode" defaultChecked />
            </div>
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
              <div className="space-y-0.5">
                <Label className="text-base text-neutral-200">Include Dev Dependencies</Label>
                <p className="text-sm text-neutral-500">Scan toolchains for issues in development dependencies.</p>
              </div>
              <Switch id="dev-deps" />
            </div>
          </CardContent>
        </Card>

        <Card className="bg-neutral-950/50 border-white/10 backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Database className="w-5 h-5 text-blue-500" />
              <CardTitle>Toolchain Ignorance</CardTitle>
            </div>
            <CardDescription>Select which package managers Envexa should skip scanning.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
              <div className="space-y-0.5">
                <Label className="text-base text-neutral-200">Skip Python (pip)</Label>
                <p className="text-sm text-neutral-500">Ignore requirements.txt and pip envs entirely.</p>
              </div>
              <Switch id="skip-pip" />
            </div>
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
              <div className="space-y-0.5">
                <Label className="text-base text-neutral-200">Skip Node (npm/bun)</Label>
                <p className="text-sm text-neutral-500">Ignore package.json files.</p>
              </div>
              <Switch id="skip-npm" />
            </div>
          </CardContent>
        </Card>

        <Card className="bg-neutral-950/50 border-white/10 backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Bell className="w-5 h-5 text-yellow-500" />
              <CardTitle>Notifications</CardTitle>
            </div>
            <CardDescription>How you want to be alerted about scanning results.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="flex items-center justify-between space-x-4 rounded-lg border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
              <div className="space-y-0.5">
                <Label className="text-base text-neutral-200">System Notifications</Label>
                <p className="text-sm text-neutral-500">Show OS level notifications when background scans complete.</p>
              </div>
              <Switch id="sys-notifs" defaultChecked />
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
