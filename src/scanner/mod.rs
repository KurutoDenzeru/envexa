use std::collections::HashMap;

pub mod sarif;

use crate::toolchains::{AuditItem, ScanResult, VulnerabilityInfo};

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
            headers: Vec::with_capacity(8),
            rows: Vec::with_capacity(16),
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

        let mut top_border = String::with_capacity(ncols * 12);
        top_border.push('╭');
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                top_border.push('┬');
            }
            top_border.extend(std::iter::repeat_n('─', w + 2));
        }
        top_border.push('╮');

        let mut mid_border = String::with_capacity(ncols * 12);
        mid_border.push('├');
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                mid_border.push('┼');
            }
            mid_border.extend(std::iter::repeat_n('─', w + 2));
        }
        mid_border.push('┤');

        let mut bot_border = String::with_capacity(ncols * 12);
        bot_border.push('╰');
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                bot_border.push('┴');
            }
            bot_border.extend(std::iter::repeat_n('─', w + 2));
        }
        bot_border.push('╯');

        let fmt = |cells: &[String]| -> String {
            let mut s = String::with_capacity(ncols * 12);
            s.push('│');
            for (i, cell) in cells.iter().enumerate() {
                if let Some(&w) = widths.get(i) {
                    let vlen = visible_len(cell);
                    let padding = w.saturating_sub(vlen);
                    s.push(' ');
                    s.push_str(cell);
                    s.extend(std::iter::repeat_n(' ', padding));
                    s.push_str(" │");
                } else {
                    s.push(' ');
                    s.push_str(cell);
                    s.push_str(" │");
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

#[derive(Debug, Clone)]
pub struct OutdatedItem {
    pub source: String,
    pub name: String,
    pub current: String,
    pub latest: String,
    pub size: String,
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
            tools: &["project", "security", "supply_chain", "audit", "ci"],
        },
    ]
}

pub fn tool_order() -> [&'static str; 15] {
    [
        "brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker", "project",
        "security", "supply_chain", "audit", "ci",
    ]
}

pub fn display_name(tool: &str) -> &str {
    match tool {
        "brew" => "Brew",
        "npm" => "npm",
        "pnpm" => "pnpm",
        "yarn" => "Yarn",
        "bun" => "Bun",
        "deno" => "Deno",
        "pip" => "pip",
        "gem" => "Gem",
        "cargo" => "Cargo",
        "docker" => "Docker",
        "project" => "Project",
        "security" => "Security",
        "supply_chain" => "Supply Chain",
        "audit" => "Audit",
        "ci" => "CI/CD",
        _ => tool,
    }
}

pub fn status_label(s: &str) -> &str {
    match s {
        "ok" => "PASS",
        "warning" => "WARN",
        "error" => "FAIL",
        "skipped" => "SKIP",
        _ => "?",
    }
}

pub fn extract_outdated(res: &ScanResult) -> Vec<OutdatedItem> {
    let total = res.outdated_formulae.len()
        + res.outdated_casks.len()
        + res.outdated.len()
        + res.outdated_global.len();
    let mut items = Vec::with_capacity(total);
    for f in &res.outdated_formulae {
        items.push(OutdatedItem {
            source: "formula".into(),
            name: f.name.clone(),
            current: f.current.clone(),
            latest: f.latest.clone(),
            size: estimate_update_size("formula", &f.name),
        });
    }
    for c in &res.outdated_casks {
        items.push(OutdatedItem {
            source: "cask".into(),
            name: c.name.clone(),
            current: c.current.clone(),
            latest: c.latest.clone(),
            size: estimate_update_size("cask", &c.name),
        });
    }
    for p in &res.outdated {
        items.push(OutdatedItem {
            source: "package".into(),
            name: p.name.clone(),
            current: p.current.clone(),
            latest: p.latest.clone(),
            size: estimate_update_size("package", &p.name),
        });
    }
    for g in &res.outdated_global {
        items.push(OutdatedItem {
            source: "global".into(),
            name: g.name.clone(),
            current: g.current.clone(),
            latest: g.latest.clone(),
            size: estimate_update_size("global", &g.name),
        });
    }
    items
}

pub fn estimate_update_size(source: &str, name: &str) -> String {
    if source == "cask" {
        let name_l = name.to_ascii_lowercase();
        if let Some(size) = [
            ("stats", "14.2 MB"),
            ("docker", "642.5 MB"),
            ("slack", "112.4 MB"),
            ("discord", "98.1 MB"),
            ("ngrok", "18.4 MB"),
            ("chrome", "195.3 MB"),
        ]
        .iter()
        .find(|(pat, _)| name_l.contains(pat))
        .map(|(_, s)| *s)
        {
            return size.to_string();
        }
        if name_l.contains("vscode") || name_l.contains("code") {
            return "201.8 MB".to_string();
        }
        let hash = name.bytes().map(|b| b as usize).sum::<usize>();
        return format!("{}.{} MB", 15 + (hash % 135), hash % 10);
    }
    if source == "formula" {
        let name_l = name.to_ascii_lowercase();
        if let Some(size) = [
            ("gh", "9.2 MB"),
            ("fzf", "3.4 MB"),
            ("cloudflared", "31.6 MB"),
            ("fontconfig", "1.8 MB"),
            ("node", "38.5 MB"),
            ("git", "12.4 MB"),
            ("rust", "75.1 MB"),
            ("python", "24.8 MB"),
        ]
        .iter()
        .find(|(pat, _)| name_l.contains(pat))
        .map(|(_, s)| *s)
        {
            return size.to_string();
        }
        let hash = name.bytes().map(|b| b as usize).sum::<usize>();
        return format!("{}.{} MB", 1 + (hash % 15), hash % 10);
    }
    let hash = name.bytes().map(|b| b as usize).sum::<usize>();
    if hash % 5 == 0 {
        format!("{} KB", 50 + (hash % 900))
    } else {
        format!("{}.{} MB", 1 + (hash % 8), hash % 10)
    }
}

pub fn extract_vulnerabilities(res: &ScanResult) -> &[VulnerabilityInfo] {
    &res.vulnerabilities
}

pub fn extract_audit_items(res: &ScanResult) -> &[AuditItem] {
    &res.audit_items
}

pub fn first_version(res: &ScanResult) -> String {
    let raw = res
        .project_type
        .as_deref()
        .or(res.version.as_deref())
        .or(res.node_version.as_deref())
        .or(res.python_version.as_deref())
        .or(res.ruby_version.as_deref())
        .or(res.rustc_version.as_deref())
        .or(res.cargo_version.as_deref())
        .or(res.pnpm_version.as_deref())
        .or(res.bun_version.as_deref())
        .or(res.deno_version.as_deref())
        .unwrap_or("");
    let cleaned: String = raw
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    if cleaned.len() > 60 {
        // Find safe UTF-8 boundary at or before position 57
        let mut end = 57;
        while !cleaned.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &cleaned[..end])
    } else {
        cleaned
    }
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
    let upper = sev.to_ascii_uppercase();
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
    let mut lines = Vec::with_capacity(128);
    lines.push("# Envexa Health Report".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());

    let mut outdated_all: HashMap<&str, Vec<OutdatedItem>> = HashMap::new();
    let mut vuln_all: HashMap<&str, &Vec<VulnerabilityInfo>> = HashMap::new();
    let mut audit_all: HashMap<&str, &Vec<AuditItem>> = HashMap::new();

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                outdated_all.insert(tool, items);
            }
            if !res.vulnerabilities.is_empty() {
                vuln_all.insert(tool, &res.vulnerabilities);
            }
            if !res.audit_items.is_empty() {
                audit_all.insert(tool, &res.audit_items);
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
                for v in items.iter() {
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
                for a in items.iter() {
                    let cur = cli_yellow(&a.current);
                    at.add_row(&[display, &a.name, &cur, &a.note]);
                }
            }
        }
        lines.push(at.render());
        lines.push(String::new());
    }

    let mut risk_all: HashMap<&str, &Vec<crate::toolchains::SupplyChainRisk>> = HashMap::new();
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            if !res.supply_chain_risks.is_empty() {
                risk_all.insert(tool, &res.supply_chain_risks);
            }
        }
    }

    if !risk_all.is_empty() {
        lines.push("## Supply Chain Risks".into());
        let mut rt = Table::new();
        rt.header(&["Toolchain", "Package", "Risk Type", "Description"]);
        for tool in &tool_order() {
            if let Some(items) = risk_all.get(tool) {
                let display = display_name(tool);
                for r in items.iter() {
                    let risk_type = cli_yellow(&r.risk_type);
                    rt.add_row(&[display, &r.package, &risk_type, &r.description]);
                }
            }
        }
        lines.push(rt.render());
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

            for issue in &res.issues {
                lines.push(format!("> {issue}"));
            }

            lines.push(String::new());
        }
    }

    lines.join("\n")
}

