use super::*;

fn parse_semver_parts(ver: &str) -> Vec<u64> {
    ver.trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse::<u64>().ok())
        .collect()
}

fn major(ver: &str) -> Option<u64> {
    parse_semver_parts(ver).first().copied()
}

async fn check_node_npm() -> Option<AuditItem> {
    let node = match run_cmd("node", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };
    let npm = match run_cmd("npm", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };

    let node_major = match major(&node) {
        Some(m) => m,
        None => return None,
    };
    let npm_major = match major(&npm) {
        Some(m) => m,
        None => return None,
    };

    let expected = if node_major >= 20 {
        10u64
    } else if node_major >= 18 {
        9u64
    } else if node_major >= 16 {
        8u64
    } else {
        6u64
    };

    if npm_major < expected {
        Some(AuditItem {
            name: "npm (vs Node)".into(),
            current: format!("node v{node_major} + npm v{npm_major}"),
            note: format!("npm v{expected}+ expected with Node v{node_major}"),
        })
    } else {
        None
    }
}

async fn check_python_pip() -> Option<AuditItem> {
    let python = match run_cmd("python3", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };
    let pip = match run_cmd("pip3", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };

    let py_major = major(&python);
    let pip_ver = pip.split_whitespace().nth(1).unwrap_or("0").to_string();
    let pip_major = major(&pip_ver);

    if let (Some(py), Some(pi)) = (py_major, pip_major) {
        if py >= 12 && pi < 24 {
            return Some(AuditItem {
                name: "pip (vs Python)".into(),
                current: format!("Python v{py} + pip v{pi}"),
                note: format!("pip v24+ recommended with Python v{py}"),
            });
        }
    }
    None
}

async fn check_brew_age() -> Option<AuditItem> {
    let out = match run_cmd("brew", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };
    let ver = out.split_whitespace().nth(1).unwrap_or("0").to_string();
    let m = match major(&ver) {
        Some(v) => v,
        None => return None,
    };
    if m < 4 {
        Some(AuditItem {
            name: "Homebrew".into(),
            current: format!("v{ver}"),
            note: "v4+ recommended (run `brew update`)".into(),
        })
    } else {
        None
    }
}

async fn check_cargo_vs_rustc() -> Option<AuditItem> {
    let rustc = match run_cmd("rustc", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };
    let cargo = match run_cmd("cargo", &["--version"], None).await {
        Ok(v) => v,
        _ => return None,
    };

    let rustc_ver = rustc.split_whitespace().nth(1).unwrap_or("0");
    let cargo_ver = cargo.split_whitespace().nth(1).unwrap_or("0");

    let rc_major = major(rustc_ver);
    let c_major = major(cargo_ver);

    if let (Some(rc), Some(c)) = (rc_major, c_major) {
        if (rc as i64 - c as i64).unsigned_abs() > 1 {
            return Some(AuditItem {
                name: "rustc vs Cargo".into(),
                current: format!("rustc v{rustc_ver}, cargo v{cargo_ver}"),
                note: "versions should track within 1 major".into(),
            });
        }
    }
    None
}

async fn check_bun_age() -> Option<AuditItem> {
    if let Ok(ver) = run_cmd("bun", &["--version"], None).await {
        let m = match major(&ver) {
            Some(v) => v,
            None => return None,
        };
        if m < 1 {
            return Some(AuditItem {
                name: "Bun".into(),
                current: format!("v{ver}"),
                note: "v1+ recommended".into(),
            });
        }
    }
    None
}

