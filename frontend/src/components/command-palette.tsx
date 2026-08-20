import { useNavigate } from "@tanstack/react-router"
import { useTheme } from "next-themes"
import { toast } from "sonner"
import {
  Home,
  ShieldAlert,
  PackageMinus,
  Boxes,
  ScrollText,
  Settings,
  RefreshCw,
  ArrowUpCircle,
  SunMoon,
} from "lucide-react"
import {
  Command,
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command"
import { Kbd } from "@/components/ui/kbd"
import { useScanData } from "@/components/scan-data-context"
import { useSettingsDialog } from "@/components/settings-dialog"
import { modLabel } from "@/lib/platform"

const views = [
  { title: "Overview", url: "/", icon: Home, keys: [modLabel, "1"] },
  {
    title: "Vulnerabilities",
    url: "/vulnerabilities",
    icon: ShieldAlert,
    keys: [modLabel, "2"],
  },
  {
    title: "Outdated",
    url: "/outdated",
    icon: PackageMinus,
    keys: [modLabel, "3"],
  },
  {
    title: "Toolchains",
    url: "/toolchains",
    icon: Boxes,
    keys: [modLabel, "4"],
  },
  {
    title: "System Logs",
    url: "/logs",
    icon: ScrollText,
    keys: [modLabel, "5"],
  },
]

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const navigate = useNavigate()
  const { refetch } = useScanData()
  const { openSettings } = useSettingsDialog()
  const { resolvedTheme, setTheme } = useTheme()

  const run = (action: () => void) => {
    onOpenChange(false)
    action()
  }

  const checkUpdates = async () => {
    const id = toast.loading("Checking for updates...")
    try {
      const res = await fetch("/api/update/check")
      const data = await res.json()
      if (data.update_available) {
        toast.success("Update available!", {
          id,
          description: `Envexa v${data.latest_version} is available (you're on v${data.current_version}).`,
          duration: 8000,
        })
      } else {
        toast.success("You're up to date", {
          id,
          description: `Envexa v${data.current_version} is the latest version.`,
        })
      }
    } catch {
      toast.error("Failed to check for updates", {
        id,
        description: "Could not reach GitHub release API.",
      })
    }
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <Command>
        <CommandInput placeholder="Search views and actions..." />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Views">
            {views.map((view) => (
              <CommandItem
                key={view.url}
                value={view.title}
                onSelect={() => run(() => navigate({ to: view.url }))}
              >
                <view.icon />
                <span>{view.title}</span>
                <CommandShortcut className="flex gap-1 tracking-normal">
                  {view.keys.map((key) => (
                    <Kbd key={key}>{key}</Kbd>
                  ))}
                </CommandShortcut>
              </CommandItem>
            ))}
          </CommandGroup>
          <CommandGroup heading="Actions">
            <CommandItem
              value="Open settings"
              onSelect={() => run(() => openSettings())}
            >
              <Settings />
              <span>Open settings</span>
              <CommandShortcut className="flex gap-1 tracking-normal">
                <Kbd>{modLabel}</Kbd>
                <Kbd>,</Kbd>
              </CommandShortcut>
            </CommandItem>
            <CommandItem
              value="Rescan now"
              onSelect={() => run(() => refetch(true))}
            >
              <RefreshCw />
              <span>Rescan now</span>
              <CommandShortcut className="tracking-normal">
                <Kbd>R</Kbd>
              </CommandShortcut>
            </CommandItem>
            <CommandItem
              value="Check updates"
              onSelect={() => run(checkUpdates)}
            >
              <ArrowUpCircle />
              <span>Check updates</span>
            </CommandItem>
            <CommandItem
              value="Toggle theme"
              onSelect={() =>
                run(() => setTheme(resolvedTheme === "dark" ? "light" : "dark"))
              }
            >
              <SunMoon />
              <span>Toggle theme</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  )
}
