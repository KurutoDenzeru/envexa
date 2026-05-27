use std::path::{Path, PathBuf};

use super::*;

fn detect_project_type(dir: &Path) -> Option<&'static str> {
    if dir.join("pnpm-lock.yaml").exists() || dir.join("pnpm-lock.yml").exists() {
        Some("pnpm")
    } else if dir.join("yarn.lock").exists() {
        Some("yarn")
    } else if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
        Some("bun")
    } else if dir.join("deno.json").exists() || dir.join("deno.jsonc").exists() {
        Some("deno")
    } else if dir.join("Cargo.toml").exists() {
        Some("cargo")
    } else if dir.join("poetry.lock").exists() {
        Some("poetry")
    } else if dir.join("Pipfile").exists() || dir.join("Pipfile.lock").exists() {
        Some("pipenv")
    } else if dir.join("requirements.txt").exists() {
        Some("requirements")
    } else if dir.join("package.json").exists() {
        Some("npm")
    } else if dir.join("go.mod").exists() {
        Some("go")
    } else if dir.join("build.gradle").exists() || dir.join("build.gradle.kts").exists() {
        Some("gradle")
    } else if dir.join("pom.xml").exists() {
        Some("maven")
    } else if dir.join("composer.json").exists() {
        Some("composer")
    } else {
        None
    }
}

async fn npm_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("npm") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "npm", &["outdated", "--json"]).await {
        if out.is_empty() || !out.starts_with('{') {
            return;
        }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(obj) = data.as_object() {
                for (name, info) in obj {
                    if name == "error" {
                        continue;
                    }
                    result.outdated.push(PackageInfo {
                        name: name.clone(),
                        current: info["current"].as_str().unwrap_or("?").to_string(),
                        latest: info["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn pnpm_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("pnpm") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "pnpm", &["outdated", "--json"]).await {
        if out.is_empty() || !out.starts_with('{') {
            return;
        }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(obj) = data.as_object() {
                for (name, info) in obj {
                    result.outdated.push(PackageInfo {
                        name: name.clone(),
                        current: info["current"].as_str().unwrap_or("?").to_string(),
                        latest: info["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn yarn_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("yarn") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "yarn", &["outdated", "--json"]).await {
        for line in out.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                let name = data["name"].as_str();
                let current = data["current"].as_str();
                let latest = data["latest"].as_str();
                if let (Some(n), Some(c), Some(l)) = (name, current, latest) {
                    result.outdated.push(PackageInfo {
                        name: n.to_string(),
                        current: c.to_string(),
                        latest: l.to_string(),
                    });
                }
            }
        }
    }
}

async fn bun_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("bun") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "bun", &["outdated", "--format=json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    result.outdated.push(PackageInfo {
                        name: item["name"].as_str().unwrap_or("?").to_string(),
                        current: item["current"].as_str().unwrap_or("?").to_string(),
                        latest: item["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn deno_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("deno") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "deno", &["outdated", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    result.outdated.push(PackageInfo {
                        name: item["name"].as_str().unwrap_or("?").to_string(),
                        current: item["current"].as_str().unwrap_or("?").to_string(),
                        latest: item["latest"].as_str().unwrap_or("?").to_string(),
                    });
                }
            }
        }
    }
}

async fn cargo_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("cargo-outdated") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "cargo", &["outdated", "--format", "json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    let name = item["name"].as_str().unwrap_or("?").to_string();
                    let current = item["project"].as_str().unwrap_or("?").to_string();
                    let latest = item["latest"].as_str().unwrap_or("?").to_string();
                    result.outdated.push(PackageInfo {
                        name,
                        current,
                        latest,
                    });
                }
            } else if let Some(obj) = data.as_object() {
                if let Some(arr) = obj.get("dependencies").and_then(|v| v.as_array()) {
                    for item in arr {
                        let name = item["name"].as_str().unwrap_or("?").to_string();
                        let current = item["project"].as_str().unwrap_or("?").to_string();
                        let latest = item["latest"].as_str().unwrap_or("?").to_string();
                        result.outdated.push(PackageInfo {
                            name,
                            current,
                            latest,
                        });
                    }
                }
            }
        }
    }
}

async fn poetry_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("poetry") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "poetry", &["show", "--outdated", "--format=json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(arr) = data.as_array() {
                for item in arr {
                    let name = item["name"].as_str().unwrap_or("?").to_string();
                    let current = item["version"].as_str().unwrap_or("?").to_string();
                    let latest = item["latest"].as_str().unwrap_or("?").to_string();
                    result.outdated.push(PackageInfo {
                        name,
                        current,
                        latest,
                    });
                }
                return;
            }
        }
    }
    if let Ok(out) = run_cmd_in(dir, "poetry", &["show", "--outdated"]).await {
        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let current = parts[1];
                let latest = parts[2];
                if (current.chars().next().is_some_and(|c| c.is_ascii_digit())
                    || current.contains('.'))
                    && (latest.chars().next().is_some_and(|c| c.is_ascii_digit())
                        || latest.contains('.'))
                {
                    result.outdated.push(PackageInfo {
                        name: name.to_string(),
                        current: current.to_string(),
                        latest: latest.to_string(),
                    });
                }
            }
        }
    }
}