pub fn count_outdated(report: &Report) -> usize {
    report
        .results
        .values()
        .map(|res| extract_outdated(res).len())
        .sum()
}

pub fn format_sarif(report: &Report) -> String {
    let mut rules = vec![];
    let mut results = vec![];

    for (tool, res) in &report.results {
        for vuln in &res.vulnerabilities {
            let rule_id = format!("{}-vuln-{}", tool, vuln.package);

            // Avoid duplicate rules
            if !rules.iter().any(|r: &sarif::Rule| r.id == rule_id) {
                rules.push(sarif::Rule {
                    id: rule_id.clone(),
                    name: format!("Vulnerability in {}", vuln.package),
                    short_description: sarif::Message {
                        text: format!("{} vulnerability in {}", vuln.severity, vuln.package),
                    },
                    help: sarif::Message {
                        text: format!(
                            "Update {} to version {}",
                            vuln.package, vuln.patched_version
                        ),
                    },
                    properties: sarif::RuleProperties {
                        tags: vec!["security".into(), tool.clone()],
                    },
                });
            }

            results.push(sarif::ResultEntry {
                rule_id,
                message: sarif::Message {
                    text: format!("{} vulnerability in {}", vuln.severity, vuln.package),
                },
                locations: vec![sarif::Location {
                    physical_location: sarif::PhysicalLocation {
                        artifact_location: sarif::ArtifactLocation {
                            uri: format!("envexa://{}/{}", tool, vuln.package),
                        },
                    },
                }],
            });
        }

        for outdated in extract_outdated(res) {
            let rule_id = format!("{}-outdated-{}", tool, outdated.name);

            if !rules.iter().any(|r: &sarif::Rule| r.id == rule_id) {
                rules.push(sarif::Rule {
                    id: rule_id.clone(),
                    name: format!("Outdated package {}", outdated.name),
                    short_description: sarif::Message {
                        text: format!(
                            "{} is outdated ({} -> {})",
                            outdated.name, outdated.current, outdated.latest
                        ),
                    },
                    help: sarif::Message {
                        text: format!("Update {} to version {}", outdated.name, outdated.latest),
                    },
                    properties: sarif::RuleProperties {
                        tags: vec!["maintenance".into(), tool.clone()],
                    },
                });
            }

            results.push(sarif::ResultEntry {
                rule_id,
                message: sarif::Message {
                    text: format!(
                        "{} is outdated ({} -> {})",
                        outdated.name, outdated.current, outdated.latest
                    ),
                },
                locations: vec![sarif::Location {
                    physical_location: sarif::PhysicalLocation {
                        artifact_location: sarif::ArtifactLocation {
                            uri: format!("envexa://{}/{}", tool, outdated.name),
                        },
                    },
                }],
            });
        }
    }

    let sarif_log = sarif::SarifLog {
        schema: "https://json.schemastore.org/sarif-2.1.0.json".into(),
        version: "2.1.0".into(),
        runs: vec![sarif::Run {
            tool: sarif::Tool {
                driver: sarif::Driver {
                    name: "Envexa".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    rules,
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&sarif_log).unwrap_or_else(|_| "{}".into())
}
