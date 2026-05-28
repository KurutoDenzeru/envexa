use std::collections::HashMap;
use std::fs;

use regex::Regex;

use super::*;

lazy_static::lazy_static! {
    static ref USES_RE: Regex = Regex::new(r"uses:\s+([a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+)@([vV]?\d+(?:\.\d+)*)").unwrap();

    static ref LATEST_ACTIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("actions/checkout", "v4");
        m.insert("actions/setup-node", "v4");
        m.insert("actions/setup-python", "v5");
        m.insert("actions/setup-go", "v5");
        m.insert("actions/setup-java", "v4");
        m.insert("actions/cache", "v4");
        m.insert("actions/upload-artifact", "v4");
        m.insert("actions/download-artifact", "v4");
        m.insert("actions/github-script", "v7");
        m.insert("actions/stale", "v9");
        m.insert("docker/setup-buildx-action", "v3");
        m.insert("docker/login-action", "v3");
        m.insert("docker/build-push-action", "v5");
        m.insert("docker/setup-qemu-action", "v3");
        m.insert("codecov/codecov-action", "v4");
        m.insert("dtinth/setup-github-actions-caching", "v1");
        m
    };
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("ci");
    let project_path = get_project_path();
    let workflows_dir = project_path.join(".github").join("workflows");

    if !workflows_dir.exists() || !workflows_dir.is_dir() {
        return ScanResult::skipped("No .github/workflows directory found");
    }

    let mut outdated_map: HashMap<String, (String, String)> = HashMap::new();

    if let Ok(entries) = fs::read_dir(workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "yml" || ext == "yaml" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for cap in USES_RE.captures_iter(&content) {
                            if let (Some(action), Some(version)) = (cap.get(1), cap.get(2)) {
                                let action_str = action.as_str();
                                let version_str = version.as_str();

                                if let Some(&latest) = LATEST_ACTIONS.get(action_str) {
                                    // Strip leading 'v' for comparison
                                    let current_v = version_str.trim_start_matches('v');
                                    let latest_v = latest.trim_start_matches('v');

                                    // Very basic major version check
                                    if let (Ok(c), Ok(l)) =
                                        (current_v.parse::<u32>(), latest_v.parse::<u32>())
                                    {
                                        if c < l {
                                            outdated_map.insert(
                                                action_str.to_string(),
                                                (version_str.to_string(), latest.to_string()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (name, (current, latest)) in outdated_map {
        result.outdated.push(PackageInfo {
            name,
            current,
            latest,
        });
    }

    let n = result.outdated.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };

    if n > 0 {
        result
            .issues
            .push(format!("{n} outdated GitHub Action(s) found"));
    }

    result
}