fn parse_pip_outdated(out: &str, result: &mut ScanResult) {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(out) {
        if let Some(arr) = data.as_array() {
            for item in arr {
                let name = item["name"].as_str().unwrap_or("?").to_string();
                let current = item["version"].as_str().unwrap_or("?").to_string();
                let latest = item["latest_version"].as_str().unwrap_or("?").to_string();
                result.outdated.push(PackageInfo {
                    name,
                    current,
                    latest,
                });
            }
        }
    }
}

async fn pipenv_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("pipenv") {
        return;
    }
    if let Ok(out) = run_cmd_in(
        dir,
        "pipenv",
        &["run", "pip", "list", "--outdated", "--format=json"],
    )
    .await
    {
        parse_pip_outdated(&out, result);
    }
}

async fn pip_venv_outdated(dir: &Path, result: &mut ScanResult) {
    let dot_venv = dir.join(".venv").join("bin").join("pip");
    let venv = dir.join("venv").join("bin").join("pip");
    let pip_cmd = if dot_venv.exists() {
        Some(dot_venv)
    } else if venv.exists() {
        Some(venv)
    } else {
        None
    };

    if let Some(pip) = pip_cmd {
        if let Ok(out) = run_cmd_in(
            dir,
            pip.to_str().unwrap_or("pip"),
            &["list", "--outdated", "--format=json"],
        )
        .await
        {
            parse_pip_outdated(&out, result);
        }
    } else {
        let cmd = if which("pip3") {
            "pip3"
        } else if which("pip") {
            "pip"
        } else {
            return;
        };
        if let Ok(out) = run_cmd_in(dir, cmd, &["list", "--outdated", "--format=json"]).await {
            parse_pip_outdated(&out, result);
        }
    }
}

async fn go_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("go") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "go", &["list", "-u", "-m", "-json", "all"]).await {
        let text = out.replace("}\n{", "}\n---\n{");
        for chunk in text.split("\n---\n") {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(chunk) {
                if let Some(update) = data.get("Update") {
                    let name = data["Path"].as_str().unwrap_or("?").to_string();
                    let current = data["Version"].as_str().unwrap_or("?").to_string();
                    let latest = update["Version"].as_str().unwrap_or("?").to_string();
                    result.outdated.push(PackageInfo {
                        name,
                        current,
                        latest,
                    });
                }
            }
        }
    }
}

async fn composer_outdated(dir: &Path, result: &mut ScanResult) {
    if !which("composer") {
        return;
    }
    if let Ok(out) = run_cmd_in(dir, "composer", &["outdated", "--format=json", "--direct"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(installed) = data.get("installed").and_then(|v| v.as_array()) {
                for item in installed {
                    let name = item["name"].as_str().unwrap_or("?").to_string();
                    let current = item["version"].as_str().unwrap_or("?").to_string();
                    let latest = item["latest"].as_str().unwrap_or("?").to_string();
                    result.outdated.push(PackageInfo {
                        name,
                        current,
                        latest,
                    });
                }
            }
        }
    }
}

async fn gradle_outdated(_dir: &Path, result: &mut ScanResult) {
    result
        .issues
        .push("Gradle dependency check requires ben-manes/gradle-versions-plugin (skipped)".into());
}

async fn maven_outdated(_dir: &Path, result: &mut ScanResult) {
    result
        .issues
        .push("Maven outdated checking requires manual parsing (skipped)".into());
}

pub async fn scan() -> ScanResult {
    let project_path = crate::core::config::load_config()
        .project_path
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let pm = match detect_project_type(&project_path) {
        Some(p) => p,
        None => return ScanResult::skipped("no project lockfile found in current directory"),
    };

    let mut result = ScanResult::new("project");
    result.project_type = Some(pm.to_string());

    match pm {
        "npm" => npm_outdated(&project_path, &mut result).await,
        "pnpm" => pnpm_outdated(&project_path, &mut result).await,
        "yarn" => yarn_outdated(&project_path, &mut result).await,
        "bun" => bun_outdated(&project_path, &mut result).await,
        "deno" => deno_outdated(&project_path, &mut result).await,
        "cargo" => cargo_outdated(&project_path, &mut result).await,
        "poetry" => poetry_outdated(&project_path, &mut result).await,
        "pipenv" => pipenv_outdated(&project_path, &mut result).await,
        "requirements" => pip_venv_outdated(&project_path, &mut result).await,
        "go" => go_outdated(&project_path, &mut result).await,
        "composer" => composer_outdated(&project_path, &mut result).await,
        "gradle" => gradle_outdated(&project_path, &mut result).await,
        "maven" => maven_outdated(&project_path, &mut result).await,
        _ => {}
    }

    let n = result.outdated.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} outdated package(s)"));
    }

    result
}
