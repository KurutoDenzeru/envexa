use super::*;

pub async fn scan() -> ScanResult {
    if !which("yarn") {
        return ScanResult::skipped("yarn not installed");
    }

    let mut result = ScanResult::new("yarn");

    if let Ok(ver) = run_cmd("yarn", &["--version"], None).await {
        result.version = Some(ver);
    }

    result
}
