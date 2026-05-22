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
            outdated_formulae: vec![],
            outdated_casks: vec![],
            outdated: vec![],
            outdated_global: vec![],
            issues: vec![],
            disk_usage: None,
            project_type: None,
            vulnerabilities: vec![],
            audit_items: vec![],
            cleanup_items: vec![],
        }
    }

    pub fn skipped(reason: &str) -> Self {
        let mut r = Self::new("");
        r.status = "skipped".to_string();
        r.issues.push(reason.to_string());
        r
    }
}

const TIMEOUT: Duration = Duration::from_secs(30);

fn which(cmd: &str) -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|d| std::path::Path::new(d).join(cmd).exists())
}

pub async fn run_cmd(program: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let cmd = Command::new(program).args(args).output();
    let output = tokio::time::timeout(TIMEOUT, cmd).await??;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn run_cmd_in(
    dir: &std::path::Path,
    program: &str,
    args: &[&str],
) -> Result<String, anyhow::Error> {
    let cmd = Command::new(program).args(args).current_dir(dir).output();
    let output = tokio::time::timeout(TIMEOUT, cmd).await??;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub mod audit;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod cleanup;
pub mod deno;
pub mod docker;
pub mod gem;
pub mod npm;
pub mod pip;
pub mod pnpm;
pub mod project;
pub mod security;
pub mod yarn;

pub async fn scan_all() -> HashMap<String, ScanResult> {
    let (b, n, p, y, bu, de, pi, g, ca, dk, pr, se, au, cl) = tokio::join!(
        brew::scan(),
        npm::scan(),
        pnpm::scan(),
        yarn::scan(),
        bun::scan(),
        deno::scan(),
        pip::scan(),
        gem::scan(),
        cargo::scan(),
        docker::scan(),
        project::scan(),
        security::scan(),
        audit::scan(),
        cleanup::scan(),
    );
    let mut results = HashMap::new();
    results.insert("brew".to_string(), b);
    results.insert("npm".to_string(), n);
    results.insert("pnpm".to_string(), p);
    results.insert("yarn".to_string(), y);
    results.insert("bun".to_string(), bu);
    results.insert("deno".to_string(), de);
    results.insert("pip".to_string(), pi);
    results.insert("gem".to_string(), g);
    results.insert("cargo".to_string(), ca);
    results.insert("docker".to_string(), dk);
    results.insert("project".to_string(), pr);
    results.insert("security".to_string(), se);
    results.insert("audit".to_string(), au);
    results.insert("cleanup".to_string(), cl);
    results
}

#[allow(dead_code)]
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
        _ => None,
    }
}
