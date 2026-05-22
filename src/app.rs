use std::collections::HashSet;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use throbber_widgets_tui::ThrobberState;

use crate::config;
use crate::scanner::{self, OutdatedItem, Report};
use crate::toolchains;

pub enum View {
    Dashboard,
    Outdated,
    Scanning,
    PackageDetail,
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

    pub detail_tool: Option<String>,
    pub detail_selection: usize,
    pub detail_items: Vec<OutdatedItem>,
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
            detail_tool: None,
            detail_selection: 0,
            detail_items: Vec::new(),
            detail_checked: HashSet::new(),
            detail_message: String::new(),
            checked_outdated: HashSet::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        loop {
            terminal.draw(|frame| crate::ui::render(frame, self))?;

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
                        KeyCode::Char(' ') => {
                            if self.detail_checked.contains(&self.detail_selection) {
                                self.detail_checked.remove(&self.detail_selection);
                            } else {
                                self.detail_checked.insert(self.detail_selection);
                            }
                        }
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            self.run_detail_updates();
                        }
                        KeyCode::Down | KeyCode::Char('n') => {
                            let n = self.detail_items.len().saturating_sub(1);
                            self.detail_selection = self.detail_selection.saturating_add(1).min(n);
                        }
                        KeyCode::Up | KeyCode::Char('p') => {
                            self.detail_selection = self.detail_selection.saturating_sub(1);
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.view = View::Dashboard;
                            self.tab_index = 0;
                            self.detail_tool = None;
                            self.detail_items.clear();
                            self.detail_checked.clear();
                            self.detail_message.clear();
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
                        let msg = self.run_checked_updates();
                        self.detail_message = msg;
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
        let items = scanner::extract_outdated(res);
        if items.is_empty() {
            return;
        }
        self.detail_tool = Some(scanner::display_name(tool).to_string());
        self.detail_selection = 0;
        self.detail_items = items;
        self.detail_checked.clear();
        self.detail_message.clear();
        self.view = View::PackageDetail;
    }

    fn run_detail_updates(&mut self) {
        let tool = match &self.detail_tool {
            Some(t) => t.clone(),
            None => return,
        };
        let mut updated = 0usize;
        let mut failed = 0usize;
        let mut errors = vec![];
        for (i, item) in self.detail_items.iter().enumerate() {
            if self.detail_checked.contains(&i) {
                match run_update(&tool, item) {
                    Ok(_) => updated += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.name, e));
                    }
                }
            }
        }
        self.detail_checked.clear();
        if errors.is_empty() {
            self.detail_message = format!("\u{2714} Updated {updated} package(s)");
        } else {
            let e = errors.join("; ");
            self.detail_message =
                format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}");
        }
    }

    fn run_checked_updates(&mut self) -> String {
        let report = match &self.report {
            Some(r) => r,
            None => return "No scan data".into(),
        };
        let mut idx = 0usize;
        let mut updated = 0usize;
        let mut failed = 0usize;
        let mut errors = vec![];
        for tool in &scanner::tool_order() {
            if let Some(res) = report.results.get(*tool) {
                for item in &scanner::extract_outdated(res) {
                    if self.checked_outdated.contains(&idx) {
                        let tool_display = scanner::display_name(tool).to_string();
                        match run_update(&tool_display, item) {
                            Ok(_) => updated += 1,
                            Err(e) => {
                                failed += 1;
                                errors.push(format!("{}: {}", item.name, e));
                            }
                        }
                    }
                    idx += 1;
                }
            }
        }
        self.checked_outdated.clear();
        if errors.is_empty() {
            format!("\u{2714} Updated {updated} package(s)")
        } else {
            let e = errors.join("; ");
            format!("\u{2714} Updated {updated} | \u{2716} Failed {failed}: {e}")
        }
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
                let n = self.detail_items.len().saturating_sub(1);
                self.detail_selection = self.detail_selection.min(n);
            }
        }
    }

    fn do_scan(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.view = View::Scanning;

        let handle = std::thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .expect("Failed to create scan runtime")
                .block_on(toolchains::scan_all())
        });

        loop {
            terminal.draw(|frame| crate::ui::render(frame, self))?;
            self.throbber_state.calc_next();
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

        terminal.draw(|frame| crate::ui::render(frame, self))?;
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
                let n = self.detail_items.len().saturating_sub(1);
                self.detail_selection = self.detail_selection.saturating_add(1).min(n);
            }
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
        }
    }
}

fn run_update(tool: &str, item: &OutdatedItem) -> Result<String, String> {
    let (cmd, args) = match tool {
        "Brew" | "Brew (dev)" => {
            let mut args = vec!["upgrade"];
            if item.source == "cask" {
                args.push("--cask");
            }
            args.push(&item.name);
            ("brew", args)
        }
        _ => return Err("auto-update not supported for this toolchain".into()),
    };
    let output = std::process::Command::new(cmd)
        .args(&args)
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
