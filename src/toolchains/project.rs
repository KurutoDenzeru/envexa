use std::path::{Path, PathBuf};

use super::*;

fn detect_project_type(dir: &Path) -> Option<&'static str> {
    if dir.join("pnpm-lock.yaml").exists() || dir.join("pnpm-lock.yml").exists() {
        Some("pnpm")
    } else if dir.join("yarn.lock").exists() {
        Some("yarn")
    } else if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
        Some("bun")
    } else if dir.join("deno.json").exists() || dir.join("deno.jsonc").exists() {
        Some("deno")
    } else if dir.join("package.json").exists() {
        Some("npm")
    } else {
        None
    }
}

async fn npm_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("npm") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "npm", &["outdated", "--json"]).await {
        if out.is_empty() || !out.starts_with('{') {
            return;
        }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(obj) = data.as_object() {
                for (name, info) in obj {
                    if name == "error" {
                        continue;
                    }
                    result.outdated.push(PackageInfo {
                        name: name.clone(),
                        current: info["current"].as_str().unwrap_or("?").to_string(),
                        latest: info["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn pnpm_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("pnpm") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "pnpm", &["outdated", "--json"]).await {
        if out.is_empty() || !out.starts_with('{') {
            return;
        }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(obj) = data.as_object() {
                for (name, info) in obj {
                    result.outdated.push(PackageInfo {
                        name: name.clone(),
                        current: info["current"].as_str().unwrap_or("?").to_string(),
                        latest: info["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn yarn_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("yarn") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "yarn", &["outdated", "--json"]).await {
        for line in out.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                let name = data["name"].as_str();
                let current = data["current"].as_str();
                let latest = data["latest"].as_str();
                if let (Some(n), Some(c), Some(l)) = (name, current, latest) {
                    result.outdated.push(PackageInfo {
                        name: n.to_string(),
                        current: c.to_string(),
                        latest: l.to_string(),
                    });
                }
            }
        }
    }
}

async fn bun_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("bun") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "bun", &["outdated", "--format=json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    result.outdated.push(PackageInfo {
                        name: item["name"].as_str().unwrap_or("?").to_string(),
                        current: item["current"].as_str().unwrap_or("?").to_string(),
                        latest: item["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn deno_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("deno") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "deno", &["outdated", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    result.outdated.push(PackageInfo {
                        name: item["name"].as_str().unwrap_or("?").to_string(),
                        current: item["current"].as_str().unwrap_or("?").to_string(),
                        latest: item["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

pub async fn scan() -> ScanResult {
    let project_path = crate::config::load_config()
        .project_path
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let pm = match detect_project_type(&project_path) {
        Some(p) => p,
        None => return ScanResult::skipped("no project lockfile found in current directory"),
    };

    let mut result = ScanResult::new("project");
    result.project_type = Some(pm.to_string());

    match pm {
        "npm" => npm_outdated(&project_path, &mut result).await,
        "pnpm" => pnpm_outdated(&project_path, &mut result).await,
        "yarn" => yarn_outdated(&project_path, &mut result).await,
        "bun" => bun_outdated(&project_path, &mut result).await,
        "deno" => deno_outdated(&project_path, &mut result).await,
        _ => {}
    }

    let n = result.outdated.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} outdated package(s)"));
    }

    result
}
