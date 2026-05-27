use super::*;

pub async fn scan() -> ScanResult {
    if !which("bun") {
        return ScanResult::skipped("bun not installed");
    }

    let mut result = ScanResult::new("bun");

    if let Ok(ver) = run_cmd("bun", &["--version"], None).await {
        result.bun_version = Some(ver);
    }

    result
}
