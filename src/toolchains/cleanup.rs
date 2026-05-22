use super::*;

fn dir_size(path: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("du")
        .args(["-sh", &path.to_string_lossy()])
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(s.split_whitespace().next().unwrap_or("?").to_string())
    } else {
        None
    }
}

async fn check_brew_cleanup(result: &mut ScanResult) {
    if !which("brew") {
        return;
    }
    if let Ok(out) = run_cmd("brew", &["cleanup", "-n"]).await {
        let lines: Vec<&str> = out.lines().collect();
        let count = lines.len();
        if count > 0 {
            let size = lines
                .iter()
                .find(|l| l.contains("Would free"))
                .and_then(|l| l.split(':').nth(1).map(|s| s.trim().to_string()))
                .unwrap_or_else(|| format!("{count} items"));
            result.cleanup_items.push(CleanupItem {
                category: "brew".into(),
                description: format!("Homebrew cached formulae ({count} items)"),
                size: Some(size),
                command: Some("brew cleanup".into()),
            });
        }
    }
}

async fn check_npm_cache(result: &mut ScanResult) {
    let home = std::env::var("HOME").unwrap_or_default();
    let npm_cache = std::path::Path::new(&home).join(".npm").join("_cacache");
    if npm_cache.exists() {
        let size = dir_size(&npm_cache).unwrap_or_else(|| "?".into());
        result.cleanup_items.push(CleanupItem {
            category: "cache".into(),
            description: "npm cache (~/.npm/_cacache)".into(),
            size: Some(size),
            command: Some("npm cache clean --force".into()),
        });
    }
}

async fn check_cargo_cache(result: &mut ScanResult) {
    let home = std::env::var("HOME").unwrap_or_default();
    let cargo_cache = std::path::Path::new(&home).join(".cargo").join("registry");
    if cargo_cache.exists() {
        let size = dir_size(&cargo_cache).unwrap_or_else(|| "?".into());
        result.cleanup_items.push(CleanupItem {
            category: "cache".into(),
            description: "Cargo registry cache (~/.cargo/registry)".into(),
            size: Some(size),
            command: Some("cargo install cargo-cache && cargo cache -a".into()),
        });
    }
}

async fn check_bun_cache(result: &mut ScanResult) {
    let home = std::env::var("HOME").unwrap_or_default();
    let bun_cache = std::path::Path::new(&home)
        .join(".bun")
        .join("install")
        .join("cache");
    if bun_cache.exists() {
        let size = dir_size(&bun_cache).unwrap_or_else(|| "?".into());
        result.cleanup_items.push(CleanupItem {
            category: "cache".into(),
            description: "bun install cache (~/.bun/install/cache)".into(),
            size: Some(size),
            command: Some("rm -rf ~/.bun/install/cache".into()),
        });
    }
}

async fn check_pip_cache(result: &mut ScanResult) {
    let home = std::env::var("HOME").unwrap_or_default();
    let cache = if cfg!(target_os = "macos") {
        std::path::Path::new(&home)
            .join("Library")
            .join("Caches")
            .join("pip")
    } else {
        std::path::Path::new(&home).join(".cache").join("pip")
    };
    if cache.exists() {
        let size = dir_size(&cache).unwrap_or_else(|| "?".into());
        result.cleanup_items.push(CleanupItem {
            category: "cache".into(),
            description: "pip cache".into(),
            size: Some(size),
            command: Some("pip3 cache purge".into()),
        });
    }
}

async fn check_docker_disk(result: &mut ScanResult) {
    if !which("docker") {
        return;
    }
    if let Ok(out) = run_cmd("docker", &["system", "df", "--format", "{{json .}}"]).await {
        for line in out.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                let typ = data["Type"].as_str().unwrap_or("unknown");
                let size = data["Size"].as_str().unwrap_or("?");
                let reclaimable = data["Reclaimable"].as_str().unwrap_or("?");
                if reclaimable != "0B" {
                    result.cleanup_items.push(CleanupItem {
                        category: "docker".into(),
                        description: format!("Docker {} disk", typ),
                        size: Some(format!("{size} ({reclaimable} reclaimable)")),
                        command: Some(match typ {
                            "Images" => "docker image prune -a".into(),
                            "Containers" => "docker container prune".into(),
                            "Volumes" => "docker volume prune".into(),
                            "Build Cache" => "docker builder prune".into(),
                            _ => "docker system prune".into(),
                        }),
                    });
                }
            }
        }
    }
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("cleanup");

    check_brew_cleanup(&mut result).await;
    check_npm_cache(&mut result).await;
    check_cargo_cache(&mut result).await;
    check_bun_cache(&mut result).await;
    check_pip_cache(&mut result).await;
    check_docker_disk(&mut result).await;

    let total = result.cleanup_items.len();
    result.status = if total > 0 {
        "warning".into()
    } else {
        "ok".into()
    };
    if total > 0 {
        result
            .issues
            .push(format!("{total} cleanup item(s) available"));
    }

    result
}
