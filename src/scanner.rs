use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::toolchains::{self, ScanResult, PackageInfo};

#[derive(Clone)]
pub struct ReportCache {
    inner: Arc<Mutex<Option<Report>>>,
}

#[derive(Clone, serde::Serialize)]
pub struct Report {
    pub timestamp: String,
    pub results: HashMap<String, ScanResult>,
}

impl ReportCache {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(None)) }
    }

    pub async fn set(&self, report: Report) {
        *self.inner.lock().await = Some(report);
    }

    pub async fn get(&self) -> Option<Report> {
        self.inner.lock().await.clone()
    }
}

fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

const ICONS: &[(&str, &str)] = &[
    ("brew", "🍺"),
    ("npm", ""),
    ("pnpm", ""),
    ("yarn", ""),
    ("bun", ""),
    ("deno", ""),
    ("pip", ""),
    ("gem", ""),
    ("cargo", "🦀"),
    ("docker", "🐳"),
];

const STATUS_EMOJI: &[(&str, &str)] = &[
    ("ok", "✅"),
    ("warning", "⚠️"),
    ("error", "❌"),
    ("skipped", "⏭️"),
];

const LABELS: &[(&str, &str)] = &[
    ("ok", "PASS"),
    ("warning", "WARN"),
    ("error", "FAIL"),
    ("skipped", "SKIP"),
];

const DISPLAY_NAMES: &[(&str, &str)] = &[
    ("brew", "Brew"),
    ("npm", "npm"),
    ("pnpm", "pnpm"),
    ("yarn", "Yarn"),
    ("bun", "Bun"),
    ("deno", "Deno"),
    ("pip", "pip"),
    ("gem", "Gem"),
    ("cargo", "Cargo"),
    ("docker", "Docker"),
];

fn tool_order() -> [&'static str; 10] {
    ["brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker"]
}

fn display_name(tool: &str) -> &str {
    DISPLAY_NAMES.iter().find(|(k, _)| *k == tool).map(|(_, v)| *v).unwrap_or(tool)
}

fn icon(tool: &str) -> &str {
    ICONS.iter().find(|(k, _)| *k == tool).map(|(_, v)| *v).unwrap_or("")
}

fn status_emoji(s: &str) -> &str {
    STATUS_EMOJI.iter().find(|(k, _)| *k == s).map(|(_, v)| *v).unwrap_or("")
}

fn status_label(s: &str) -> &str {
    LABELS.iter().find(|(k, _)| *k == s).map(|(_, v)| *v).unwrap_or("?")
}

fn extract_outdated(res: &ScanResult) -> Vec<&PackageInfo> {
    let mut items = vec![];
    for key in ["outdated_formulae", "outdated_casks", "outdated_global", "outdated"] {
        match key {
            "outdated_formulae" => items.extend(res.outdated_formulae.iter()),
            "outdated_casks" => items.extend(res.outdated_casks.iter()),
            "outdated_global" => items.extend(res.outdated_global.iter()),
            "outdated" => items.extend(res.outdated.iter()),
            _ => {}
        }
    }
    items
}

fn status_text(res: &ScanResult) -> String {
    let n = extract_outdated(res).len();
    if res.status == "warning" {
        format!("WARN ({n})")
    } else {
        status_label(&res.status).to_string()
    }
}

fn first_version(res: &ScanResult) -> String {
    let fields = ["version", "node_version", "python_version", "ruby_version",
                   "rustc_version", "cargo_version", "pnpm_version", "bun_version", "deno_version"];
    for f in fields {
        let val = match f {
            "version" => &res.version,
            "node_version" => &res.node_version,
            "python_version" => &res.python_version,
            "ruby_version" => &res.ruby_version,
            "rustc_version" => &res.rustc_version,
            "cargo_version" => &res.cargo_version,
            "pnpm_version" => &res.pnpm_version,
            "bun_version" => &res.bun_version,
            "deno_version" => &res.deno_version,
            _ => &None,
        };
        if let Some(v) = val {
            return v.clone();
        }
    }
    String::new()
}

