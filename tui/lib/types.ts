// Data contract mirrors internal/report (shared with the bash toolchains,
// the Go bridge, and the web dashboard) and internal/config.
export interface PackageInfo {
  name: string;
  current: string;
  latest: string;
}

export interface VulnerabilityInfo {
  package: string;
  severity: string;
  title: string;
  cve: string | null;
  patched_version: string;
  dependency_path?: string[];
}

export interface AuditItem {
  name: string;
  current: string;
  note: string;
}

export interface SupplyChainRisk {
  package: string;
  risk_type: string;
  description: string;
}

export interface ScanResult {
  tool: string;
  status: string;
  version?: string;
  node_version?: string;
  python_version?: string;
  ruby_version?: string;
  rustc_version?: string;
  cargo_version?: string;
  pnpm_version?: string;
  bun_version?: string;
  deno_version?: string;
  installed_count?: number;
  disk_usage?: unknown;
  outdated?: PackageInfo[];
  outdated_formulae?: PackageInfo[];
  outdated_casks?: PackageInfo[];
  outdated_global?: PackageInfo[];
  issues?: string[];
  project_type?: string;
  vulnerabilities?: VulnerabilityInfo[];
  audit_items?: AuditItem[];
  supply_chain_risks?: SupplyChainRisk[];
}

export interface OutdatedItem {
  source: string;
  name: string;
  current: string;
  latest: string;
  size: string;
}

export interface Report {
  timestamp: string;
  results: Record<string, ScanResult>;
  // Flattened view for the outdated table — built by the TUI bridge, same as
  // the Go bridge's Outdated field.
  outdated: OutdatedItem[];
}

export interface UserConfig {
  cache_ttl_minutes: number;
  project_path: string | null;
  recent_project_paths: string[];
  favorite_project_paths: string[];
  auto_scan_on_startup: boolean;
  theme: string;
  verbose_logs: boolean;
  scan_timeout_secs: number;
  daemon_interval_secs: number;
  export_format: string;
  enabled_scanners: string[] | null;
  log_retention_days: number;
}

export interface Detail {
  source: string;
  name: string;
  current: string;
  latest: string;
}
