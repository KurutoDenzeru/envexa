use super::*;

pub async fn scan() -> ScanResult {
    if !which("docker") {
        return ScanResult::skipped("Docker not installed");
    }

    let mut result = ScanResult::new("docker");

    let info_cmd = Command::new("docker")
        .args(["info", "--format", "{{json .}}"])
        .output();

    let (ver_res, info_res) = tokio::join!(
        run_cmd("docker", &["--version"], None),
        tokio::time::timeout(Duration::from_secs(10), info_cmd)
    );

    if let Ok(ver) = ver_res {
        result.version = Some(ver);
    }

    let info_check = match info_res {
        Ok(Ok(out)) => out,
        _ => {
            result.status = "error".into();
            result.issues.push("Docker daemon not running".into());
            return result;
        }
    };

    if info_check.status.success() {
        if let Ok(info) =
            serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&info_check.stdout))
        {
            let mut disk = serde_json::Map::new();
            if let Some(driver) = info["Driver"].as_str() {
                disk.insert("driver".into(), serde_json::Value::String(driver.into()));
            }
            result.disk_usage = Some(serde_json::Value::Object(disk));
        }
    }

    result
}
