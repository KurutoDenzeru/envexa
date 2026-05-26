use std::collections::HashSet;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use throbber_widgets_tui::ThrobberState;

use crate::config;
use crate::scanner::{self, OutdatedItem, Report};
use crate::toolchains;
use crate::toolchains::{AuditItem, CleanupItem, VulnerabilityInfo};

pub enum View {
    Dashboard,
    Outdated,
    Scanning,
    PackageDetail,
    Updating,
}

pub struct App {
    pub report: Option<Report>,
    pub view: View,
    pub dashboard_selection: usize,
    pub outdated_selection: usize,
    pub tab_index: usize,
    pub search_mode: bool,
    pub search_query: String,
    pub throbber_state: ThrobberState,
    pub progress_counter: usize,

    pub detail_tool: Option<String>,
    pub detail_key: Option<String>,
    pub detail_selection: usize,
    pub detail_items: Vec<OutdatedItem>,
    pub detail_vulns: Vec<VulnerabilityInfo>,
    pub detail_audits: Vec<AuditItem>,
    pub detail_cleanup: Vec<CleanupItem>,
    pub detail_checked: HashSet<usize>,
    pub detail_message: String,

    pub checked_outdated: HashSet<usize>,
}

impl App {
    pub fn new() -> Self {
        let report = config::read_cache().map(|e| e.report);
        Self {
            report,
            view: View::Dashboard,
            dashboard_selection: 0,
            outdated_selection: 0,
            tab_index: 0,
            search_mode: false,
            search_query: String::new(),
            throbber_state: ThrobberState::default(),
            progress_counter: 0,
            detail_tool: None,
            detail_key: None,
            detail_selection: 0,
            detail_items: Vec::new(),
            detail_vulns: Vec::new(),
            detail_audits: Vec::new(),
            detail_cleanup: Vec::new(),
            detail_checked: HashSet::new(),
            detail_message: String::new(),
            checked_outdated: HashSet::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        loop {
            terminal.draw(|frame| crate::tui::ui::render(frame, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => {}
                }

                if self.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            self.search_mode = false;
                            self.search_query.clear();
                        }
                        KeyCode::Enter => {
                            self.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.clamp_selection();
                        }
                        KeyCode::Char(c) if !c.is_control() => {
                            self.search_query.push(c);
                            self.clamp_selection();
                        }
                        _ => {}
                    }
                    continue;
                }

                if matches!(self.view, View::PackageDetail) {
                    match key.code {
                        KeyCode::Char(' ')
                            if !matches!(
                                self.detail_key.as_deref(),
                                Some("security") | Some("audit") | Some("cleanup")
                            ) =>
                        {
                            if self.detail_checked.contains(&self.detail_selection) {
                                self.detail_checked.remove(&self.detail_selection);
                            } else {
                                self.detail_checked.insert(self.detail_selection);
                            }
                        }
                        KeyCode::Char(' ') => {}
                        KeyCode::Char('y') | KeyCode::Char('Y')
                            if !matches!(
                                self.detail_key.as_deref(),
                                Some("security") | Some("audit") | Some("cleanup")
                            ) =>
                        {
                            self.do_detail_updates(&mut terminal)?;
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            self.export_detail_report();
                        }
                        KeyCode::Down | KeyCode::Char('n') => {
                            let n = self.detail_len().saturating_sub(1);
                            self.detail_selection = self.detail_selection.saturating_add(1).min(n);
                        }
                        KeyCode::Up | KeyCode::Char('p') => {
                            self.detail_selection = self.detail_selection.saturating_sub(1);
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.view = View::Dashboard;
                            self.tab_index = 0;
                            self.detail_tool = None;
                            self.detail_key = None;
                            self.clear_detail();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Esc | KeyCode::Char('h') => {
                        self.view = View::Dashboard;
                        self.tab_index = 0;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.search_mode = false;
                        self.search_query.clear();
                        self.do_scan(&mut terminal)?
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        self.view = View::Outdated;
                        self.tab_index = 1;
                    }
                    KeyCode::Char('u') | KeyCode::Char('U')
                        if matches!(self.view, View::Outdated)
                            && !self.checked_outdated.is_empty() =>
                    {
                        self.do_checked_updates(&mut terminal)?;
                    }
                    KeyCode::Char(' ') => {
                        if matches!(self.view, View::Outdated) {
                            if self.checked_outdated.contains(&self.outdated_selection) {
                                self.checked_outdated.remove(&self.outdated_selection);
                            } else {
                                self.checked_outdated.insert(self.outdated_selection);
                            }
                        }
                    }
                    KeyCode::Char('/') => {
                        self.search_mode = true;
                        self.dashboard_selection = 0;
                        self.outdated_selection = 0;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.tab_index = (self.tab_index + 1).min(1);
                        self.view = if self.tab_index == 0 {
                            View::Dashboard
                        } else {
                            View::Outdated
                        };
                    }
                    KeyCode::Left | KeyCode::Char('j') => {
                        self.tab_index = self.tab_index.saturating_sub(1);
                        self.view = if self.tab_index == 0 {
                            View::Dashboard
                        } else {
                            View::Outdated
                        };
                    }
                    KeyCode::Down | KeyCode::Char('n') => self.next_item(),
                    KeyCode::Up | KeyCode::Char('p') => self.prev_item(),
                    KeyCode::Enter => {
                        if matches!(self.view, View::Dashboard) {
                            self.open_detail();
                        }
                    }
                    _ => {}
                }
            }
        }

        ratatui::restore();
        Ok(())
    }

