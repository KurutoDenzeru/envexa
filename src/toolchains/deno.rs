use super::*;

pub async fn scan() -> ScanResult {
    if !which("deno") {
        return ScanResult::skipped("deno not installed");
    }

    let mut result = ScanResult::new("deno");

    if let Ok(ver) = run_cmd("deno", &["--version"]).await {
        result.deno_version = Some(ver);
    }

    result
}
