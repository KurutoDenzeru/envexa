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

async fn check_node_npm(result: &mut ScanResult) {
    let node = match run_cmd("node", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };
    let npm = match run_cmd("npm", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };

    let node_major = match major(&node) {
        Some(m) => m,
        None => return,
    };
    let npm_major = match major(&npm) {
        Some(m) => m,
        None => return,
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
        result.audit_items.push(AuditItem {
            name: "npm (vs Node)".into(),
            current: format!("node v{node_major} + npm v{npm_major}"),
            note: format!("npm v{expected}+ expected with Node v{node_major}"),
        });
    }
}

async fn check_python_pip(result: &mut ScanResult) {
    let python = match run_cmd("python3", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };
    let pip = match run_cmd("pip3", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };

    let py_major = major(&python);
    let pip_ver = pip.split_whitespace().nth(1).unwrap_or("0").to_string();
    let pip_major = major(&pip_ver);

    if let (Some(py), Some(pi)) = (py_major, pip_major) {
        if py >= 12 && pi < 24 {
            result.audit_items.push(AuditItem {
                name: "pip (vs Python)".into(),
                current: format!("Python v{py} + pip v{pi}"),
                note: format!("pip v24+ recommended with Python v{py}"),
            });
        }
    }
}

async fn check_brew_age(result: &mut ScanResult) {
    let out = match run_cmd("brew", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };
    let ver = out.split_whitespace().nth(1).unwrap_or("0").to_string();
    let m = match major(&ver) {
        Some(v) => v,
        None => return,
    };
    if m < 4 {
        result.audit_items.push(AuditItem {
            name: "Homebrew".into(),
            current: format!("v{ver}"),
            note: "v4+ recommended (run `brew update`)".into(),
        });
    }
}

async fn check_cargo_vs_rustc(result: &mut ScanResult) {
    let rustc = match run_cmd("rustc", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };
    let cargo = match run_cmd("cargo", &["--version"]).await {
        Ok(v) => v,
        _ => return,
    };

    let rustc_ver = rustc.split_whitespace().nth(1).unwrap_or("0");
    let cargo_ver = cargo.split_whitespace().nth(1).unwrap_or("0");

    let rc_major = major(rustc_ver);
    let c_major = major(cargo_ver);

    if let (Some(rc), Some(c)) = (rc_major, c_major) {
        if (rc as i64 - c as i64).unsigned_abs() > 1 {
            result.audit_items.push(AuditItem {
                name: "rustc vs Cargo".into(),
                current: format!("rustc v{rustc_ver}, cargo v{cargo_ver}"),
                note: "versions should track within 1 major".into(),
            });
        }
    }
}

async fn check_bun_age(result: &mut ScanResult) {
    if let Ok(ver) = run_cmd("bun", &["--version"]).await {
        let m = match major(&ver) {
            Some(v) => v,
            None => return,
        };
        if m < 1 {
            result.audit_items.push(AuditItem {
                name: "Bun".into(),
                current: format!("v{ver}"),
                note: "v1+ recommended".into(),
            });
        }
    }
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("audit");

    check_node_npm(&mut result).await;
    check_python_pip(&mut result).await;
    check_brew_age(&mut result).await;
    check_cargo_vs_rustc(&mut result).await;
    check_bun_age(&mut result).await;

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
