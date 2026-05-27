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
            let readonly_detail = matches!(app.detail_key.as_deref(), Some("audit"));
            let mut spans = vec![
                Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::DarkGray)),
                Span::raw(" nav "),
            ];
            if app.detail_key.as_deref() == Some("security") {
                spans.extend([
                    Span::styled("[F]", Style::default().fg(Color::Green)),
                    Span::raw(" fix "),
                ]);
            } else if !readonly_detail {
                spans.extend([
                    Span::styled("[Space]", Style::default().fg(Color::Yellow)),
                    Span::raw(" toggle "),
                    Span::styled("[Y]", Style::default().fg(Color::Green)),
                    Span::raw(" update/clean "),
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
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Min(12),
        ],
        _ => vec![
            Constraint::Length(6),
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
        let num_blocks = ((readiness * 10.0).round() as usize).min(10);
        let bar_span = Span::styled("■".repeat(num_blocks), Style::default().fg(readiness_color));
        let empty_span = Span::styled(
            "░".repeat(10 - num_blocks),
            Style::default().fg(Color::DarkGray),
        );
        let extra_text = if inner.width >= 28 {
            format!("  {} outdated  {} vulns", project_outdated, vuln_count)
        } else {
            String::new()
        };

        let compact = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("Ready ", Style::default().fg(Color::White).bold()),
                Span::styled(
                    format!("{:>3}% ", (readiness * 100.0).round() as u64),
                    Style::default().fg(readiness_color).bold(),
                ),
                bar_span,
                empty_span,
            ]),
            Line::from(vec![
                Span::styled("Risk  ", Style::default().fg(Color::White)),
                Span::styled(format!("{risk}/100"), Style::default().fg(readiness_color)),
                Span::raw(extra_text),
            ]),
        ]));
        frame.render_widget(compact, inner);
        return;
    }

    let (chunks, has_spacers) = if inner.height >= 11 {
        (
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Gauge
                    Constraint::Length(1), // Spacer
                    Constraint::Length(3), // Summary text
                    Constraint::Length(1), // Spacer
                    Constraint::Min(3),    // BarChart
                ])
                .split(inner),
            true,
        )
    } else {
        (
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Gauge
                    Constraint::Length(4), // Summary text (with 1 trailing blank line)
                    Constraint::Min(3),    // BarChart
                ])
                .split(inner),
            false,
        )
    };

    let gauge_chunk = chunks[0];
    let summary_chunk = if has_spacers { chunks[2] } else { chunks[1] };
    let chart_chunk = if has_spacers { chunks[4] } else { chunks[2] };

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
        gauge_chunk,
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
    frame.render_widget(summary, summary_chunk);

    let signal_data = [
        ("Pkg", project_outdated as u64),
        ("Cri", critical as u64),
        ("Hgh", high as u64),
        ("Med", moderate as u64),
        ("Oth", other as u64),
        ("Aud", audit_count as u64),
    ];
    let max_signal = signal_data
        .iter()
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(1)
        .max(1);

    let (bar_width, bar_gap) = if inner.width >= 45 {
        (5, 3)
    } else if inner.width >= 39 {
        (4, 3)
    } else if inner.width >= 34 {
        (4, 2)
    } else if inner.width >= 28 {
        (3, 2)
    } else if inner.width >= 22 {
        (2, 2)
    } else {
        (2, 1)
    };

    let chart = BarChart::default()
        .data(&signal_data)
        .max(max_signal)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .bar_style(Style::default().fg(Color::Magenta))
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::DarkGray));
    if chart_chunk.height > 1 {
        frame.render_widget(chart, chart_chunk);
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
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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
        let tooling_height = if area.height >= 22 {
            9
        } else if area.height >= 15 {
            7
        } else if area.height >= 12 {
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
        if inner.height >= 4 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Min(0),
                ])
                .split(inner);
            frame.render_stateful_widget(throbber, chunks[0], &mut app.throbber_state);

            let ratio = 1.0 - (0.96_f64).powi(app.progress_counter as i32);
            let pct = (ratio * 100.0).round() as u64;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::NONE))
                .gauge_style(Style::default().fg(Color::Cyan))
                .label(Span::styled(
                    format!(" scanning... {}% ", pct),
                    Style::default().fg(Color::White).bold(),
                ))
                .ratio(ratio);
            frame.render_widget(gauge, chunks[2]);

            // Real-time scan logs and tips
            let step = app.progress_counter;
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spin_char = spinner_frames[step % spinner_frames.len()];
            let mut log_lines = vec![Line::from("")];

            // Step 1: Project Environment (started at step 0)
            if step > 0 {
                let sym = if step < 10 { spin_char } else { "✔" };
                let color = if step < 10 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Checking project environment & lockfiles...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 2: Homebrew (started at step 10)
            if step >= 10 {
                let sym = if step < 22 { spin_char } else { "✔" };
                let color = if step < 22 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Auditing system runtimes & Homebrew formulas...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 3: Web Runtimes (started at step 22)
            if step >= 22 {
                let sym = if step < 35 { spin_char } else { "✔" };
                let color = if step < 35 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Scanning Web Development runtimes (npm, pnpm, Bun, Deno)...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 4: Cargo / pip / gem (started at step 35)
            if step >= 35 {
                let sym = if step < 48 { spin_char } else { "✔" };
                let color = if step < 48 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Checking compiler dependencies (cargo-outdated, pip, gem)...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 5: Security Advisory (started at step 48)
            if step >= 48 {
                let sym = if step < 62 { spin_char } else { "✔" };
                let color = if step < 62 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Running security advisory audits (RustSec, npm audit, pip-audit)...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 6: Docker / Cleanup (started at step 62)
            if step >= 62 {
                let sym = if step < 75 { spin_char } else { "✔" };
                let color = if step < 75 { Color::Cyan } else { Color::Green };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Checking Docker daemons & local cache reclaimables...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Step 7: Compiling results (started at step 75)
            if step >= 75 {
                log_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", spin_char),
                        Style::default().fg(Color::Cyan).bold(),
                    ),
                    Span::styled(
                        "Compiling final health reports and consolidating cache...",
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Rotating tool tips
            let tips = [
                "💡 Tip: Press '/' to filter outdated packages by name or manager in real-time.",
                "💡 Tip: Use the 'Space' key to select packages and 'U' to update them concurrently.",
                "💡 Tip: Run 'envexa scan' in your terminal to get a clean Markdown report for your CI.",
                "💡 Tip: Change your default target project directory in '~/.envexa/config.json'.",
                "💡 Tip: Cleanup scanner checks docker, npm, and cargo local cache spaces.",
                "💡 Tip: Outdated tab shows global as well as local project dependencies.",
            ];
            let tip_index = (step / 80) % tips.len();
            let current_tip = tips[tip_index];

            log_lines.push(Line::from(""));
            log_lines.push(Line::from(vec![Span::styled(
                format!("  {}", current_tip),
                Style::default().fg(Color::DarkGray),
            )]));

            frame.render_widget(Paragraph::new(log_lines), chunks[3]);
        } else {
            frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
        }
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

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Outdated Packages (0) "))
            .border_style(Style::default().fg(Color::Green));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  \u{2714} ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    "All packages are completely up to date!",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

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

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Vulnerabilities (0) "))
            .border_style(Style::default().fg(Color::Green));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  \u{2714} ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    "No security vulnerabilities detected!",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

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

    let bottom_msg = if !app.detail_message.is_empty() {
        format!(
            "  {}  |  [E] Export Report  [Esc] Back ",
            app.detail_message
        )
    } else {
        "  [E] Export Report  |  [Esc] Back ".to_string()
    };

    let table_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {tool} — Vulnerabilities ({}) ", items.len()))
        .title_bottom(bottom_msg)
        .border_style(Style::default().fg(Color::Red));

    // Responsive 2-column layout
    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let left_area = chunks[0];
        let right_area = chunks[1];

        // Draw Left Table
        let table = Table::new(rows, detail_table_constraints(left_area.width, "security"))
            .header(header)
            .block(table_block)
            .column_spacing(1);
        frame.render_widget(table, left_area);

        // Draw Right Stats and Detail Panel
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Min(1)])
            .split(right_area);

        // Right Top: Scorecard with line gauge
        let (crit, high, mod_cnt, other) = severity_counts(items);
        let score = 100_usize
            .saturating_sub(crit * 25 + high * 10 + mod_cnt * 5)
            .clamp(0, 100) as u16;
        let score_color = if score >= 90 {
            Color::Green
        } else if score >= 70 {
            Color::Yellow
        } else {
            Color::Red
        };

        let gauge = LineGauge::default()
            .block(Block::default().title(" Security Health Score "))
            .filled_style(Style::default().fg(score_color))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .ratio(score as f64 / 100.0);

        let overview_text = vec![Line::from(vec![
            Span::styled("Critical: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", crit),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" High: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", high),
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Mod: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", mod_cnt),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Low: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", other),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        let overview_block = Block::default()
            .borders(Borders::ALL)
            .title(" Security Scorecard ")
            .border_style(Style::default().fg(Color::Magenta));

        let overview_inner = overview_block.inner(right_chunks[0]);
        frame.render_widget(overview_block, right_chunks[0]);

        if overview_inner.width > 0 && overview_inner.height > 0 {
            let metric_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Length(2)])
                .split(overview_inner);

            frame.render_widget(gauge, metric_chunks[0]);
            frame.render_widget(Paragraph::new(overview_text), metric_chunks[1]);
        }

        // Right Bottom: Dynamic Selection Card
        if let Some(vuln) = items.get(app.detail_selection) {
            let cve = vuln.cve.as_deref().unwrap_or("None");
            let card_border_color = match vuln.severity.to_ascii_lowercase().as_str() {
                "critical" => Color::Red,
                "high" => Color::LightRed,
                "moderate" | "medium" => Color::Yellow,
                "low" => Color::Blue,
                _ => Color::DarkGray,
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled("Package: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &vuln.package,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Severity: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&vuln.severity, severity_style(&vuln.severity)),
                ]),
                Line::from(vec![
                    Span::styled("CVE ID: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cve, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("Patched: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&vuln.patched_version, Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Title / Description:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(&vuln.title, Style::default().fg(Color::White))),
            ];

            let detail_card = Paragraph::new(lines)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Selected Vulnerability ")
                        .border_style(Style::default().fg(card_border_color)),
                );

            frame.render_widget(detail_card, right_chunks[1]);
        } else {
            let empty_card = Paragraph::new("No item selected.").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Selected Vulnerability "),
            );
            frame.render_widget(empty_card, right_chunks[1]);
        }
    } else {
        // Fallback layout (narrow terminal)
        let table = Table::new(rows, detail_table_constraints(area.width, "security"))
            .header(header)
            .block(table_block)
            .column_spacing(1);
        frame.render_widget(table, area);
    }
}