pub fn format_report(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Envexa Health Report".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());

    let mut outdated_all: HashMap<&str, Vec<&PackageInfo>> = HashMap::new();
    let mut dashboard_rows = vec![];

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                outdated_all.insert(tool, items);
            }

            let icon = icon(tool);
            let status_emoji = status_emoji(&res.status);
            let status_txt = status_text(res);
            let ver = first_version(res);
            let display = display_name(tool);
            dashboard_rows.push(format!("| {icon} {display:7} | {status_emoji} {status_txt:<16} | {ver} |"));
        }
    }

    lines.push("## Dashboard".into());
    lines.push("| Toolchain | Status | Version |".into());
    lines.push("|-----------|--------|---------|".into());
    for row in &dashboard_rows {
        lines.push(row.clone());
    }
    lines.push(String::new());

    if !outdated_all.is_empty() {
        lines.push("## Outdated Packages".into());
        lines.push(String::new());
        lines.push("| Toolchain | Package | Current | Latest |".into());
        lines.push("|-----------|---------|---------|--------|".into());
        for tool in &tool_order() {
            if let Some(items) = outdated_all.get(tool) {
                let display = display_name(tool);
                let ic = icon(tool);
                for item in items {
                    lines.push(format!("| {ic} {display} | {} | {} | {} |", item.name, item.current, item.latest));
                }
            }
        }
        lines.push(String::new());
    }

    lines.push("## Per-Toolchain Details".into());
    lines.push(String::new());

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let ic = icon(tool);
            let label = status_label(&res.status);
            let display = display_name(tool);
            lines.push(format!("### {ic} [{label}] {display}"));

            if res.status == "skipped" {
                let reason = res.issues.first().map(|s| s.as_str()).unwrap_or("Skipped");
                lines.push(format!("> {reason}"));
                lines.push(String::new());
                continue;
            }

            let version_labels = [
                ("version", &res.version),
                ("node_version", &res.node_version),
                ("python_version", &res.python_version),
                ("ruby_version", &res.ruby_version),
                ("rustc_version", &res.rustc_version),
                ("cargo_version", &res.cargo_version),
                ("pnpm_version", &res.pnpm_version),
                ("bun_version", &res.bun_version),
                ("deno_version", &res.deno_version),
            ];
            let ver_parts: Vec<String> = version_labels.iter()
                .filter_map(|(key, val)| val.as_ref().map(|v| {
                    let label = match *key {
                        "version" => "Version",
                        "node_version" => "Node",
                        "python_version" => "Python",
                        "ruby_version" => "Ruby",
                        "rustc_version" => "Rust",
                        "cargo_version" => "Cargo",
                        "pnpm_version" => "pnpm",
                        "bun_version" => "Bun",
                        "deno_version" => "Deno",
                        _ => key,
                    };
                    format!("**{label}:** {v}")
                }))
                .collect();
            if !ver_parts.is_empty() {
                lines.push(ver_parts.join(" | "));
            }

            if let Some(count) = res.installed_count {
                lines.push(format!("**Formulae:** {count}"));
            }

            if let Some(ref disk) = res.disk_usage {
                if let Some(obj) = disk.as_object() {
                    for (typ, info) in obj {
                        let size = info.get("size").and_then(|v| v.as_str()).unwrap_or("?");
                        lines.push(format!("- **{typ}:** {size}"));
                    }
                }
            }

            let outdated_items = extract_outdated(res);
            if !outdated_items.is_empty() {
                lines.push(String::new());
                lines.push("| Package | Current | Latest |".into());
                lines.push("|---------|---------|--------|".into());
                for item in &outdated_items {
                    lines.push(format!("| {} | {} | {} |", item.name, item.current, item.latest));
                }
            }

            for issue in &res.issues {
                lines.push(format!("> {issue}"));
            }

            lines.push(String::new());
        }
    }

    lines.join("\n")
}

pub fn format_status(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Envexa Status".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());
    lines.push("| Toolchain | Status | Count |".into());
    lines.push("|-----------|--------|-------|".into());

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let ic = icon(tool);
            let emoji = status_emoji(&res.status);
            let label = status_label(&res.status);
            let n = extract_outdated(res).len();
            let count = if n > 0 { format!("({n})") } else { String::new() };
            let display = display_name(tool);
            lines.push(format!("| {ic} {display} | {emoji} {label} | {count} |"));
        }
    }

    lines.push(String::new());
    lines.push("Run `/envexa:scan` for full report or `/envexa:outdated` for details.".into());
    lines.join("\n")
}

pub fn format_outdated(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Outdated Packages".into());
    lines.push(String::new());
    lines.push("| Toolchain | Package | Current | Latest |".into());
    lines.push("|-----------|---------|---------|--------|".into());

    let mut has_anything = false;
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                has_anything = true;
                let display = display_name(tool);
                for item in &items {
                    lines.push(format!("| {display} | {} | {} | {} |", item.name, item.current, item.latest));
                }
            }
        }
    }

    if !has_anything {
        return "All packages are up to date!".into();
    }

    lines.join("\n")
}

pub async fn scan_and_cache(cache: &ReportCache, chain: &str) -> String {
    let results = if chain == "all" {
        toolchains::scan_all().await
    } else if let Some(res) = toolchains::scan_one(chain).await {
        let mut map = HashMap::new();
        map.insert(chain.to_string(), res);
        map
    } else {
        return format!("Unknown chain: {chain}. Options: all, brew, npm, pnpm, yarn, bun, deno, pip, gem, cargo, docker");
    };

    let report = Report {
        timestamp: now_iso(),
        results,
    };

    cache.set(report.clone()).await;
    format_report(&report)
}
