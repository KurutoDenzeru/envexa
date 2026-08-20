import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Kbd, KbdGroup } from "@/components/ui/kbd"
import { modLabel } from "@/lib/platform"

export interface ShortcutGroup {
  heading: string
  shortcuts: { keys: string[][]; description: string }[]
}

export const shortcutGroups: ShortcutGroup[] = [
  {
    heading: "General",
    shortcuts: [
      { keys: [[modLabel, "K"]], description: "Open command palette" },
      { keys: [["?"]], description: "Show keyboard shortcuts" },
      { keys: [["R"]], description: "Rescan now" },
      { keys: [["/"]], description: "Focus search" },
      { keys: [[modLabel, "B"]], description: "Toggle sidebar" },
      { keys: [["Esc"]], description: "Close dialogs / clear search" },
    ],
  },
  {
    heading: "Navigation",
    shortcuts: [
      { keys: [[modLabel, "1"]], description: "Overview" },
      { keys: [[modLabel, "2"]], description: "Vulnerabilities" },
      { keys: [[modLabel, "3"]], description: "Outdated" },
      { keys: [[modLabel, "4"]], description: "Toolchains" },
      { keys: [[modLabel, "5"]], description: "System Logs" },
      { keys: [[modLabel, ","]], description: "Settings" },
    ],
  },
]

interface ShortcutsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ShortcutsDialog({ open, onOpenChange }: ShortcutsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Keyboard shortcuts</DialogTitle>
          <DialogDescription>
            Navigate the dashboard without leaving the keyboard.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4">
          {shortcutGroups.map((group) => (
            <div key={group.heading} className="flex flex-col gap-1">
              <h3 className="px-1 text-xs font-medium text-muted-foreground">
                {group.heading}
              </h3>
              {group.shortcuts.map((shortcut) => (
                <div
                  key={shortcut.description}
                  className="flex items-center justify-between rounded-md px-1 py-1.5 text-sm"
                >
                  <span>{shortcut.description}</span>
                  <KbdGroup>
                    {shortcut.keys.map((combo) => (
                      <KbdGroup key={combo.join("+")}>
                        {combo.map((key, index) => (
                          <span key={key} className="flex items-center gap-1">
                            {index > 0 && (
                              <span className="text-xs text-muted-foreground">
                                +
                              </span>
                            )}
                            <Kbd>{key}</Kbd>
                          </span>
                        ))}
                      </KbdGroup>
                    ))}
                  </KbdGroup>
                </div>
              ))}
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  )
}