    fn open_detail(&mut self) {
        let report = match &self.report {
            Some(r) => r,
            None => return,
        };
        let tools = self.filtered_tools();
        let tool = match tools.get(self.dashboard_selection) {
            Some(t) => t,
            None => return,
        };
        let res = match report.results.get(*tool) {
            Some(r) => r,
            None => return,
        };

        self.detail_tool = Some(scanner::display_name(tool).to_string());
        self.detail_key = Some(tool.to_string());
        self.detail_selection = 0;
        self.detail_checked.clear();
        self.detail_message.clear();

        match *tool {
            "security" => {
                let vulns = scanner::extract_vulnerabilities(res);
                self.detail_vulns = vulns.to_vec();
                self.detail_items.clear();
                self.detail_audits.clear();
                self.detail_cleanup.clear();
            }
            "audit" => {
                let audits = scanner::extract_audit_items(res);
                self.detail_audits = audits.to_vec();
                self.detail_items.clear();
                self.detail_vulns.clear();
                self.detail_cleanup.clear();
            }
            "cleanup" => {
                let cleanup = scanner::extract_cleanup_items(res);
                self.detail_cleanup = cleanup.to_vec();
                self.detail_items.clear();
                self.detail_vulns.clear();
                self.detail_audits.clear();
            }
            _ => {
                let items = scanner::extract_outdated(res);
                self.detail_items = items;
                self.detail_vulns.clear();
                self.detail_audits.clear();
                self.detail_cleanup.clear();
            }
        }

        self.view = View::PackageDetail;
    }

    fn do_detail_updates(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tool = match &self.detail_tool {
            Some(t) => t.clone(),
            None => return Ok(()),
        };
        let work: Vec<(String, OutdatedItem)> = self
            .detail_items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.detail_checked.contains(i))
            .map(|(_, item)| (tool.clone(), item.clone()))
            .collect();
        self.detail_checked.clear();
        if work.is_empty() {
            return Ok(());
        }

        self.view = View::Updating;
        self.progress_counter = 0;
        let _count = work.len();

