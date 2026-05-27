use super::*;
use regex::Regex;

fn gem_outdated_re() -> Regex {
    Regex::new(r"^(\S+)\s+\((\S+)\s*(?:[<>]?\s*)?(\S*)\)").unwrap()
}

pub async fn scan() -> ScanResult {
    if !which("ruby") {
        return ScanResult::skipped("Ruby not installed");
    }

    let mut result = ScanResult::new("gem");

    let has_gem = which("gem");

    if has_gem {
        let (ruby_ver, outdated) = tokio::join!(
            run_cmd("ruby", &["--version"], None),
            run_cmd("gem", &["outdated"], None)
        );

        if let Ok(ver) = ruby_ver {
            result.ruby_version = Some(ver);
        }

        if let Ok(out) = outdated {
            let re = gem_outdated_re();
            for line in out.lines() {
                if let Some(cap) = re.captures(line) {
                    result.outdated.push(PackageInfo {
                        name: cap[1].to_string(),
                        current: cap[2].to_string(),
                        latest: if cap.get(3).is_none_or(|m| m.as_str().is_empty()) {
                            "?".to_string()
                        } else {
                            cap[3].to_string()
                        },
                    });
                }
            }
        }
    } else {
        if let Ok(ver) = run_cmd("ruby", &["--version"], None).await {
            result.ruby_version = Some(ver);
        }
        result.status = "warning".into();
        result.issues.push("gem CLI not found".into());
        return result;
    }

    let n = result.outdated.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    if n > 0 {
        result.issues.push(format!("{n} outdated gem(s)"));
    }

    result
}
