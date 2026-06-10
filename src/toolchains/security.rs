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

    let mut dependency_path = Vec::new();
    if let Some(nodes) = info["nodes"].as_array() {
        if let Some(first_node) = nodes.first().and_then(|n| n.as_str()) {
            let path_parts: Vec<&str> = first_node.split("node_modules/").filter(|s| !s.is_empty()).collect();
            for part in path_parts {
                let clean = part.trim_end_matches('/');
                if !clean.is_empty() {
                    dependency_path.push(clean.to_string());
                }
            }
        }
    }

    out.push(VulnerabilityInfo {
        package: name.to_string(),
        severity: severity.to_uppercase(),
        title: title.to_string(),
        cve,
        patched_version: patched.to_string(),
        dependency_path,
    });
}

async fn npm_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("package-lock.json").exists()
        && !project_path.join("package.json").exists()
    {
        return vulns;
    }
    if !which("npm") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(project_path, "npm", &["audit", "--json"], None).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(vulnerabilities) = data["vulnerabilities"].as_object() {
                for (name, info) in vulnerabilities {
                    parse_vuln(name, info, &mut vulns);
                }
            }
        }
    }
    vulns
}

async fn pnpm_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("pnpm-lock.yaml").exists() && !project_path.join("pnpm-lock.yml").exists()
    {
        return vulns;
    }
    if !which("pnpm") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(project_path, "pnpm", &["audit", "--json"], None).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(vulnerabilities) = data["vulnerabilities"].as_object() {
                for (name, info) in vulnerabilities {
                    parse_vuln(name, info, &mut vulns);
                }
            }
        }
    }
    vulns
}

async fn bun_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("bun.lockb").exists() && !project_path.join("bun.lock").exists() {
        return vulns;
    }
    if !which("bun") {
        return vulns;
    }

    if let Ok(out) = run_cmd_in(project_path, "bun", &["audit", "--json"], None).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            // New Bun format: root object is a map of package names to an array of vulnerability objects
            if let Some(obj) = data.as_object() {
                for (pkg_name, vulns_val) in obj {
                    if let Some(vuln_arr) = vulns_val.as_array() {
                        for v in vuln_arr {
                            vulns.push(VulnerabilityInfo {
                                package: pkg_name.clone(),
                                severity: v["severity"].as_str().unwrap_or("unknown").to_uppercase(),
                                title: v["title"].as_str().unwrap_or("?").to_string(),
                                cve: v["cve"].as_array().and_then(|a| a.first()).and_then(|c| c.as_str()).map(|s| s.to_string()),
                                patched_version: String::new(), // bun output often doesn't give a simple patched version
                                dependency_path: Vec::new(),
                            });
                        }
                    }
                }
                return vulns;
            }
        }
    }

    if let Ok(out) = run_cmd_in(project_path, "bun", &["audit"], None).await {
        let mut in_section = false;
        for line in out.lines() {
            if !in_section {
                if line.contains("Package") && line.contains("Severity") {
                    in_section = true;
                }
                continue;
            }
            if line.contains("└") || line.is_empty() {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();
            if parts.len() >= 3 {
                let name = parts[0].trim();
                let severity = parts.get(1).unwrap_or(&"unknown").trim().to_uppercase();
                let remainder = parts[2..].join(" ");
                if !name.is_empty() {
                    vulns.push(VulnerabilityInfo {
                        package: name.to_string(),
                        severity,
                        title: remainder,
                        cve: None,
                        patched_version: String::new(),
                        dependency_path: Vec::new(),
                    });
                }
            }
        }
    }
    vulns
}

async fn cargo_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("Cargo.toml").exists() && !project_path.join("Cargo.lock").exists() {
        return vulns;
    }
    if !which("cargo-audit") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(project_path, "cargo-audit", &["audit", "--json"], None).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(list) = data["vulnerabilities"]["list"].as_array() {
                for item in list {
                    let pkg = &item["package"];
                    let advisory = &item["advisory"];
                    vulns.push(VulnerabilityInfo {
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
                        dependency_path: Vec::new(),
                    });
                }
            }
        }
    }
    vulns
}

async fn pip_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("requirements.txt").exists()
        && !project_path.join("Pipfile").exists()
        && !project_path.join("Pipfile.lock").exists()
        && !project_path.join("poetry.lock").exists()
    {
        return vulns;
    }
    if !which("pip-audit") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(
        project_path,
        "pip-audit",
        &["--format", "json", "--desc", "--no-deps"],
        None,
    )
    .await
    {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(deps) = data["dependencies"].as_array() {
                for dep in deps {
                    if let Some(v_arr) = dep["vulns"].as_array() {
                        for v in v_arr {
                            vulns.push(VulnerabilityInfo {
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
                                dependency_path: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    vulns
}

async fn go_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("go.mod").exists() {
        return vulns;
    }
    if !which("govulncheck") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(project_path, "govulncheck", &["-json", "./..."], None).await {
        for line in out.lines() {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(vuln) = data.get("osv") {
                    let id = vuln["id"].as_str().unwrap_or("?");
                    let details = vuln["details"].as_str().unwrap_or("?");
                    let aliases = vuln["aliases"].as_array();
                    let cve = aliases
                        .and_then(|a| {
                            a.iter()
                                .find_map(|v| v.as_str().filter(|s| s.starts_with("CVE-")))
                        })
                        .map(|s| s.to_string());
                    vulns.push(VulnerabilityInfo {
                        package: id.to_string(),
                        severity: "HIGH".to_string(),
                        title: details.to_string(),
                        cve,
                        patched_version: "?".to_string(),
                        dependency_path: Vec::new(),
                    });
                }
            }
        }
    }
    vulns
}

async fn composer_audit(project_path: &std::path::Path) -> Vec<VulnerabilityInfo> {
    let mut vulns = Vec::new();
    if !project_path.join("composer.json").exists() {
        return vulns;
    }
    if !which("composer") {
        return vulns;
    }
    if let Ok(out) = run_cmd_in(project_path, "composer", &["audit", "--format=json"], None).await {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out) {
            if let Some(advisories) = data.get("advisories").and_then(|a| a.as_object()) {
                for (pkg, list) in advisories {
                    if let Some(arr) = list.as_array() {
                        for adv in arr {
                            vulns.push(VulnerabilityInfo {
                                package: pkg.to_string(),
                                severity: "HIGH".to_string(),
                                title: adv["title"].as_str().unwrap_or("?").to_string(),
                                cve: adv["cve"].as_str().map(|s| s.to_string()),
                                patched_version: "?".to_string(),
                                dependency_path: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    vulns
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("security");
    let project_path = get_project_path();

    let (npm_res, pnpm_res, bun_res, cargo_res, pip_res, go_res, php_res) = tokio::join!(
        npm_audit(&project_path),
        pnpm_audit(&project_path),
        bun_audit(&project_path),
        cargo_audit(&project_path),
        pip_audit(&project_path),
        go_audit(&project_path),
        composer_audit(&project_path)
    );

    result.vulnerabilities.extend(npm_res);
    result.vulnerabilities.extend(pnpm_res);
    result.vulnerabilities.extend(bun_res);
    result.vulnerabilities.extend(cargo_res);
    result.vulnerabilities.extend(pip_res);
    result.vulnerabilities.extend(go_res);
    result.vulnerabilities.extend(php_res);

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
