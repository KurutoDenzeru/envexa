use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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
                    KeyCode::Char('q') => break,
                    KeyCode::Esc | KeyCode::Char('h') => {
                        self.view = View::Dashboard;
                        self.tab_index = 0;
                    }
                    KeyCode::Char('s') => self.do_scan(&mut terminal)?,
                    KeyCode::Char('o') => {
                        self.view = View::Outdated;
                        self.tab_index = 1;
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
                    KeyCode::Down => self.next_item(),
                    KeyCode::Up => self.prev_item(),
                    _ => {}
                }
            }
        }

        ratatui::restore();
        Ok(())
    }

    fn do_scan(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.view = View::Scanning;
        terminal.draw(|frame| crate::ui::render(frame, self))?;

        let results =
            tokio::runtime::Handle::current().block_on(async { toolchains::scan_all().await });

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
