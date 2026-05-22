use std::collections::HashMap;

use crate::toolchains::{PackageInfo, ScanResult};

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
            widths[i] = widths[i].max(h.len());
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < ncols {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let sep_body: Vec<String> = widths.iter().map(|w| "-".repeat(w + 2)).collect();
        let sep = format!("+{}+", sep_body.join("+"));

        let fmt = |cells: &[String]| -> String {
            let mut s = String::from("|");
            for (i, cell) in cells.iter().enumerate() {
                if let Some(&w) = widths.get(i) {
                    s.push_str(&format!(" {:<w$} |", cell, w = w));
                } else {
                    s.push_str(&format!(" {} |", cell));
                }
            }
            s
        };

        let mut out = String::new();
        out.push_str(&sep);
        out.push('\n');
        out.push_str(&fmt(&self.headers));
        out.push('\n');
        out.push_str(&sep);
        out.push('\n');
        for row in &self.rows {
            out.push_str(&fmt(row));
            out.push('\n');
        }
        out.push_str(&sep);
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
];

fn tool_order() -> [&'static str; 10] {
    [
        "brew", "npm", "pnpm", "yarn", "bun", "deno", "pip", "gem", "cargo", "docker",
    ]
}

fn display_name(tool: &str) -> &str {
    DISPLAY_NAMES
        .iter()
        .find(|(k, _)| *k == tool)
        .map(|(_, v)| *v)
        .unwrap_or(tool)
}

fn status_label(s: &str) -> &str {
    LABELS
        .iter()
        .find(|(k, _)| *k == s)
        .map(|(_, v)| *v)
        .unwrap_or("?")
}

fn extract_outdated(res: &ScanResult) -> Vec<&PackageInfo> {
    let mut items = vec![];
    for key in [
        "outdated_formulae",
        "outdated_casks",
        "outdated_global",
        "outdated",
    ] {
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

fn first_version(res: &ScanResult) -> String {
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

pub fn format_report(report: &Report) -> String {
    let results = &report.results;
    let mut lines = vec![];
    lines.push("# Envexa Health Report".into());
    lines.push(format!("**Generated:** {}", report.timestamp));
    lines.push(String::new());

    let mut outdated_all: HashMap<&str, Vec<&PackageInfo>> = HashMap::new();

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let items = extract_outdated(res);
            if !items.is_empty() {
                outdated_all.insert(tool, items);
            }
        }
    }

    lines.push("## Dashboard".into());
    let mut dt = Table::new();
    dt.header(&["Toolchain", "Status", "Version"]);
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let display = display_name(tool);
            let label = status_label(&res.status);
            let ver = first_version(res);
            dt.add_row(&[display, label, &ver]);
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
                    ot.add_row(&[display, &item.name, &item.current, &item.latest]);
                }
            }
        }
        lines.push(ot.render());
        lines.push(String::new());
    }

    lines.push("## Per-Toolchain Details".into());
    lines.push(String::new());

    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let label = status_label(&res.status);
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
                    pt.add_row(&[&item.name, &item.current, &item.latest]);
                }
                lines.push(pt.render());
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

    let mut t = Table::new();
    t.header(&["Toolchain", "Status", "Count"]);
    for tool in &tool_order() {
        if let Some(res) = results.get(*tool) {
            let label = status_label(&res.status);
            let n = extract_outdated(res).len();
            let count = if n > 0 { n.to_string() } else { "-".into() };
            let display = display_name(tool);
            t.add_row(&[display, label, &count]);
        }
    }
    lines.push(t.render());

    lines.push(String::new());
    lines.push("Run `/envexa:scan` for full report or `/envexa:outdated` for details.".into());
    lines.join("\n")
}

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
                    t.add_row(&[display, &item.name, &item.current, &item.latest]);
                }
            }
        }
    }

    if !has_anything {
        return "All packages are up to date!".into();
    }

    format!("# Outdated Packages\n\n{}", t.render())
}
