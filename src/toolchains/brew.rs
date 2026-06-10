use super::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

#[derive(Deserialize, Debug)]
struct JwsPayload {
    payload: String,
}

#[derive(Deserialize, Debug)]
struct BrewApiCache {
    metadata: Option<ApiMetadata>,
    formulae: HashMap<String, FormulaInfo>,
    casks: HashMap<String, CaskInfo>,
}

#[derive(Deserialize, Debug)]
struct ApiMetadata {
    homebrew_version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct FormulaInfo {
    stable_version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CaskInfo {
    version: Option<String>,
}

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

fn get_installed(path: &Path) -> HashMap<String, String> {
    let mut installed = HashMap::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if let Ok(versions) = fs::read_dir(entry.path()) {
                    let mut version_dirs: Vec<_> = versions
                        .flatten()
                        .filter(|v| v.file_type().is_ok_and(|t| t.is_dir()))
                        .map(|v| v.file_name().to_string_lossy().to_string())
                        .collect();
                    version_dirs.sort();
                    if let Some(latest) = version_dirs.pop() {
                        installed.insert(name, latest);
                    }
                }
            }
        }
    }
    installed
}

#[allow(clippy::type_complexity)]
fn get_latest_versions_from_cache() -> Option<(HashMap<String, String>, HashMap<String, String>, Option<String>)> {
    let home = std::env::var("HOME").ok()?;
    let api_dir = PathBuf::from(home).join("Library/Caches/Homebrew/api/internal");
    
    let mut cache_path = None;
    if let Ok(entries) = fs::read_dir(&api_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("packages.") && name.ends_with(".jws.json") {
                cache_path = Some(entry.path());
                break;
            }
        }
    }
    
    let cache_path = cache_path?;
    let content = fs::read_to_string(cache_path).ok()?;
    let jws: JwsPayload = serde_json::from_str(&content).ok()?;
    let data: BrewApiCache = serde_json::from_str(&jws.payload).ok()?;
    
    let mut formulae_latest = HashMap::new();
    for (name, info) in data.formulae {
        if let Some(v) = info.stable_version {
            formulae_latest.insert(name, v);
        }
    }
    
    let mut casks_latest = HashMap::new();
    for (name, info) in data.casks {
        if let Some(v) = info.version {
            casks_latest.insert(name, v);
        }
    }
    
    let version = data.metadata.and_then(|m| m.homebrew_version);
    
    Some((formulae_latest, casks_latest, version))
}

fn scan_taps(base: &Path) -> HashMap<String, String> {
    let mut tap_versions = HashMap::new();
    let taps_dir = base.join("Library/Taps");
    if !taps_dir.exists() {
        return tap_versions;
    }
    
    let mut dirs_to_visit = vec![taps_dir];
    let re = Regex::new(r#"version\s+['"]([^'"]+)['"]"#).unwrap();
    
    while let Some(dir) = dirs_to_visit.pop() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue;
                    }
                    if ft.is_dir() {
                        dirs_to_visit.push(entry.path());
                    } else if ft.is_file() && entry.path().extension().is_some_and(|ext| ext == "rb") {
                        let tap_name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Some(caps) = re.captures(&content) {
                                tap_versions.insert(tap_name, caps[1].to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    tap_versions
}

fn is_outdated(current: &str, latest: &str) -> bool {
    if current == latest {
        return false;
    }
    if current.starts_with(&format!("{}_", latest)) {
        return false;
    }
    if latest == ":latest" || latest == "latest" {
        return false;
    }
    true
}

pub async fn scan() -> ScanResult {
    if !which("brew") {
        return ScanResult::skipped("Homebrew not installed");
    }

    let mut result = ScanResult::new("brew");

    let base = if Path::new("/opt/homebrew").exists() {
        Path::new("/opt/homebrew")
    } else if Path::new("/usr/local").exists() {
        Path::new("/usr/local")
    } else {
        return ScanResult::skipped("Homebrew not installed");
    };

    if let Some((core_formulae, core_casks, cache_version)) = get_latest_versions_from_cache() {
        let tap_versions = scan_taps(base);
        
        let installed_formulae = get_installed(&base.join("Cellar"));
        let installed_casks = get_installed(&base.join("Caskroom"));
        
        result.installed_count = Some((installed_formulae.len() + installed_casks.len()) as u64);
        
        for (name, current) in installed_formulae {
            let latest = core_formulae.get(&name).or_else(|| tap_versions.get(&name));
            if let Some(latest) = latest {
                if is_outdated(&current, latest) {
                    result.outdated_formulae.push(PackageInfo {
                        name,
                        current,
                        latest: latest.clone(),
                    });
                }
            }
        }
        
        for (name, current) in installed_casks {
            let latest = core_casks.get(&name).or_else(|| tap_versions.get(&name));
            if let Some(latest) = latest {
                if is_outdated(&current, latest) {
                    result.outdated_casks.push(PackageInfo {
                        name,
                        current,
                        latest: latest.clone(),
                    });
                }
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

        if let Some(v) = cache_version {
            result.version = Some(v);
        } else if let Ok(ver) = run_cmd("brew", &["--version"], None).await {
            result.version = ver.split_whitespace().nth(1).map(|s| s.to_string());
        }

        return result;
    }

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
                        current: f
                            .installed_versions
                            .into_iter()
                            .next()
                            .unwrap_or_else(|| "?".to_string()),
                        latest: f.current_version,
                    });
                }
                for c in data.casks {
                    result.outdated_casks.push(PackageInfo {
                        name: c.name,
                        current: c
                            .installed_versions
                            .into_iter()
                            .next()
                            .unwrap_or_else(|| "?".to_string()),
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
