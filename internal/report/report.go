// Package report holds the scan data contract shared by the bash toolchains,
// the Go bridge, and the TUI — the JSON shapes of src/toolchains/mod.rs
// ScanResult and src/scanner/mod.rs Report.
package report

import "encoding/json"

type PackageInfo struct {
	Name    string `json:"name"`
	Current string `json:"current"`
	Latest  string `json:"latest"`
}

type ScanResult struct {
	Tool             string              `json:"tool"`
	Status           string              `json:"status"`
	Version          string              `json:"version,omitempty"`
	NodeVersion      string              `json:"node_version,omitempty"`
	PythonVersion    string              `json:"python_version,omitempty"`
	RubyVersion      string              `json:"ruby_version,omitempty"`
	RustcVersion     string              `json:"rustc_version,omitempty"`
	CargoVersion     string              `json:"cargo_version,omitempty"`
	PnpmVersion      string              `json:"pnpm_version,omitempty"`
	BunVersion       string              `json:"bun_version,omitempty"`
	DenoVersion      string              `json:"deno_version,omitempty"`
	InstalledCount   *uint64             `json:"installed_count,omitempty"`
	DiskUsage        json.RawMessage     `json:"disk_usage,omitempty"`
	Outdated         []PackageInfo       `json:"outdated,omitempty"`
	OutdatedGlobal   []PackageInfo       `json:"outdated_global,omitempty"`
	Issues           []string            `json:"issues,omitempty"`
	ProjectType      string              `json:"project_type,omitempty"`
	Vulnerabilities  []VulnerabilityInfo `json:"vulnerabilities,omitempty"`
	AuditItems       []AuditItem         `json:"audit_items,omitempty"`
	SupplyChainRisks []SupplyChainRisk   `json:"supply_chain_risks,omitempty"`
}

type VulnerabilityInfo struct {
	Package        string   `json:"package"`
	Severity       string   `json:"severity"`
	Title          string   `json:"title"`
	CVE            *string  `json:"cve"`
	PatchedVersion string   `json:"patched_version"`
	DependencyPath []string `json:"dependency_path,omitempty"`
}

type AuditItem struct {
	Name    string `json:"name"`
	Current string `json:"current"`
	Note    string `json:"note"`
}

type SupplyChainRisk struct {
	Package     string `json:"package"`
	RiskType    string `json:"risk_type"`
	Description string `json:"description"`
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
