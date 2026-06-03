use super::*;
use std::path::Path;

async fn check_stale_branches(dir: &Path) -> Vec<String> {
    let mut issues = vec![];
    if !which("git") {
        return issues;
    }
    if let Ok(out) = run_cmd_in(
        dir,
        "git",
        &[
            "for-each-ref",
            "--sort=-committerdate",
            "refs/heads/",
            "--format=%(refname:short)|%(committerdate:unix)",
        ],
        None,
    )
    .await
    {
        let now = chrono::Local::now().timestamp();
        for line in out.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let name = parts[0];
                if let Ok(ts) = parts[1].parse::<i64>() {
                    let days_old = (now - ts) / (24 * 3600);
                    if days_old > 30 {
                        issues.push(format!(
                            "Branch '{}' is stale ({} days old)",
                            name, days_old
                        ));
                    }
                }
            }
        }
    }
    issues
}

async fn check_uncommitted_changes(dir: &Path) -> Vec<String> {
    let mut issues = vec![];
    if !which("git") {
        return issues;
    }
    if let Ok(out) = run_cmd_in(dir, "git", &["status", "--porcelain"], None).await {
        let changes = out.lines().count();
        if changes > 0 {
            issues.push(format!("{} uncommitted changes", changes));
        }
    }
    issues
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("git");
    let project_path = get_project_path();

    if !project_path.join(".git").exists() {
        return ScanResult::skipped("Not a git repository");
    }

    let (stale_res, uncommitted_res) = tokio::join!(
        check_stale_branches(&project_path),
        check_uncommitted_changes(&project_path),
    );

    result.issues.extend(stale_res);
    result.issues.extend(uncommitted_res);

    let n = result.issues.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    result
}
