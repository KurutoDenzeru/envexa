use super::*;

#[derive(serde::Deserialize)]
struct BrewItem {
    name: String,
    installed_versions: Vec<String>,
    current_version: String,
}

#[derive(serde::Deserialize)]
struct BrewOutdated {
    formulae: Vec<BrewItem>,
    casks: Vec<BrewItem>,
}

pub async fn scan() -> ScanResult {
    if !which("brew") {
        return ScanResult::skipped("Homebrew not installed");
    }

    let mut result = ScanResult::new("brew");

    let (ver_res, outdated_res, list_res) = tokio::join!(
        run_cmd("brew", &["--version"], None),
        run_cmd("brew", &["outdated", "--greedy", "--json"], None),
        run_cmd("brew", &["list", "--formula", "--versions"], None)
    );

    if let Ok(ver) = ver_res {
        result.version = ver.split_whitespace().nth(1).map(|s| s.to_string());
    }

    if let Ok(out) = outdated_res {
        if !out.is_empty() {
            if let Ok(data) = serde_json::from_str::<BrewOutdated>(&out) {
                for f in data.formulae {
                    result.outdated_formulae.push(PackageInfo {
                        name: f.name,
                        current: f.installed_versions.into_iter().next().unwrap_or_else(|| "?".to_string()),
                        latest: f.current_version,
                    });
                }
                for c in data.casks {
                    result.outdated_casks.push(PackageInfo {
                        name: c.name,
                        current: c.installed_versions.into_iter().next().unwrap_or_else(|| "?".to_string()),
                        latest: c.current_version,
                    });
                }
            }
        }
    }

    if let Ok(out) = list_res {
        if !out.is_empty() {
            result.installed_count = Some(out.lines().count() as u64);
        }
    }

    let total = result.outdated_formulae.len() + result.outdated_casks.len();
    result.status = if total == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if total > 0 {
        result.issues.push(format!("{total} outdated package(s)"));
    }

    result
}
