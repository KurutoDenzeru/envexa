use super::*;

pub async fn scan() -> ScanResult {
    if !which("pnpm") {
        return ScanResult::skipped("pnpm not installed");
    }

    let mut result = ScanResult::new("pnpm");

    if let Ok(ver) = run_cmd("node", &["--version"], None).await {
        result.node_version = Some(ver);
    }

    if let Ok(ver) = run_cmd("pnpm", &["--version"], None).await {
        result.pnpm_version = Some(ver);
    }

    result
}
