import { useState, useEffect, useCallback, useRef } from "react"
import {
  FolderOpen,
  Check,
  ArrowRight,
  ChevronUp,
  Folder,
  CornerDownLeft,
  Star,
} from "lucide-react"
import { toast } from "sonner"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

interface ProjectData {
  current: string
  recent: string[]
  favorites: string[]
}

interface DirEntry {
  name: string
  full_path: string
}

interface DirsResponse {
  path: string
  parent: string | null
  entries: DirEntry[]
}

function shortenPath(p: string): string {
  const parts = p.replace(/\/$/, "").split("/")
  if (parts.length <= 2) return p
  return parts.slice(-2).join("/")
}

export function ProjectPathSelector({
  onPathChanged,
}: {
  onPathChanged?: () => void
}) {
  const [project, setProject] = useState<ProjectData | null>(null)
  const [open, setOpen] = useState(false)
  const [switching, setSwitching] = useState<string | null>(null)
  const [inputValue, setInputValue] = useState("")

  // Directory browser state
  const [browsePath, setBrowsePath] = useState<string | null>(null)
  const [dirs, setDirs] = useState<DirsResponse | null>(null)
  const [dirsLoading, setDirsLoading] = useState(false)
  const dirCache = useRef<Map<string, DirsResponse>>(new Map())

  const fetchProject = useCallback(async () => {
    try {
      const res = await fetch("/api/project")
      if (res.ok) {
        const data = await res.json()
        setProject(data)
        setInputValue(data.current)
      }
    } catch {
      // silently fail
    }
  }, [])

  const fetchDirs = useCallback(async (path: string) => {
    const cached = dirCache.current.get(path)
    if (cached) {
      setDirs(cached)
      setBrowsePath(cached.path)
      return
    }
    setDirsLoading(true)
    try {
      const res = await fetch(`/api/project/dirs?path=${encodeURIComponent(path)}`)
      if (res.ok) {
        const data: DirsResponse = await res.json()
        dirCache.current.set(path, data)
        setDirs(data)
        setBrowsePath(data.path)
      }
    } catch {
      // silently fail
    } finally {
      setDirsLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchProject()
  }, [fetchProject])

  // When dialog opens, start browsing from current path
  useEffect(() => {
    if (open && project) {
      setInputValue(project.current)
      fetchDirs(project.current)
    }
  }, [open, project, fetchDirs])

  const handleSwitch = async (path: string) => {
    setSwitching(path)
    try {
      const res = await fetch("/api/project", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      })
      if (!res.ok) {
        const err = await res.text()
        toast.error(err || "Failed to switch project")
        setSwitching(null)
        return
      }
      const data: ProjectData = await res.json()
      setProject(data)
      setInputValue(data.current)
      setOpen(false)
      toast.success("Project switched")
      onPathChanged?.()
    } catch {
      toast.error("Failed to switch project")
      setSwitching(null)
    }
  }

  const handleInputSubmit = () => {
    const trimmed = inputValue.trim()
    if (!trimmed) return
    if (trimmed === project?.current) {
      setOpen(false)
      return
    }
    handleSwitch(trimmed)
  }

  const navigateUp = () => {
    if (dirs?.parent) {
      fetchDirs(dirs.parent)
      setInputValue(dirs.parent)
    }
  }

  const navigateInto = (path: string) => {
    fetchDirs(path)
    setInputValue(path)
  }

  const toggleFavorite = async (path: string) => {
    try {
      const res = await fetch("/api/project/favorite", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      })
      if (res.ok) {
        const data: ProjectData = await res.json()
        setProject(data)
      }
    } catch {
      // silently fail
    }
  }

  const display = project?.current ?? "…"

  return (
    <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) dirCache.current.clear() }}>
      <DialogTrigger
        render={
          <button
            className={cn(
              "flex items-center gap-2 px-3 py-1.5 rounded-md",
              "bg-muted/50 border border-border/50",
              "hover:bg-muted transition-colors",
              "text-sm font-medium text-foreground",
              "max-w-[280px] truncate"
            )}
            title={display}
          />
        }
      >
        <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />
        <span className="truncate">{shortenPath(display)}</span>
      </DialogTrigger>
      <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Project Path</DialogTitle>
        </DialogHeader>

        {/* Path input */}
        <form
          onSubmit={(e) => {
            e.preventDefault()
            handleInputSubmit()
          }}
          className="flex gap-2"
        >
          <input
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder="/path/to/project"
            className={cn(
              "flex-1 px-3 py-2 text-sm rounded-md",
              "bg-muted border border-border/50",
              "text-foreground placeholder:text-muted-foreground/50",
              "outline-none focus:ring-1 focus:ring-ring",
              "font-mono"
            )}
          />
          <button
            type="submit"
            disabled={
              switching !== null || inputValue.trim() === project?.current
            }
            className={cn(
              "px-3 py-2 rounded-md text-sm font-medium",
              "bg-primary text-primary-foreground",
              "hover:bg-primary/90 transition-colors",
              "disabled:opacity-50 disabled:cursor-not-allowed",
              "flex items-center gap-1.5 shrink-0"
            )}
          >
            <ArrowRight className="h-4 w-4" />
            Switch
          </button>
        </form>

        {/* Directory browser */}
        <div className="rounded-lg border border-border/50 overflow-hidden">
          {/* Browser header with current location and up button */}
          <div className="flex items-center gap-2 px-3 py-2 bg-muted/50 border-b border-border/50">
            <button
              onClick={navigateUp}
              disabled={!dirs?.parent}
              className={cn(
                "p-1 rounded hover:bg-muted transition-colors",
                "disabled:opacity-30 disabled:cursor-not-allowed"
              )}
              title="Go to parent directory"
            >
              <ChevronUp className="h-4 w-4" />
            </button>
            <span className="text-xs text-muted-foreground font-mono truncate flex-1">
              {browsePath ?? "…"}
            </span>
            {dirs?.parent && (
              <button
                onClick={() => inputValue.trim() && handleSwitch(inputValue.trim())}
                disabled={switching !== null || inputValue.trim() === project?.current}
                className={cn(
                  "text-xs px-2 py-1 rounded",
                  "bg-green-600/20 text-green-400 hover:bg-green-600/30",
                  "transition-colors",
                  "disabled:opacity-50 disabled:cursor-not-allowed",
                  "flex items-center gap-1"
                )}
              >
                <CornerDownLeft className="h-3 w-3" />
                Select
              </button>
            )}
          </div>

          {/* Directory entries */}
          <div className="max-h-[280px] overflow-y-auto">
            {dirsLoading ? (
              <div className="flex items-center justify-center py-6 text-sm text-muted-foreground">
                Loading…
              </div>
            ) : dirs && dirs.entries.length > 0 ? (
              dirs.entries.map((entry) => {
                const isActive = entry.full_path === project?.current
                return (
                  <button
                    key={entry.full_path}
                    onClick={() => navigateInto(entry.full_path)}
                    onMouseEnter={() => {
                      const p = entry.full_path
                      if (!dirCache.current.has(p)) {
                        fetch(`/api/project/dirs?path=${encodeURIComponent(p)}`)
                          .then(r => r.ok ? r.json() : null)
                          .then(data => { if (data) dirCache.current.set(p, data) })
                          .catch(() => {})
                      }
                    }}
                    className={cn(
                      "w-full flex items-center gap-2 px-3 py-2 text-left",
                      "hover:bg-muted/50 transition-colors",
                      "border-b border-border/30 last:border-0",
                      isActive && "bg-muted/30"
                    )}
                  >
                    <Folder
                      className={cn(
                        "h-4 w-4 shrink-0",
                        isActive ? "text-green-500" : "text-muted-foreground"
                      )}
                    />
                    <span className="text-sm truncate flex-1">{entry.name}</span>
                    {isActive && (
                      <Check className="h-3.5 w-3.5 shrink-0 text-green-500" />
                    )}
                  </button>
                )
              })
            ) : (
              <div className="flex items-center justify-center py-6 text-sm text-muted-foreground">
                No subdirectories
              </div>
            )}
          </div>
        </div>

        {/* Favorite projects */}
        {project && project.favorites.length > 0 && (
          <div>
            <p className="text-xs font-medium text-muted-foreground mb-1.5 px-1 flex items-center gap-1.5">
              <Star className="h-3 w-3 fill-yellow-500 text-yellow-500" />
              Favorites
            </p>
            <div className="divide-y divide-border/50 rounded-lg border border-border/50 overflow-hidden">
              {project.favorites.map((p) => {
                const isActive = p === project.current
                const isSwitching = switching === p
                return (
                  <div
                    key={p}
                    className={cn(
                      "flex items-center gap-2 px-3 py-2",
                      "hover:bg-muted/50 transition-colors",
                      "border-b border-border/30 last:border-0",
                      isActive && "bg-muted/30"
                    )}
                  >
                    <button
                      onClick={() => !isActive && handleSwitch(p)}
                      disabled={isActive || isSwitching}
                      className={cn(
                        "flex-1 flex items-center gap-2 text-left min-w-0",
                        "disabled:cursor-default"
                      )}
                      title={p}
                    >
                      <FolderOpen
                        className={cn(
                          "h-4 w-4 shrink-0",
                          isActive ? "text-green-500" : "text-yellow-500"
                        )}
                      />
                      <span className="text-sm font-mono truncate">
                        {shortenPath(p)}
                      </span>
                      {isActive && (
                        <Check className="h-3.5 w-3.5 shrink-0 text-green-500" />
                      )}
                      {isSwitching && (
                        <span className="h-4 w-4 shrink-0 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
                      )}
                    </button>
                    <button
                      onClick={() => toggleFavorite(p)}
                      className="p-1 rounded hover:bg-muted transition-colors shrink-0"
                      title="Remove from favorites"
                    >
                      <Star className="h-3.5 w-3.5 fill-yellow-500 text-yellow-500" />
                    </button>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {/* Recent projects */}
        {project && project.recent.length > 0 && (
          <div>
            <p className="text-xs font-medium text-muted-foreground mb-1.5 px-1">
              Recent Projects
            </p>
            <div className="divide-y divide-border/50 rounded-lg border border-border/50 overflow-hidden">
              {project.recent.map((p) => {
                const isActive = p === project.current
                const isSwitching = switching === p
                const isFavorited = project.favorites.includes(p)
                return (
                  <div
                    key={p}
                    className={cn(
                      "flex items-center gap-2 px-3 py-2",
                      "hover:bg-muted/50 transition-colors",
                      "border-b border-border/30 last:border-0",
                      isActive && "bg-muted/30"
                    )}
                  >
                    <button
                      onClick={() => !isActive && handleSwitch(p)}
                      disabled={isActive || isSwitching}
                      className={cn(
                        "flex-1 flex items-center gap-2 text-left min-w-0",
                        "disabled:cursor-default"
                      )}
                      title={p}
                    >
                      <FolderOpen
                        className={cn(
                          "h-4 w-4 shrink-0",
                          isActive ? "text-green-500" : "text-muted-foreground"
                        )}
                      />
                      <span className="text-sm font-mono truncate">
                        {shortenPath(p)}
                      </span>
                      {isActive && (
                        <Check className="h-3.5 w-3.5 shrink-0 text-green-500" />
                      )}
                      {isSwitching && (
                        <span className="h-4 w-4 shrink-0 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
                      )}
                    </button>
                    <button
                      onClick={() => toggleFavorite(p)}
                      className="p-1 rounded hover:bg-muted transition-colors shrink-0"
                      title={isFavorited ? "Remove from favorites" : "Add to favorites"}
                    >
                      <Star
                        className={cn(
                          "h-3.5 w-3.5",
                          isFavorited
                            ? "fill-yellow-500 text-yellow-500"
                            : "text-muted-foreground"
                        )}
                      />
                    </button>
                  </div>
                )
              })}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
