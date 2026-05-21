use super::*;

pub async fn scan() -> ScanResult {
    if !which("python3") {
        return ScanResult::skipped("Python not installed");
    }

    let mut result = ScanResult::new("pip");

    if let Ok(ver) = run_cmd("python3", &["--version"]).await {
        result.python_version = Some(ver);
    }

    if !which("pip3") {
        return result;
    }

    if let Ok(ver) = run_cmd("pip3", &["--version"]).await {
        result.version = Some(ver);
    }

    if let Ok(out) = run_cmd("pip3", &["list", "--outdated", "--format=json"]).await {
        if let Ok(packages) = serde_json::from_str::<Vec<serde_json::Value>>(&out) {
            for pkg in packages {
                result.outdated.push(PackageInfo {
                    name: pkg["name"].as_str().unwrap_or("?").to_string(),
                    current: pkg["version"].as_str().unwrap_or("?").to_string(),
                    latest: pkg["latest_version"].as_str().unwrap_or("?").to_string(),
                });
            }
        }
    }

    result.status = if result.outdated.is_empty() {
        "ok".into()
    } else {
        "warning".into()
    };

    result
}
