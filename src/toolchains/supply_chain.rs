use super::*;
use std::path::Path;

async fn check_npm_scripts(project_path: &Path) -> Vec<SupplyChainRisk> {
    let mut risks = Vec::new();
    let pkg_json_path = project_path.join("package.json");
    if !pkg_json_path.exists() {
        return risks;
    }

    if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = data.get("dependencies").and_then(|d| d.as_object()) {
                for (pkg, _) in deps {
                    let node_module_pkg = project_path.join("node_modules").join(pkg).join("package.json");
                    if node_module_pkg.exists() {
                        if let Ok(mod_content) = std::fs::read_to_string(&node_module_pkg) {
                            if let Ok(mod_data) = serde_json::from_str::<serde_json::Value>(&mod_content) {
                                if let Some(scripts) = mod_data.get("scripts").and_then(|s| s.as_object()) {
                                    if scripts.contains_key("postinstall") || scripts.contains_key("preinstall") || scripts.contains_key("install") {
                                        risks.push(SupplyChainRisk {
                                            package: pkg.to_string(),
                                            risk_type: "Install Script".to_string(),
                                            description: "Package executes scripts automatically on install".to_string(),
                                        });
                                    }
                                }
                                if mod_data.get("deprecated").is_some() {
                                    risks.push(SupplyChainRisk {
                                        package: pkg.to_string(),
                                        risk_type: "Deprecated".to_string(),
                                        description: "Package is marked as deprecated by its author".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    risks
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("supply_chain");
    let project_path = get_project_path();

    let npm_risks = check_npm_scripts(&project_path).await;
    
    result.supply_chain_risks.extend(npm_risks);

    let n = result.supply_chain_risks.len();
    result.status = if n == 0 {
        "ok".into()
    } else if n <= 3 {
        "warning".into()
    } else {
        "error".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} supply chain risk(s) detected"));
    }

    result
}
