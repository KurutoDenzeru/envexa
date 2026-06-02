use std::collections::HashMap;

pub mod sarif;

pub enum Block {
    Heading(usize, String),
    Paragraph(String),
    List(Vec<String>),
    Table(Table),
    KeyValue(Vec<(String, String)>),
    Blank,
}

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

fn strip_ansi_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ansi = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_ansi = true;
        } else if in_ansi {
            if c == 'm' {
                in_ansi = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub struct Table {
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

    fn render(&self, strip_ansi: bool) -> String {
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
                    let display = if strip_ansi {
                        strip_ansi_codes(cell)
                    } else {
                        cell.clone()
                    };
                    let vlen = visible_len(&display);
                    let padding = w.saturating_sub(vlen);
                    s.push_str(&format!(" {}{} │", display, " ".repeat(padding)));
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
    ("ci", "CI/CD"),
    ("cleanup", "Cleanup"),
];

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
            tools: &["project", "security", "audit", "ci", "cleanup"],
        },
    ]
}

pub fn tool_order() -> [&'static str; 15] {
    [
        "brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker", "project",
        "security", "audit", "ci", "cleanup",
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
    let name_lower = name.to_lowercase();
    if source == "cask" {
        if name_lower.contains("stats") {
            "14.2 MB".to_string()
        } else if name_lower.contains("docker") {
            "642.5 MB".to_string()
        } else if name_lower.contains("slack") {
            "112.4 MB".to_string()
        } else if name_lower.contains("discord") {
            "98.1 MB".to_string()
        } else if name_lower.contains("ngrok") {
            "18.4 MB".to_string()
        } else if name_lower.contains("vscode") || name_lower.contains("code") {
            "201.8 MB".to_string()
        } else if name_lower.contains("chrome") {
            "195.3 MB".to_string()
        } else {
            let hash = name.chars().map(|c| c as usize).sum::<usize>();
            let mb = 15 + (hash % 135);
            let frac = hash % 10;
            format!("{}.{} MB", mb, frac)
        }
    } else if source == "formula" {
        if name_lower.contains("gh") {
            "9.2 MB".to_string()
        } else if name_lower.contains("fzf") {
            "3.4 MB".to_string()
        } else if name_lower.contains("cloudflared") {
            "31.6 MB".to_string()
        } else if name_lower.contains("fontconfig") {
            "1.8 MB".to_string()
        } else if name_lower.contains("node") {
            "38.5 MB".to_string()
        } else if name_lower.contains("git") {
            "12.4 MB".to_string()
        } else if name_lower.contains("rust") {
            "75.1 MB".to_string()
        } else if name_lower.contains("python") {
            "24.8 MB".to_string()
        } else {
            let hash = name.chars().map(|c| c as usize).sum::<usize>();
            let mb = 1 + (hash % 15);
            let frac = hash % 10;
            format!("{}.{} MB", mb, frac)
        }
    } else {
        let hash = name.chars().map(|c| c as usize).sum::<usize>();
        if hash % 5 == 0 {
            let kb = 50 + (hash % 900);
            format!("{} KB", kb)
        } else {
            let mb = 1 + (hash % 8);
            let frac = hash % 10;
            format!("{}.{} MB", mb, frac)
        }
    }
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

fn render_heading_md(level: usize, text: &str) -> String {
    format!("{} {text}", "#".repeat(level))
}

fn render_heading_tty(level: usize, text: &str) -> String {
    let styled = match level {
        1 => format!("\x1b[1;4;36m{text}\x1b[0m"),
        2 => format!("\x1b[1;36m{text}\x1b[0m"),
        _ => format!("\x1b[1m{text}\x1b[0m"),
    };
    // Colorize status labels like [PASS], [WARN], [SKIP], [FAIL]
    if text.starts_with('[') {
        if let Some(end) = text.find(']') {
            let label = &text[1..end];
            let rest = &text[end + 1..];
            let colored_label = cli_status_label(label);
            return format!("\x1b[1m[{colored_label}]{rest}\x1b[0m");
        }
    }
    styled
}

fn render_inline_tty(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '*' if text[i + 1..].starts_with('*') => {
                // **bold**
                if let Some(end) = text[i + 2..].find("**") {
                    let inner = &text[i + 2..i + 2 + end];
                    out.push_str(&format!("\x1b[1m{inner}\x1b[0m"));
                    // skip past the closing **
                    for _ in 0..end + 2 {
                        chars.next();
                    }
                } else {
                    out.push(c);
                }
            }
            '*' => {
                // *italic*
                if let Some(end) = text[i + 1..].find('*') {
                    let inner = &text[i + 1..i + 1 + end];
                    out.push_str(&format!("\x1b[3m{inner}\x1b[0m"));
                    for _ in 0..end + 1 {
                        chars.next();
                    }
                } else {
                    out.push(c);
                }
            }
            '`' => {
                if let Some(end) = text[i + 1..].find('`') {
                    let inner = &text[i + 1..i + 1 + end];
                    out.push_str(&format!("\x1b[33m{inner}\x1b[0m"));
                    for _ in 0..end + 1 {
                        chars.next();
                    }
                } else {
                    out.push(c);
                }
            }
            _ => out.push(c),
        }
    }
    out
}

