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

fn source_style(source: &str) -> Style {
    match source {
        "formula" => Style::default().fg(Color::Blue),
        "cask" => Style::default().fg(Color::Magenta),
        "global" => Style::default().fg(Color::Cyan),
        "package" => Style::default().fg(Color::DarkGray),
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

fn status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let bar = if app.search_mode {
        let query = format!(" / {}█", app.search_query);
        Line::from(vec![
            Span::styled("Search:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(query),
            Span::styled("  Esc", Style::default().fg(Color::DarkGray)),
            Span::raw(" clear"),
        ])
    } else {
        let hints = vec![
            Span::styled(" [S]", Style::default().fg(Color::Green)),
            Span::raw("can "),
            Span::styled("[O]", Style::default().fg(Color::Yellow)),
            Span::raw("utdated "),
            Span::styled("[/]", Style::default().fg(Color::Cyan)),
            Span::raw("Search "),
            Span::styled("\u{2190}\u{2192}", Style::default().fg(Color::DarkGray)),
            Span::raw(" tabs "),
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::DarkGray)),
            Span::raw(" nav  "),
            Span::styled("^C", Style::default().fg(Color::Red)),
            Span::styled(" Exit", Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled("[Q]", Style::default().fg(Color::DarkGray)),
            Span::raw("uit"),
        ];
        Line::from(hints)
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(bar).block(block), area);
}

fn truncated_cell(text: &str, max: usize) -> Cell<'static> {
    let display = if text.len() > max {
        format!("{}…", &text[..max.saturating_sub(1)])
    } else {
        text.to_string()
    };
    Cell::from(display)
}

fn count_statuses(report: &crate::scanner::Report) -> (usize, usize, usize, usize) {
    let mut pass = 0usize;
    let mut warn = 0;
    let mut fail = 0;
    let mut skip = 0;
    for tool in &crate::scanner::tool_order() {
        if let Some(res) = report.results.get(*tool) {
            match res.status.as_str() {
                "ok" => pass += 1,
                "warning" => warn += 1,
                "error" => fail += 1,
                "skipped" => skip += 1,
                _ => {}
            }
        }
    }
    (pass, warn, fail, skip)
}

fn scan_age(timestamp: &str) -> String {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S") {
        let elapsed = chrono::Local::now().naive_local() - dt;
        let mins = elapsed.num_minutes();
        if mins < 1 {
            "just now".into()
        } else if mins < 60 {
            format!("{mins}m ago")
        } else {
            format!("{}h ago", elapsed.num_hours())
        }
    } else {
        String::new()
    }
}

fn dashboard_stats_line(frame: &mut Frame, area: Rect, report: &crate::scanner::Report) {
    let (pass, warn, fail, skip) = count_statuses(report);
    let outdated = crate::scanner::count_outdated(report);
    let age = scan_age(&report.timestamp);
    let items = vec![
        Span::styled(format!(" \u{25CF} {pass} "), Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled(format!("\u{25CF} {warn} "), Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(format!("\u{25CF} {fail} "), Style::default().fg(Color::Red)),
        Span::raw(" "),
        Span::styled(format!("\u{25CF} {skip} "), Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            format!("\u{25C9} {outdated} outdated"),
            Style::default().fg(if outdated > 0 { Color::Yellow } else { Color::Green }),
        ),
        Span::raw("  "),
        Span::styled(
            format!("\u{23F0} {age}"),
            Style::default().fg(Color::DarkGray),
        ),
    ];
    let block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(Paragraph::new(Line::from(items)).block(block), area);
}

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "\u{2728} Envexa",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Scan your dev environment to get started."),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  \u{25B6} Press [S]",
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" to scan all toolchains"),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  \u{25B6} Press [O]",
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" to view outdated packages"),
                ]),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Welcome ")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            frame.render_widget(text, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    dashboard_stats_line(frame, chunks[0], report);

    let header_cells = ["", " Toolchain ", " Status ", " Version ", " Outdated ", " Issues "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let q = app.search_query.to_lowercase();
    let rows: Vec<Row> = scanner::tool_order()
        .iter()
        .filter_map(|tool| {
            let res = report.results.get(*tool)?;
            if !q.is_empty() && app.search_mode {
                let name = scanner::display_name(tool).to_lowercase();
                if !name.contains(&q) && !tool.contains(&q) {
                    return None;
                }
            }
            let display = scanner::display_name(tool);
            let label = scanner::status_label(&res.status);
            let style = status_style(&res.status);
            let ver = scanner::first_version(res);
            let outdated_count = scanner::extract_outdated(res).len();
            let outdated_str = if outdated_count > 0 {
                outdated_count.to_string()
            } else {
                String::new()
            };
            let issues_str = res
                .issues
                .first()
                .map(|s| s.as_str())
                .unwrap_or("");
            Some((tool, display, label, style, ver, outdated_str, issues_str))
        })
        .enumerate()
        .map(|(i, (_tool, display, label, style, ver, outdated_str, issues_str))| {
            let sel = i == app.dashboard_selection;
            let indicator = if sel { "\u{25b8} " } else { "  " };
            let row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(display),
                Cell::from(label).style(style),
                Cell::from(ver),
                truncated_cell(&outdated_str, 8),
                truncated_cell(issues_str, 20),
            ])
            .height(1);
            if sel {
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
    let filtered = app.search_mode && !q.is_empty();
    let count = rows.len();
    let subtitle = if filtered {
        format!(" Dashboard  ({count} matched) ")
    } else if total_outdated > 0 {
        format!(" Dashboard  |  {total_outdated} outdated ")
    } else {
        " Dashboard  |  all up to date ".into()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Min(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(subtitle)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .column_spacing(1);

    frame.render_widget(table, chunks[1]);
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

    let header_cells = ["", " Toolchain ", " Source ", " Package ", " Current ", " Latest "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let q = app.search_query.to_lowercase();
    let mut items: Vec<(String, scanner::OutdatedItem)> = Vec::new();
    for tool in &scanner::tool_order() {
        if let Some(res) = report.results.get(*tool) {
            let pkgs = scanner::extract_outdated(res);
            if !pkgs.is_empty() {
                let display = scanner::display_name(tool).to_string();
                for pkg in pkgs {
                    if !q.is_empty() && app.search_mode {
                        let tool_lower = display.to_lowercase();
                        if !tool_lower.contains(&q)
                            && !pkg.name.to_lowercase().contains(&q)
                            && !pkg.source.contains(&q)
                        {
                            continue;
                        }
                    }
                    items.push((display.clone(), pkg));
                }
            }
        }
    }

    if items.is_empty() {
        let msg = if app.search_mode && !q.is_empty() {
            format!("  No packages match \"{q}\" ")
        } else {
            "  All packages are up to date! ".into()
        };
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                msg,
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
        .map(|(i, (tool, pkg))| {
            let sel = i == app.outdated_selection;
            let indicator = if sel { "\u{25b8} " } else { "  " };
            let row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(tool.as_str()),
                Cell::from(pkg.source.as_str()).style(source_style(&pkg.source)),
                Cell::from(pkg.name.as_str()),
                Cell::from(pkg.current.as_str()),
                Cell::from(pkg.latest.as_str()),
            ]);
            if sel {
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
    let title = if app.search_mode && !q.is_empty() {
        format!(" Outdated Packages ({total} matched) ")
    } else {
        format!(" Outdated Packages ({total}) ")
    };
    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
            Constraint::Length(18),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
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

    status_bar(frame, chunks[3], app);
}
