use std::collections::HashSet;
use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use std::collections::HashMap;
use tokio::sync::mpsc;

use throbber_widgets_tui::ThrobberState;

use crate::core::config;
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

pub enum AppEvent {
    Tick,
    Input(crossterm::event::KeyEvent),
    ScanFinished(HashMap<String, crate::toolchains::ScanResult>),
    UpdateProgress {
        package_name: String,
        downloaded_mb: f64,
    },
    UpdateFinished {
        result_msg: String,
        updated_packages: Vec<(String, String, String)>, // (tool_key, name, source)
    },
    CleanupFinished {
        result_msg: String,
        new_report_res: Option<crate::toolchains::ScanResult>,
    },
    SecurityFixFinished {
        result_msg: String,
        new_report_res: Option<crate::toolchains::ScanResult>,
    },
}

pub struct UiState {
    pub view: View,
    pub dashboard_selection: usize,
    pub outdated_selection: usize,
    pub tab_index: usize,
    pub search_mode: bool,
    pub search_query: String,
    pub throbber_state: ThrobberState,
    pub progress_counter: usize,
    pub tick_count: usize,
    pub checked_outdated: HashSet<usize>,
    pub update_package_name: String,
    pub update_downloaded_mb: f64,
}

pub struct DetailState {
    pub tool: Option<String>,
    pub key: Option<String>,
    pub selection: usize,
    pub items: Vec<OutdatedItem>,
    pub vulns: Vec<VulnerabilityInfo>,
    pub audits: Vec<AuditItem>,
    pub cleanup: Vec<CleanupItem>,
    pub checked: HashSet<usize>,
    pub message: String,
}

