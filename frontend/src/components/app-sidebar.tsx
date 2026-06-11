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
import { Home, ShieldAlert, Boxes, Settings, ScrollText } from "lucide-react"
import { Link } from "@tanstack/react-router"
import { Separator } from "@/components/ui/separator"

const navItems = [
  { title: "Overview", url: "/", icon: Home },
  { title: "Vulnerabilities", url: "/vulnerabilities", icon: ShieldAlert },
  { title: "Toolchains", url: "/toolchains", icon: Boxes },
  { title: "System Logs", url: "/logs", icon: ScrollText },
]

export function AppSidebar() {
  return (
    <Sidebar collapsible="icon" className="border-r border-border bg-card/50 backdrop-blur-xl">
      <SidebarHeader className="h-14 border-b border-border p-4 flex flex-row items-center gap-3 text-foreground group-data-[collapsible=icon]:!p-2 group-data-[collapsible=icon]:justify-center">
        <img src="/bulldozer.png" alt="Envexa Logo" className="h-8 w-8 shrink-0 object-contain" />
        <span className="font-bold text-lg tracking-tight group-data-[collapsible=icon]:hidden">Envexa</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-muted-foreground/60 group-data-[collapsible=icon]:hidden">Dashboards</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu className="gap-1">
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton 
                    tooltip={item.title}
                    render={
                      <Link 
                        to={item.url} 
                        className="flex w-full items-center gap-3 transition-colors duration-200 group-data-[collapsible=icon]:justify-center"
                        activeProps={{ className: "bg-muted text-foreground font-medium" }}
                        inactiveProps={{ className: "text-muted-foreground hover:bg-muted/50 hover:text-foreground" }}
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
      <SidebarFooter className="p-2 group-data-[collapsible=icon]:!p-2 border-t border-border text-xs text-muted-foreground/60 space-y-2">
        <SidebarMenu className="gap-1">
          <SidebarMenuItem>
            <SidebarMenuButton 
              tooltip="Settings"
              render={
                <Link 
                  to="/settings" 
                  className="flex w-full items-center gap-3 transition-colors duration-200 group-data-[collapsible=icon]:justify-center"
                  activeProps={{ className: "bg-muted text-foreground font-medium" }}
                  inactiveProps={{ className: "text-muted-foreground hover:bg-muted/50 hover:text-foreground" }}
                />
              }
            >
              <Settings className="h-4 w-4 shrink-0" />
              <span className="group-data-[collapsible=icon]:hidden">Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <Separator className="bg-border/50 my-1" />
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton 
              tooltip="Scanner Service Active"
              className="flex w-full items-center gap-3 transition-colors duration-200 group-data-[collapsible=icon]:justify-center cursor-default hover:bg-muted/50"
            >
              <div className="w-2 h-2 shrink-0 rounded-full bg-green-500 animate-pulse group-data-[collapsible=icon]:w-2.5 group-data-[collapsible=icon]:h-2.5"></div>
              <span className="group-data-[collapsible=icon]:hidden text-foreground">Scanner Service Active</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
