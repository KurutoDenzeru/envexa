use std::collections::HashMap;

use crate::toolchains::{AuditItem, CleanupItem, ScanResult, VulnerabilityInfo};

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_ansi = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_ansi = true;
        } else if in_ansi {
            if c == 'm' {
                in_ansi = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn new() -> Self {
        Table {
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    fn header(&mut self, cols: &[&str]) {
        self.headers = cols.iter().map(|c| c.to_string()).collect();
    }

    fn add_row(&mut self, cols: &[&str]) {
        self.rows.push(cols.iter().map(|c| c.to_string()).collect());
    }

    fn render(&self) -> String {
        let ncols = self.headers.len();
        if ncols == 0 {
            return String::new();
        }

        let mut widths = vec![0usize; ncols];
        for (i, h) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(visible_len(h));
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < ncols {
                    widths[i] = widths[i].max(visible_len(cell));
                }
            }
        }

        let top_sep_parts: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        let top_border = format!("╭{}╮", top_sep_parts.join("┬"));

        let mid_sep_parts: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        let mid_border = format!("├{}┤", mid_sep_parts.join("┼"));

        let bot_sep_parts: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        let bot_border = format!("╰{}╯", bot_sep_parts.join("┴"));

        let fmt = |cells: &[String]| -> String {
            let mut s = String::from("│");
            for (i, cell) in cells.iter().enumerate() {
                if let Some(&w) = widths.get(i) {
                    let vlen = visible_len(cell);
                    let padding = w.saturating_sub(vlen);
                    s.push_str(&format!(" {}{} │", cell, " ".repeat(padding)));
                } else {
                    s.push_str(&format!(" {} │", cell));
                }
            }
            s
        };

        let mut out = String::new();
        out.push_str(&top_border);
        out.push('\n');
        out.push_str(&fmt(&self.headers));
        out.push('\n');
        out.push_str(&mid_border);
        out.push('\n');
        for row in &self.rows {
            out.push_str(&fmt(row));
            out.push('\n');
        }
        out.push_str(&bot_border);
        out
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub timestamp: String,
    pub results: HashMap<String, ScanResult>,
}

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
    ("project", "Project"),
    ("security", "Security"),
    ("audit", "Audit"),
    ("git", "Git"),
    ("cleanup", "Cleanup"),
];

#[derive(Debug, Clone)]
pub struct OutdatedItem {
    pub source: String,
    pub name: String,
    pub current: String,
    pub latest: String,
}

pub struct ToolCategory {
    pub name: &'static str,
    pub tools: &'static [&'static str],
}

pub fn tool_categories() -> [ToolCategory; 3] {
    [
        ToolCategory {
            name: "System & Runtime",
            tools: &["brew", "cargo", "docker", "pip", "gem"],
        },
        ToolCategory {
            name: "Web Development",
            tools: &["npm", "pnpm", "yarn", "bun", "deno"],
        },
        ToolCategory {
            name: "Project Tooling",
            tools: &["project", "security", "audit", "git", "cleanup"],
        },
    ]
}

pub fn tool_order() -> [&'static str; 15] {
    [
        "brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker", "project",
        "security", "audit", "git", "cleanup",
    ]
}

pub fn display_name(tool: &str) -> &str {
    DISPLAY_NAMES
        .iter()
        .find(|(k, _)| *k == tool)
        .map(|(_, v)| *v)
        .unwrap_or(tool)
}

pub fn status_label(s: &str) -> &str {
    LABELS
        .iter()
        .find(|(k, _)| *k == s)
        .map(|(_, v)| *v)
        .unwrap_or("?")
}

pub fn extract_outdated(res: &ScanResult) -> Vec<OutdatedItem> {
    let mut items = vec![];
    for f in &res.outdated_formulae {
        items.push(OutdatedItem {
            source: "formula".into(),
            name: f.name.clone(),
            current: f.current.clone(),
            latest: f.latest.clone(),
        });
    }
    for c in &res.outdated_casks {
        items.push(OutdatedItem {
            source: "cask".into(),
            name: c.name.clone(),
            current: c.current.clone(),
            latest: c.latest.clone(),
        });
    }
    for p in &res.outdated {
        items.push(OutdatedItem {
            source: "package".into(),
            name: p.name.clone(),
            current: p.current.clone(),
            latest: p.latest.clone(),
        });
    }
    for g in &res.outdated_global {
        items.push(OutdatedItem {
            source: "global".into(),
            name: g.name.clone(),
            current: g.current.clone(),
            latest: g.latest.clone(),
        });
    }
    items
}

