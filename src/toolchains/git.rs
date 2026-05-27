use super::*;
use std::path::Path;

async fn check_stale_branches(dir: &Path, result: &mut ScanResult) {
    if !which("git") {
        return;
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
                        result.issues.push(format!(
                            "Branch '{}' is stale ({} days old)",
                            name, days_old
                        ));
                    }
                }
            }
        }
    }
}

async fn check_uncommitted_changes(dir: &Path, result: &mut ScanResult) {
    if let Ok(out) = run_cmd_in(dir, "git", &["status", "--porcelain"], None).await {
        let changes = out.lines().count();
        if changes > 0 {
            result
                .issues
                .push(format!("{} uncommitted changes", changes));
        }
    }
}

async fn check_large_git_dir(dir: &Path, result: &mut ScanResult) {
    let git_dir = dir.join(".git");
    if git_dir.exists() {
        if let Ok(out) = run_cmd("du", &["-sk", git_dir.to_str().unwrap_or(".git")], None).await {
            if let Some(size_str) = out.split_whitespace().next() {
                if let Ok(size_kb) = size_str.parse::<u64>() {
                    let size_mb = size_kb / 1024;
                    if size_mb > 500 {
                        result.cleanup_items.push(CleanupItem {
                            category: "git".into(),
                            description: "Large .git directory".into(),
                            size: Some(format!("{} MB", size_mb)),
                            command: Some("git gc --prune=now".into()),
                        });
                    }
                }
            }
        }
    }
}

pub async fn scan() -> ScanResult {
    let mut result = ScanResult::new("git");
    let project_path = get_project_path();

    if !project_path.join(".git").exists() {
        return ScanResult::skipped("Not a git repository");
    }

    check_stale_branches(&project_path, &mut result).await;
    check_uncommitted_changes(&project_path, &mut result).await;
    check_large_git_dir(&project_path, &mut result).await;

    let n = result.issues.len() + result.cleanup_items.len();
    result.status = if n == 0 {
        "ok".into()
    } else {
        "warning".into()
    };
    result
}
