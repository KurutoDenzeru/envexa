use super::*;

fn parse_vuln(name: &str, info: &serde_json::Value, out: &mut Vec<VulnerabilityInfo>) {
    let severity = info["severity"].as_str().unwrap_or("unknown");
    let _range = info["range"].as_str().unwrap_or("?");
    let patched = info["fixAvailable"]
        .as_object()
        .and_then(|f| f["version"].as_str())
        .unwrap_or("?");
    let title = info["via"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v["title"].as_str())
        .unwrap_or("unknown");

    let cve = info["via"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v["cve"].as_str())
        .or_else(|| {
            info["via"].as_array().and_then(|arr| {
                arr.iter()
                    .find_map(|v| v.as_str().filter(|s| s.starts_with("CVE-")))
            })
        })
        .map(|s| s.to_string());

    out.push(VulnerabilityInfo {
        package: name.to_string(),
        severity: severity.to_uppercase(),
        title: title.to_string(),
        cve,
        patched_version: patched.to_string(),
    });
}

async fn npm_audit(result: &mut ScanResult) {
    if !which("npm") {
        return;
    }
    if let Ok(out) = run_cmd("npm", &["audit", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(vulns) = data["vulnerabilities"].as_object() {
                for (name, info) in vulns {
                    parse_vuln(name, info, &mut result.vulnerabilities);
                }
            }
        }
    }
}

async fn pnpm_audit(result: &mut ScanResult) {
    if !which("pnpm") {
        return;
    }
    if let Ok(out) = run_cmd("pnpm", &["audit", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(vulns) = data["vulnerabilities"].as_object() {
                for (name, info) in vulns {
                    parse_vuln(name, info, &mut result.vulnerabilities);
                }
            }
        }
    }
}

async fn bun_audit(result: &mut ScanResult) {
    if !which("bun") {
        return;
    }
    if let Ok(out) = run_cmd("bun", &["audit"]).await {
        for line in out.lines() {
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() >= 3 {
                let name = parts[0].trim();
                let severity = parts.get(1).unwrap_or(&"unknown").trim().to_uppercase();
                let remainder = parts[2..].join(" ");
                if !name.is_empty() && !severity.is_empty() {
                    result.vulnerabilities.push(VulnerabilityInfo {
                        package: name.to_string(),
                        severity,
                        title: remainder,
                        cve: None,
                        patched_version: String::new(),
                    });
                }
            }
        }
    }
}

async fn cargo_audit(result: &mut ScanResult) {
    if !which("cargo-audit") {
        return;
    }
    if let Ok(out) = run_cmd("cargo-audit", &["audit", "--json"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(list) = data["vulnerabilities"]["list"].as_array() {
                for item in list {
                    let pkg = &item["package"];
                    let advisory = &item["advisory"];
                    result.vulnerabilities.push(VulnerabilityInfo {
                        package: pkg["name"].as_str().unwrap_or("?").to_string(),
                        severity: advisory["severity"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_uppercase(),
                        title: advisory["title"].as_str().unwrap_or("?").to_string(),
                        cve: advisory["aliases"]
                            .as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        patched_version: advisory["patched_versions"]
                            .as_str()
                            .unwrap_or("?")
                            .to_string(),
                    });
                }
            }
        }
    }
}

async fn pip_audit(result: &mut ScanResult) {
    if !which("pip-audit") {
        return;
    }
    if let Ok(out) = run_cmd("pip-audit", &["--format", "json", "--desc", "--no-deps"]).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(deps) = data["dependencies"].as_array() {
                for dep in deps {
                    if let Some(vulns) = dep["vulns"].as_array() {
                        for v in vulns {
                            result.vulnerabilities.push(VulnerabilityInfo {
                                package: dep["name"].as_str().unwrap_or("?").to_string(),
                                severity: v["severity"]
                                    .as_str()
                                    .unwrap_or("unknown")
                                    .to_uppercase(),
                                title: v["description"]
                                    .as_str()
                                    .unwrap_or(v["id"].as_str().unwrap_or("?"))
                                    .to_string(),
                                cve: v["aliases"]
                                    .as_array()
                                    .and_then(|a| a.first())
                                    .and_then(|c| c.as_str())
                                    .map(|s| s.to_string()),
                                patched_version: v["fixed_version"]
                                    .as_str()
                                    .unwrap_or("?")
                                    .to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("security");

    npm_audit(&mut result).await;
    pnpm_audit(&mut result).await;
    bun_audit(&mut result).await;
    cargo_audit(&mut result).await;
    pip_audit(&mut result).await;

    let n = result.vulnerabilities.len();
    result.status = if n == 0 {
        "ok".into()
    } else if n <= 3 {
        "warning".into()
    } else {
        "error".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} vulnerability(ies) found"));
    }

    result
}