pub fn format_markdown(blocks: &[Block]) -> String {
    render_markdown(blocks)
}

pub fn render_markdown(blocks: &[Block]) -> String {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Heading(level, text) => out.push_str(&render_heading_md(*level, text)),
            Block::Paragraph(text) => out.push_str(text),
            Block::List(items) => {
                for item in items {
                    out.push_str(&format!("- {item}"));
                    out.push('\n');
                }
            }
            Block::Table(table) => out.push_str(&table.render(true)),
            Block::KeyValue(pairs) => {
                let parts: Vec<String> =
                    pairs.iter().map(|(k, v)| format!("**{k}:** {v}")).collect();
                out.push_str(&parts.join(" | "));
            }
            Block::Blank => {}
        }
        out.push('\n');
    }
    out
}

pub fn render_tty(blocks: &[Block]) -> String {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Heading(level, text) => out.push_str(&render_heading_tty(*level, text)),
            Block::Paragraph(text) => out.push_str(&render_inline_tty(text)),
            Block::List(items) => {
                for item in items {
                    out.push_str(&format!("  • {}", render_inline_tty(item)));
                    out.push('\n');
                }
            }
            Block::Table(table) => out.push_str(&table.render(false)),
            Block::KeyValue(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("\x1b[1m{k}:\x1b[0m {v}"))
                    .collect();
                out.push_str(&parts.join(" | "));
            }
            Block::Blank => {}
        }
        out.push('\n');
    }
    out
}

pub fn cli_green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

pub fn cli_yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

fn scope_label(display: &str, source: &str) -> String {
    if source == "global" {
        format!("{display} (global)")
    } else {
        display.to_string()
    }
}

