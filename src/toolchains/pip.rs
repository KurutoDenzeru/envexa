use super::*;

pub async fn scan() -> ScanResult {
    if !which("python3") {
        return ScanResult::skipped("Python not installed");
    }

    let mut result = ScanResult::new("pip");

    let has_pip3 = which("pip3");

    if has_pip3 {
        let (py_ver, pip_ver, outdated) = tokio::join!(
            run_cmd("python3", &["--version"], None),
            run_cmd("pip3", &["--version"], None),
            run_cmd("pip3", &["list", "--outdated", "--format=json"], None)
        );

        if let Ok(ver) = py_ver {
            result.python_version = Some(ver);
        }
        if let Ok(ver) = pip_ver {
            result.version = Some(ver);
        }
        if let Ok(out) = outdated {
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
    } else {
        if let Ok(ver) = run_cmd("python3", &["--version"], None).await {
            result.python_version = Some(ver);
        }
    }

    result.status = if result.outdated.is_empty() {
        "ok".into()
    } else {
        "warning".into()
    };

    result
}
