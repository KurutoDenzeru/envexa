import { createFileRoute, redirect } from "@tanstack/react-router"
import { SETTINGS_TABS, type SettingsTab } from "@/components/settings-dialog"

// Settings moved from a page to a dialog (#32). Keep the route as a redirect
// so existing links/bookmarks (including legacy ?tab= deep links) land on the
// dashboard with the dialog open at the matching category.
export const Route = createFileRoute("/settings")({
  validateSearch: (search: Record<string, unknown>): { tab?: SettingsTab } => ({
    tab: SETTINGS_TABS.includes(search.tab as SettingsTab)
      ? (search.tab as SettingsTab)
      : undefined,
  }),
  beforeLoad: ({ search }) => {
    throw redirect({
      to: "/",
      search: { settings: search.tab ?? "general" },
    })
  },
})