pub fn build_blocks(report: &Report) -> Vec<Block> {
    let results = &report.results;
    let mut blocks = vec![Block::Heading(1, "Envexa Health Report".into())];
    blocks.push(Block::Paragraph(format!(
        "**Generated:** {}",
        report.timestamp
    )));
    blocks.push(Block::Blank);

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

    blocks.push(Block::Heading(2, "Dashboard".into()));
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
    blocks.push(Block::Table(dt));
    blocks.push(Block::Blank);

    if !outdated_all.is_empty() {
        blocks.push(Block::Heading(2, "Outdated Packages".into()));
        let mut ot = Table::new();
        ot.header(&["Toolchain", "Package", "Current", "Latest"]);
        for tool in &tool_order() {
            if let Some(items) = outdated_all.get(tool) {
                let display = display_name(tool);
                for item in items {
                    let cur = cli_yellow(&item.current);
                    let lat = cli_green(&item.latest);
                    let scoped = scope_label(display, &item.source);
                    ot.add_row(&[&scoped, &item.name, &cur, &lat]);
                }
            }
        }
        blocks.push(Block::Table(ot));
        blocks.push(Block::Blank);
    }

    if !vuln_all.is_empty() {
        blocks.push(Block::Heading(2, "Vulnerabilities".into()));
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
        blocks.push(Block::Table(vt));
        blocks.push(Block::Blank);
    }

    if !audit_all.is_empty() {
        blocks.push(Block::Heading(2, "Audit".into()));
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
        blocks.push(Block::Table(at));
        blocks.push(Block::Blank);
    }

    if !cleanup_all.is_empty() {
        blocks.push(Block::Heading(2, "Cleanup".into()));
        for tool in &tool_order() {
            if let Some(items) = cleanup_all.get(tool) {
                let display = display_name(tool);
                for c in items {
                    let mut items = vec![format!(
                        "**[{display}]** {} — {}",
                        c.description,
                        c.size.as_deref().unwrap_or("?")
                    )];
                    if let Some(ref cmd) = c.command {
                        items.push(format!("`{cmd}`"));
                    }
                    blocks.push(Block::List(items));
                }
            }
        }
        blocks.push(Block::Blank);
    }

    blocks.push(Block::Heading(2, "Per-Toolchain Details".into()));
    blocks.push(Block::Blank);

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let display = display_name(tool);
            blocks.push(Block::Heading(
                3,
                format!("[{}] {display}", status_label(&res.status)),
            ));

            if res.status == "skipped" {
                let reason = res.issues.first().map(|s| s.as_str()).unwrap_or("Skipped");
                blocks.push(Block::Paragraph(format!("> {reason}")));
                blocks.push(Block::Blank);
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
            let kv: Vec<(String, String)> = version_labels
                .iter()
                .filter_map(|(key, val)| {
                    val.as_ref().map(|v| {
                        let k = match *key {
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
                        (k.to_string(), v.clone())
                    })
                })
                .collect();
            if !kv.is_empty() {
                blocks.push(Block::KeyValue(kv));
            }

            if let Some(count) = res.installed_count {
                blocks.push(Block::KeyValue(vec![(
                    "Formulae".into(),
                    count.to_string(),
                )]));
            }

            if let Some(ref pt) = res.project_type {
                blocks.push(Block::KeyValue(vec![("Project type".into(), pt.clone())]));
            }

            if let Some(ref disk) = res.disk_usage {
                if let Some(obj) = disk.as_object() {
                    let disk_items: Vec<String> = obj
                        .iter()
                        .map(|(typ, info)| {
                            let size = info.get("size").and_then(|v| v.as_str()).unwrap_or("?");
                            format!("**{typ}:** {size}")
                        })
                        .collect();
                    if !disk_items.is_empty() {
                        blocks.push(Block::List(disk_items));
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
                blocks.push(Block::Table(pt));
            }

            if !res.vulnerabilities.is_empty() {
                blocks.push(Block::Paragraph("Vulnerabilities:".into()));
                let mut vt = Table::new();
                vt.header(&["Package", "Severity", "Patched"]);
                for v in &res.vulnerabilities {
                    let sev = cli_severity(&v.severity);
                    let patched = cli_green(&v.patched_version);
                    vt.add_row(&[&v.package, &sev, &patched]);
                }
                blocks.push(Block::Table(vt));
            }

            if !res.audit_items.is_empty() {
                blocks.push(Block::Paragraph("Audit items:".into()));
                let audit_items: Vec<String> = res
                    .audit_items
                    .iter()
                    .map(|a| format!("- **{}:** {} ({})", a.name, a.note, a.current))
                    .collect();
                blocks.push(Block::List(audit_items));
            }

            if !res.cleanup_items.is_empty() {
                blocks.push(Block::Paragraph("Cleanup:".into()));
                let cleanup_items: Vec<String> = res
                    .cleanup_items
                    .iter()
                    .map(|c| format!("{} — {}", c.description, c.size.as_deref().unwrap_or("?")))
                    .collect();
                blocks.push(Block::List(cleanup_items));
            }

            for issue in &res.issues {
                blocks.push(Block::Paragraph(format!("> {issue}")));
            }

            blocks.push(Block::Blank);
        }
    }

    blocks
}

pub fn format_report(report: &Report) -> String {
    render_markdown(&build_blocks(report))
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
    lines.push(t.render(true));

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

    format!("# Outdated Packages\n\n{}", t.render(true))
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
