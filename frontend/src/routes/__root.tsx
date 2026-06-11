import { createRootRoute, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"

export const Route = createRootRoute({
  component: () => (
    <SidebarProvider>
      <div className="flex min-h-screen w-full bg-neutral-950 font-sans text-neutral-100">
        <AppSidebar />
        <main className="flex-1 flex flex-col relative overflow-hidden">
          <header className="h-14 border-b border-white/10 flex items-center px-4 bg-neutral-950/50 backdrop-blur-md sticky top-0 z-10">
            <SidebarTrigger />
          </header>
          <div className="flex-1 overflow-auto p-4 md:p-8">
            <Outlet />
          </div>
        </main>
      </div>
      <TanStackRouterDevtools />
    </SidebarProvider>
  ),
  notFoundComponent: () => (
    <div className="flex items-center justify-center min-h-screen bg-neutral-950 text-neutral-100">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">404</h1>
        <p className="text-neutral-400">Page not found</p>
      </div>
    </div>
  ),
})
