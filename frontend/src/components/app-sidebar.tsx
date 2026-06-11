import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { Home, ShieldAlert, Boxes, Settings, Hexagon } from "lucide-react"
import { Link } from "@tanstack/react-router"

const navItems = [
  { title: "Overview", url: "/", icon: Home },
  { title: "Vulnerabilities", url: "/vulnerabilities", icon: ShieldAlert },
  { title: "Toolchains", url: "/toolchains", icon: Boxes },
]

export function AppSidebar() {
  return (
    <Sidebar collapsible="icon" className="border-r border-white/10 bg-neutral-950/50 backdrop-blur-xl">
      <SidebarHeader className="p-4 flex flex-row items-center gap-2 text-neutral-100 group-data-[collapsible=icon]:!p-2 group-data-[collapsible=icon]:justify-center">
        <Hexagon className="h-6 w-6 shrink-0 text-blue-500" />
        <span className="font-bold text-lg tracking-tight group-data-[collapsible=icon]:hidden">Envexa</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-neutral-500 group-data-[collapsible=icon]:hidden">Dashboards</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton 
                    tooltip={item.title}
                    render={
                      <Link 
                        to={item.url} 
                        className="flex w-full items-center gap-3 transition-colors duration-200 group-data-[collapsible=icon]:justify-center"
                        activeProps={{ className: "bg-white/10 text-white font-medium" }}
                        inactiveProps={{ className: "text-neutral-400 hover:bg-white/5 hover:text-white" }}
                      />
                    }
                  >
                    <item.icon className="h-4 w-4 shrink-0" />
                    <span className="group-data-[collapsible=icon]:hidden">{item.title}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="p-4 border-t border-white/10 text-xs text-neutral-500 space-y-4">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton 
              tooltip="Settings"
              render={
                <Link 
                  to="/settings" 
                  className="flex w-full items-center gap-3 transition-colors duration-200 group-data-[collapsible=icon]:justify-center"
                  activeProps={{ className: "bg-white/10 text-white font-medium" }}
                  inactiveProps={{ className: "text-neutral-400 hover:bg-white/5 hover:text-white" }}
                />
              }
            >
              <Settings className="h-4 w-4 shrink-0" />
              <span className="group-data-[collapsible=icon]:hidden">Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <div className="flex items-center gap-2 group-data-[collapsible=icon]:justify-center">
          <div className="w-2 h-2 shrink-0 rounded-full bg-green-500 animate-pulse"></div>
          <span className="group-data-[collapsible=icon]:hidden">Scanner Service Active</span>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
