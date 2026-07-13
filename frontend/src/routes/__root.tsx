import { createRootRoute, Outlet } from '@tanstack/react-router'
import { useState, useEffect } from 'react'
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { ThemeProvider } from "@/components/theme-provider"
import { ThemeProvider as NextThemesProvider } from "next-themes"
import { Toaster } from "@/components/ui/sonner"
import { ProjectPathSelector } from "@/components/project-path-selector"
import { ScanDataProvider } from "@/components/scan-data-context"

type ScannerStatus = "warming" | "active" | "error"

const STATUS_CONFIG: Record<ScannerStatus, { color: string; label: string }> = {
  warming: { color: "bg-orange-500", label: "Warming up..." },
  active: { color: "bg-emerald-500", label: "Scanner Service Active" },
  error: { color: "bg-red-500", label: "Failed to load environment report" },
}

export const Route = createRootRoute({
  component: () => {
    const [status, setStatus] = useState<ScannerStatus>("warming")
    const defaultOpen = document.cookie
      .split("; ")
      .find((row) => row.startsWith("sidebar_state="))
      ?.split("=")[1] !== "false";

    useEffect(() => {
      const handler = (e: CustomEvent<ScannerStatus>) => setStatus(e.detail)
      window.addEventListener("scanner-status", handler as EventListener)
      return () => window.removeEventListener("scanner-status", handler as EventListener)
    }, [])

    return (
      <ThemeProvider defaultTheme="dark" storageKey="envexa-ui-theme">
        <NextThemesProvider attribute="class" defaultTheme="dark" enableSystem>
          <ScanDataProvider>
            <Toaster position="top-right" />
            <SidebarProvider defaultOpen={defaultOpen}>
            <div className="flex min-h-screen w-full bg-background font-sans text-foreground">
            <AppSidebar />
              <main className="flex-1 flex flex-col min-h-0">
              <header className="h-14 shrink-0 border-b border-border flex items-center px-4 bg-background/80 backdrop-blur-md sticky top-0 z-10">
                <div className="flex items-center">
                  <SidebarTrigger />
                </div>
                <div className="flex-1 flex justify-center">
                  <ProjectPathSelector onPathChanged={() => window.location.reload()} />
                </div>
                <div className="flex items-center gap-2 text-sm text-muted-foreground shrink-0">
                  <div className={`w-2 h-2 rounded-full animate-pulse ${STATUS_CONFIG[status].color}`} />
                  <span>{STATUS_CONFIG[status].label}</span>
                </div>
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
    );
  },
  notFoundComponent: () => (
    <div className="flex items-center justify-center min-h-screen bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">404</h1>
        <p className="text-muted-foreground">Page not found</p>
      </div>
    </div>
  ),
})
