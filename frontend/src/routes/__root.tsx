import { createRootRoute, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { ThemeProvider } from "@/components/theme-provider"

export const Route = createRootRoute({
  component: () => (
    <ThemeProvider defaultTheme="dark" storageKey="envexa-ui-theme">
      <SidebarProvider>
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
        <TanStackRouterDevtools />
      </SidebarProvider>
    </ThemeProvider>
  ),
  notFoundComponent: () => (
    <div className="flex items-center justify-center min-h-screen bg-background text-foreground">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">404</h1>
        <p className="text-muted-foreground">Page not found</p>
      </div>
    </div>
  ),
})
