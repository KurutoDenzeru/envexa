use super::*;

async fn dir_size(path: &std::path::Path) -> Option<String> {
    let output = tokio::process::Command::new("du")
        .args(["-sh", &path.to_string_lossy()])
        .output()
        .await
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(s.split_whitespace().next().unwrap_or("?").to_string())
    } else {
        None
    }
}

async fn check_brew_cleanup() -> Option<CleanupItem> {
    if !which("brew") {
        return None;
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
            return Some(CleanupItem {
                category: "brew".into(),
                description: format!("Homebrew cached formulae ({count} items)"),
                size: Some(size),
                command: Some("brew cleanup".into()),
            });
        }
    }
    None
}

async fn check_npm_cache() -> Option<CleanupItem> {
    let home = std::env::var("HOME").unwrap_or_default();
    let npm_cache = std::path::Path::new(&home).join(".npm").join("_cacache");
    if npm_cache.exists() {
        let size = dir_size(&npm_cache).await.unwrap_or_else(|| "?".into());
        Some(CleanupItem {
            category: "cache".into(),
            description: "npm cache (~/.npm/_cacache)".into(),
            size: Some(size),
            command: Some("npm cache clean --force".into()),
        })
    } else {
        None
    }
}

async fn check_cargo_cache() -> Option<CleanupItem> {
    let home = std::env::var("HOME").unwrap_or_default();
    let cargo_cache = std::path::Path::new(&home).join(".cargo").join("registry");
    if cargo_cache.exists() {
        let size = dir_size(&cargo_cache).await.unwrap_or_else(|| "?".into());
        Some(CleanupItem {
            category: "cache".into(),
            description: "Cargo registry cache (~/.cargo/registry)".into(),
            size: Some(size),
            command: Some("cargo install cargo-cache && cargo cache -a".into()),
        })
    } else {
        None
    }
}

async fn check_bun_cache() -> Option<CleanupItem> {
    let home = std::env::var("HOME").unwrap_or_default();
    let bun_cache = std::path::Path::new(&home)
        .join(".bun")
        .join("install")
        .join("cache");
    if bun_cache.exists() {
        let size = dir_size(&bun_cache).await.unwrap_or_else(|| "?".into());
        Some(CleanupItem {
            category: "cache".into(),
            description: "bun install cache (~/.bun/install/cache)".into(),
            size: Some(size),
            command: Some("rm -rf ~/.bun/install/cache".into()),
        })
    } else {
        None
    }
}

async fn check_pip_cache() -> Option<CleanupItem> {
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
        let size = dir_size(&cache).await.unwrap_or_else(|| "?".into());
        Some(CleanupItem {
            category: "cache".into(),
            description: "pip cache".into(),
            size: Some(size),
            command: Some("pip3 cache purge".into()),
        })
    } else {
        None
    }
}

async fn check_docker_disk() -> Vec<CleanupItem> {
    let mut items = Vec::new();
    if !which("docker") {
        return items;
    }
    if let Ok(out) = run_cmd("docker", &["system", "df", "--format", "{{json .}}"]).await {
        for line in out.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                let typ = data["Type"].as_str().unwrap_or("unknown");
                let size = data["Size"].as_str().unwrap_or("?");
                let reclaimable = data["Reclaimable"].as_str().unwrap_or("?");
                if reclaimable != "0B" {
                    items.push(CleanupItem {
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
    items
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("cleanup");

    let (brew_res, npm_res, cargo_res, bun_res, pip_res, docker_res) = tokio::join!(
        check_brew_cleanup(),
        check_npm_cache(),
        check_cargo_cache(),
        check_bun_cache(),
        check_pip_cache(),
        check_docker_disk()
    );

    if let Some(item) = brew_res {
        result.cleanup_items.push(item);
    }
    if let Some(item) = npm_res {
        result.cleanup_items.push(item);
    }
    if let Some(item) = cargo_res {
        result.cleanup_items.push(item);
    }
    if let Some(item) = bun_res {
        result.cleanup_items.push(item);
    }
    if let Some(item) = pip_res {
        result.cleanup_items.push(item);
    }
    result.cleanup_items.extend(docker_res);

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
