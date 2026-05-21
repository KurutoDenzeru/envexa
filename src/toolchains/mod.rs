use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub current: String,
    pub latest: String,
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

async fn run_cmd(program: &str, args: &[&str]) -> Result<String, anyhow::Error> {
    let cmd = Command::new(program).args(args).output();
    let output = tokio::time::timeout(TIMEOUT, cmd).await??;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub mod brew;
pub mod cargo;
pub mod gem;
pub mod npm;
pub mod pip;
pub mod docker;
pub mod pnpm;
pub mod deno;
pub mod bun;
pub mod yarn;

pub async fn scan_all() -> HashMap<String, ScanResult> {
    let (b, n, p, y, bu, de, pi, g, ca, dk) = tokio::join!(
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
        _ => None,
    }
}
