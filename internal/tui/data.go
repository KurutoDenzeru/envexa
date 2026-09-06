package tui

import "github.com/KurutoDenzeru/envexa/internal/report"

// Data contract lives in internal/report (shared with the Go bridge); aliases
// keep the TUI view code short. Mock data covers the spike benchmark/test path.

type (
	PackageInfo       = report.PackageInfo
	ScanResult        = report.ScanResult
	OutdatedItem      = report.OutdatedItem
	Report            = report.Report
	VulnerabilityInfo = report.VulnerabilityInfo
	AuditItem         = report.AuditItem
	SupplyChainRisk   = report.SupplyChainRisk
)

func mockReport() Report {
	return Report{
		Timestamp: "2026-09-06T12:00:00Z",
		Results: map[string]ScanResult{
			"brew":     {Tool: "brew", Status: "ok", Version: "4.6.2"},
			"npm":      {Tool: "npm", Status: "warn", Version: "11.4.2", Issues: []string{"3 outdated package(s)"}},
			"pnpm":     {Tool: "pnpm", Status: "ok", Version: "10.15.0"},
			"yarn":     {Tool: "yarn", Status: "skipped"},
			"bun":      {Tool: "bun", Status: "ok", Version: "1.2.23"},
			"deno":     {Tool: "deno", Status: "ok", Version: "2.5.1"},
			"pip":      {Tool: "pip", Status: "error", Issues: []string{"no pyproject.toml"}},
			"gem":      {Tool: "gem", Status: "ok", Version: "3.7.1"},
			"cargo":    {Tool: "cargo", Status: "ok", Version: "1.91.0"},
			"docker":   {Tool: "docker", Status: "skipped"},
			"project":  {Tool: "project", Status: "warn", Issues: []string{"2 missing configs"}},
			"security": {Tool: "security", Status: "error", Issues: []string{"1 critical advisory"}},
			"audit":    {Tool: "audit", Status: "warn", Issues: []string{"1 low severity"}},
		},
		Outdated: []OutdatedItem{
			{Source: "npm", Name: "typescript", Current: "5.9.2", Latest: "5.9.3", Size: "62 MB"},
			{Source: "npm", Name: "vite", Current: "7.1.5", Latest: "8.0.0", Size: "24 MB"},
			{Source: "brew", Name: "ripgrep", Current: "14.1.1", Latest: "14.2.0", Size: "4 MB"},
			{Source: "cargo", Name: "serde", Current: "1.0.219", Latest: "1.0.225", Size: "2 MB"},
			{Source: "deno", Name: "dax", Current: "0.42.0", Latest: "0.44.1", Size: "1 MB"},
		},
	}
}
