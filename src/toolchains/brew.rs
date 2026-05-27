use super::*;

pub async fn scan() -> ScanResult {
    if !which("brew") {
        return ScanResult::skipped("Homebrew not installed");
    }

    let mut result = ScanResult::new("brew");

    if let Ok(ver) = run_cmd("brew", &["--version"], None).await {
        result.version = ver.split_whitespace().nth(1).map(|s| s.to_string());
    }

    if let Ok(out) = run_cmd("brew", &["outdated", "--json"], None).await {
        if !out.is_empty() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
                if let Some(formulae) = data["formulae"].as_array() {
                    for f in formulae {
                        result.outdated_formulae.push(PackageInfo {
                            name: f["name"].as_str().unwrap_or("?").to_string(),
                            current: f["installed_versions"][0]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            latest: f["current_version"].as_str().unwrap_or("?").to_string(),
                        });
                    }
                }
            }
        }
    }

    if let Ok(out) = run_cmd("brew", &["outdated", "--cask", "--greedy", "--json"], None).await {
        if !out.is_empty() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
                if let Some(casks) = data["casks"].as_array() {
                    for c in casks {
                        result.outdated_casks.push(PackageInfo {
                            name: c["name"].as_str().unwrap_or("?").to_string(),
                            current: c["installed_versions"][0]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            latest: c["current_version"].as_str().unwrap_or("?").to_string(),
                        });
                    }
                }
            }
        }
    }

    if let Ok(out) = run_cmd("brew", &["list", "--formula", "--versions"], None).await {
        if !out.is_empty() {
            result.installed_count = Some(out.lines().count() as u64);
        }
    }

    let total = result.outdated_formulae.len() + result.outdated_casks.len();
    result.status = if total == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if total > 0 {
        result.issues.push(format!("{total} outdated package(s)"));
    }

    result
}
