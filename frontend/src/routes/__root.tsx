import { createRootRoute, Outlet } from '@tanstack/react-router'
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { ThemeProvider } from "@/components/theme-provider"
import { ThemeProvider as NextThemesProvider } from "next-themes"
import { Toaster } from "@/components/ui/sonner"

export const Route = createRootRoute({
  component: () => {
    const defaultOpen = document.cookie
      .split("; ")
      .find((row) => row.startsWith("sidebar_state="))
      ?.split("=")[1] !== "false";

    return (
      <ThemeProvider defaultTheme="dark" storageKey="envexa-ui-theme">
        <NextThemesProvider attribute="class" defaultTheme="dark" enableSystem>
          <Toaster position="top-right" />
          <SidebarProvider defaultOpen={defaultOpen}>
          <div className="flex min-h-screen w-full bg-background font-sans text-foreground">
          <AppSidebar />
          <main className="flex-1 flex flex-col relative overflow-hidden">
            <header className="h-14 border-b border-border flex items-center px-4 bg-background/80 backdrop-blur-md sticky top-0 z-10">
              <SidebarTrigger />
            </header>
            <div className="flex-1 overflow-auto p-4 md:p-8">
              <Outlet />
            </div>
          </main>
        </div>
      </SidebarProvider>
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
