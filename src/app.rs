use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::config;
use crate::scanner::{self, Report};
use crate::toolchains;

pub enum View {
    Dashboard,
    Outdated,
    Scanning,
}

pub struct App {
    pub report: Option<Report>,
    pub view: View,
    pub dashboard_selection: usize,
    pub outdated_selection: usize,
    pub tab_index: usize,
    pub search_mode: bool,
    pub search_query: String,
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
                    _ => {}
                }
            }
        }

        ratatui::restore();
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
                                        let tool_name =
                                            scanner::display_name(tool).to_lowercase();
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
        }
    }

    fn do_scan(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.view = View::Scanning;
        terminal.draw(|frame| crate::ui::render(frame, self))?;

        let results = std::thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .expect("Failed to create scan runtime")
                .block_on(toolchains::scan_all())
        })
        .join()
        .expect("Scan thread panicked");

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
                let n = self
                    .report
                    .as_ref()
                    .map_or(0, |r| r.results.len().saturating_sub(1));
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
        }
    }

    fn prev_item(&mut self) {
        match self.view {
            View::Dashboard => {
                self.dashboard_selection = self.dashboard_selection.saturating_sub(1)
            }
            View::Outdated => self.outdated_selection = self.outdated_selection.saturating_sub(1),
            View::Scanning => {}
        }
    }
}
