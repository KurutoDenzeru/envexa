use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub current: String,
    pub latest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    pub package: String,
    pub severity: String,
    pub title: String,
    pub cve: Option<String>,
    pub patched_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditItem {
    pub name: String,
    pub current: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupItem {
    pub category: String,
    pub description: String,
    pub size: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub tool: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruby_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rustc_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnpm_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bun_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deno_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outdated_formulae: Vec<PackageInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outdated_casks: Vec<PackageInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outdated: Vec<PackageInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outdated_global: Vec<PackageInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_usage: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vulnerabilities: Vec<VulnerabilityInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_items: Vec<AuditItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cleanup_items: Vec<CleanupItem>,
}

impl ScanResult {
    pub fn new(tool: &str) -> Self {
        Self {
            tool: tool.to_string(),
            status: "ok".to_string(),
            version: None,
            node_version: None,
            python_version: None,
            ruby_version: None,
            rustc_version: None,
            cargo_version: None,
            pnpm_version: None,
            bun_version: None,
            deno_version: None,
            installed_count: None,
            outdated_formulae: Vec::with_capacity(4),
            outdated_casks: Vec::with_capacity(4),
            outdated: Vec::with_capacity(8),
            outdated_global: Vec::with_capacity(8),
            issues: Vec::with_capacity(2),
            disk_usage: None,
            project_type: None,
            vulnerabilities: Vec::with_capacity(4),
            audit_items: Vec::with_capacity(4),
            cleanup_items: Vec::with_capacity(4),
        }
    }

    pub fn skipped(reason: &str) -> Self {
        let mut r = Self::new("");
        r.status = "skipped".to_string();
        r.issues.push(reason.to_string());
        r
    }
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

use std::collections::HashMap as StdHashMap;
use std::sync::{LazyLock, Mutex};

static WHICH_CACHE: LazyLock<Mutex<StdHashMap<String, bool>>> =
    LazyLock::new(|| Mutex::new(StdHashMap::new()));

pub fn which(cmd: &str) -> bool {
    {
        let cache = WHICH_CACHE.lock().unwrap();
        if let Some(&found) = cache.get(cmd) {
            return found;
        }
    }
    let found = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|d| std::path::Path::new(d).join(cmd).exists());
    WHICH_CACHE.lock().unwrap().insert(cmd.to_string(), found);
    found
}

pub async fn run_cmd(
    program: &str,
    args: &[&str],
    timeout: Option<Duration>,
) -> Result<String, anyhow::Error> {
    let cmd = Command::new(program).args(args).output();
    let to_wait = timeout.unwrap_or(DEFAULT_TIMEOUT);
    let output = tokio::time::timeout(to_wait, cmd).await??;
    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned());
    Ok(stdout.trim().to_string())
}

pub async fn run_cmd_in(
    dir: &std::path::Path,
    program: &str,
    args: &[&str],
    timeout: Option<Duration>,
) -> Result<String, anyhow::Error> {
    let cmd = Command::new(program).args(args).current_dir(dir).output();
    let to_wait = timeout.unwrap_or(DEFAULT_TIMEOUT);
    let output = tokio::time::timeout(to_wait, cmd).await??;
    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned());
    Ok(stdout.trim().to_string())
}

pub fn get_project_path() -> std::path::PathBuf {
    crate::core::config::load_config()
        .project_path
        .filter(|p| !p.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

pub mod audit;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod ci;
pub mod cleanup;
pub mod deno;
pub mod docker;
pub mod gem;
pub mod git;
pub mod npm;
pub mod pip;
pub mod pnpm;
pub mod project;
pub mod security;
pub mod yarn;

pub async fn scan_all() -> HashMap<String, ScanResult> {
    scan_all_with(30, None).await
}

pub async fn scan_all_with(
    timeout_secs: u64,
    enabled: Option<&[String]>,
) -> HashMap<String, ScanResult> {
    let project_dir = get_project_path();
    let ignore = crate::core::config::EnvexaIgnore::load(&project_dir);

    use futures::stream::{self, StreamExt};

    macro_rules! scanner_task {
        ($name:ident) => {
            Box::pin(async { (stringify!($name), $name::scan().await) })
        };
    }

    #[allow(clippy::type_complexity)]
    let tasks: Vec<
        std::pin::Pin<Box<dyn std::future::Future<Output = (&'static str, ScanResult)> + Send>>,
    > = vec![
        scanner_task!(brew),
        scanner_task!(npm),
        scanner_task!(pnpm),
        scanner_task!(yarn),
        scanner_task!(bun),
        scanner_task!(deno),
        scanner_task!(pip),
        scanner_task!(gem),
        scanner_task!(cargo),
        scanner_task!(docker),
        scanner_task!(project),
        scanner_task!(security),
        scanner_task!(audit),
        scanner_task!(cleanup),
        scanner_task!(git),
        scanner_task!(ci),
    ];

    let mut results = HashMap::new();

    let mut buffered = stream::iter(tasks).buffer_unordered(16);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

    loop {
        let result = tokio::time::timeout_at(deadline, buffered.next()).await;
        match result {
            Ok(Some((name, mut res))) => {
                if let Some(enabled_list) = enabled {
                    if !enabled_list.iter().any(|e| e == name) {
                        results.insert(
                            name.to_string(),
                            ScanResult::skipped("Disabled by user settings"),
                        );
                        continue;
                    }
                }

                if ignore.should_ignore_tool(name) {
                    results.insert(
                        name.to_string(),
                        ScanResult::skipped("Ignored by .envexaignore"),
                    );
                    continue;
                }

                res.outdated
                    .retain(|p| !ignore.should_ignore_package(&p.name));
                res.outdated_global
                    .retain(|p| !ignore.should_ignore_package(&p.name));
                res.outdated_formulae
                    .retain(|p| !ignore.should_ignore_package(&p.name));
                res.outdated_casks
                    .retain(|p| !ignore.should_ignore_package(&p.name));

                res.vulnerabilities.retain(|v| {
                    !ignore.should_ignore_package(&v.package)
                        && !ignore.should_ignore_vuln(&v.package)
                        && v.cve
                            .as_ref()
                            .map(|cve| !ignore.should_ignore_vuln(cve))
                            .unwrap_or(true)
                });

                results.insert(name.to_string(), res);
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    results
}

pub async fn scan_one(name: &str) -> Option<ScanResult> {
    match name {
        "brew" => Some(brew::scan().await),
        "npm" => Some(npm::scan().await),
        "pnpm" => Some(pnpm::scan().await),
        "yarn" => Some(yarn::scan().await),
        "bun" => Some(bun::scan().await),
        "deno" => Some(deno::scan().await),
        "pip" => Some(pip::scan().await),
        "gem" => Some(gem::scan().await),
        "cargo" => Some(cargo::scan().await),
        "docker" => Some(docker::scan().await),
        "project" => Some(project::scan().await),
        "security" => Some(security::scan().await),
        "audit" => Some(audit::scan().await),
        "cleanup" => Some(cleanup::scan().await),
        "git" => Some(git::scan().await),
        "ci" => Some(ci::scan().await),
        _ => None,
    }
}
