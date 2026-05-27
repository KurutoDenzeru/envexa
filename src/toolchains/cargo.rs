use super::*;

pub async fn scan() -> ScanResult {
    if !which("rustc") {
        return ScanResult::skipped("Rust not installed");
    }

    let mut result = ScanResult::new("cargo");

    if let Ok(ver) = run_cmd("rustc", &["--version"], None).await {
        result.rustc_version = Some(ver);
    }

    if which("cargo") {
        if let Ok(ver) = run_cmd("cargo", &["--version"], None).await {
            result.cargo_version = Some(ver);
        }
    }

    if !which("cargo-outdated") {
        result
            .issues
            .push("cargo-outdated not installed (run: cargo install cargo-outdated)".into());
        return result;
    }

    let project_path = get_project_path();
    if let Ok(out) = run_cmd_in(&project_path, "cargo-outdated", &["--format=json"], None).await {
        result.outdated = parse_outdated(&out);
    }

    result.status = if result.outdated.is_empty() {
        "ok".into()
    } else {
        "warning".into()
    };

    result
}

#[derive(serde::Deserialize)]
struct CargoOutdatedDependency {
    name: String,
    project_version: Option<String>,
    latest_version: Option<String>,
}

#[derive(serde::Deserialize)]
struct CargoOutdated {
    dependencies: Vec<CargoOutdatedDependency>,
}

pub fn parse_outdated(out: &str) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    if let Ok(data) = serde_json::from_str::<CargoOutdated>(out) {
        for c in data.dependencies {
            packages.push(PackageInfo {
                name: c.name,
                current: c.project_version.unwrap_or_else(|| "?".to_string()),
                latest: c.latest_version.unwrap_or_else(|| "?".to_string()),
            });
        }
    }
    packages
}
