use super::*;

pub async fn scan() -> ScanResult {
    if !which("rustc") {
        return ScanResult::skipped("Rust not installed");
    }

    let mut result = ScanResult::new("cargo");

    if let Ok(ver) = run_cmd("rustc", &["--version"]).await {
        result.rustc_version = Some(ver);
    }

    if which("cargo") {
        if let Ok(ver) = run_cmd("cargo", &["--version"]).await {
            result.cargo_version = Some(ver);
        }
    }

    if !which("cargo-outdated") {
        result.issues.push("cargo-outdated not installed (run: cargo install cargo-outdated)".into());
        return result;
    }

    if let Ok(out) = run_cmd("cargo-outdated", &["--format=json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(crates) = data["dependencies"].as_array() {
                for c in crates {
                    result.outdated.push(PackageInfo {
                        name: c["name"].as_str().unwrap_or("?").to_string(),
                        current: c["project_version"].as_str().unwrap_or("?").to_string(),
                        latest: c["latest_version"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }

    result.status = if result.outdated.is_empty() { "ok".into() } else { "warning".into() };

    result
}
