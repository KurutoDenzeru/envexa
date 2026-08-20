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
import {
  Home,
  ShieldAlert,
  Boxes,
  Settings,
  ScrollText,
  PackageMinus,
} from "lucide-react"
import { Link } from "@tanstack/react-router"
import { Kbd, KbdGroup } from "@/components/ui/kbd"
import { useModifierHeld } from "@/hooks/use-hotkeys"
import { useSettingsDialog } from "@/components/settings-dialog"
import { modLabel } from "@/lib/platform"

const navItems = [
  { title: "Overview", url: "/", icon: Home, hint: "1" },
  {
    title: "Vulnerabilities",
    url: "/vulnerabilities",
    icon: ShieldAlert,
    hint: "2",
  },
  { title: "Outdated", url: "/outdated", icon: PackageMinus, hint: "3" },
  { title: "Toolchains", url: "/toolchains", icon: Boxes, hint: "4" },
  { title: "System Logs", url: "/logs", icon: ScrollText, hint: "5" },
]

export function AppSidebar() {
  const modHeld = useModifierHeld()
  const { openSettings } = useSettingsDialog()

  return (
    <Sidebar
      collapsible="icon"
      className="border-r border-border bg-card/50 backdrop-blur-xl"
    >
      <SidebarHeader className="flex h-14 flex-row items-center gap-3 border-b border-border p-4 text-foreground group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:!p-2">
        <img
          src="/bulldozer.png"
          alt="Envexa Logo"
          className="h-8 w-8 shrink-0 object-contain"
        />
        <span className="text-lg font-bold tracking-tight group-data-[collapsible=icon]:hidden">
          Envexa
        </span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel className="text-muted-foreground/60 group-data-[collapsible=icon]:hidden">
            Dashboards
          </SidebarGroupLabel>
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
                        activeProps={{
                          className: "bg-muted text-foreground font-medium",
                        }}
                        inactiveProps={{
                          className:
                            "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                        }}
                      />
                    }
                  >
                    <item.icon className="h-4 w-4 shrink-0" />
                    <span className="group-data-[collapsible=icon]:hidden">
                      {item.title}
                    </span>
                    {modHeld && (
                      <Kbd className="ml-auto group-data-[collapsible=icon]:hidden">
                        {item.hint}
                      </Kbd>
                    )}
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="flex flex-col gap-1 border-t border-border p-2 text-xs text-muted-foreground/60 group-data-[collapsible=icon]:!p-2">
        <SidebarMenu className="gap-1">
          <SidebarMenuItem>
            <SidebarMenuButton
              tooltip="Settings"
              onClick={() => openSettings()}
              className="flex w-full items-center gap-3 text-muted-foreground transition-colors duration-200 group-data-[collapsible=icon]:justify-center hover:bg-muted/50 hover:text-foreground"
            >
              <Settings className="h-4 w-4 shrink-0" />
              <span className="group-data-[collapsible=icon]:hidden">
                Settings
              </span>
              <KbdGroup className="ml-auto group-data-[collapsible=icon]:hidden">
                <Kbd>{modLabel}</Kbd>
                <Kbd>,</Kbd>
              </KbdGroup>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
