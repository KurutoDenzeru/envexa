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

#[allow(dead_code)]
fn config_path() -> PathBuf {
    dir().join("config.json")
}

fn ensure() -> std::io::Result<()> {
    std::fs::create_dir_all(dir())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub cache_ttl_days: u64,
    pub project_path: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            cache_ttl_days: 7,
            project_path: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub report: Report,
    pub cached_at: String,
    pub ttl_days: u64,
}

#[allow(dead_code)]
pub fn load_config() -> UserConfig {
    let path = config_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[allow(dead_code)]
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

pub fn write_cache(report: &Report, ttl_days: u64) -> std::io::Result<()> {
    ensure()?;
    let entry = CacheEntry {
        report: report.clone(),
        cached_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        ttl_days,
    };
    std::fs::write(cache_path(), serde_json::to_string_pretty(&entry)?)
}

#[allow(dead_code)]
pub fn cache_expired(entry: &CacheEntry) -> bool {
    chrono::NaiveDateTime::parse_from_str(&entry.cached_at, "%Y-%m-%dT%H:%M:%S")
        .map(|dt| {
            let expiry = dt + chrono::Duration::days(entry.ttl_days as i64);
            chrono::Local::now().naive_local() > expiry
        })
        .unwrap_or(true)
}

#[allow(dead_code)]
pub fn remove_all() -> std::io::Result<()> {
    let d = dir();
    if d.exists() {
        std::fs::remove_dir_all(&d)?;
    }
    Ok(())
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
