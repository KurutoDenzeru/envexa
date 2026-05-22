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
}

impl App {
    pub fn new() -> Self {
        let report = config::read_cache().map(|e| e.report);
        Self {
            report,
            view: View::Dashboard,
            dashboard_selection: 0,
            outdated_selection: 0,
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
                    KeyCode::Esc => self.view = View::Dashboard,
                    KeyCode::Char('s') => self.do_scan(&mut terminal)?,
                    KeyCode::Char('o') => self.view = View::Outdated,
                    KeyCode::Char('h') => self.view = View::Dashboard,
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.prev(),
                    KeyCode::Enter => self.select(),
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
        std::thread::sleep(Duration::from_millis(200));
        terminal.draw(|frame| crate::ui::render(frame, self))?;
        Ok(())
    }

    fn next(&mut self) {
        let n = self.items_count();
        match self.view {
            View::Dashboard => {
                self.dashboard_selection = self.dashboard_selection.saturating_add(1).min(n)
            }
            View::Outdated => {
                self.outdated_selection = self.outdated_selection.saturating_add(1).min(n)
            }
            View::Scanning => {}
        }
    }

    fn prev(&mut self) {
        match self.view {
            View::Dashboard => {
                self.dashboard_selection = self.dashboard_selection.saturating_sub(1)
            }
            View::Outdated => self.outdated_selection = self.outdated_selection.saturating_sub(1),
            View::Scanning => {}
        }
    }

    fn select(&mut self) {}

    fn items_count(&self) -> usize {
        match self.view {
            View::Dashboard => self
                .report
                .as_ref()
                .map_or(0, |r| r.results.len().saturating_sub(1)),
            View::Outdated => self
                .report
                .as_ref()
                .map(scanner::count_outdated)
                .unwrap_or(0)
                .saturating_sub(1),
            View::Scanning => 0,
        }
    }
}
