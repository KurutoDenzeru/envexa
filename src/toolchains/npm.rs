use super::*;

pub async fn scan() -> ScanResult {
    if !which("node") {
        return ScanResult::skipped("Node.js not installed");
    }

    let mut result = ScanResult::new("npm");

    if let Ok(ver) = run_cmd("node", &["--version"], None).await {
        result.node_version = Some(ver);
    }

    if !which("npm") {
        return result;
    }

    if let Ok(ver) = run_cmd("npm", &["--version"], None).await {
        result.version = Some(ver);
    }

    if let Ok(out) = run_cmd("npm", &["outdated", "-g", "--json"], None).await {
        result.outdated_global = parse_outdated(&out);
    }

    result.status = if result.outdated_global.is_empty() {
        "ok".into()
    } else {
        "warning".into()
    };

    result
}

pub fn parse_outdated(out: &str) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(out) {
        if let Some(obj) = data.as_object() {
            for (name, info) in obj {
                packages.push(PackageInfo {
                    name: name.clone(),
                    current: info["current"].as_str().unwrap_or("?").to_string(),
                    latest: info["latest"].as_str().unwrap_or("?").to_string(),
                });
            }
        }
    }
    packages
}
