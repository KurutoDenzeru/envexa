use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Table, Tabs},
    Frame,
};
use tui_piechart::{LegendAlignment, LegendLayout, LegendPosition, PieChart, PieSlice, Resolution};

use crate::scanner;
use crate::tui::app::{App, View};

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

fn title_bar(frame: &mut Frame, area: Rect, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let art = Paragraph::new(Text::from(ENVEXA_LOGO))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(art, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("By KurutoDenzeru", Style::default().fg(Color::DarkGray)),
            Span::raw(" \u{2502} "),
            Span::styled(
                concat!("v", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .alignment(Alignment::Center),
        chunks[2],
    );

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, chunks[3]);
}

fn tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let titles = vec![" Dashboard ", " Outdated "];
    let selected = match app.view {
        View::Dashboard => 0,
        View::Outdated => 1,
        View::Scanning | View::PackageDetail | View::Updating => app.tab_index,
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
    let (text, style) = match app.view {
        View::Updating => (
            Line::from(vec![Span::styled(
                " Updating packages... ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Style::default().fg(Color::White).bg(Color::Black),
        ),
        View::PackageDetail => {
            let msg = if !app.detail_message.is_empty() {
                format!("  {}", app.detail_message)
            } else {
                String::new()
            };
            (
                Line::from(vec![
                    Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::DarkGray)),
                    Span::raw(" nav "),
                    Span::styled("[Space]", Style::default().fg(Color::Yellow)),
                    Span::raw(" toggle "),
                    Span::styled("[Y]", Style::default().fg(Color::Green)),
                    Span::raw(" update "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" back"),
                    Span::styled(msg, Style::default().fg(Color::White)),
                ]),
                Style::default().fg(Color::White).bg(Color::Black),
            )
        }
        _ if app.search_mode => {
            let query = format!(" / {}█", app.search_query);
            (
                Line::from(vec![
                    Span::styled(
                        "Search:",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(query),
                    Span::styled("  Esc", Style::default().fg(Color::DarkGray)),
                    Span::raw(" clear"),
                ]),
                Style::default(),
            )
        }
        _ => {
            let update_msg = if matches!(app.view, View::Outdated) && !app.detail_message.is_empty()
            {
                format!("  {}", app.detail_message)
            } else {
                String::new()
            };
            (
                Line::from(vec![
                    Span::styled(" [S]", Style::default().fg(Color::Green)),
                    Span::raw("can "),
                    Span::styled("[O]", Style::default().fg(Color::Yellow)),
                    Span::raw("utdated "),
                    Span::styled("[/]", Style::default().fg(Color::Cyan)),
                    Span::raw("earch "),
                    Span::styled("\u{2190}\u{2192}", Style::default().fg(Color::DarkGray)),
                    Span::raw(" tabs "),
                    Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::DarkGray)),
                    Span::raw(" nav "),
                    Span::styled("[U]", Style::default().fg(Color::Green)),
                    Span::raw("pdate "),
                    Span::styled("^C", Style::default().fg(Color::Red)),
                    Span::styled(" Exit", Style::default().fg(Color::Red)),
                    Span::raw("  "),
                    Span::styled("[Q]", Style::default().fg(Color::DarkGray)),
                    Span::raw("uit"),
                    Span::styled(update_msg, Style::default().fg(Color::White)),
                ]),
                Style::default().fg(Color::White).bg(Color::Black),
            )
        }
    };
    let block = Block::default().style(style);
    frame.render_widget(Paragraph::new(text).block(block), area);
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
        Span::styled(
            format!(" \u{25CF} {pass} "),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25CF} {warn} "),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(format!("\u{25CF} {fail} "), Style::default().fg(Color::Red)),
        Span::raw(" "),
        Span::styled(
            format!("\u{25CF} {skip} "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("\u{25C9} {outdated} outdated"),
            Style::default().fg(if outdated > 0 {
                Color::Yellow
            } else {
                Color::Green
            }),
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

const ENVEXA_LOGO: &str = "\
███████╗███╗   ██╗██╗   ██╗███████╗██╗  ██╗ █████╗ \n\
██╔════╝████╗  ██║██║   ██║██╔════╝╚██╗██╔╝██╔══██╗\n\
█████╗  ██╔██╗ ██║██║   ██║█████╗   ╚███╔╝ ███████║\n\
██╔══╝  ██║╚██╗██║╚██╗ ██╔╝██╔══╝   ██╔██╗ ██╔══██║\n\
███████╗██║ ╚████║ ╚████╔╝ ███████╗██╔╝ ██╗██║  ██║\n\
╚══════╝╚═╝  ╚═══╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝";

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            let text = Paragraph::new(Text::from(vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Press [S] to scan your environment",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(vec![
                    Span::styled("[S]", Style::default().fg(Color::Green)),
                    Span::raw(" Scan  "),
                    Span::styled("[O]", Style::default().fg(Color::Yellow)),
                    Span::raw(" Outdated"),
                ]),
            ]))
            .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }
    };

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(46), Constraint::Min(1)])
        .split(area);

    let (pass, warn, fail, skip) = count_statuses(report);

    let pass_label = format!("PASS ({pass})");
    let warn_label = format!("WARN ({warn})");
    let fail_label = format!("FAIL ({fail})");
    let skip_label = format!("SKIP ({skip})");

    let mut slices = Vec::new();
    if pass > 0 {
        slices.push(PieSlice::new(&pass_label, pass as f64, Color::Green));
    }
    if warn > 0 {
        slices.push(PieSlice::new(&warn_label, warn as f64, Color::Yellow));
    }
    if fail > 0 {
        slices.push(PieSlice::new(&fail_label, fail as f64, Color::Red));
    }
    if skip > 0 {
        slices.push(PieSlice::new(&skip_label, skip as f64, Color::DarkGray));
    }

    if slices.is_empty() {
        slices.push(PieSlice::new("EMPTY", 1.0, Color::DarkGray));
    }

    let piechart = PieChart::new(slices)
        .resolution(Resolution::Braille)
        .show_legend(true)
        .legend_position(LegendPosition::Top)
        .legend_layout(LegendLayout::Horizontal)
        .legend_alignment(LegendAlignment::Center)
        .show_percentages(false)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Overview ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(piechart, layout[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(layout[1]);

    let top_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(right_chunks[0]);

    let total = pass + warn + fail + skip;
    let health = if total > 0 {
        pass as f64 / total as f64
    } else {
        0.0
    };

    frame.render_widget(
        LineGauge::default()
            .filled_style(Style::default().fg(Color::Green))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .ratio(health),
        top_panel[0],
    );

    dashboard_stats_line(frame, top_panel[1], report);

    let summary = Paragraph::new(Line::from(vec![
        Span::styled(" [S]", Style::default().fg(Color::Green)),
        Span::raw("can  "),
        Span::styled("[O]", Style::default().fg(Color::Yellow)),
        Span::raw("utdated  "),
        Span::styled("[/]", Style::default().fg(Color::Cyan)),
        Span::raw("Search  "),
        Span::styled("^C", Style::default().fg(Color::Red)),
        Span::raw(" Exit  "),
        Span::styled("[Q]", Style::default().fg(Color::DarkGray)),
        Span::raw("uit"),
    ]))
    .style(Style::default().fg(Color::White))
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(summary, top_panel[2]);

    let q = app.search_query.to_lowercase();
    let mut category_tables: Vec<Table> = Vec::new();
    let mut category_heights: Vec<u16> = Vec::new();
    let mut tool_index = 0;

    for cat in scanner::tool_categories() {
        let visible_tools: Vec<&&str> = cat
            .tools
            .iter()
            .filter(|t| {
                if !report.results.contains_key(**t) {
                    return false;
                }
                if !q.is_empty() && app.search_mode {
                    let name = scanner::display_name(t).to_lowercase();
                    if !name.contains(&q) && !t.contains(&q) {
                        return false;
                    }
                }
                true
            })
            .collect();

        if visible_tools.is_empty() {
            continue;
        }

        let mut rows: Vec<Row> = Vec::new();
        for tool in visible_tools {
            let res = report.results.get(*tool).unwrap();
            let display = scanner::display_name(tool);
            let label = scanner::status_label(&res.status);
            let style = status_style(&res.status);
            let ver = scanner::first_version(res);
            let (outdated_str, issues_str): (String, &str) = match *tool {
                "security" => {
                    let n = res.vulnerabilities.len();
                    (
                        if n > 0 { n.to_string() } else { String::new() },
                        res.vulnerabilities
                            .first()
                            .map(|v| Box::leak(Box::new(format!("{} {}", n, v.severity))) as &str)
                            .unwrap_or(res.issues.first().map(|s| s.as_str()).unwrap_or("")),
                    )
                }
                "audit" => {
                    let n = res.audit_items.len();
                    (
                        String::new(),
                        if n > 0 {
                            Box::leak(Box::new(format!("{n} item(s)")))
                        } else {
                            res.issues.first().map(|s| s.as_str()).unwrap_or("")
                        },
                    )
                }
                "cleanup" => {
                    let _total = res
                        .cleanup_items
                        .iter()
                        .filter_map(|c| c.size.as_ref())
                        .count();
                    (
                        String::new(),
                        res.cleanup_items
                            .first()
                            .and_then(|c| c.size.as_deref())
                            .unwrap_or(res.issues.first().map(|s| s.as_str()).unwrap_or("")),
                    )
                }
                _ => {
                    let outdated_count = scanner::extract_outdated(res).len();
                    let outdated_str = if outdated_count > 0 {
                        outdated_count.to_string()
                    } else {
                        String::new()
                    };
                    let issues_str = res.issues.first().map(|s| s.as_str()).unwrap_or("");
                    (outdated_str, issues_str)
                }
            };
            let sel = tool_index == app.dashboard_selection;
            let indicator = if sel { "\u{25b8} " } else { "  " };
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(display),
                Cell::from(label).style(style),
                Cell::from(ver),
                truncated_cell(&outdated_str, 8),
                truncated_cell(issues_str, 20),
            ])
            .height(1);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            rows.push(row);
            tool_index += 1;
        }

        let h = 2 + 1 + rows.len() as u16;

        let cat_header = Row::new(
            [
                "",
                " Toolchain ",
                " Status ",
                " Version ",
                " Outdated ",
                " Issues ",
            ]
            .iter()
            .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

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
        .header(cat_header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", cat.name))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

        category_heights.push(h);
        category_tables.push(table);
    }

    let constraints: Vec<Constraint> = category_heights
        .iter()
        .map(|h| Constraint::Length(*h))
        .collect();

    let total_outdated = scanner::count_outdated(report);
    if !constraints.is_empty() {
        let cat_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(right_chunks[1]);
        for (i, table) in category_tables.into_iter().enumerate() {
            frame.render_widget(table, cat_chunks[i]);
        }
    } else if !q.is_empty() && total_outdated > 0 {
        let text = Paragraph::new(Text::from(Line::from(Span::raw(
            "No matches found for filter.",
        ))))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, right_chunks[1]);
    }
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

    let header_cells = [
        "",
        " Toolchain ",
        " Source ",
        " Package ",
        " Current ",
        " Latest ",
    ]
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
            Line::from(vec![Span::styled(msg, Style::default().fg(Color::Green))]),
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
            let checked = app.checked_outdated.contains(&i);
            let cb = if checked { "[x]" } else { "[ ]" };
            let indicator = if sel {
                format!("{cb}\u{25b8}")
            } else {
                format!("{cb} ")
            };
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(tool.as_str()),
                Cell::from(pkg.source.as_str()).style(source_style(&pkg.source)),
                Cell::from(pkg.name.as_str()),
                Cell::from(pkg.current.as_str()),
                Cell::from(pkg.latest.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let total = items.len();
    let checked_count = app.checked_outdated.len();
    let title = if app.search_mode && !q.is_empty() {
        format!(" Outdated Packages ({total} matched) ")
    } else if checked_count > 0 {
        format!(" Outdated Packages ({total})  —  {checked_count} selected ")
    } else {
        format!(" Outdated Packages ({total}) ")
    };
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
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

fn render_scanning(frame: &mut Frame, area: Rect, app: &mut App) {
    let throbber = throbber_widgets_tui::Throbber::default()
        .label("Scanning all toolchains...")
        .style(Style::default().fg(Color::Cyan))
        .throbber_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
        .use_type(throbber_widgets_tui::WhichUse::Spin);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Envexa ")
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // title bar (1 pad + 6 art + 1 subtitle + 1 border)
            Constraint::Length(1), // tab bar
            Constraint::Length(1), // gap
            Constraint::Length(1), // shortcuts (status_bar)
            Constraint::Min(1),    // content
        ])
        .split(frame.area());

    title_bar(frame, chunks[0], app);
    tab_bar(frame, chunks[1], app);
    status_bar(frame, chunks[3], app);

    match app.view {
        View::Dashboard => render_dashboard(frame, chunks[4], app),
        View::Outdated => render_outdated(frame, chunks[4], app),
        View::Scanning => render_scanning(frame, chunks[4], app),
        View::PackageDetail => render_package_detail(frame, chunks[4], app),
        View::Updating => render_updating(frame, chunks[4], app),
    }
}