        let handle = std::thread::spawn(move || {
            let mut updated = 0usize;
            let mut failed = 0usize;
            let mut errors = vec![];
            for (tool, item) in &work {
                match run_update(tool, item) {
                    Ok(_) => updated += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.name, e));
                    }
                }
            }
            if errors.is_empty() {
                format!("\u{2714} Updated {updated} package(s)")
            } else {
                let e = errors.join("; ");
                format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}")
            }
        });

        loop {
            terminal.draw(|frame| crate::tui::ui::render(frame, self))?;
            self.throbber_state.calc_next();
            self.progress_counter += 1;
            if handle.is_finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let result = handle.join().unwrap();
        self.detail_message = result;
        self.view = View::PackageDetail;
        Ok(())
    }

    fn collect_checked_work(&self) -> Vec<(String, OutdatedItem)> {
        let report = match &self.report {
            Some(r) => r,
            None => return vec![],
        };
        let mut work = vec![];
        let mut idx = 0usize;
        for tool in &scanner::tool_order() {
            if let Some(res) = report.results.get(*tool) {
                for item in &scanner::extract_outdated(res) {
                    if self.checked_outdated.contains(&idx) {
                        work.push((scanner::display_name(tool).to_string(), item.clone()));
                    }
                    idx += 1;
                }
            }
        }
        work
    }

    fn do_checked_updates(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let work = self.collect_checked_work();
        self.checked_outdated.clear();
        if work.is_empty() {
            return Ok(());
        }

        self.view = View::Updating;
        self.progress_counter = 0;
        let _count = work.len();

        let handle = std::thread::spawn(move || {
            let mut updated = 0usize;
            let mut failed = 0usize;
            let mut errors = vec![];
            for (tool, item) in &work {
                match run_update(tool, item) {
                    Ok(_) => updated += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.name, e));
                    }
                }
            }
            if errors.is_empty() {
                format!("\u{2714} Updated {updated} package(s)")
            } else {
                let e = errors.join("; ");
                format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}")
            }
        });

        loop {
            terminal.draw(|frame| crate::tui::ui::render(frame, self))?;
            self.throbber_state.calc_next();
            self.progress_counter += 1;
            if handle.is_finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let result = handle.join().unwrap();
        self.detail_message = result;
        self.view = View::Outdated;
        Ok(())
    }

    fn filtered_tools(&self) -> Vec<&'static str> {
        let report = match &self.report {
            Some(r) => r,
            None => return vec![],
        };
        let q = self.search_query.to_lowercase();
        scanner::tool_order()
            .iter()
            .copied()
            .filter(|tool| {
                if q.is_empty() || !self.search_mode {
                    return true;
                }
                let name = scanner::display_name(tool).to_lowercase();
                name.contains(&q) || tool.contains(&q)
            })
            .filter(|tool| report.results.contains_key(*tool))
            .collect()
    }

    fn clamp_selection(&mut self) {
        match self.view {
            View::Dashboard => {
                let n = self.filtered_tools().len().saturating_sub(1);
                self.dashboard_selection = self.dashboard_selection.min(n);
            }
            View::Outdated => {
                let n: usize = self
                    .report
                    .as_ref()
                    .map(|r| {
                        let q = self.search_query.to_lowercase();
                        let mut count = 0usize;
                        for tool in &scanner::tool_order() {
                            if let Some(res) = r.results.get(*tool) {
                                for item in &scanner::extract_outdated(res) {
                                    if q.is_empty() || !self.search_mode {
                                        count += 1;
                                    } else {
                                        let tool_name = scanner::display_name(tool).to_lowercase();
                                        if tool_name.contains(&q)
                                            || item.name.to_lowercase().contains(&q)
                                            || item.source.contains(&q)
                                        {
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        count
                    })
                    .unwrap_or(0)
                    .saturating_sub(1);
                self.outdated_selection = self.outdated_selection.min(n);
            }
            View::Scanning => {}
            View::PackageDetail => {
                let n = self.detail_len().saturating_sub(1);
                self.detail_selection = self.detail_selection.min(n);
            }
            View::Updating => {}
        }
    }

    fn do_scan(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.view = View::Scanning;
        self.progress_counter = 0;

        let rt_handle = tokio::runtime::Handle::current();
        let handle = std::thread::spawn(move || rt_handle.block_on(toolchains::scan_all()));

        loop {
            terminal.draw(|frame| crate::tui::ui::render(frame, self))?;
            self.throbber_state.calc_next();
            self.progress_counter += 1;
            if handle.is_finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let results = handle.join().expect("Scan thread panicked");

        let report = Report {
            timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            results,
        };
        let _ = config::write_cache(&report, 7);

        self.report = Some(report);
        self.view = View::Dashboard;
        self.tab_index = 0;
        self.dashboard_selection = 0;
        std::thread::sleep(Duration::from_millis(200));

        terminal.draw(|frame| crate::tui::ui::render(frame, self))?;
        Ok(())
    }

    fn next_item(&mut self) {
        match self.view {
            View::Dashboard => {
                let n = self.filtered_tools().len().saturating_sub(1);
                self.dashboard_selection = self.dashboard_selection.saturating_add(1).min(n);
            }
            View::Outdated => {
                let n = self
                    .report
                    .as_ref()
                    .map(scanner::count_outdated)
                    .unwrap_or(0)
                    .saturating_sub(1);
                self.outdated_selection = self.outdated_selection.saturating_add(1).min(n);
            }
            View::Scanning => {}
            View::PackageDetail => {
                let n = self.detail_len().saturating_sub(1);
                self.detail_selection = self.detail_selection.saturating_add(1).min(n);
            }
            View::Updating => {}
        }
    }

    fn prev_item(&mut self) {
        match self.view {
            View::Dashboard => {
                self.dashboard_selection = self.dashboard_selection.saturating_sub(1)
            }
            View::Outdated => self.outdated_selection = self.outdated_selection.saturating_sub(1),
            View::Scanning => {}
            View::PackageDetail => self.detail_selection = self.detail_selection.saturating_sub(1),
            View::Updating => {}
        }
    }

    fn detail_len(&self) -> usize {
        match self.detail_key.as_deref() {
            Some("security") => self.detail_vulns.len(),
            Some("audit") => self.detail_audits.len(),
            Some("cleanup") => self.detail_cleanup.len(),
            _ => self.detail_items.len(),
        }
    }

    fn clear_detail(&mut self) {
        self.detail_items.clear();
        self.detail_vulns.clear();
        self.detail_audits.clear();
        self.detail_cleanup.clear();
        self.detail_checked.clear();
        self.detail_message.clear();
    }

    pub fn export_detail_report(&mut self) {
        let key = match self.detail_key.as_deref() {
            Some(k) => k,
            None => return,
        };
        let timestamp = self
            .report
            .as_ref()
            .map(|r| r.timestamp.as_str())
            .unwrap_or("Unknown");

        let (filename, content) = match key {
            "security" => {
                let filename = "envexa_security_report.md";
                let mut out = "# Envexa Security Vulnerability Report\n\n".to_string();
                out.push_str(&format!("* **Generated**: {}\n", timestamp));
                out.push_str(&format!(
                    "* **Total Vulnerabilities**: {}\n\n",
                    self.detail_vulns.len()
                ));
                out.push_str("| Package | Severity | CVE | Title | Patched In |\n");
                out.push_str("| --- | --- | --- | --- | --- |\n");
                for v in &self.detail_vulns {
                    let cve = v.cve.as_deref().unwrap_or("-");
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        v.package, v.severity, cve, v.title, v.patched_version
                    ));
                }
                (filename, out)
            }
            "audit" => {
                let filename = "envexa_audit_report.md";
                let mut out = "# Envexa System & Toolchain Audit Report\n\n".to_string();
                out.push_str(&format!("* **Generated**: {}\n", timestamp));
                out.push_str(&format!(
                    "* **Total Audit Issues**: {}\n\n",
                    self.detail_audits.len()
                ));
                out.push_str("| Name | Current State | Note / Recommendation |\n");
                out.push_str("| --- | --- | --- |\n");
                for a in &self.detail_audits {
                    out.push_str(&format!("| {} | {} | {} |\n", a.name, a.current, a.note));
                }
                (filename, out)
            }
            _ => return,
        };

        match std::fs::write(filename, content) {
            Ok(_) => {
                self.detail_message = format!("\u{2714} Exported to {}", filename);
            }
            Err(e) => {
                self.detail_message = format!("\u{2716} Export failed: {}", e);
            }
        }
    }
}

fn run_update(tool: &str, item: &OutdatedItem) -> Result<String, String> {
    let project_path = toolchains::get_project_path();
    let is_package = item.source == "package";

    let (cmd, args, run_in_project) = match tool {
        "Brew" | "Brew (dev)" => {
            let mut args = vec!["upgrade".to_string()];
            if item.source == "cask" {
                args.push("--cask".to_string());
            }
            args.push(item.name.clone());
            ("brew".to_string(), args, false)
        }
        "npm" => {
            if is_package {
                (
                    "npm".to_string(),
                    vec!["install".to_string(), format!("{}@latest", item.name)],
                    true,
                )
            } else {
                (
                    "npm".to_string(),
                    vec![
                        "install".to_string(),
                        "-g".to_string(),
                        format!("{}@latest", item.name),
                    ],
                    false,
                )
            }
        }
        "pnpm" => {
            if is_package {
                (
                    "pnpm".to_string(),
                    vec!["add".to_string(), format!("{}@latest", item.name)],
                    true,
                )
            } else {
                (
                    "pnpm".to_string(),
                    vec![
                        "add".to_string(),
                        "-g".to_string(),
                        format!("{}@latest", item.name),
                    ],
                    false,
                )
            }
        }
        "Yarn" => {
            if is_package {
                (
                    "yarn".to_string(),
                    vec!["upgrade".to_string(), format!("{}@latest", item.name)],
                    true,
                )
            } else {
                return Err("global auto-update not supported for Yarn".into());
            }
        }
        "Bun" => {
            if is_package {
                (
                    "bun".to_string(),
                    vec!["add".to_string(), format!("{}@latest", item.name)],
                    true,
                )
            } else {
                (
                    "bun".to_string(),
                    vec![
                        "add".to_string(),
                        "-g".to_string(),
                        format!("{}@latest", item.name),
                    ],
                    false,
                )
            }
        }
        "pip" => {
            if is_package {
                (
                    "pip3".to_string(),
                    vec![
                        "install".to_string(),
                        "--upgrade".to_string(),
                        item.name.clone(),
                    ],
                    true,
                )
            } else {
                (
                    "pip3".to_string(),
                    vec![
                        "install".to_string(),
                        "--upgrade".to_string(),
                        item.name.clone(),
                    ],
                    false,
                )
            }
        }
        "Gem" => (
            "gem".to_string(),
            vec!["update".to_string(), item.name.clone()],
            false,
        ),
        "Cargo" => (
            "cargo".to_string(),
            vec!["install".to_string(), item.name.clone()],
            false,
        ),
        _ => return Err(format!("auto-update not supported for {}", tool)),
    };

    let mut command = std::process::Command::new(&cmd);
    command.args(&args);
    if run_in_project {
        command.current_dir(&project_path);
    }

    let output = command
        .output()
        .map_err(|e| format!("command failed: {e}"))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok("done".into())
        } else {
            Ok(stdout)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "unknown error".into()
        } else {
            stderr
        })
    }
}
