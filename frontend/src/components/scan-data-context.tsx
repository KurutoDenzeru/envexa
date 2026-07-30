import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react"

export interface ScanReport {
  timestamp: string
  results: Record<string, any>
}

interface ScanDataContextValue {
  report: ScanReport | null
  loading: boolean
  error: boolean
  refetch: (force?: boolean) => Promise<void>
}

const ScanDataContext = createContext<ScanDataContextValue | null>(null)

export function ScanDataProvider({ children }: { children: ReactNode }) {
  const [report, setReport] = useState<ScanReport | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(false)

  const refetch = useCallback(async (force = false) => {
    setLoading(true)
    setError(false)
    window.dispatchEvent(new CustomEvent("scanner-status", { detail: "warming" }))
    try {
      const url = force ? "/api/scan?force=true" : "/api/scan"
      const res = await fetch(url)
      const data: unknown = await res.json()
      setReport(data as ScanReport)
      window.dispatchEvent(new CustomEvent("scanner-status", { detail: "active" }))
    } catch (e) {
      console.error("Failed to fetch report", e)
      setError(true)
      window.dispatchEvent(new CustomEvent("scanner-status", { detail: "error" }))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refetch()
  }, [refetch])

  return (
    <ScanDataContext.Provider value={{ report, loading, error, refetch }}>
      {children}
    </ScanDataContext.Provider>
  )
}

export function useScanData() {
  const ctx = useContext(ScanDataContext)
  if (!ctx) throw new Error("useScanData must be used within ScanDataProvider")
  return ctx
}
