use super::*;

pub async fn scan() -> ScanResult {
    if !which("pnpm") {
        return ScanResult::skipped("pnpm not installed");
    }

    let mut result = ScanResult::new("pnpm");

    let (node_ver, pnpm_ver) = tokio::join!(
        run_cmd("node", &["--version"], None),
        run_cmd("pnpm", &["--version"], None)
    );

    if let Ok(ver) = node_ver {
        result.node_version = Some(ver);
    }

    if let Ok(ver) = pnpm_ver {
        result.pnpm_version = Some(ver);
    }

    result
}
