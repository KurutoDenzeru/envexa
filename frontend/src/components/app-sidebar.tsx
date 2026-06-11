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
  { title: "Settings", url: "/settings", icon: Settings },
]

export function AppSidebar() {
  return (
    <Sidebar className="border-r border-white/10 bg-neutral-950/50 backdrop-blur-xl">
      <SidebarHeader className="p-4 flex flex-row items-center gap-2 text-neutral-100">
        <Hexagon className="h-6 w-6 text-blue-500" />
        <span className="font-bold text-lg tracking-tight">Envexa</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-neutral-500">Dashboards</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton className="hover:bg-white/10 hover:text-white transition-colors duration-200" tooltip={item.title}>
                    <Link to={item.url} className="flex w-full items-center gap-3">
                      <item.icon className="h-4 w-4" />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="p-4 border-t border-white/10 text-xs text-neutral-500">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
          Scanner Service Active
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
