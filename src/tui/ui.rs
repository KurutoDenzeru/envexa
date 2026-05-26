use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{BarChart, Block, Borders, Cell, Gauge, LineGauge, Paragraph, Row, Table, Tabs},
    Frame,
};
use tui_piechart::{LegendAlignment, LegendLayout, LegendPosition, PieChart, PieSlice, Resolution};

use crate::scanner;
use crate::tui::app::{App, View};

fn status_style(status: &str) -> Style {
    let style = Style::default().fg(status_color(status));
    match status {
        "ok" | "warning" | "error" => style.add_modifier(Modifier::BOLD),
        _ => style,
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

fn status_color(status: &str) -> Color {
    match status {
        "ok" => Color::Green,
        "warning" => Color::Yellow,
        "error" => Color::Red,
        "skipped" => Color::DarkGray,
        _ => Color::White,
    }
}

fn severity_style(severity: &str) -> Style {
    match severity.to_ascii_lowercase().as_str() {
        "critical" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "high" => Style::default().fg(Color::Red),
        "moderate" | "medium" => Style::default().fg(Color::Yellow),
        "low" => Style::default().fg(Color::Blue),
        _ => Style::default().fg(Color::DarkGray),
    }
}

fn severity_counts(vulns: &[crate::toolchains::VulnerabilityInfo]) -> (usize, usize, usize, usize) {
    let mut critical = 0usize;
    let mut high = 0usize;
    let mut moderate = 0usize;
    let mut other = 0usize;

    for vuln in vulns {
        match vuln.severity.to_ascii_lowercase().as_str() {
            "critical" => critical += 1,
            "high" => high += 1,
            "moderate" | "medium" => moderate += 1,
            _ => other += 1,
        }
    }

    (critical, high, moderate, other)
}

fn render_minimal(frame: &mut Frame, area: Rect, msg: &str) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn title_bar(frame: &mut Frame, area: Rect, _app: &App) {
    if area.height == 0 {
        return;
    }
    if area.height < 9 || area.width < 72 {
        let title = Paragraph::new(Line::from(vec![
            Span::styled("Envexa", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" "),
            Span::styled(
                concat!("v", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, area);
        return;
    }

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
            let readonly_detail = matches!(
                app.detail_key.as_deref(),
                Some("security") | Some("audit") | Some("cleanup")
            );
            let mut spans = vec![
                Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::DarkGray)),
                Span::raw(" nav "),
            ];
            if !readonly_detail {
                spans.extend([
                    Span::styled("[Space]", Style::default().fg(Color::Yellow)),
                    Span::raw(" toggle "),
                    Span::styled("[Y]", Style::default().fg(Color::Green)),
                    Span::raw(" update "),
                ]);
            }
            spans.extend([
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" back"),
                Span::styled(msg, Style::default().fg(Color::White)),
            ]);
            (
                Line::from(spans),
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
    if max == 0 {
        return Cell::from(String::new());
    }
    let display = if text.chars().count() > max {
        let mut s: String = text.chars().take(max.saturating_sub(1)).collect();
        s.push('…');
        s
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

fn render_dashboard_health_panel(
    frame: &mut Frame,
    area: Rect,
    report: &crate::scanner::Report,
    pass: usize,
    warn: usize,
    fail: usize,
    skip: usize,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let total = pass + warn + fail + skip;
    let health = if total > 0 {
        pass as f64 / total as f64
    } else {
        0.0
    };

    if area.height == 1 || area.width < 42 {
        dashboard_stats_line(frame, area, report);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(
        LineGauge::default()
            .filled_style(Style::default().fg(Color::Green))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .ratio(health),
        chunks[0],
    );

    dashboard_stats_line(frame, chunks[1], report);

    if chunks[2].height > 0 && area.width >= 56 {
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
        frame.render_widget(summary, chunks[2]);
    }
}

fn render_overview_pie(
    frame: &mut Frame,
    area: Rect,
    pass: usize,
    warn: usize,
    fail: usize,
    skip: usize,
) {
    if area.width < 24 || area.height < 7 {
        return;
    }

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
        .show_legend(area.width >= 36 && area.height >= 10)
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
    frame.render_widget(piechart, area);
}

fn dashboard_table_constraints(width: u16) -> [Constraint; 6] {
    if width < 64 {
        [
            Constraint::Length(1),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Min(4),
        ]
    } else if width < 88 {
        [
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Length(13),
            Constraint::Length(8),
            Constraint::Min(8),
        ]
    } else {
        [
            Constraint::Length(2),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Min(15),
        ]
    }
}

fn outdated_table_constraints(width: u16) -> [Constraint; 6] {
    if width < 72 {
        [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(10),
        ]
    } else {
        [
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
            Constraint::Length(18),
            Constraint::Length(18),
        ]
    }
}

fn detail_table_constraints(width: u16, kind: &str) -> Vec<Constraint> {
    match kind {
        "outdated" if width < 72 => vec![
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
        "outdated" => vec![
            Constraint::Length(5),
            Constraint::Min(18),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Length(18),
        ],
        "security" if width < 84 => vec![
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Min(14),
            Constraint::Length(10),
        ],
        "security" => vec![
            Constraint::Min(16),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Min(20),
            Constraint::Length(14),
        ],
        "audit" if width < 64 => vec![
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Min(16),
        ],
        "audit" => vec![
            Constraint::Min(16),
            Constraint::Length(10),
            Constraint::Min(30),
        ],
        "cleanup" if width < 80 => vec![
            Constraint::Length(10),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Min(12),
        ],
        _ => vec![
            Constraint::Length(12),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Min(20),
        ],
    }
}

fn project_tooling_risk(
    project_outdated: usize,
    critical: usize,
    high: usize,
    moderate: usize,
    other: usize,
    audit_items: usize,
) -> u64 {
    let score = critical * 30
        + high * 20
        + moderate * 12
        + other * 6
        + audit_items * 8
        + project_outdated.min(10) * 2;
    score.min(100) as u64
}

fn dashboard_cells(tool: &str, res: &crate::toolchains::ScanResult) -> (String, String) {
    match tool {
        "security" => {
            let n = res.vulnerabilities.len();
            let signal = if n > 0 { n.to_string() } else { String::new() };
            let focus = if n > 0 {
                let (critical, high, moderate, other) = severity_counts(&res.vulnerabilities);
                format!("C{critical} H{high} M{moderate} O{other}")
            } else {
                res.issues.first().cloned().unwrap_or_default()
            };
            (signal, focus)
        }
        "audit" => {
            let n = res.audit_items.len();
            let signal = if n > 0 { n.to_string() } else { String::new() };
            let focus = res
                .audit_items
                .first()
                .map(|a| format!("{}: {}", a.name, a.note))
                .or_else(|| res.issues.first().cloned())
                .unwrap_or_default();
            (signal, focus)
        }
        "cleanup" => {
            let n = res.cleanup_items.len();
            let signal = if n > 0 { n.to_string() } else { String::new() };
            let focus = res
                .cleanup_items
                .first()
                .and_then(|c| c.size.clone())
                .or_else(|| res.issues.first().cloned())
                .unwrap_or_default();
            (signal, focus)
        }
        _ => {
            let outdated_count = scanner::extract_outdated(res).len();
            let signal = if outdated_count > 0 {
                outdated_count.to_string()
            } else {
                String::new()
            };
            let focus = res.issues.first().cloned().unwrap_or_default();
            (signal, focus)
        }
    }
}

fn project_tooling_cells(tool: &str, res: &crate::toolchains::ScanResult) -> (String, String) {
    match tool {
        "project" => {
            let outdated = scanner::extract_outdated(res).len();
            let signal = if outdated > 0 {
                format!("{outdated} pkg")
            } else if res.status == "skipped" {
                "-".into()
            } else {
                "current".into()
            };
            let focus = res
                .project_type
                .as_ref()
                .map(|kind| format!("{kind} project"))
                .or_else(|| res.issues.first().cloned())
                .unwrap_or_else(|| "lockfile detected".into());
            (signal, focus)
        }
        "security" => {
            let n = res.vulnerabilities.len();
            let signal = if n > 0 {
                format!("{n} vuln")
            } else {
                "clean".into()
            };
            let focus = if n > 0 {
                let (critical, high, moderate, other) = severity_counts(&res.vulnerabilities);
                format!("C{critical} H{high} M{moderate} O{other}")
            } else {
                "no known vulns".into()
            };
            (signal, focus)
        }
        "audit" => {
            let n = res.audit_items.len();
            let signal = if n > 0 {
                format!("{n} item")
            } else {
                "aligned".into()
            };
            let focus = res
                .audit_items
                .first()
                .map(|item| format!("{}: {}", item.name, item.note))
                .or_else(|| res.issues.first().cloned())
                .unwrap_or_else(|| "runtime pairs ok".into());
            (signal, focus)
        }
        _ => dashboard_cells(tool, res),
    }
}

fn render_project_tooling_panel(frame: &mut Frame, area: Rect, report: &crate::scanner::Report) {
    if area.width < 18 || area.height < 3 {
        render_minimal(frame, area, "Project Tooling");
        return;
    }

    let project = report.results.get("project");
    let security = report.results.get("security");
    let audit = report.results.get("audit");

    let project_outdated = project
        .map(scanner::extract_outdated)
        .map(|items| items.len())
        .unwrap_or(0);
    let project_type = project
        .and_then(|res| res.project_type.as_deref())
        .unwrap_or("unknown");
    let project_status = project.map(|res| res.status.as_str()).unwrap_or("skipped");
    let security_status = security.map(|res| res.status.as_str()).unwrap_or("skipped");
    let audit_status = audit.map(|res| res.status.as_str()).unwrap_or("skipped");

    let empty_vulns: &[crate::toolchains::VulnerabilityInfo] = &[];
    let vulnerabilities = security
        .map(|res| res.vulnerabilities.as_slice())
        .unwrap_or(empty_vulns);
    let (critical, high, moderate, other) = severity_counts(vulnerabilities);
    let vuln_count = vulnerabilities.len();
    let audit_count = audit.map(|res| res.audit_items.len()).unwrap_or(0);
    let risk = project_tooling_risk(
        project_outdated,
        critical,
        high,
        moderate,
        other,
        audit_count,
    );
    let readiness = 1.0 - (risk as f64 / 100.0);
    let readiness_color = if risk >= 70 {
        Color::Red
    } else if risk >= 35 {
        Color::Yellow
    } else {
        Color::Green
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Project Tooling ")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 || inner.width < 16 {
        return;
    }

    if inner.height < 7 || inner.width < 34 {
        let compact = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("Ready ", Style::default().fg(readiness_color).bold()),
                Span::raw(format!("{:>3}%", (readiness * 100.0).round() as u64)),
            ]),
            Line::from(vec![
                Span::styled("Risk ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{risk}/100")),
            ]),
        ]));
        frame.render_widget(compact, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(3),
        ])
        .split(inner);

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(readiness_color))
            .label(Span::styled(
                format!(
                    " readiness {:>3}% | risk {risk}/100 ",
                    (readiness * 100.0).round() as u64
                ),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ))
            .ratio(readiness),
        chunks[0],
    );

    let summary = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Project ", Style::default().fg(Color::Cyan)),
            Span::styled(
                scanner::status_label(project_status),
                status_style(project_status),
            ),
            Span::raw(format!("  {project_type} / {project_outdated} outdated")),
        ]),
        Line::from(vec![
            Span::styled("Security", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled(
                scanner::status_label(security_status),
                status_style(security_status),
            ),
            Span::raw(format!("  {vuln_count} vulns")),
        ]),
        Line::from(vec![
            Span::styled("Audit   ", Style::default().fg(Color::Yellow)),
            Span::styled(
                scanner::status_label(audit_status),
                status_style(audit_status),
            ),
            Span::raw(format!("  {audit_count} checks flagged")),
        ]),
    ]));
    frame.render_widget(summary, chunks[1]);

    let signal_data = [
        ("Pkg", project_outdated as u64),
        ("Crit", critical as u64),
        ("High", high as u64),
        ("Mod", moderate as u64),
        ("Other", other as u64),
        ("Audit", audit_count as u64),
    ];
    let max_signal = signal_data
        .iter()
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(1)
        .max(1);
    let bar_width = match inner.width {
        0..=42 => 3,
        43..=58 => 4,
        _ => 5,
    };
    let bar_gap = if inner.width < 44 { 0 } else { 1 };
    let chart = BarChart::default()
        .data(&signal_data)
        .max(max_signal)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .bar_style(Style::default().fg(Color::Magenta))
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::DarkGray));
    if chunks[2].height > 1 {
        frame.render_widget(chart, chunks[2]);
    }
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

    if area.width < 24 || area.height < 6 {
        let (pass, warn, fail, skip) = count_statuses(report);
        render_minimal(frame, area, &format!("Envexa {pass}/{warn}/{fail}/{skip}"));
        return;
    }

    let (pass, warn, fail, skip) = count_statuses(report);

    let table_area = if area.width >= 104 && area.height >= 18 {
        let left_width = (area.width / 3).clamp(38, 52);
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(left_width), Constraint::Min(1)])
            .split(area);
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(7)])
            .split(layout[0]);

        render_overview_pie(frame, left_chunks[0], pass, warn, fail, skip);
        render_project_tooling_panel(frame, left_chunks[1], report);

        let header_height = if layout[1].height >= 6 { 4 } else { 2 };
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(1)])
            .split(layout[1]);
        render_dashboard_health_panel(frame, right_chunks[0], report, pass, warn, fail, skip);
        right_chunks[1]
    } else {
        let tooling_height = if area.height >= 20 {
            8
        } else if area.height >= 13 {
            5
        } else {
            0
        };
        let header_height = if area.height >= 10 { 2 } else { 1 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(tooling_height),
                Constraint::Length(header_height),
                Constraint::Min(1),
            ])
            .split(area);
        if tooling_height > 0 {
            render_project_tooling_panel(frame, chunks[0], report);
        }
        render_dashboard_health_panel(frame, chunks[1], report, pass, warn, fail, skip);
        chunks[2]
    };

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
            let (outdated_str, issues_str) = if cat.name == "Project Tooling" {
                project_tooling_cells(tool, res)
            } else {
                dashboard_cells(tool, res)
            };
            let sel = tool_index == app.dashboard_selection;
            let indicator = if sel { "\u{25b8} " } else { "  " };
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(display),
                Cell::from(label).style(style),
                Cell::from(ver),
                truncated_cell(&outdated_str, 8),
                truncated_cell(&issues_str, 20),
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

        let metric_header = if cat.name == "Project Tooling" {
            "Signal "
        } else {
            "Outdated "
        };
        let focus_header = if cat.name == "Project Tooling" {
            "Focus "
        } else {
            "Issues "
        };
        let cat_header = Row::new(
            [
                "",
                "Toolchain ",
                "Status ",
                "Version ",
                metric_header,
                focus_header,
            ]
            .iter()
            .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .height(1);

        let table = Table::new(rows, dashboard_table_constraints(table_area.width))
            .header(cat_header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", cat.name))
                    .border_style(Style::default().fg(if cat.name == "Project Tooling" {
                        Color::Magenta
                    } else {
                        Color::Cyan
                    })),
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
            .split(table_area);
        for (i, table) in category_tables.into_iter().enumerate() {
            if cat_chunks[i].height > 0 {
                frame.render_widget(table, cat_chunks[i]);
            }
        }
    } else if !q.is_empty() && total_outdated > 0 {
        let text = Paragraph::new(Text::from(Line::from(Span::raw(
            "No matches found for filter.",
        ))))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, table_area);
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
        "Toolchain ",
        "Source ",
        "Package ",
        "Current ",
        "Latest ",
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
    let table = Table::new(rows, outdated_table_constraints(area.width))
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
    if inner.width > 0 && inner.height > 0 {
        frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < 16 || area.height < 4 {
        render_minimal(frame, area, "Envexa");
        return;
    }

    let title_height = if area.height >= 18 && area.width >= 72 {
        9
    } else {
        2
    };
    let tab_height = if area.height >= 7 { 1 } else { 0 };
    let gap_height = if area.height >= 12 { 1 } else { 0 };
    let status_height = if area.height >= 6 { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Length(tab_height),
            Constraint::Length(gap_height),
            Constraint::Length(status_height),
            Constraint::Min(1),
        ])
        .split(area);

    title_bar(frame, chunks[0], app);
    if tab_height > 0 {
        tab_bar(frame, chunks[1], app);
    }
    if status_height > 0 {
        status_bar(frame, chunks[3], app);
    }

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

    let header_cells = ["", "Package ", "Source ", "Current ", "Latest "]
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

    let table = Table::new(rows, detail_table_constraints(area.width, "outdated"))
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
    let header_cells = ["Package ", "Severity ", "CVE ", "Title ", "Patched In "]
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
                Cell::from(v.severity.as_str()).style(severity_style(&v.severity)),
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

    let table = Table::new(rows, detail_table_constraints(area.width, "security"))
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
    let header_cells = ["Name ", "Current ", "Note "]
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

    let table = Table::new(rows, detail_table_constraints(area.width, "audit"))
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
    let header_cells = ["Category ", "Description ", "Size ", "Command "]
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

    let table = Table::new(rows, detail_table_constraints(area.width, "cleanup"))
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
    if inner.width > 0 && inner.height > 0 {
        frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
    }
}