fn render_audit_items(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_audits;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Audit Items (0) "))
            .border_style(Style::default().fg(Color::Green));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  \u{2714} ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    "System and toolchains are aligned! No issues flagged.",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

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

    let bottom_msg = if !app.detail_message.is_empty() {
        format!(
            "  {}  |  [E] Export Report  [Esc] Back ",
            app.detail_message
        )
    } else {
        "  [E] Export Report  |  [Esc] Back ".to_string()
    };

    let table_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {tool} — Audit Items ({}) ", items.len()))
        .title_bottom(bottom_msg)
        .border_style(Style::default().fg(Color::Yellow));

    // Responsive 2-column layout
    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let left_area = chunks[0];
        let right_area = chunks[1];

        // Draw Left Table
        let table = Table::new(rows, detail_table_constraints(left_area.width, "audit"))
            .header(header)
            .block(table_block)
            .column_spacing(1);
        frame.render_widget(table, left_area);

        // Draw Right Stats and Detail Panel
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(right_area);

        // Right Top: Alignment meter with LineGauge
        let alignment_score = 100_usize.saturating_sub(items.len() * 20).clamp(0, 100) as u16;
        let gauge_color = if alignment_score >= 80 {
            Color::Green
        } else if alignment_score >= 50 {
            Color::Yellow
        } else {
            Color::Red
        };

        let gauge = LineGauge::default()
            .block(Block::default().title(" System Alignment Score "))
            .filled_style(Style::default().fg(gauge_color))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .ratio(alignment_score as f64 / 100.0);

        frame.render_widget(gauge, right_chunks[0]);

        // Right Bottom: Dynamic Recommendation Card
        if let Some(audit) = items.get(app.detail_selection) {
            let lines = vec![
                Line::from(vec![
                    Span::styled("Audit Rule: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &audit.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Current State: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&audit.current, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Recommendation / Note:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(&audit.note, Style::default().fg(Color::White))),
            ];

            let detail_card = Paragraph::new(lines)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Recommendation Detail ")
                        .border_style(Style::default().fg(Color::Yellow)),
                );

            frame.render_widget(detail_card, right_chunks[1]);
        } else {
            let empty_card = Paragraph::new("No item selected.").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Recommendation Detail "),
            );
            frame.render_widget(empty_card, right_chunks[1]);
        }
    } else {
        // Fallback layout (narrow terminal)
        let table = Table::new(rows, detail_table_constraints(area.width, "audit"))
            .header(header)
            .block(table_block)
            .column_spacing(1);
        frame.render_widget(table, area);
    }
}

