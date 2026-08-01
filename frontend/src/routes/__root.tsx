import { createRootRoute, Outlet } from "@tanstack/react-router"
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { ThemeProvider } from "@/components/theme-provider"
import { ThemeProvider as NextThemesProvider } from "next-themes"
import { Toaster } from "@/components/ui/sonner"
import { ProjectPathSelector } from "@/components/project-path-selector"
import { ScanDataProvider, useScanData } from "@/components/scan-data-context"
import { Button } from "@/components/ui/button"
import { RefreshCw } from "lucide-react"
import { formatRelativeTime } from "@/lib/utils"

function ProjectPathWithScan() {
  const { refetch } = useScanData()
  return (
    <ProjectPathSelector
      onPathChanged={() => window.location.reload()}
      onSwitchAndScan={() => refetch(true)}
    />
  )
}

function NavbarScanStatus() {
  const { report, refetch } = useScanData()
  return (
    <div className="flex shrink-0 items-center gap-3 text-sm text-muted-foreground">
      <span className="hidden whitespace-nowrap md:inline">
        Last scanned{" "}
        {report?.timestamp ? formatRelativeTime(report.timestamp) : "just now"}
      </span>
      <Button
        variant="outline"
        size="sm"
        onClick={() => refetch(true)}
        className="gap-1.5"
      >
        <RefreshCw className="h-3.5 w-3.5" />
        Rescan Now
      </Button>
    </div>
  )
}

export const Route = createRootRoute({
  component: () => {
    const defaultOpen =
      document.cookie
        .split("; ")
        .find((row) => row.startsWith("sidebar_state="))
        ?.split("=")[1] !== "false"

    return (
      <ThemeProvider defaultTheme="dark" storageKey="envexa-ui-theme">
        <NextThemesProvider attribute="class" defaultTheme="dark" enableSystem>
          <ScanDataProvider>
            <Toaster position="top-right" />
            <SidebarProvider defaultOpen={defaultOpen}>
              <div className="flex min-h-screen w-full bg-background font-sans text-foreground">
                <AppSidebar />
                <main className="flex min-h-0 min-w-0 flex-1 flex-col">
                  <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center border-b border-border bg-background/80 px-4 backdrop-blur-md">
                    <div className="flex items-center">
                      <SidebarTrigger />
                    </div>
                    <div className="flex flex-1 justify-center">
                      <ProjectPathWithScan />
                    </div>
                    <NavbarScanStatus />
                  </header>
                  <div className="flex-1 overflow-auto p-4 md:p-8">
                    <Outlet />
                  </div>
                </main>
              </div>
            </SidebarProvider>
          </ScanDataProvider>
        </NextThemesProvider>
      </ThemeProvider>
    )
  },
  notFoundComponent: () => (
    <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
      <div className="text-center">
        <h1 className="mb-4 text-4xl font-bold">404</h1>
        <p className="text-muted-foreground">Page not found</p>
      </div>
    </div>
  ),
})
