package tui

// Data shapes mirror the Rust side (src/toolchains/mod.rs ScanResult serde
// fields, src/scanner/mod.rs Report) so bash scanners feed both runtimes
// unchanged. Spike ships mock data until the bash scanners are wired in.

type PackageInfo struct {
	Name    string `json:"name"`
	Current string `json:"current"`
	Latest  string `json:"latest"`
}

type ScanResult struct {
	Tool           string        `json:"tool"`
	Status         string        `json:"status"`
	Version        string        `json:"version,omitempty"`
	NodeVersion    string        `json:"node_version,omitempty"`
	PythonVersion  string        `json:"python_version,omitempty"`
	RubyVersion    string        `json:"ruby_version,omitempty"`
	RustcVersion   string        `json:"rustc_version,omitempty"`
	CargoVersion   string        `json:"cargo_version,omitempty"`
	PnpmVersion    string        `json:"pnpm_version,omitempty"`
	BunVersion     string        `json:"bun_version,omitempty"`
	DenoVersion    string        `json:"deno_version,omitempty"`
	InstalledCount *uint64       `json:"installed_count,omitempty"`
	Outdated       []PackageInfo `json:"outdated,omitempty"`
	OutdatedGlobal []PackageInfo `json:"outdated_global,omitempty"`
	Issues         []string      `json:"issues,omitempty"`
}

type OutdatedItem struct {
	Source  string `json:"source"`
	Name    string `json:"name"`
	Current string `json:"current"`
	Latest  string `json:"latest"`
	Size    string `json:"size"`
}

type Report struct {
	Timestamp string                `json:"timestamp"`
	Results   map[string]ScanResult `json:"results"`
	Outdated  []OutdatedItem        `json:"outdated"`
}

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
