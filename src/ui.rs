use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, View};
use crate::scanner;

fn status_style(status: &str) -> Style {
    match status {
        "ok" => Style::default().fg(Color::Green),
        "warning" => Style::default().fg(Color::Yellow),
        "error" => Style::default().fg(Color::Red),
        "skipped" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}

fn header(frame: &mut Frame, area: Rect) {
    let title = Span::styled(
        " Envexa ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    let hint = vec![
        Span::styled(" [S]", Style::default().fg(Color::Green)),
        Span::raw("can "),
        Span::styled("[O]", Style::default().fg(Color::Yellow)),
        Span::raw("utdated "),
        Span::styled("[H]", Style::default().fg(Color::Cyan)),
        Span::raw("ome "),
        Span::styled("[Q]", Style::default().fg(Color::Red)),
        Span::raw("uit"),
    ];
    let mut spans = vec![title];
    spans.extend(hint);
    let header = Paragraph::new(Line::from(spans));
    frame.render_widget(header, area);
}

fn footer(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(
            " j/k/\u{2191}\u{2193}",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" navigate", Style::default().fg(Color::DarkGray)),
        Span::styled("  Enter", Style::default().fg(Color::Yellow)),
        Span::styled(" select", Style::default().fg(Color::DarkGray)),
    ]);
    let footer = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, area);
}

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            let text = Paragraph::new(Text::from(vec![
                Line::from("No scan data yet."),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "S",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to start a scan."),
                ]),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
            frame.render_widget(text, area);
            return;
        }
    };

    let header_cells = ["Toolchain", "Status", "Version"]
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
                row.style(Style::default().bg(Color::DarkGray))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(rows, Constraint::from_lengths([15, 10, 25]))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Dashboard "))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(table, area);
}

fn render_outdated(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            let text = Paragraph::new("No scan data. Press S to scan first.");
            frame.render_widget(text, area);
            return;
        }
    };

    let header_cells = ["Toolchain", "Package", "Current", "Latest"]
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
                row.style(Style::default().bg(Color::DarkGray))
            } else {
                row
            }
        })
        .collect();

    let total = items.len();
    let subtitle = format!(" Outdated Packages ({total}) ");
    let table = Table::new(rows, Constraint::from_lengths([12, 26, 22, 22]))
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(subtitle))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(table, area);
}

fn render_scanning(frame: &mut Frame, area: Rect, _app: &App) {
    let text = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from("  Scanning your dev environment..."),
        Line::from(""),
        Line::from("  This usually takes 3-4 seconds."),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Scanning "))
    .style(Style::default().fg(Color::Cyan));
    frame.render_widget(text, area);
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    header(frame, chunks[0]);

    match app.view {
        View::Dashboard => render_dashboard(frame, chunks[1], app),
        View::Outdated => render_outdated(frame, chunks[1], app),
        View::Scanning => render_scanning(frame, chunks[1], app),
    }

    footer(frame, chunks[2]);
}
