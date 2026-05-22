use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::app::{App, View};
use crate::scanner;

fn status_style(status: &str) -> Style {
    match status {
        "ok" => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        "warning" => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        "error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "skipped" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}

fn title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let title = Span::styled(
        " Envexa ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let version = Span::styled(
        concat!(" v", env!("CARGO_PKG_VERSION")),
        Style::default().fg(Color::DarkGray),
    );
    let cache_status = match &app.report {
        Some(r) => {
            let n = scanner::count_outdated(r);
            if n > 0 {
                Span::styled(
                    format!(" {n} outdated "),
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                )
            } else {
                Span::styled(" up to date ", Style::default().fg(Color::Green))
            }
        }
        None => Span::styled(" no data ", Style::default().fg(Color::DarkGray)),
    };
    let bar = Line::from(vec![title, version, Span::raw("  "), cache_status]);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(bar).block(block), area);
}

fn tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let titles = vec![" Dashboard ", " Outdated "];
    let selected = match app.view {
        View::Dashboard => 0,
        View::Outdated => 1,
        View::Scanning => app.tab_index,
    };
    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(tabs, area);
}

fn key_hints(frame: &mut Frame, area: Rect) {
    let hints = vec![
        Span::styled(" S", Style::default().fg(Color::Green)),
        Span::raw("can "),
        Span::styled("O", Style::default().fg(Color::Yellow)),
        Span::raw("utdated "),
        Span::styled("\u{2190}\u{2192}", Style::default().fg(Color::DarkGray)),
        Span::raw(" tabs "),
        Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::DarkGray)),
        Span::raw(" nav "),
        Span::styled("Q", Style::default().fg(Color::Red)),
        Span::raw("uit"),
    ];
    let bar = Line::from(hints);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(bar).block(block), area);
}

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Welcome to "),
                    Span::styled(
                        "Envexa",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Press "),
                    Span::styled(
                        "S",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to scan your dev environment."),
                ]),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            frame.render_widget(text, area);
            return;
        }
    };

    let header_cells = [" Toolchain ", " Status ", " Version "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = scanner::tool_order()
        .iter()
        .filter_map(|tool| {
            let res = report.results.get(*tool)?;
            let display = scanner::display_name(tool);
            let label = scanner::status_label(&res.status);
            let style = status_style(&res.status);
            let ver = scanner::first_version(res);
            Some(
                Row::new(vec![
                    Cell::from(display),
                    Cell::from(label).style(style),
                    Cell::from(ver),
                ])
                .height(1),
            )
        })
        .enumerate()
        .map(|(i, row)| {
            if i == app.dashboard_selection {
                row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect();

    let total_outdated = scanner::count_outdated(report);
    let subtitle = if total_outdated > 0 {
        format!(" Dashboard  |  {total_outdated} outdated ")
    } else {
        " Dashboard  |  all up to date ".into()
    };

    let table = Table::new(rows, Constraint::from_lengths([15, 10, 30]))
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(subtitle)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(2);

    frame.render_widget(table, area);
}

fn render_outdated(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            frame.render_widget(
                Paragraph::new("No scan data. Press S to scan first.")
                    .block(Block::default().borders(Borders::ALL)),
                area,
            );
            return;
        }
    };

    let header_cells = [" Toolchain ", " Package ", " Current ", " Latest "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let mut items: Vec<(String, String, String, String)> = Vec::new();
    for tool in &scanner::tool_order() {
        if let Some(res) = report.results.get(*tool) {
            let pkgs = scanner::extract_outdated(res);
            if !pkgs.is_empty() {
                let display = scanner::display_name(tool);
                for pkg in &pkgs {
                    items.push((
                        display.to_string(),
                        pkg.name.clone(),
                        pkg.current.clone(),
                        pkg.latest.clone(),
                    ));
                }
            }
        }
    }

    if items.is_empty() {
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  All packages are up to date! ",
                Style::default().fg(Color::Green),
            )]),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Outdated Packages ")
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(text, area);
        return;
    }

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, (tool, name, cur, lat))| {
            let row = Row::new(vec![
                Cell::from(tool.as_str()),
                Cell::from(name.as_str()),
                Cell::from(cur.as_str()),
                Cell::from(lat.as_str()),
            ]);
            if i == app.outdated_selection {
                row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect();

    let total = items.len();
    let table = Table::new(rows, Constraint::from_lengths([12, 26, 22, 22]))
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Outdated Packages ({total}) "))
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_scanning(frame: &mut Frame, area: Rect, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let text = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Scanning ",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(""),
        Line::from("  Checking Homebrew, npm, pnpm, Yarn, Bun, Deno,"),
        Line::from("  pip, Gem, Cargo, and Docker..."),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Envexa ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(text, chunks[0]);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Progress ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent(50)
        .label("running...");
    frame.render_widget(gauge, chunks[1]);
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    title_bar(frame, chunks[0], app);
    tab_bar(frame, chunks[1], app);

    match app.view {
        View::Dashboard => render_dashboard(frame, chunks[2], app),
        View::Outdated => render_outdated(frame, chunks[2], app),
        View::Scanning => render_scanning(frame, chunks[2], app),
    }

    key_hints(frame, chunks[3]);
}