fn parse_size_to_mb(size_str: &str) -> f64 {
    let clean: String = size_str
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect();

    let numeric_part: String = clean
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    let Ok(val) = numeric_part.parse::<f64>() else {
        return 0.0;
    };

    let suffix = &clean[numeric_part.len()..];
    let suffix_lower = suffix.to_lowercase();

    if suffix_lower.starts_with('g') {
        val * 1024.0
    } else if suffix_lower.starts_with('m') {
        val
    } else if suffix_lower.starts_with('k') {
        val / 1024.0
    } else if suffix_lower.starts_with('b') {
        val / (1024.0 * 1024.0)
    } else {
        0.0
    }
}

fn get_cleanup_label(item: &crate::toolchains::CleanupItem) -> String {
    let desc = item.description.to_lowercase();
    if desc.contains("npm") {
        "NPM".to_string()
    } else if desc.contains("cargo") {
        "Cargo".to_string()
    } else if desc.contains("bun") {
        "Bun".to_string()
    } else if desc.contains("pip") {
        "Pip".to_string()
    } else if desc.contains("docker") {
        "Docker".to_string()
    } else if desc.contains("homebrew") || desc.contains("brew") {
        "Homebrew".to_string()
    } else {
        item.category.to_uppercase()
    }
}

fn get_label_color(label: &str) -> Color {
    match label {
        "NPM" => Color::Red,
        "Cargo" => Color::LightRed,
        "Bun" => Color::Yellow,
        "Pip" => Color::Green,
        "Docker" => Color::Blue,
        "Homebrew" => Color::Magenta,
        _ => Color::Cyan,
    }
}