async fn check_env_managers() -> Vec<AuditItem> {
    let mut items = vec![];
    let project_dir = get_project_path();

    let nvmrc = project_dir.join(".nvmrc");
    let node_version_file = project_dir.join(".node-version");
    let tool_versions = project_dir.join(".tool-versions");
    let mise_toml = project_dir.join("mise.toml");
    let sdkmanrc = project_dir.join(".sdkmanrc");

    let mut expected_node = None;
    let mut expected_python = None;
    let mut expected_java = None;

    if let Ok(content) = std::fs::read_to_string(&nvmrc) {
        expected_node = Some(("nvmrc", content.trim().to_string()));
    } else if let Ok(content) = std::fs::read_to_string(&node_version_file) {
        expected_node = Some(("node-version", content.trim().to_string()));
    }

    if let Ok(content) = std::fs::read_to_string(&tool_versions) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if parts[0] == "nodejs" && expected_node.is_none() {
                    expected_node = Some(("tool-versions", parts[1].to_string()));
                } else if parts[0] == "python" {
                    expected_python = Some(("tool-versions", parts[1].to_string()));
                } else if parts[0] == "java" {
                    expected_java = Some(("tool-versions", parts[1].to_string()));
                }
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string(&mise_toml) {
        let mut in_tools = false;
        for line in content.lines() {
            let line = line.trim();
            if line == "[tools]" {
                in_tools = true;
                continue;
            } else if line.starts_with('[') {
                in_tools = false;
            }
            if in_tools {
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if k == "node" && expected_node.is_none() {
                        expected_node = Some(("mise.toml", v.to_string()));
                    } else if k == "python" && expected_python.is_none() {
                        expected_python = Some(("mise.toml", v.to_string()));
                    } else if k == "java" && expected_java.is_none() {
                        expected_java = Some(("mise.toml", v.to_string()));
                    }
                }
            }
        }
    }

    if let Some((source, expected)) = expected_node {
        if let Ok(node) = run_cmd("node", &["--version"], None).await {
            let current = node.trim().trim_start_matches('v');
            let expected_clean = expected.trim_start_matches('v');
            if !current.starts_with(expected_clean) {
                items.push(AuditItem {
                    name: "Node Environment".into(),
                    current: format!("node v{current}"),
                    note: format!("expected v{expected_clean} from .{source}"),
                });
            }
        }
    }

    let python_version_file = project_dir.join(".python-version");
    if let Ok(content) = std::fs::read_to_string(&python_version_file) {
        if expected_python.is_none() {
            expected_python = Some(("python-version", content.trim().to_string()));
        }
    }

    if let Some((source, expected)) = expected_python {
        if let Ok(python) = run_cmd("python3", &["--version"], None).await {
            let current = python.split_whitespace().nth(1).unwrap_or("0");
            if !current.starts_with(&expected) {
                items.push(AuditItem {
                    name: "Python Environment".into(),
                    current: format!("python v{current}"),
                    note: format!("expected v{expected} from .{source}"),
                });
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string(&sdkmanrc) {
        for line in content.lines() {
            if line.starts_with("java=") && expected_java.is_none() {
                expected_java = Some(("sdkmanrc", line.trim_start_matches("java=").trim().to_string()));
            }
        }
    }

    if let Some((source, expected)) = expected_java {
        if let Ok(java) = run_cmd("java", &["-version"], None).await {
            let current = java.lines().next().unwrap_or("").split('"').nth(1).unwrap_or("0");
            if !current.starts_with(&expected) {
                items.push(AuditItem {
                    name: "Java Environment".into(),
                    current: format!("java v{current}"),
                    note: format!("expected v{expected} from .{source}"),
                });
            }
        }
    }
    items
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("audit");

    let (node_res, python_res, brew_res, cargo_res, bun_res, env_res) = tokio::join!(
        check_node_npm(),
        check_python_pip(),
        check_brew_age(),
        check_cargo_vs_rustc(),
        check_bun_age(),
        check_env_managers()
    );

    if let Some(item) = node_res {
        result.audit_items.push(item);
    }
    if let Some(item) = python_res {
        result.audit_items.push(item);
    }
    if let Some(item) = brew_res {
        result.audit_items.push(item);
    }
    if let Some(item) = cargo_res {
        result.audit_items.push(item);
    }
    if let Some(item) = bun_res {
        result.audit_items.push(item);
    }
    for item in env_res {
        result.audit_items.push(item);
    }

    let n = result.audit_items.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} audit item(s)"));
    }

    result
}
