use super::*;

pub async fn scan() -> ScanResult {
    if !which("node") {
        return ScanResult::skipped("Node.js not installed");
    }

    let mut result = ScanResult::new("npm");

    if let Ok(ver) = run_cmd("node", &["--version"]).await {
        result.node_version = Some(ver);
    }

    if !which("npm") {
        return result;
    }

    if let Ok(ver) = run_cmd("npm", &["--version"]).await {
        result.version = Some(ver);
    }

    if let Ok(out) = run_cmd("npm", &["outdated", "-g", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(obj) = data.as_object() {
                for (name, info) in obj {
                    result.outdated_global.push(PackageInfo {
                        name: name.clone(),
                        current: info["current"].as_str().unwrap_or("?").to_string(),
                        latest: info["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }

    result.status = if result.outdated_global.is_empty() {
        "ok".into()
    } else {
        "warning".into()
    };

    result
}
