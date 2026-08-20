import { useEffect, useRef, useState } from "react"
import { isMac } from "@/lib/platform"

export const VIEW_ROUTES = [
  "/",
  "/vulnerabilities",
  "/outdated",
  "/toolchains",
  "/logs",
] as const

interface HotkeyHandlers {
  onRescan: () => void
  onNavigate: (to: string) => void
  onSettings: () => void
  onPalette: () => void
  onShortcuts: () => void
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  )
}

export function useHotkeys(handlers: HotkeyHandlers) {
  const latest = useRef(handlers)
  latest.current = handlers

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat) return

      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault()
        latest.current.onPalette()
        return
      }

      if (event.key === "Escape") {
        // Dialogs handle their own Esc; here we only release a focused search input.
        const active = document.activeElement
        if (
          active instanceof HTMLElement &&
          active.hasAttribute("data-search-input")
        ) {
          active.blur()
        }
        return
      }

      if (event.altKey) return
      if (isEditableTarget(event.target)) return
      // Portaled dialogs/dialog overlays mount only while open.
      if (document.querySelector('[role="dialog"]')) return

      const mod = isMac ? event.metaKey : event.ctrlKey
      if (mod) {
        // mod+b is handled by the sidebar's own listener.
        if (event.key === ",") {
          event.preventDefault()
          latest.current.onSettings()
          return
        }
        const index = Number.parseInt(event.key, 10) - 1
        if (index >= 0 && index < VIEW_ROUTES.length) {
          event.preventDefault()
          latest.current.onNavigate(VIEW_ROUTES[index])
        }
        return
      }

      if (event.ctrlKey || event.metaKey) return

      switch (event.key) {
        case "r":
          event.preventDefault()
          latest.current.onRescan()
          break
        case "/": {
          event.preventDefault()
          document.querySelector<HTMLElement>("[data-search-input]")?.focus()
          break
        }
        case "?":
          event.preventDefault()
          latest.current.onShortcuts()
          break
      }
    }

    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [])
}

// Tracks the OS modifier (Cmd on macOS, Ctrl elsewhere) so UI can reveal
// mod-based shortcut hints only while the key is physically held.
export function useModifierHeld(): boolean {
  const [held, setHeld] = useState(false)

  useEffect(() => {
    const sync = (event: KeyboardEvent) =>
      setHeld(isMac ? event.metaKey : event.ctrlKey)
    const reset = () => setHeld(false)

    window.addEventListener("keydown", sync)
    window.addEventListener("keyup", sync)
    window.addEventListener("blur", reset)
    return () => {
      window.removeEventListener("keydown", sync)
      window.removeEventListener("keyup", sync)
      window.removeEventListener("blur", reset)
    }
  }, [])

  return held
}