pub fn extract_vulnerabilities(res: &ScanResult) -> &[VulnerabilityInfo] {
    &res.vulnerabilities
}

pub fn extract_audit_items(res: &ScanResult) -> &[AuditItem] {
    &res.audit_items
}

pub fn extract_cleanup_items(res: &ScanResult) -> &[CleanupItem] {
    &res.cleanup_items
}

pub fn first_version(res: &ScanResult) -> String {
    if let Some(ref pt) = res.project_type {
        return pt.clone();
    }
    let fields = [
        "version",
        "node_version",
        "python_version",
        "ruby_version",
        "rustc_version",
        "cargo_version",
        "pnpm_version",
        "bun_version",
        "deno_version",
    ];
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

pub fn cli_status_label(s: &str) -> String {
    let raw = status_label(s);
    match s {
        "ok" => format!("\x1b[32m{}\x1b[0m", raw),
        "warning" => format!("\x1b[33m{}\x1b[0m", raw),
        "error" => format!("\x1b[31m{}\x1b[0m", raw),
        "skipped" => format!("\x1b[90m{}\x1b[0m", raw),
        _ => raw.to_string(),
    }
}

pub fn cli_severity(sev: &str) -> String {
    let upper = sev.to_uppercase();
    if upper.contains("CRITICAL") {
        format!("\x1b[1;31m{}\x1b[0m", sev)
    } else if upper.contains("HIGH") {
        format!("\x1b[31m{}\x1b[0m", sev)
    } else if upper.contains("MEDIUM") || upper.contains("MODERATE") {
        format!("\x1b[33m{}\x1b[0m", sev)
    } else if upper.contains("LOW") {
        format!("\x1b[34m{}\x1b[0m", sev)
    } else {
        sev.to_string()
    }
}

pub fn cli_green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

pub fn cli_yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

pub fn format_report(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Envexa Health Report".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());

    let mut outdated_all: HashMap<&str, Vec<OutdatedItem>> = HashMap::new();
    let mut vuln_all: HashMap<&str, Vec<VulnerabilityInfo>> = HashMap::new();
    let mut audit_all: HashMap<&str, Vec<AuditItem>> = HashMap::new();
    let mut cleanup_all: HashMap<&str, Vec<CleanupItem>> = HashMap::new();

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                outdated_all.insert(tool, items);
            }
            if !res.vulnerabilities.is_empty() {
                vuln_all.insert(tool, res.vulnerabilities.clone());
            }
            if !res.audit_items.is_empty() {
                audit_all.insert(tool, res.audit_items.clone());
            }
            if !res.cleanup_items.is_empty() {
                cleanup_all.insert(tool, res.cleanup_items.clone());
            }
        }
    }

    lines.push("## Dashboard".into());
    let mut dt = Table::new();
    dt.header(&["Toolchain", "Status", "Version"]);
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let display = display_name(tool);
            let label = cli_status_label(&res.status);
            let ver = first_version(res);
            dt.add_row(&[display, &label, &ver]);
        }
    }
    lines.push(dt.render());
    lines.push(String::new());

    if !outdated_all.is_empty() {
        lines.push("## Outdated Packages".into());
        let mut ot = Table::new();
        ot.header(&["Toolchain", "Package", "Current", "Latest"]);
        for tool in &tool_order() {
            if let Some(items) = outdated_all.get(tool) {
                let display = display_name(tool);
                for item in items {
                    let cur = cli_yellow(&item.current);
                    let lat = cli_green(&item.latest);
                    ot.add_row(&[display, &item.name, &cur, &lat]);
                }
            }
        }
        lines.push(ot.render());
        lines.push(String::new());
    }

    if !vuln_all.is_empty() {
        lines.push("## Vulnerabilities".into());
        let mut vt = Table::new();
        vt.header(&["Toolchain", "Package", "Severity", "Patched"]);
        for tool in &tool_order() {
            if let Some(items) = vuln_all.get(tool) {
                let display = display_name(tool);
                for v in items {
                    let sev = cli_severity(&v.severity);
                    let patched = cli_green(&v.patched_version);
                    vt.add_row(&[display, &v.package, &sev, &patched]);
                }
            }
        }
        lines.push(vt.render());
        lines.push(String::new());
    }

    if !audit_all.is_empty() {
        lines.push("## Audit".into());
        let mut at = Table::new();
        at.header(&["Toolchain", "Name", "Current", "Note"]);
        for tool in &tool_order() {
            if let Some(items) = audit_all.get(tool) {
                let display = display_name(tool);
                for a in items {
                    let cur = cli_yellow(&a.current);
                    at.add_row(&[display, &a.name, &cur, &a.note]);
                }
            }
        }
        lines.push(at.render());
        lines.push(String::new());
    }

    if !cleanup_all.is_empty() {
        lines.push("## Cleanup".into());
        for tool in &tool_order() {
            if let Some(items) = cleanup_all.get(tool) {
                let display = display_name(tool);
                for c in items {
                    lines.push(format!(
                        "- **[{display}]** {} — {}",
                        c.description,
                        c.size.as_deref().unwrap_or("?")
                    ));
                    if let Some(ref cmd) = c.command {
                        lines.push(format!("  `{cmd}`"));
                    }
                }
            }
        }
        lines.push(String::new());
    }

    lines.push("## Per-Toolchain Details".into());
    lines.push(String::new());

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let label = cli_status_label(&res.status);
            let display = display_name(tool);
            lines.push(format!("### [{label}] {display}"));

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
            let ver_parts: Vec<String> = version_labels
                .iter()
                .filter_map(|(key, val)| {
                    val.as_ref().map(|v| {
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
                    })
                })
                .collect();
            if !ver_parts.is_empty() {
                lines.push(ver_parts.join(" | "));
            }

            if let Some(count) = res.installed_count {
                lines.push(format!("**Formulae:** {count}"));
            }

            if let Some(ref pt) = res.project_type {
                lines.push(format!("**Project type:** {pt}"));
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
                let mut pt = Table::new();
                pt.header(&["Package", "Current", "Latest"]);
                for item in &outdated_items {
                    let cur = cli_yellow(&item.current);
                    let lat = cli_green(&item.latest);
                    pt.add_row(&[&item.name, &cur, &lat]);
                }
                lines.push(pt.render());
            }

            if !res.vulnerabilities.is_empty() {
                lines.push("Vulnerabilities:".into());
                let mut vt = Table::new();
                vt.header(&["Package", "Severity", "Patched"]);
                for v in &res.vulnerabilities {
                    let sev = cli_severity(&v.severity);
                    let patched = cli_green(&v.patched_version);
                    vt.add_row(&[&v.package, &sev, &patched]);
                }
                lines.push(vt.render());
            }

            if !res.audit_items.is_empty() {
                lines.push("Audit items:".into());
                for a in &res.audit_items {
                    lines.push(format!("- **{}:** {} ({})", a.name, a.note, a.current));
                }
            }

            if !res.cleanup_items.is_empty() {
                lines.push("Cleanup:".into());
                for c in &res.cleanup_items {
                    lines.push(format!(
                        "- {} — {}",
                        c.description,
                        c.size.as_deref().unwrap_or("?")
                    ));
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

#[allow(dead_code)]
pub fn format_status(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Envexa Status".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());

    let mut t = Table::new();
    t.header(&["Toolchain", "Status", "Count"]);
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let label = cli_status_label(&res.status);
            let n = extract_outdated(res).len();
            let count = if n > 0 {
                cli_yellow(&n.to_string())
            } else {
                "-".into()
            };
            let display = display_name(tool);
            t.add_row(&[display, &label, &count]);
        }
    }
    lines.push(t.render());

    lines.push(String::new());
    lines.push("Run `/envexa:scan` for full report or `/envexa:outdated` for details.".into());
    lines.join("\n")
}

pub fn count_outdated(report: &Report) -> usize {
    report
        .results
        .values()
        .map(|res| extract_outdated(res).len())
        .sum()
}

#[allow(dead_code)]
pub fn format_outdated(report: &Report) -> String {
    let results = &report.results;

    let mut t = Table::new();
    t.header(&["Toolchain", "Package", "Current", "Latest"]);
    let mut has_anything = false;
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                has_anything = true;
                let display = display_name(tool);
                for item in &items {
                    let cur = cli_yellow(&item.current);
                    let lat = cli_green(&item.latest);
                    t.add_row(&[display, &item.name, &cur, &lat]);
                }
            }
        }
    }

    if !has_anything {
        return "All packages are up to date!".into();
    }

    format!("# Outdated Packages\n\n{}", t.render())
}