pub struct App {
    pub report: Option<Report>,
    pub ui: UiState,
    pub detail: DetailState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let report = config::read_cache().map(|e| e.report);
        Self {
            report,
            ui: UiState {
                view: View::Dashboard,
                dashboard_selection: 0,
                outdated_selection: 0,
                tab_index: 0,
                search_mode: false,
                search_query: String::new(),
                throbber_state: ThrobberState::default(),
                progress_counter: 0,
                tick_count: 0,
                checked_outdated: HashSet::new(),
                update_package_name: String::new(),
                update_downloaded_mb: 0.0,
            },
            detail: DetailState {
                tool: None,
                key: None,
                selection: 0,
                items: Vec::new(),
                vulns: Vec::new(),
                audits: Vec::new(),
                cleanup: Vec::new(),
                checked: HashSet::new(),
                message: String::new(),
            },
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
        let tx_in = tx.clone();

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(Duration::from_millis(150));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if tx_in.send(AppEvent::Tick).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Key(key))) = reader.next() => {
                        if key.kind == KeyEventKind::Press
                            && tx_in.send(AppEvent::Input(key)).is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });

        loop {
            terminal.draw(|frame| crate::tui::ui::render(frame, self))?;

            if let Some(event) = rx.recv().await {
                match event {
                    AppEvent::Tick => {
                        self.ui.tick_count = self.ui.tick_count.wrapping_add(1);
                        if matches!(self.ui.view, View::Scanning | View::Updating) {
                            self.ui.throbber_state.calc_next();
                            self.ui.progress_counter += 1;
                        }
                    }
                    AppEvent::Input(key) => {
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            break;
                        }
                        if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                            break;
                        }

                        if self.ui.search_mode {
                            match key.code {
                                KeyCode::Esc => {
                                    self.ui.search_mode = false;
                                    self.ui.search_query.clear();
                                }
                                KeyCode::Enter => {
                                    self.ui.search_mode = false;
                                }
                                KeyCode::Backspace => {
                                    self.ui.search_query.pop();
                                    self.clamp_selection();
                                }
                                KeyCode::Char(c) if !c.is_control() => {
                                    self.ui.search_query.push(c);
                                    self.clamp_selection();
                                }
                                _ => {}
                            }
                            continue;
                        }

                        if matches!(self.ui.view, View::PackageDetail) {
                            match key.code {
                                KeyCode::Char(' ')
                                    if !matches!(
                                        self.detail.key.as_deref(),
                                        Some("security") | Some("audit")
                                    ) =>
                                {
                                    if self.detail.checked.contains(&self.detail.selection) {
                                        self.detail.checked.remove(&self.detail.selection);
                                    } else {
                                        self.detail.checked.insert(self.detail.selection);
                                    }
                                }
                                KeyCode::Char(' ') => {}
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    match self.detail.key.as_deref() {
                                        Some("cleanup") => {
                                            self.do_detail_cleanups(tx.clone())?;
                                        }
                                        Some("security") | Some("audit") => {}
                                        _ => {
                                            self.do_detail_updates(tx.clone())?;
                                        }
                                    }
                                }
                                KeyCode::Char('f') | KeyCode::Char('F') => {
                                    if matches!(self.detail.key.as_deref(), Some("security")) {
                                        self.do_detail_security_fixes(tx.clone())?;
                                    }
                                }
                                KeyCode::Char('e') | KeyCode::Char('E') => {
                                    self.export_detail_report();
                                }
                                KeyCode::Down | KeyCode::Char('n') => {
                                    let n = self.detail_len().saturating_sub(1);
                                    self.detail.selection =
                                        self.detail.selection.saturating_add(1).min(n);
                                }
                                KeyCode::Up | KeyCode::Char('p') => {
                                    self.detail.selection = self.detail.selection.saturating_sub(1);
                                }
                                KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                                    self.ui.view = View::Dashboard;
                                    self.ui.tab_index = 0;
                                    self.detail.tool = None;
                                    self.detail.key = None;
                                    self.clear_detail();
                                }
                                _ => {}
                            }
                            continue;
                        }

                        match key.code {
                            KeyCode::Esc | KeyCode::Char('h') => {
                                self.ui.view = View::Dashboard;
                                self.ui.tab_index = 0;
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                self.ui.search_mode = false;
                                self.ui.search_query.clear();
                                self.do_scan(tx.clone())?;
                            }
                            KeyCode::Char('o') | KeyCode::Char('O') => {
                                self.ui.view = View::Outdated;
                                self.ui.tab_index = 1;
                            }
                            KeyCode::Char('u') | KeyCode::Char('U')
                                if matches!(self.ui.view, View::Outdated)
                                    && !self.ui.checked_outdated.is_empty() =>
                            {
                                self.do_checked_updates(tx.clone())?;
                            }
                            KeyCode::Char(' ') => {
                                if matches!(self.ui.view, View::Outdated) {
                                    if self
                                        .ui
                                        .checked_outdated
                                        .contains(&self.ui.outdated_selection)
                                    {
                                        self.ui
                                            .checked_outdated
                                            .remove(&self.ui.outdated_selection);
                                    } else {
                                        self.ui.checked_outdated.insert(self.ui.outdated_selection);
                                    }
                                }
                            }
                            KeyCode::Char('/') => {
                                self.ui.search_mode = true;
                                self.ui.dashboard_selection = 0;
                                self.ui.outdated_selection = 0;
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                self.ui.tab_index = (self.ui.tab_index + 1).min(1);
                                self.ui.view = if self.ui.tab_index == 0 {
                                    View::Dashboard
                                } else {
                                    View::Outdated
                                };
                            }
                            KeyCode::Left | KeyCode::Char('j') => {
                                self.ui.tab_index = self.ui.tab_index.saturating_sub(1);
                                self.ui.view = if self.ui.tab_index == 0 {
                                    View::Dashboard
                                } else {
                                    View::Outdated
                                };
                            }
                            KeyCode::Down | KeyCode::Char('n') => self.next_item(),
                            KeyCode::Up | KeyCode::Char('p') => self.prev_item(),
                            KeyCode::Enter => {
                                if matches!(self.ui.view, View::Dashboard) {
                                    self.open_detail();
                                }
                            }
                            _ => {}
                        }
                    }
                    AppEvent::ScanFinished(results) => {
                        let report = Report {
                            timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                            results,
                        };
                        let _ = config::write_cache(&report, 7);

                        self.report = Some(report);
                        self.ui.view = View::Dashboard;
                        self.ui.tab_index = 0;
                        self.ui.dashboard_selection = 0;
                    }
                    AppEvent::UpdateProgress {
                        package_name,
                        downloaded_mb,
                    } => {
                        self.ui.update_package_name = package_name;
                        self.ui.update_downloaded_mb = downloaded_mb;
                    }
                    AppEvent::UpdateFinished {
                        result_msg,
                        updated_packages,
                    } => {
                        self.detail.message = result_msg;

                        if let Some(ref mut report) = self.report {
                            for (tool_key, name, source) in &updated_packages {
                                if let Some(res) = report.results.get_mut(tool_key) {
                                    match source.as_str() {
                                        "formula" => {
                                            res.outdated_formulae.retain(|pkg| pkg.name != *name);
                                        }
                                        "cask" => {
                                            res.outdated_casks.retain(|pkg| pkg.name != *name);
                                        }
                                        "package" => {
                                            res.outdated.retain(|pkg| pkg.name != *name);
                                        }
                                        "global" => {
                                            res.outdated_global.retain(|pkg| pkg.name != *name);
                                        }
                                        _ => {
                                            res.outdated.retain(|pkg| pkg.name != *name);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(ref tool_key) = self.detail.key {
                            for (tk, name, _) in &updated_packages {
                                if tk == tool_key {
                                    self.detail.items.retain(|item| item.name != *name);
                                }
                            }
                        }

                        if self.detail.key.is_some() {
                            self.ui.view = View::PackageDetail;
                        } else {
                            self.ui.view = View::Outdated;
                        }
                        self.clamp_selection();
                    }
                    AppEvent::CleanupFinished {
                        result_msg,
                        new_report_res,
                    } => {
                        self.detail.message = result_msg;
                        self.ui.view = View::PackageDetail;

                        if let Some(res) = new_report_res {
                            if let Some(ref mut report) = self.report {
                                report.results.insert("cleanup".to_string(), res.clone());
                                self.detail.cleanup =
                                    crate::scanner::extract_cleanup_items(&res).to_vec();
                            }
                        }
                        self.clamp_selection();
                    }
                    AppEvent::SecurityFixFinished {
                        result_msg,
                        new_report_res,
                    } => {
                        self.detail.message = result_msg;
                        self.ui.view = View::PackageDetail;

                        if let Some(res) = new_report_res {
                            if let Some(ref mut report) = self.report {
                                report.results.insert("security".to_string(), res.clone());
                                self.detail.vulns =
                                    crate::scanner::extract_vulnerabilities(&res).to_vec();
                            }
                        }
                        self.clamp_selection();
                    }
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
        let tool = match tools.get(self.ui.dashboard_selection) {
            Some(t) => t,
            None => return,
        };
        let res = match report.results.get(*tool) {
            Some(r) => r,
            None => return,
        };

        self.detail.tool = Some(scanner::display_name(tool).to_string());
        self.detail.key = Some(tool.to_string());
        self.detail.selection = 0;
        self.detail.checked.clear();
        self.detail.message.clear();

        match *tool {
            "security" => {
                let vulns = scanner::extract_vulnerabilities(res);
                self.detail.vulns = vulns.to_vec();
                self.detail.items.clear();
                self.detail.audits.clear();
                self.detail.cleanup.clear();
            }
            "audit" => {
                let audits = scanner::extract_audit_items(res);
                self.detail.audits = audits.to_vec();
                self.detail.items.clear();
                self.detail.vulns.clear();
                self.detail.cleanup.clear();
            }
            "cleanup" => {
                let cleanup = scanner::extract_cleanup_items(res);
                self.detail.cleanup = cleanup.to_vec();
                self.detail.items.clear();
                self.detail.vulns.clear();
                self.detail.audits.clear();
            }
            _ => {
                let items = scanner::extract_outdated(res);
                self.detail.items = items;
                self.detail.vulns.clear();
                self.detail.audits.clear();
                self.detail.cleanup.clear();
            }
        }

        self.ui.view = View::PackageDetail;
    }

    fn do_detail_updates(
        &mut self,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tool_key = match &self.detail.key {
            Some(k) => k.clone(),
            None => return Ok(()),
        };
        let tool_display = match &self.detail.tool {
            Some(t) => t.clone(),
            None => return Ok(()),
        };
        let work: Vec<(String, String, OutdatedItem)> = self
            .detail
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.detail.checked.contains(i))
            .map(|(_, item)| (tool_key.clone(), tool_display.clone(), item.clone()))
            .collect();
        self.detail.checked.clear();
        if work.is_empty() {
            return Ok(());
        }

        self.ui.view = View::Updating;
        self.ui.progress_counter = 0;
        self.ui.update_package_name = String::new();
        self.ui.update_downloaded_mb = 0.0;
        let _count = work.len();

        tokio::spawn(async move {
            let mut updated = 0usize;
            let mut failed = 0usize;
            let mut errors = vec![];
            let mut successful_updates = vec![];
            for (tk, td, item) in &work {
                match run_update(td, item, tx.clone()).await {
                    Ok(_) => {
                        updated += 1;
                        successful_updates.push((
                            tk.clone(),
                            item.name.clone(),
                            item.source.clone(),
                        ));
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.name, e));
                    }
                }
            }
            let result_msg = if errors.is_empty() {
                format!("\u{2714} Updated {updated} package(s)")
            } else {
                let e = errors.join("; ");
                format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}")
            };
            let _ = tx.send(AppEvent::UpdateFinished {
                result_msg,
                updated_packages: successful_updates,
            });
        });
        Ok(())
    }

    fn do_detail_cleanups(
        &mut self,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let work: Vec<CleanupItem> = self
            .detail
            .cleanup
            .iter()
            .enumerate()
            .filter(|(i, _)| self.detail.checked.contains(i))
            .map(|(_, item)| item.clone())
            .collect();
        self.detail.checked.clear();
        if work.is_empty() {
            return Ok(());
        }

        self.ui.view = View::Updating;
        self.ui.progress_counter = 0;

        tokio::spawn(async move {
            let mut cleaned = 0usize;
            let mut failed = 0usize;
            let mut errors = vec![];
            for item in &work {
                if let Some(ref cmd_str) = item.command {
                    let mut command = tokio::process::Command::new("sh");
                    command.arg("-c").arg(cmd_str);
                    match command.output().await {
                        Ok(output) if output.status.success() => cleaned += 1,
                        Ok(output) => {
                            failed += 1;
                            let err_msg =
                                String::from_utf8_lossy(&output.stderr).trim().to_string();
                            errors.push(format!("{}: {}", item.description, err_msg));
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("{}: {}", item.description, e));
                        }
                    }
                } else {
                    failed += 1;
                    errors.push(format!("{}: no command configured", item.description));
                }
            }
            let result_msg = if errors.is_empty() {
                format!("\u{2714} Successfully cleaned up {cleaned} item(s)")
            } else {
                let e = errors.join("; ");
                format!("\u{2714} Cleaned up {cleaned} | \u{2716} Failed {failed}: {e}")
            };

            let new_report_res = toolchains::scan_one("cleanup").await;
            let _ = tx.send(AppEvent::CleanupFinished {
                result_msg,
                new_report_res,
            });
        });
        Ok(())
    }

    fn do_detail_security_fixes(
        &mut self,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.detail.vulns.is_empty() {
            return Ok(());
        }

        self.ui.view = View::Updating;
        self.ui.progress_counter = 0;

        tokio::spawn(async move {
            let project_path = toolchains::get_project_path();
            let mut fixed = 0usize;
            let mut errors = vec![];

            if project_path.join("package.json").exists() && toolchains::which("npm") {
                let mut cmd = tokio::process::Command::new("npm");
                cmd.current_dir(&project_path).args(["audit", "fix"]);
                match cmd.output().await {
                    Ok(out) if out.status.success() => fixed += 1,
                    Ok(out) => errors.push(format!(
                        "npm audit fix: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    )),
                    Err(e) => errors.push(format!("npm audit fix error: {}", e)),
                }
            }

            if project_path.join("Cargo.toml").exists() && toolchains::which("cargo") {
                let mut cmd = tokio::process::Command::new("cargo");
                cmd.current_dir(&project_path).args(["update"]);
                match cmd.output().await {
                    Ok(out) if out.status.success() => fixed += 1,
                    Ok(out) => errors.push(format!(
                        "cargo update: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    )),
                    Err(e) => errors.push(format!("cargo update error: {}", e)),
                }
            }

            let result_msg = if errors.is_empty() {
                if fixed > 0 {
                    "\u{2714} Successfully ran automated security fixes".to_string()
                } else {
                    "\u{2714} No automated fixes supported for this project type".to_string()
                }
            } else {
                let e = errors.join("; ");
                format!("\u{2716} Fix attempted with errors: {e}")
            };

            let new_report_res = toolchains::scan_one("security").await;
            let _ = tx.send(AppEvent::SecurityFixFinished {
                result_msg,
                new_report_res,
            });
        });
        Ok(())
    }

    fn collect_checked_work(&self) -> Vec<(String, String, OutdatedItem)> {
        let report = match &self.report {
            Some(r) => r,
            None => return vec![],
        };
        let mut work = vec![];
        let mut idx = 0usize;
        for tool in &scanner::tool_order() {
            if let Some(res) = report.results.get(*tool) {
                for item in &scanner::extract_outdated(res) {
                    if self.ui.checked_outdated.contains(&idx) {
                        work.push((
                            tool.to_string(),
                            scanner::display_name(tool).to_string(),
                            item.clone(),
                        ));
                    }
                    idx += 1;
                }
            }
        }
        work
    }

    fn do_checked_updates(
        &mut self,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let work = self.collect_checked_work();
        self.ui.checked_outdated.clear();
        if work.is_empty() {
            return Ok(());
        }

        self.ui.view = View::Updating;
        self.ui.progress_counter = 0;
        self.ui.update_package_name = String::new();
        self.ui.update_downloaded_mb = 0.0;
        let _count = work.len();

        tokio::spawn(async move {
            let mut updated = 0usize;
            let mut failed = 0usize;
            let mut errors = vec![];
            let mut successful_updates = vec![];
            for (tk, td, item) in &work {
                match run_update(td, item, tx.clone()).await {
                    Ok(_) => {
                        updated += 1;
                        successful_updates.push((
                            tk.clone(),
                            item.name.clone(),
                            item.source.clone(),
                        ));
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.name, e));
                    }
                }
            }
            let result_msg = if errors.is_empty() {
                format!("\u{2714} Updated {updated} package(s)")
            } else {
                let e = errors.join("; ");
                format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}")
            };
            let _ = tx.send(AppEvent::UpdateFinished {
                result_msg,
                updated_packages: successful_updates,
            });
        });
        Ok(())
    }

    fn filtered_tools(&self) -> Vec<&'static str> {
        let report = match &self.report {
            Some(r) => r,
            None => return vec![],
        };
        let q = self.ui.search_query.to_lowercase();
        let mut tools = vec![];
        for cat in scanner::tool_categories() {
            for tool in cat.tools {
                let matches_search = if q.is_empty() || !self.ui.search_mode {
                    true
                } else {
                    let name = scanner::display_name(tool).to_lowercase();
                    name.contains(&q) || tool.contains(&q)
                };
                if matches_search && report.results.contains_key(*tool) {
                    tools.push(*tool);
                }
            }
        }
        tools
    }

    fn clamp_selection(&mut self) {
        match self.ui.view {
            View::Dashboard => {
                let n = self.filtered_tools().len().saturating_sub(1);
                self.ui.dashboard_selection = self.ui.dashboard_selection.min(n);
            }
            View::Outdated => {
                let n: usize = self
                    .report
                    .as_ref()
                    .map(|r| {
                        let q = self.ui.search_query.to_lowercase();
                        let mut count = 0usize;
                        for tool in &scanner::tool_order() {
                            if let Some(res) = r.results.get(*tool) {
                                for item in &scanner::extract_outdated(res) {
                                    if q.is_empty() || !self.ui.search_mode {
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
                self.ui.outdated_selection = self.ui.outdated_selection.min(n);
            }
            View::Scanning => {}
            View::PackageDetail => {
                let n = self.detail_len().saturating_sub(1);
                self.detail.selection = self.detail.selection.min(n);
            }
            View::Updating => {}
        }
    }

    fn do_scan(
        &mut self,
        tx: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.view = View::Scanning;
        self.ui.progress_counter = 0;

        tokio::spawn(async move {
            let results = toolchains::scan_all().await;
            let _ = tx.send(AppEvent::ScanFinished(results));
        });
        Ok(())
    }

    fn next_item(&mut self) {
        match self.ui.view {
            View::Dashboard => {
                let n = self.filtered_tools().len().saturating_sub(1);
                self.ui.dashboard_selection = self.ui.dashboard_selection.saturating_add(1).min(n);
            }
            View::Outdated => {
                let n = self
                    .report
                    .as_ref()
                    .map(scanner::count_outdated)
                    .unwrap_or(0)
                    .saturating_sub(1);
                self.ui.outdated_selection = self.ui.outdated_selection.saturating_add(1).min(n);
            }
            View::Scanning => {}
            View::PackageDetail => {
                let n = self.detail_len().saturating_sub(1);
                self.detail.selection = self.detail.selection.saturating_add(1).min(n);
            }
            View::Updating => {}
        }
    }

    fn prev_item(&mut self) {
        match self.ui.view {
            View::Dashboard => {
                self.ui.dashboard_selection = self.ui.dashboard_selection.saturating_sub(1)
            }
            View::Outdated => {
                self.ui.outdated_selection = self.ui.outdated_selection.saturating_sub(1)
            }
            View::Scanning => {}
            View::PackageDetail => self.detail.selection = self.detail.selection.saturating_sub(1),
            View::Updating => {}
        }
    }

    fn detail_len(&self) -> usize {
        match self.detail.key.as_deref() {
            Some("security") => self.detail.vulns.len(),
            Some("audit") => self.detail.audits.len(),
            Some("cleanup") => self.detail.cleanup.len(),
            _ => self.detail.items.len(),
        }
    }

    fn clear_detail(&mut self) {
        self.detail.items.clear();
        self.detail.vulns.clear();
        self.detail.audits.clear();
        self.detail.cleanup.clear();
        self.detail.checked.clear();
        self.detail.message.clear();
    }

    pub fn export_detail_report(&mut self) {
        let key = match self.detail.key.as_deref() {
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
                    self.detail.vulns.len()
                ));
                out.push_str("| Package | Severity | CVE | Title | Patched In |\n");
                out.push_str("| --- | --- | --- | --- | --- |\n");
                for v in &self.detail.vulns {
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
                    self.detail.audits.len()
                ));
                out.push_str("| Name | Current State | Note / Recommendation |\n");
                out.push_str("| --- | --- | --- |\n");
                for a in &self.detail.audits {
                    out.push_str(&format!("| {} | {} | {} |\n", a.name, a.current, a.note));
                }
                (filename, out)
            }
            _ => return,
        };

        match std::fs::write(filename, content) {
            Ok(_) => {
                self.detail.message = format!("\u{2714} Exported to {}", filename);
            }
            Err(e) => {
                self.detail.message = format!("\u{2716} Export failed: {}", e);
            }
        }
    }
}

async fn run_update(
    tool: &str,
    item: &OutdatedItem,
    tx: mpsc::UnboundedSender<AppEvent>,
) -> Result<String, String> {
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

    let _ = tx.send(AppEvent::UpdateProgress {
        package_name: item.name.clone(),
        downloaded_mb: 0.0,
    });

    let mut command = tokio::process::Command::new(&cmd);
    command.args(&args);
    if run_in_project {
        command.current_dir(&project_path);
    }
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| format!("command failed to spawn: {e}"))?;

    use tokio::io::{AsyncBufReadExt, BufReader};
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to open stderr".to_string())?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut total_downloaded_mb = 0.0;
    let mut stderr_accum = String::new();
    let mut stdout_accum = String::new();

    let name_clone = item.name.clone();

    loop {
        tokio::select! {
            res = stdout_reader.next_line() => {
                match res {
                    Ok(Some(line)) => {
                        if let Some(mb) = parse_downloaded_mb(&line) {
                            if mb > total_downloaded_mb {
                                total_downloaded_mb = mb;
                                let _ = tx.send(AppEvent::UpdateProgress {
                                    package_name: name_clone.clone(),
                                    downloaded_mb: total_downloaded_mb,
                                });
                            }
                        }
                        if contains_password_prompt(&line) {
                            let _ = child.kill().await;
                            return Err("Requires interactive password/sudo input, which is not supported in background updates. Run manually in terminal.".into());
                        }
                        if stdout_accum.len() < 1000 {
                            if !stdout_accum.is_empty() {
                                stdout_accum.push('\n');
                            }
                            stdout_accum.push_str(&line);
                        }
                    }
                    Ok(None) => {},
                    Err(_) => {}
                }
            }
            res = stderr_reader.next_line() => {
                match res {
                    Ok(Some(line)) => {
                        if let Some(mb) = parse_downloaded_mb(&line) {
                            if mb > total_downloaded_mb {
                                total_downloaded_mb = mb;
                                let _ = tx.send(AppEvent::UpdateProgress {
                                    package_name: name_clone.clone(),
                                    downloaded_mb: total_downloaded_mb,
                                });
                            }
                        }
                        if contains_password_prompt(&line) {
                            let _ = child.kill().await;
                            return Err("Requires interactive password/sudo input, which is not supported in background updates. Run manually in terminal.".into());
                        }
                        if stderr_accum.len() < 1000 {
                            if !stderr_accum.is_empty() {
                                stderr_accum.push('\n');
                            }
                            stderr_accum.push_str(&line);
                        }
                    }
                    Ok(None) => {},
                    Err(_) => {}
                }
            }
            status = child.wait() => {
                let exit_status = status.map_err(|e| format!("failed to wait for child: {e}"))?;
                if exit_status.success() {
                    let combined = if !stdout_accum.is_empty() { stdout_accum } else { "done".to_string() };
                    return Ok(combined);
                } else {
                    let err_msg = if !stderr_accum.is_empty() {
                        stderr_accum
                    } else if !stdout_accum.is_empty() {
                        stdout_accum
                    } else {
                        format!("exit code: {:?}", exit_status.code())
                    };
                    return Err(err_msg);
                }
            }
        }
    }
}

fn parse_downloaded_mb(line: &str) -> Option<f64> {
    use regex::Regex;
    thread_local! {
        static RE: Regex = Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(MB|MiB|KB|KiB|GB|GiB|M|K|G)\b").unwrap();
    }
    RE.with(|re| {
        if let Some(caps) = re.captures(line) {
            if let (Some(num_str), Some(unit)) = (caps.get(1), caps.get(2)) {
                if let Ok(num) = num_str.as_str().parse::<f64>() {
                    let unit_lower = unit.as_str().to_lowercase();
                    if unit_lower.starts_with('g') {
                        return Some(num * 1024.0);
                    } else if unit_lower.starts_with('m') {
                        return Some(num);
                    } else if unit_lower.starts_with('k') {
                        return Some(num / 1024.0);
                    }
                }
            }
        }
        None
    })
}

fn contains_password_prompt(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("password:")
        || lower.contains("[sudo] password")
        || lower.contains("enter password")
}
