use std::path::PathBuf;

use crate::scanner::Report;

pub fn dir() -> PathBuf {
    dirs().into_iter().next().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".envexa")
    })
}

fn dirs() -> Vec<PathBuf> {
    let mut d = Vec::new();
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        d.push(PathBuf::from(xdg).join("envexa"));
    }
    if let Ok(home) = std::env::var("HOME") {
        d.push(
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("envexa"),
        );
    }
    if let Ok(home) = std::env::var("HOME") {
        d.push(PathBuf::from(home).join(".envexa"));
    }
    d
}

fn cache_path() -> PathBuf {
    dir().join("cache.json")
}

fn config_path() -> PathBuf {
    dir().join("config.json")
}

fn ensure() -> std::io::Result<()> {
    std::fs::create_dir_all(dir())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub cache_ttl_minutes: u64,
    pub project_path: Option<String>,
    #[serde(default)]
    pub auto_scan_on_startup: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub verbose_logs: bool,
    #[serde(default = "default_scan_timeout")]
    pub scan_timeout_secs: u64,
    #[serde(default = "default_daemon_interval")]
    pub daemon_interval_secs: u64,
    #[serde(default = "default_export_format")]
    pub export_format: String,
    pub enabled_scanners: Option<Vec<String>>,
    #[serde(default = "default_log_retention")]
    pub log_retention_days: u64,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_scan_timeout() -> u64 {
    30
}

fn default_daemon_interval() -> u64 {
    14400
}

fn default_export_format() -> String {
    "markdown".to_string()
}

fn default_log_retention() -> u64 {
    7
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            cache_ttl_minutes: 15,
            project_path: None,
            auto_scan_on_startup: false,
            theme: default_theme(),
            verbose_logs: false,
            scan_timeout_secs: default_scan_timeout(),
            daemon_interval_secs: default_daemon_interval(),
            export_format: default_export_format(),
            enabled_scanners: None,
            log_retention_days: default_log_retention(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub report: Report,
    pub cached_at: String,
    pub ttl_minutes: u64,
}

pub fn load_config() -> UserConfig {
    let path = config_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &UserConfig) -> std::io::Result<()> {
    ensure()?;
    std::fs::write(config_path(), serde_json::to_string_pretty(cfg)?)
}

pub fn read_cache() -> Option<CacheEntry> {
    let path = cache_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn write_cache(report: &Report, ttl_minutes: u64) -> std::io::Result<()> {
    ensure()?;
    let entry = CacheEntry {
        report: report.clone(),
        cached_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        ttl_minutes,
    };
    std::fs::write(cache_path(), serde_json::to_string_pretty(&entry)?)
}

pub fn cache_expired(entry: &CacheEntry) -> bool {
    chrono::NaiveDateTime::parse_from_str(&entry.cached_at, "%Y-%m-%dT%H:%M:%S")
        .map(|dt| {
            let expiry = dt + chrono::Duration::minutes(entry.ttl_minutes as i64);
            chrono::Local::now().naive_local() > expiry
        })
        .unwrap_or(true)
}

pub fn logs_path() -> PathBuf {
    dir().join("logs.json")
}

pub fn read_logs(retention_days: u64) -> Vec<(chrono::DateTime<chrono::Local>, String)> {
    let path = logs_path();
    let logs: Vec<(chrono::DateTime<chrono::Local>, String)> = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    if retention_days == 0 {
        return logs;
    }

    let now = chrono::Local::now();
    let cutoff = now - chrono::Duration::days(retention_days as i64);

    logs.into_iter()
        .filter(|(time, _)| *time >= cutoff)
        .collect()
}

pub fn write_logs(logs: &[(chrono::DateTime<chrono::Local>, String)]) -> std::io::Result<()> {
    ensure()?;
    std::fs::write(logs_path(), serde_json::to_string_pretty(logs)?)
}

#[derive(Debug, Default)]
pub struct EnvexaIgnore {
    pub packages: Vec<String>,
    pub vulnerabilities: Vec<String>,
    pub toolchains: Vec<String>,
}

impl EnvexaIgnore {
    pub fn load(dir: &std::path::Path) -> Self {
        let mut ignore = Self::default();
        let path = dir.join(".envexaignore");
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(pkg) = line.strip_prefix("package:") {
                    ignore.packages.push(pkg.trim().to_string());
                } else if let Some(vuln) = line.strip_prefix("cve:") {
                    ignore.vulnerabilities.push(vuln.trim().to_string());
                } else if let Some(tool) = line.strip_prefix("tool:") {
                    ignore.toolchains.push(tool.trim().to_string());
                } else if let Some(pkg) = line.strip_prefix("pkg:") {
                    ignore.packages.push(pkg.trim().to_string());
                } else if let Some(vuln) = line.strip_prefix("vulnerability:") {
                    ignore.vulnerabilities.push(vuln.trim().to_string());
                }
            }
        }
        ignore
    }

    pub fn should_ignore_package(&self, pkg: &str) -> bool {
        self.packages.iter().any(|i| pkg.contains(i))
    }

    pub fn should_ignore_vuln(&self, vuln: &str) -> bool {
        self.vulnerabilities.iter().any(|i| vuln.contains(i))
    }

    pub fn should_ignore_tool(&self, tool: &str) -> bool {
        self.toolchains.iter().any(|i| tool == i)
    }
}