fn render_cleanup_items(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail_cleanup;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Cleanup (0) "))
            .border_style(Style::default().fg(Color::Green));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  \u{2714} ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    "Your environment is fully clean! No cache cleanup needed.",
                    Style::default().fg(Color::White),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

    let header_cells = ["", "Category ", "Description ", "Size ", "Command "]
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
            let checked = app.detail_checked.contains(&i);
            let cb = if checked { "[x]" } else { "[ ]" };
            let indicator = if sel {
                format!("{cb}\u{25b8}")
            } else {
                format!("{cb} ")
            };
            let size = c.size.as_deref().unwrap_or("-");
            let cmd = c.command.as_deref().unwrap_or("-");
            let mut row = Row::new(vec![
                Cell::from(indicator),
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {tool} — Cleanup ({}) ", items.len()))
        .border_style(Style::default().fg(Color::Green));

    let inner_area = block.inner(area);

    if inner_area.width >= 80 && inner_area.height >= 8 {
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(inner_area);

        let mut category_sizes: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for item in items {
            if let Some(ref sz) = item.size {
                let label = get_cleanup_label(item);
                let bytes = parse_size_to_mb(sz);
                *category_sizes.entry(label).or_insert(0.0) += bytes;
            }
        }

        let mut sorted_categories: Vec<(String, f64)> = category_sizes.into_iter().collect();
        sorted_categories
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let slice_labels: Vec<String> = sorted_categories
            .iter()
            .map(|(label, mb)| {
                if *mb >= 1024.0 {
                    format!("{} ({:.1}G)", label, mb / 1024.0)
                } else {
                    format!("{} ({:.1}M)", label, mb)
                }
            })
            .collect();

        let mut slices = Vec::new();
        for (i, (label, mb)) in sorted_categories.iter().enumerate() {
            if *mb > 0.0 {
                let color = get_label_color(label);
                slices.push(PieSlice::new(&slice_labels[i], *mb, color));
            }
        }

        if slices.is_empty() {
            slices.push(PieSlice::new("EMPTY", 1.0, Color::DarkGray));
        }

        let piechart = PieChart::new(slices)
            .resolution(Resolution::Braille)
            .show_legend(chunks[0].width >= 24 && chunks[0].height >= 8)
            .legend_position(LegendPosition::Top)
            .legend_layout(LegendLayout::Vertical)
            .legend_alignment(LegendAlignment::Center)
            .show_percentages(false)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Heatmap ")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(piechart, chunks[0]);

        let table = Table::new(rows, detail_table_constraints(chunks[1].width, "cleanup"))
            .header(header)
            .column_spacing(1);
        frame.render_widget(table, chunks[1]);
    } else {
        let table = Table::new(rows, detail_table_constraints(area.width, "cleanup"))
            .header(header)
            .block(block)
            .column_spacing(1);
        frame.render_widget(table, area);
    }
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
        if inner.height >= 4 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Min(0),
                ])
                .split(inner);
            frame.render_stateful_widget(throbber, chunks[0], &mut app.throbber_state);

            let ratio = 1.0 - (0.96_f64).powi(app.progress_counter as i32);
            let pct = (ratio * 100.0).round() as u64;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::NONE))
                .gauge_style(Style::default().fg(Color::Green))
                .label(Span::styled(
                    format!(" updating... {}% ", pct),
                    Style::default().fg(Color::White).bold(),
                ))
                .ratio(ratio);
            frame.render_widget(gauge, chunks[2]);
        } else {
            frame.render_stateful_widget(throbber, inner, &mut app.throbber_state);
        }
    }
}
