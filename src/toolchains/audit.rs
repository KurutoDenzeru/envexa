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
    let node = match run_cmd("node", &["--version"]).await {
        Ok(v) => v,
        _ => return None,
    };
    let npm = match run_cmd("npm", &["--version"]).await {
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
    let python = match run_cmd("python3", &["--version"]).await {
        Ok(v) => v,
        _ => return None,
    };
    let pip = match run_cmd("pip3", &["--version"]).await {
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
    let out = match run_cmd("brew", &["--version"]).await {
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
    let rustc = match run_cmd("rustc", &["--version"]).await {
        Ok(v) => v,
        _ => return None,
    };
    let cargo = match run_cmd("cargo", &["--version"]).await {
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
    if let Ok(ver) = run_cmd("bun", &["--version"]).await {
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

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("audit");

    let (node_res, python_res, brew_res, cargo_res, bun_res) = tokio::join!(
        check_node_npm(),
        check_python_pip(),
        check_brew_age(),
        check_cargo_vs_rustc(),
        check_bun_age()
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