fn render_package_detail(frame: &mut Frame, area: Rect, app: &App) {
    let tool = match &app.detail_tool {
        Some(t) => t.clone(),
        None => return,
    };

    match app.detail_key.as_deref() {
        Some("security") => render_vulnerabilities(frame, area, &tool, app),
        Some("audit") => render_audit_items(frame, area, &tool, app),
        Some("cleanup") => render_cleanup_items(frame, area, &tool, app),
        _ => render_outdated_detail(frame, area, &tool, app),
    }
}

fn render_outdated_detail(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_items;

    let header_cells = ["", " Package ", " Source ", " Current ", " Latest "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let sel = i == app.detail_selection;
            let checked = app.detail_checked.contains(&i);
            let cb = if checked { "[x]" } else { "[ ]" };
            let indicator = if sel {
                format!("{cb}\u{25b8}")
            } else {
                format!("{cb} ")
            };
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(item.name.as_str()),
                Cell::from(item.source.as_str()).style(source_style(&item.source)),
                Cell::from(item.current.as_str()),
                Cell::from(item.latest.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let sub = if !app.detail_message.is_empty() {
        format!("  {}", app.detail_message)
    } else {
        String::new()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(18),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Outdated Packages ({}) ", items.len()))
            .title_bottom(sub)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_vulnerabilities(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_vulns;
    let header_cells = [
        " Package ",
        " Severity ",
        " CVE ",
        " Title ",
        " Patched In ",
    ]
    .iter()
    .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let sel = i == app.detail_selection;
            let cve = v.cve.as_deref().unwrap_or("-");
            let mut row = Row::new(vec![
                Cell::from(v.package.as_str()),
                Cell::from(v.severity.as_str()).style(match v.severity.as_str() {
                    "critical" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "high" => Style::default().fg(Color::Red),
                    "moderate" => Style::default().fg(Color::Yellow),
                    _ => Style::default().fg(Color::DarkGray),
                }),
                Cell::from(cve),
                Cell::from(v.title.as_str()),
                Cell::from(v.patched_version.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(16),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Min(20),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Vulnerabilities ({}) ", items.len()))
            .border_style(Style::default().fg(Color::Red)),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_audit_items(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_audits;
    let header_cells = [" Name ", " Current ", " Note "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let sel = i == app.detail_selection;
            let mut row = Row::new(vec![
                Cell::from(a.name.as_str()),
                Cell::from(a.current.as_str()),
                Cell::from(a.note.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(16),
            Constraint::Length(10),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Audit Items ({}) ", items.len()))
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_cleanup_items(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_cleanup;
    let header_cells = [" Category ", " Description ", " Size ", " Command "]
        .iter()
        .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let sel = i == app.detail_selection;
            let size = c.size.as_deref().unwrap_or("-");
            let cmd = c.command.as_deref().unwrap_or("-");
            let mut row = Row::new(vec![
                Cell::from(c.category.as_str()),
                Cell::from(c.description.as_str()),
                Cell::from(size),
                Cell::from(cmd),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Cleanup ({}) ", items.len()))
            .border_style(Style::default().fg(Color::Green)),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_updating(frame: &mut Frame, area: Rect, app: &mut App) {
    let throbber = throbber_widgets_tui::Throbber::default()
        .label("Updating packages...")
        .style(Style::default().fg(Color::Green))
        .throbber_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
        .use_type(throbber_widgets_tui::WhichUse::Spin);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Updating ")
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
}
