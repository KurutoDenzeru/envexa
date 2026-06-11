use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        BarChart, Block, Borders, Cell, Clear, Gauge, LineGauge, List, ListItem, ListState,
        Paragraph, Row, Table, TableState, Tabs,
    },
    Frame,
};
use tui_piechart::{LegendAlignment, LegendLayout, LegendPosition, PieChart, PieSlice, Resolution};

use crate::scanner;
use crate::tui::app::{detect_lockfiles, App, View, ALL_SCANNERS};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn get_cwd_display() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let cwd_str = cwd.to_string_lossy().to_string();
    if let Ok(home) = std::env::var("HOME") {
        if cwd_str.starts_with(&home) {
            return cwd_str.replacen(&home, "~", 1);
        }
    }
    cwd_str
}

fn status_style(status: &str, theme: &Theme) -> Style {
    let style = Style::default().fg(status_color(status, theme));
    match status {
        "ok" | "warning" | "error" => style.add_modifier(Modifier::BOLD),
        _ => style,
    }
}

fn source_style(source: &str, theme: &Theme) -> Style {
    match source {
        "formula" => Style::default().fg(theme.secondary),
        "cask" => Style::default().fg(theme.secondary),
        "global" => Style::default().fg(theme.primary),
        "package" => Style::default().fg(theme.text_muted),
        _ => Style::default(),
    }
}

fn status_color(status: &str, theme: &Theme) -> Color {
    match status {
        "ok" => theme.success,
        "warning" => theme.warning,
        "error" => theme.error,
        "skipped" => theme.text_muted,
        _ => theme.text_normal,
    }
}

fn severity_style(severity: &str, theme: &Theme) -> Style {
    match severity.to_ascii_lowercase().as_str() {
        "critical" => Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD),
        "high" => Style::default().fg(theme.error),
        "moderate" | "medium" => Style::default().fg(theme.warning),
        "low" => Style::default().fg(theme.secondary),
        _ => Style::default().fg(theme.text_muted),
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

fn render_minimal(frame: &mut Frame, area: Rect, msg: &str, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(app.theme().text_muted)),
        area,
    );
}

fn title_bar(frame: &mut Frame, area: Rect, app: &App) {
    if area.height == 0 {
        return;
    }
    if area.height < 9 || area.width < 72 {
        let title = Paragraph::new(Line::from(vec![
            Span::styled("Envexa", Style::default().fg(app.theme().primary).bold()),
            Span::raw(" "),
            Span::styled(
                concat!("v", env!("CARGO_PKG_VERSION")),
                Style::default().fg(app.theme().text_muted),
            ),
            Span::raw("  "),
            Span::styled(
                app.config
                    .project_path
                    .clone()
                    .map(|p| {
                        if p == get_cwd_display() {
                            String::new()
                        } else {
                            p
                        }
                    })
                    .unwrap_or_default(),
                Style::default().fg(app.theme().text_muted),
            ),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(app.theme().text_muted)),
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
        .style(Style::default().fg(app.theme().primary));
    frame.render_widget(art, chunks[1]);

    let path_label = app
        .config
        .project_path
        .clone()
        .unwrap_or_else(get_cwd_display);

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            path_label,
            Style::default().fg(app.theme().text_muted),
        )]))
        .alignment(Alignment::Center),
        chunks[2],
    );

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(app.theme().text_muted));
    frame.render_widget(block, chunks[3]);
}

fn tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let titles = vec![" Dashboard ", " Outdated ", " Logs ", " Settings "];
    let selected = match app.ui.view {
        View::Dashboard => 0,
        View::Outdated => 1,
        View::Logs => 2,
        View::Settings => 3,
        View::Scanning | View::PackageDetail | View::Updating => app.ui.tab_index,
    };
    let tabs = Tabs::new(titles)
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(app.theme().primary)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(app.theme().text_muted))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(tabs, area);
}

fn status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (text, style) = match app.ui.view {
        View::Updating => (
            Line::from(vec![Span::styled(
                " Updating packages... ",
                Style::default()
                    .fg(app.theme().success)
                    .add_modifier(Modifier::BOLD),
            )]),
            Style::default()
                .fg(app.theme().text_normal)
                .bg(app.theme().background),
        ),
        View::PackageDetail => {
            let msg = if !app.detail.message.is_empty() {
                format!("  {}", app.detail.message)
            } else {
                String::new()
            };
            let readonly_detail = matches!(app.detail.key.as_deref(), Some("audit"));
            let mut spans = vec![
                Span::styled(
                    " [\u{2191}\u{2193}]",
                    Style::default().fg(app.theme().text_muted),
                ),
                Span::raw(" nav "),
            ];
            if app.detail.key.as_deref() == Some("security") {
                spans.extend([
                    Span::styled("[F]", Style::default().fg(app.theme().success)),
                    Span::raw(" fix "),
                ]);
            } else if !readonly_detail {
                spans.extend([
                    Span::styled("[Space]", Style::default().fg(app.theme().warning)),
                    Span::raw(" toggle "),
                    Span::styled("[Y]", Style::default().fg(app.theme().success)),
                    Span::raw(" update/clean "),
                ]);
            }
            spans.extend([
                Span::styled("[Esc]", Style::default().fg(app.theme().error)),
                Span::raw(" back"),
                Span::styled(msg, Style::default().fg(app.theme().text_normal)),
            ]);
            (
                Line::from(spans),
                Style::default()
                    .fg(app.theme().text_normal)
                    .bg(app.theme().background),
            )
        }
        _ if app.ui.search_mode => {
            let query = format!(" / {}█", app.ui.search_query);
            (
                Line::from(vec![
                    Span::styled(
                        "Search:",
                        Style::default()
                            .fg(app.theme().warning)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(query),
                    Span::styled("  Esc", Style::default().fg(app.theme().text_muted)),
                    Span::raw(" clear"),
                ]),
                Style::default(),
            )
        }
        _ => {
            let update_msg =
                if matches!(app.ui.view, View::Outdated) && !app.detail.message.is_empty() {
                    format!("  {}", app.detail.message)
                } else {
                    String::new()
                };
            let mut spans = vec![
                Span::styled(" [S]", Style::default().fg(app.theme().success)),
                Span::raw("can "),
                Span::styled("[O]", Style::default().fg(app.theme().warning)),
                Span::raw("utdated "),
                Span::styled("[/]", Style::default().fg(app.theme().primary)),
                Span::raw("earch "),
                Span::styled(
                    "\u{2190}\u{2192}",
                    Style::default().fg(app.theme().text_muted),
                ),
                Span::raw(" tabs "),
                Span::styled(
                    "\u{2191}\u{2193}",
                    Style::default().fg(app.theme().text_muted),
                ),
                Span::raw(" nav "),
                Span::styled("[U]", Style::default().fg(app.theme().success)),
                Span::raw("pdate "),
                Span::styled("^C", Style::default().fg(app.theme().error)),
                Span::styled(" Exit", Style::default().fg(app.theme().error)),
                Span::raw("  "),
                Span::styled("[Q]", Style::default().fg(app.theme().text_muted)),
                Span::raw("uit"),
            ];
            if matches!(app.ui.view, View::Settings) {
                spans.extend([
                    Span::raw("  "),
                    Span::styled("[Space/Enter]", Style::default().fg(app.theme().warning)),
                    Span::raw(" toggle"),
                ]);
            }
            spans.push(Span::styled(
                update_msg,
                Style::default().fg(app.theme().text_normal),
            ));
            (
                Line::from(spans),
                Style::default()
                    .fg(app.theme().text_normal)
                    .bg(app.theme().background),
            )
        }
    };
    let block = Block::default().style(style);
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn marquee_text(text: &str, max_width: usize, tick: usize, is_selected: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len <= max_width || max_width == 0 || !is_selected {
        if len > max_width && max_width > 0 {
            let mut s: String = chars
                .into_iter()
                .take(max_width.saturating_sub(1))
                .collect();
            s.push('…');
            return s;
        }
        return text.to_string();
    }

    let distance = len - max_width;
    let pause = 5;
    let period = (distance + pause) * 2;
    let step = tick % period;

    let offset = if step < pause {
        0
    } else if step < pause + distance {
        step - pause
    } else if step < pause * 2 + distance {
        distance
    } else {
        period - step
    };

    chars[offset..offset + max_width].iter().collect()
}

fn dashboard_max_widths(width: u16) -> (usize, usize) {
    if width < 64 {
        (6, (width.saturating_sub(39)) as usize)
    } else if width < 88 {
        (8, (width.saturating_sub(49)) as usize)
    } else {
        (8, (width.saturating_sub(57)) as usize)
    }
}

fn outdated_max_width(width: u16) -> usize {
    if width < 72 {
        (width.saturating_sub(48)) as usize
    } else {
        (width.saturating_sub(70)) as usize
    }
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

fn dashboard_stats_line(frame: &mut Frame, area: Rect, report: &crate::scanner::Report, app: &App) {
    let (pass, warn, fail, skip) = count_statuses(report);
    let outdated = crate::scanner::count_outdated(report);
    let age = scan_age(&report.timestamp);
    let items = vec![
        Span::styled(
            format!(" \u{25CF} {pass} "),
            Style::default().fg(app.theme().success),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25CF} {warn} "),
            Style::default().fg(app.theme().warning),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25CF} {fail} "),
            Style::default().fg(app.theme().error),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25CF} {skip} "),
            Style::default().fg(app.theme().text_muted),
        ),
        Span::raw("  "),
        Span::styled(
            format!("\u{25C9} {outdated} outdated"),
            Style::default().fg(if outdated > 0 {
                app.theme().warning
            } else {
                app.theme().success
            }),
        ),
        Span::raw("  "),
        Span::styled(
            format!("\u{23F0} {age}"),
            Style::default().fg(app.theme().text_muted),
        ),
    ];
    let block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(app.theme().background));
    frame.render_widget(Paragraph::new(Line::from(items)).block(block), area);
}

#[allow(clippy::too_many_arguments)]
fn render_dashboard_health_panel(
    frame: &mut Frame,
    area: Rect,
    report: &crate::scanner::Report,
    pass: usize,
    warn: usize,
    fail: usize,
    skip: usize,
    app: &App,
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
        dashboard_stats_line(frame, area, report, app);
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
            .filled_style(Style::default().fg(app.theme().success))
            .unfilled_style(Style::default().fg(app.theme().text_muted))
            .ratio(health),
        chunks[0],
    );

    dashboard_stats_line(frame, chunks[1], report, app);

    if chunks[2].height > 0 && area.width >= 56 {
        let summary = Paragraph::new(Line::from(vec![
            Span::styled(" [S]", Style::default().fg(app.theme().success)),
            Span::raw("can  "),
            Span::styled("[O]", Style::default().fg(app.theme().warning)),
            Span::raw("utdated  "),
            Span::styled("[/]", Style::default().fg(app.theme().primary)),
            Span::raw("Search  "),
            Span::styled("^C", Style::default().fg(app.theme().error)),
            Span::raw(" Exit  "),
            Span::styled("[Q]", Style::default().fg(app.theme().text_muted)),
            Span::raw("uit"),
        ]))
        .style(Style::default().fg(app.theme().text_normal))
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
    app: &App,
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
        slices.push(PieSlice::new(&pass_label, pass as f64, app.theme().success));
    }
    if warn > 0 {
        slices.push(PieSlice::new(&warn_label, warn as f64, app.theme().warning));
    }
    if fail > 0 {
        slices.push(PieSlice::new(&fail_label, fail as f64, app.theme().error));
    }
    if skip > 0 {
        slices.push(PieSlice::new(
            &skip_label,
            skip as f64,
            app.theme().text_muted,
        ));
    }

    if slices.is_empty() {
        slices.push(PieSlice::new("EMPTY", 1.0, app.theme().text_muted));
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
                .border_style(Style::default().fg(app.theme().primary)),
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

fn outdated_table_constraints(width: u16) -> [Constraint; 7] {
    if width < 72 {
        [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ]
    } else {
        [
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(10),
        ]
    }
}

fn detail_table_constraints(width: u16, kind: &str) -> Vec<Constraint> {
    match kind {
        "outdated" if width < 72 => vec![
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
        "outdated" => vec![
            Constraint::Length(5),
            Constraint::Min(18),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
        "security" if width < 84 => vec![
            Constraint::Percentage(25),
            Constraint::Length(9),
            Constraint::Length(11),
            Constraint::Percentage(35),
            Constraint::Length(11),
        ],
        "security" => vec![
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Percentage(40),
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
        "supply_chain" => vec![
            Constraint::Min(20),
            Constraint::Length(15),
            Constraint::Min(30),
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

/// Shared helper: renders a stateful table with header, selection highlight,
/// and responsive constraints. Used by all four detail render functions.
#[allow(clippy::too_many_arguments)]
fn render_item_table(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    title: String,
    border_style: Style,
    header_cells: &[&str],
    rows: &[Row],
    kind: &str,
    title_bottom: Option<&str>,
) {
    let header = Row::new(
        header_cells
            .iter()
            .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD)),
    )
    .style(
        Style::default()
            .bg(app.theme().secondary)
            .fg(app.theme().text_normal),
    )
    .height(1);

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);
    if let Some(msg) = title_bottom {
        block = block.title_bottom(msg);
    }

    let table = Table::new(rows.to_vec(), detail_table_constraints(area.width, kind))
        .header(header)
        .block(block)
        .column_spacing(1);
    let mut state = TableState::default();
    state.select(Some(app.detail.selection));
    frame.render_stateful_widget(table, area, &mut state);
}

fn dashboard_cells(
    tool: &str,
    res: &crate::toolchains::ScanResult,
    _app: &App,
) -> (String, String) {
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

fn project_tooling_cells(
    tool: &str,
    res: &crate::toolchains::ScanResult,
    app: &App,
) -> (String, String) {
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
        _ => dashboard_cells(tool, res, app),
    }
}

fn render_project_tooling_panel(
    frame: &mut Frame,
    area: Rect,
    report: &crate::scanner::Report,
    app: &App,
) {
    if area.width < 18 || area.height < 3 {
        render_minimal(frame, area, "Project Tooling", app);
        return;
    }

    let project = report.results.get("project");
    let security = report.results.get("security");
    let audit = report.results.get("audit");
    let supply_chain = report.results.get("supply_chain");

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
    let supply_chain_status = supply_chain
        .map(|res| res.status.as_str())
        .unwrap_or("skipped");

    let vulnerabilities = security
        .map(|res| res.vulnerabilities.as_slice())
        .unwrap_or(&[]);
    let (critical, high, moderate, other) = severity_counts(vulnerabilities);
    let vuln_count = vulnerabilities.len();
    let audit_count = audit.map(|res| res.audit_items.len()).unwrap_or(0);
    let risks = supply_chain
        .map(|res| res.supply_chain_risks.len())
        .unwrap_or(0);
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
        app.theme().error
    } else if risk >= 35 {
        app.theme().warning
    } else {
        app.theme().success
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Project Tooling ")
        .border_style(Style::default().fg(app.theme().secondary));
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
            Style::default().fg(app.theme().text_muted),
        );
        let extra_text = if inner.width >= 28 {
            format!("  {} outdated  {} vulns", project_outdated, vuln_count)
        } else {
            String::new()
        };

        let compact = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "Ready ",
                    Style::default().fg(app.theme().text_normal).bold(),
                ),
                Span::styled(
                    format!("{:>3}% ", (readiness * 100.0).round() as u64),
                    Style::default().fg(readiness_color).bold(),
                ),
                bar_span,
                empty_span,
            ]),
            Line::from(vec![
                Span::styled("Risk  ", Style::default().fg(app.theme().text_normal)),
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
                    .fg(app.theme().text_normal)
                    .add_modifier(Modifier::BOLD),
            ))
            .ratio(readiness),
        gauge_chunk,
    );

    let summary = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Project ", Style::default().fg(app.theme().primary)),
            Span::styled(
                scanner::status_label(project_status),
                status_style(project_status, &app.theme()),
            ),
            Span::raw(format!("  {project_type} / {project_outdated} outdated")),
        ]),
        Line::from(vec![
            Span::styled("Security", Style::default().fg(app.theme().error)),
            Span::raw(" "),
            Span::styled(
                scanner::status_label(security_status),
                status_style(security_status, &app.theme()),
            ),
            Span::raw(format!("  {vuln_count} vulns")),
        ]),
        Line::from(vec![
            Span::styled("Audit   ", Style::default().fg(app.theme().warning)),
            Span::raw(" "),
            Span::styled(
                scanner::status_label(audit_status),
                status_style(audit_status, &app.theme()),
            ),
            Span::raw(format!("  {audit_count} checks flagged")),
        ]),
        Line::from(vec![
            Span::styled("Supply  ", Style::default().fg(app.theme().secondary)),
            Span::raw(" "),
            Span::styled(
                scanner::status_label(supply_chain_status),
                status_style(supply_chain_status, &app.theme()),
            ),
            Span::raw(format!("  {risks} risks")),
        ]),
    ]));
    frame.render_widget(summary, summary_chunk);

    let signal_data = [
        ("Outdated", project_outdated as u64),
        ("Critical", critical as u64),
        ("High", high as u64),
        ("Medium", moderate as u64),
        ("Other", other as u64),
        ("Audit", audit_count as u64),
    ];
    let max_signal = signal_data
        .iter()
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(1)
        .max(1);

    let (bar_width, bar_gap) = if inner.width >= 60 {
        (8, 2)
    } else if inner.width >= 54 {
        (8, 1)
    } else if inner.width >= 45 {
        (5, 2)
    } else if inner.width >= 39 {
        (4, 2)
    } else if inner.width >= 34 {
        (4, 1)
    } else if inner.width >= 28 {
        (3, 1)
    } else if inner.width >= 22 {
        (2, 1)
    } else {
        (2, 0)
    };

    let chart = BarChart::default()
        .data(&signal_data)
        .max(max_signal)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .bar_style(Style::default().fg(app.theme().secondary))
        .value_style(Style::default().fg(app.theme().text_normal))
        .label_style(Style::default().fg(app.theme().text_muted));
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
                    Style::default().fg(app.theme().text_muted),
                )]),
                Line::from(vec![
                    Span::styled("[S]", Style::default().fg(app.theme().success)),
                    Span::raw(" Scan  "),
                    Span::styled("[O]", Style::default().fg(app.theme().warning)),
                    Span::raw(" Outdated"),
                ]),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().bg(app.theme().background));
            frame.render_widget(text, area);
            return;
        }
    };

    if area.width < 24 || area.height < 6 {
        let (pass, warn, fail, skip) = count_statuses(report);
        render_minimal(
            frame,
            area,
            &format!("Envexa {pass}/{warn}/{fail}/{skip}"),
            app,
        );
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

        render_overview_pie(frame, left_chunks[0], pass, warn, fail, skip, app);
        render_project_tooling_panel(frame, left_chunks[1], report, app);

        let header_height = if layout[1].height >= 6 { 4 } else { 2 };
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(1)])
            .split(layout[1]);
        render_dashboard_health_panel(frame, right_chunks[0], report, pass, warn, fail, skip, app);
        right_chunks[1]
    } else if area.height >= 24 {
        // Vertical stacking fallback for Portrait modes
        let top_height = (area.height / 2).clamp(12, 18);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(top_height), Constraint::Min(1)])
            .split(area);

        // Tooling taking the top half
        render_project_tooling_panel(frame, layout[0], report, app);

        // Health panel and Overview Pie side-by-side in the bottom half
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(layout[1]);

        let health_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(1)])
            .split(bottom_chunks[0]);

        render_dashboard_health_panel(frame, health_area[0], report, pass, warn, fail, skip, app);
        render_overview_pie(frame, bottom_chunks[1], pass, warn, fail, skip, app);
        health_area[1]
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
            render_project_tooling_panel(frame, chunks[0], report, app);
        }
        render_dashboard_health_panel(frame, chunks[1], report, pass, warn, fail, skip, app);
        chunks[2]
    };

    let q = app.ui.search_query.to_lowercase();
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
                if !q.is_empty() && app.ui.search_mode {
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
            let style = status_style(&res.status, &app.theme());
            let ver = scanner::first_version(res);
            let (outdated_str, issues_str) = if cat.name == "Project Tooling" {
                project_tooling_cells(tool, res, app)
            } else {
                dashboard_cells(tool, res, app)
            };
            let sel = tool_index == app.ui.dashboard_selection;
            let indicator = if sel { "\u{25b8} " } else { "  " };

            let (max_col4, max_col5) = dashboard_max_widths(table_area.width);
            let out_str = marquee_text(&outdated_str, max_col4, app.ui.tick_count, sel);
            let iss_str = marquee_text(&issues_str, max_col5, app.ui.tick_count, sel);

            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(display),
                Cell::from(label).style(style),
                Cell::from(ver),
                Cell::from(out_str),
                Cell::from(iss_str),
            ])
            .height(1);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
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
        .style(
            Style::default()
                .bg(app.theme().secondary)
                .fg(app.theme().text_normal),
        )
        .height(1);

        let table = Table::new(rows, dashboard_table_constraints(table_area.width))
            .header(cat_header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", cat.name))
                    .border_style(Style::default().fg(if cat.name == "Project Tooling" {
                        app.theme().secondary
                    } else {
                        app.theme().primary
                    })),
            )
            .column_spacing(1);

        category_heights.push(h);
        category_tables.push(table);
    }

    let mut constraints: Vec<Constraint> = category_heights
        .iter()
        .map(|h| Constraint::Length(*h))
        .collect();

    constraints.push(Constraint::Length(2));

    let total_outdated = scanner::count_outdated(report);
    if !category_tables.is_empty() {
        let cat_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(table_area);
        for (i, table) in category_tables.into_iter().enumerate() {
            if cat_chunks[i].height > 0 {
                frame.render_widget(table, cat_chunks[i]);
            }
        }

        if let Some(footer_area) = cat_chunks.last() {
            if footer_area.height > 0 {
                let footer = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "🚧 Envexa ",
                        Style::default().fg(app.theme().primary).bold(),
                    ),
                    Span::styled(
                        concat!("v", env!("CARGO_PKG_VERSION")),
                        Style::default().fg(app.theme().text_normal).bold(),
                    ),
                    Span::styled("  •  ", Style::default().fg(app.theme().text_muted)),
                    Span::styled(
                        "Crafted by Kuruto Denzeru",
                        Style::default()
                            .fg(app.theme().text_muted)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Span::raw("  "),
                ]))
                .alignment(Alignment::Right);
                frame.render_widget(footer, *footer_area);
            }
        }
    } else if !q.is_empty() && total_outdated > 0 {
        let text = Paragraph::new(Text::from(Line::from(Span::raw(
            "No matches found for filter.",
        ))))
        .style(Style::default().fg(app.theme().text_muted));
        frame.render_widget(text, table_area);
    }
}

fn render_outdated(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.report {
        Some(r) => r,
        None => {
            frame.render_widget(
                Paragraph::new("No scan data. Press S to scan first.").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .bg(app.theme().background),
                ),
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
        "Size ",
    ]
    .iter()
    .map(|h| Cell::from(*h).add_modifier(Modifier::BOLD));
    let header = Row::new(header_cells)
        .style(
            Style::default()
                .bg(app.theme().secondary)
                .fg(app.theme().text_normal),
        )
        .height(1);

    let q = app.ui.search_query.to_lowercase();
    let mut items: Vec<(String, scanner::OutdatedItem)> = Vec::new();
    for tool in &scanner::tool_order() {
        if let Some(res) = report.results.get(*tool) {
            let pkgs = scanner::extract_outdated(res);
            if !pkgs.is_empty() {
                let display = scanner::display_name(tool).to_string();
                for pkg in pkgs {
                    if !q.is_empty() && app.ui.search_mode {
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
        let msg = if app.ui.search_mode && !q.is_empty() {
            format!("  No packages match \"{q}\" ")
        } else {
            "  All packages are up to date! ".into()
        };
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                msg,
                Style::default().fg(app.theme().success),
            )]),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Outdated Packages ")
                .border_style(Style::default().fg(app.theme().success)),
        );
        frame.render_widget(text, area);
        return;
    }

    let max_col3 = outdated_max_width(area.width);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, (tool, pkg))| {
            let sel = i == app.ui.outdated_selection;
            let checked = app.ui.checked_outdated.contains(&i);
            let cb = if checked { "[x]" } else { "[ ]" };
            let indicator = if sel {
                format!("{cb}\u{25b8}")
            } else {
                format!("{cb} ")
            };
            let pkg_name = marquee_text(&pkg.name, max_col3, app.ui.tick_count, sel);
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(tool.as_str()),
                Cell::from(pkg.source.as_str()).style(source_style(&pkg.source, &app.theme())),
                Cell::from(pkg_name),
                Cell::from(pkg.current.as_str()),
                Cell::from(pkg.latest.as_str()),
                Cell::from(pkg.size.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let total = items.len();
    let checked_count = app.ui.checked_outdated.len();
    let title = if app.ui.search_mode && !q.is_empty() {
        format!(" Outdated Packages ({total} matched) ")
    } else if checked_count > 0 {
        format!(" Outdated Packages ({total})  —  {checked_count} selected ")
    } else {
        format!(" Outdated Packages ({total}) ")
    };
    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let left_area = chunks[0];
        let right_area = chunks[1];

        let table = Table::new(rows.clone(), outdated_table_constraints(left_area.width))
            .header(header.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .border_style(Style::default().fg(app.theme().warning)),
            )
            .column_spacing(1);

        let mut state = TableState::default();
        state.select(Some(app.ui.outdated_selection));
        frame.render_stateful_widget(table, left_area, &mut state);

        let out_items: Vec<crate::scanner::OutdatedItem> =
            items.iter().map(|(_, pkg)| pkg.clone()).collect();
        let tool_name = items
            .get(app.ui.outdated_selection)
            .map(|(t, _)| t.as_str())
            .unwrap_or("Unknown");
        render_outdated_detail_panels(
            frame,
            right_area,
            tool_name,
            &out_items,
            app.ui.outdated_selection,
            app,
            true,
        );
    } else if area.height >= 24 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let table = Table::new(rows.clone(), outdated_table_constraints(chunks[0].width))
            .header(header.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.clone())
                    .border_style(Style::default().fg(app.theme().warning)),
            )
            .column_spacing(1);

        let mut state = TableState::default();
        state.select(Some(app.ui.outdated_selection));
        frame.render_stateful_widget(table, chunks[0], &mut state);

        let out_items: Vec<crate::scanner::OutdatedItem> =
            items.iter().map(|(_, pkg)| pkg.clone()).collect();
        let tool_name = items
            .get(app.ui.outdated_selection)
            .map(|(t, _)| t.as_str())
            .unwrap_or("Unknown");
        render_outdated_detail_panels(
            frame,
            chunks[1],
            tool_name,
            &out_items,
            app.ui.outdated_selection,
            app,
            false,
        );
    } else {
        let table = Table::new(rows, outdated_table_constraints(area.width))
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(app.theme().warning)),
            )
            .column_spacing(1);

        let mut state = TableState::default();
        state.select(Some(app.ui.outdated_selection));
        frame.render_stateful_widget(table, area, &mut state);
    }
}

fn render_scanning(frame: &mut Frame, area: Rect, app: &mut App) {
    let throbber = throbber_widgets_tui::Throbber::default()
        .label("Scanning all toolchains...")
        .style(Style::default().fg(app.theme().primary))
        .throbber_style(
            Style::default()
                .fg(app.theme().primary)
                .add_modifier(Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
        .use_type(throbber_widgets_tui::WhichUse::Spin);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Envexa ")
        .border_style(Style::default().fg(app.theme().primary));

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
            frame.render_stateful_widget(throbber, chunks[0], &mut app.ui.throbber_state);

            let ratio = 1.0 - (0.96_f64).powi(app.ui.progress_counter as i32);
            let pct = (ratio * 100.0).round() as u64;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::NONE))
                .gauge_style(Style::default().fg(app.theme().primary))
                .label(Span::styled(
                    format!(" scanning... {}% ", pct),
                    Style::default().fg(app.theme().text_normal).bold(),
                ))
                .ratio(ratio);
            frame.render_widget(gauge, chunks[2]);

            let step = app.ui.progress_counter % 60;
            let spin_char = SPINNER_FRAMES[step % SPINNER_FRAMES.len()];
            let mut log_lines = vec![Line::from("")];

            if step > 0 {
                let (sym, color) = if step < 10 {
                    (spin_char, app.theme().primary)
                } else {
                    ("\u{2714}", app.theme().success)
                };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Checking project environment & lockfiles...",
                        Style::default().fg(app.theme().text_normal),
                    ),
                ]));
            }

            if step >= 10 {
                let (sym, color) = if step < 22 {
                    (spin_char, app.theme().primary)
                } else {
                    ("\u{2714}", app.theme().success)
                };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Auditing system runtimes & Homebrew formulas...",
                        Style::default().fg(app.theme().text_normal),
                    ),
                ]));
            }

            if step >= 22 {
                let (sym, color) = if step < 35 {
                    (spin_char, app.theme().primary)
                } else {
                    ("\u{2714}", app.theme().success)
                };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Scanning web development runtimes...",
                        Style::default().fg(app.theme().text_normal),
                    ),
                ]));
            }

            if step >= 35 {
                let (sym, color) = if step < 45 {
                    (spin_char, app.theme().primary)
                } else {
                    ("\u{2714}", app.theme().success)
                };
                log_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", sym), Style::default().fg(color).bold()),
                    Span::styled(
                        "Auditing project security & dependencies...",
                        Style::default().fg(app.theme().text_normal),
                    ),
                ]));
            }

            if chunks[3].height > 0 {
                let progress = Paragraph::new(log_lines).alignment(Alignment::Left);
                frame.render_widget(progress, chunks[3]);
            }
        } else {
            frame.render_stateful_widget(throbber, inner, &mut app.ui.throbber_state);
        }
    }
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let theme = app.theme();
    frame.render_widget(Block::default().bg(theme.background), area);

    if area.width < 16 || area.height < 4 {
        render_minimal(frame, area, "Envexa", app);
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

    match app.ui.view {
        View::Dashboard => render_dashboard(frame, chunks[4], app),
        View::Outdated => render_outdated(frame, chunks[4], app),
        View::Settings => render_settings(frame, chunks[4], app),
        View::Logs => render_logs(frame, chunks[4], app),
        View::Scanning => render_scanning(frame, chunks[4], app),
        View::PackageDetail => render_package_detail(frame, chunks[4], app),
        View::Updating => render_updating(frame, chunks[4], app),
    }
}

fn render_logs(frame: &mut Frame, area: Rect, app: &App) {
    let mut log_items = Vec::new();
    let theme = app.theme();

    for (time, action) in &app.logs {
        let mut level = "INFO";
        let mut source = "system";
        let mut message = action.as_str();

        if message.starts_with("INFO: ") {
            level = "INFO";
            message = &message["INFO: ".len()..];
        } else if message.starts_with("WARN: ") {
            level = "WARN";
            message = &message["WARN: ".len()..];
        } else if message.starts_with("ERROR: ") {
            level = "ERROR";
            message = &message["ERROR: ".len()..];
        } else if message.starts_with("DEBUG: ") {
            level = "DEBUG";
            message = &message["DEBUG: ".len()..];
        }

        let mut msg_owned = message.to_string();
        if let Some(start_idx) = msg_owned.rfind('[') {
            if let Some(end_idx) = msg_owned.rfind(']') {
                if start_idx < end_idx {
                    source = &message[start_idx + 1..end_idx];
                    msg_owned = message[..start_idx].trim().to_string();
                }
            }
        }

        let time_str = time.format("%H:%M:%S").to_string();

        let level_color = match level {
            "INFO" => theme.success,
            "WARN" => theme.warning,
            "ERROR" => theme.error,
            "DEBUG" => theme.primary,
            _ => theme.text_normal,
        };

        let source_color = match source.to_lowercase().as_str() {
            "rust" => Color::Rgb(255, 123, 114),    // #ff7b72
            "node" => Color::Rgb(126, 231, 135),    // #7ee787
            "python" => Color::Rgb(121, 192, 255),  // #79c0ff
            "system" => Color::Rgb(165, 214, 255),  // #a5d6ff
            "watcher" => Color::Rgb(210, 168, 255), // #d2a8ff
            _ => theme.text_muted,
        };

        let msg_color = match level {
            "ERROR" => theme.error,
            "WARN" => theme.warning,
            _ => theme.text_normal,
        };

        log_items.push(Line::from(vec![
            Span::styled(
                format!("{:<10}", time_str),
                Style::default().fg(theme.text_muted),
            ),
            Span::styled(
                format!("{:<7}", level),
                Style::default()
                    .fg(level_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}] ", source), Style::default().fg(source_color)),
            Span::styled(msg_owned, Style::default().fg(msg_color)),
        ]));
    }

    let logs_path = crate::core::config::logs_path()
        .to_string_lossy()
        .to_string();
    let title = format!(" Terminal — {} ", logs_path);

    let paragraph = Paragraph::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(theme.text_normal)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(theme.primary)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((app.ui.logs_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn render_settings(frame: &mut Frame, area: Rect, app: &mut App) {
    let items = [
        (
            "Cache TTL (Minutes)",
            format!("{}m", app.config.cache_ttl_minutes),
        ),
        (
            "Auto-Scan on Startup",
            if app.config.auto_scan_on_startup {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        ),
        ("Project Path", {
            let path = app.config.project_path.clone().unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            });
            let path_buf = std::path::PathBuf::from(&path);
            if !path_buf.exists() {
                format!("{} (not found)", path)
            } else if !path_buf.is_dir() {
                format!("{} (not a directory)", path)
            } else {
                let lockfiles = detect_lockfiles(&path_buf);
                if lockfiles.is_empty() {
                    format!("{} (no lockfiles)", path)
                } else {
                    format!("{} ({})", path, lockfiles.join(", "))
                }
            }
        }),
        ("Scan Timeout", format!("{}s", app.config.scan_timeout_secs)),
        (
            "Daemon Interval",
            match app.config.daemon_interval_secs {
                3600 => "1h",
                14400 => "4h",
                28800 => "8h",
                86400 => "24h",
                _ => "?",
            }
            .to_string(),
        ),
        ("Enabled Scanners", {
            let scanner_count = ALL_SCANNERS.len();
            let enabled = app
                .config
                .enabled_scanners
                .as_ref()
                .map_or(format!("All ({})", scanner_count), |v| {
                    format!("{}/{}", v.len(), scanner_count)
                });
            enabled
        }),
        ("Export Format", app.config.export_format.clone()),
        ("Theme", app.config.theme.clone()),
        (
            "Verbose Logs",
            if app.config.verbose_logs { "On" } else { "Off" }.to_string(),
        ),
        (
            "Log Retention",
            match app.config.log_retention_days {
                1 => "1 Day".to_string(),
                7 => "7 Days".to_string(),
                14 => "14 Days".to_string(),
                30 => "30 Days".to_string(),
                0 => "Unlimited".to_string(),
                _ => format!("{} Days", app.config.log_retention_days),
            },
        ),
        ("Update Envexa", format!("v{}", crate::core::cli::VERSION)),
    ];

    let mut rows = Vec::new();
    for (i, (label, val)) in items.iter().enumerate() {
        let sel = i == app.ui.settings_selection;
        let bg = if sel {
            app.theme().text_muted
        } else {
            Color::Reset
        };
        let fg = if sel {
            app.theme().text_normal
        } else {
            Color::Gray
        };

        let row = Row::new(vec![
            Cell::from(if sel { " \u{25B6} " } else { "   " }),
            Cell::from(*label),
            Cell::from(val.clone()),
        ])
        .style(Style::default().bg(bg).fg(fg))
        .height(2);
        rows.push(row);
    }

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ];

    let theme_color = match app.config.theme.as_str() {
        "dark" => app.theme().text_muted,
        "light" => app.theme().text_normal,
        _ => app.theme().secondary,
    };

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Application Settings ")
                .border_style(Style::default().fg(theme_color)),
        )
        .column_spacing(1);

    frame.render_widget(table, area);

    if app.ui.is_self_updating || !app.ui.update_message.is_empty() {
        let msg_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(2),
            width: area.width,
            height: 1,
        };

        if app.ui.is_self_updating {
            let label = if app.ui.update_message.is_empty() {
                "Checking for updates...".to_string()
            } else {
                app.ui.update_message.clone()
            };

            let throbber = throbber_widgets_tui::Throbber::default()
                .label(label)
                .style(Style::default().fg(app.theme().primary))
                .throbber_style(
                    Style::default()
                        .fg(app.theme().primary)
                        .add_modifier(Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
                .use_type(throbber_widgets_tui::WhichUse::Spin);

            // Render the throbber in the center
            let centered = centered_rect(50, 100, msg_area);
            frame.render_stateful_widget(throbber, centered, &mut app.ui.throbber_state);
        } else if !app.ui.settings_edit_mode {
            let msg = Paragraph::new(app.ui.update_message.clone())
                .style(Style::default().fg(app.theme().text_normal))
                .alignment(Alignment::Center);
            frame.render_widget(msg, msg_area);
        }
    }

    if app.ui.settings_edit_mode && app.ui.settings_selection == 5 {
        let scanner_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width.saturating_sub(4).min(50),
            height: (ALL_SCANNERS.len() as u16 + 2).min(area.height.saturating_sub(4)),
        };

        let enabled = app.config.enabled_scanners.as_deref().unwrap_or(&[]);

        let block = Block::default()
            .title(" Enabled Scanners ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme().primary))
            .bg(app.theme().background);

        let inner = block.inner(scanner_area);
        frame.render_widget(Clear, scanner_area);
        frame.render_widget(block, scanner_area);

        for (i, tool) in ALL_SCANNERS.iter().enumerate() {
            if (i as u16) >= inner.height {
                break;
            }
            let is_focused = i == app.ui.settings_scanner_focus;
            let is_all = app.config.enabled_scanners.is_none();
            let is_checked = is_all || enabled.iter().any(|e| e == tool);

            let checkbox = if is_checked { "[x]" } else { "[ ]" };
            let indicator = if is_focused { "\u{25B6}" } else { " " };

            let display = crate::scanner::display_name(tool);

            let line = Line::from(vec![
                Span::styled(
                    format!(" {} {} ", indicator, checkbox),
                    Style::default().fg(if is_focused {
                        app.theme().primary
                    } else if is_checked {
                        app.theme().success
                    } else {
                        app.theme().text_muted
                    }),
                ),
                Span::styled(
                    display,
                    Style::default().fg(if is_focused {
                        app.theme().text_normal
                    } else {
                        app.theme().text_muted
                    }),
                ),
            ]);

            let row_area = Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(line), row_area);
        }
    } else if app.ui.settings_edit_mode {
        let opts = app.get_settings_options();
        let items: Vec<ListItem> = opts
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let is_selected = i == app.ui.settings_edit_selection;
                let bg = if is_selected {
                    app.theme().text_muted
                } else {
                    Color::Reset
                };
                let fg = if is_selected {
                    app.theme().text_normal
                } else {
                    Color::Gray
                };
                let prefix = if is_selected { " \u{25B6} " } else { "   " };

                let desc = match app.ui.settings_selection {
                    0 => match val.as_str() {
                        "5m" => "Aggressive caching, frequent scans",
                        "15m" => "Balanced caching, recommended",
                        "30m" => "Long caching, saves resources",
                        "60m" => "Very long caching, for slow networks",
                        _ => "",
                    },
                    1 => match val.as_str() {
                        "On" => "Automatically scan when the app starts",
                        "Off" => "Wait for manual scan command",
                        _ => "",
                    },
                    2 => {
                        // No descriptions for project path - save space
                        ""
                    }
                    3 => match val.as_str() {
                        "10s" => "Fast scans, may timeout on slow machines",
                        "30s" => "Balanced timeout, recommended",
                        "60s" => "Long timeout, for slow networks",
                        "120s" => "Very long timeout, for CI/server environments",
                        _ => "",
                    },
                    4 => match val.as_str() {
                        "1h" => "Scan every hour",
                        "4h" => "Scan every 4 hours, recommended",
                        "8h" => "Scan every 8 hours, save resources",
                        "24h" => "Scan once per day",
                        _ => "",
                    },
                    6 => match val.as_str() {
                        "markdown" => "Human-readable Markdown report",
                        "sarif" => "Machine-readable SARIF for CI integration",
                        "json" => "Raw JSON dump of all scan results",
                        _ => "",
                    },
                    7 => match val.as_str() {
                        "default" => "Envexa standard colors",
                        "dark" => "Plain dark theme",
                        "light" => "Plain light theme",
                        "dracula" => "Vibrant dark theme",
                        "nord" => "Arctic, north-bluish colors",
                        "monokai" => "High-contrast dark theme",
                        "solarized-dark" => "Low-contrast dark theme",
                        "oceanic" => "Oceanic dark colors",
                        "catppuccin-mocha" => "Warm dark theme with muted pastels",
                        "catppuccin-latte" => "Warm light theme with soft pastels",
                        "gruvbox-dark" => "Retro dark with warm orange tones",
                        "gruvbox-light" => "Retro light with warm orange tones",
                        "tokyo-night" => "Deep blue-heavy night theme",
                        "rose-pine" => "Calm pine forest dark theme",
                        _ => "",
                    },
                    8 => match val.as_str() {
                        "On" => "Show detailed background logs",
                        "Off" => "Hide detailed logs",
                        _ => "",
                    },
                    _ => "",
                };

                let mut lines = vec![Line::from(vec![Span::styled(
                    format!("{}{}", prefix, val),
                    Style::default().add_modifier(Modifier::BOLD),
                )])];
                if !desc.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            desc,
                            Style::default().fg(if is_selected {
                                app.theme().text_normal
                            } else {
                                app.theme().text_muted
                            }),
                        ),
                    ]));
                }

                ListItem::new(lines).style(Style::default().bg(bg).fg(fg))
            })
            .collect();

        let popup_area = if app.ui.settings_selection == 2 {
            // Smaller popup for project path folder browser to avoid overlap
            centered_rect(55, 60, area)
        } else {
            centered_rect(50, 50, area)
        };
        let title = match app.ui.settings_selection {
            0 => " Cache TTL ",
            1 => " Auto-Scan ",
            2 => " Project Path ",
            3 => " Scan Timeout ",
            4 => " Daemon Interval ",
            5 => " Enabled Scanners ",
            6 => " Export Format ",
            7 => " Theme ",
            8 => " Verbose Logs ",
            _ => " Options ",
        };

        // Add footer hint for project path
        let block = if app.ui.settings_selection == 2 {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme().primary))
                .bg(app.theme().background)
                .title_bottom(
                    Line::from(vec![
                        Span::styled(" ↑↓ Navigate ", Style::default().fg(app.theme().text_muted)),
                        Span::styled("│ ", Style::default().fg(app.theme().text_muted)),
                        Span::styled("Enter Select ", Style::default().fg(app.theme().text_muted)),
                        Span::styled("│ ", Style::default().fg(app.theme().text_muted)),
                        Span::styled("Esc Back", Style::default().fg(app.theme().text_muted)),
                    ])
                    .alignment(Alignment::Center),
                )
        } else {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme().primary))
                .bg(app.theme().background)
        };

        let list = List::new(items).block(block);

        frame.render_widget(Clear, popup_area);

        let mut state = ListState::default();
        state.select(Some(app.ui.settings_edit_selection));
        frame.render_stateful_widget(list, popup_area, &mut state);
    }

    if app.ui.input_mode {
        let n_completions = app.ui.input_completions.len();
        let completion_rows = n_completions.min(8) as u16;
        let popup_height = (30 + completion_rows * 2).min(70);
        let popup_area = centered_rect(60, popup_height, area);
        let input_path = if let Some(idx) = app.ui.input_buffer.find("  [") {
            &app.ui.input_buffer[..idx]
        } else {
            app.ui.input_buffer.trim()
        };
        let valid = !input_path.is_empty() && std::path::Path::new(input_path).is_dir();

        let body = if app.ui.input_buffer.is_empty() {
            "Enter an absolute path, e.g. /Users/me/my-project".to_string()
        } else if valid {
            let p = std::path::Path::new(input_path);
            let file_count = std::fs::read_dir(p).map(|e| e.count()).unwrap_or(0);
            let lockfiles = detect_lockfiles(p);
            if lockfiles.is_empty() {
                format!(" \u{2713} {file_count} items  {}", input_path)
            } else {
                format!(
                    " \u{2713} {file_count} items  {}  [{}]",
                    input_path,
                    lockfiles.join(", ")
                )
            }
        } else {
            format!(" \u{2717} Path not found: {}", input_path)
        };

        let status_color = if valid {
            app.theme().success
        } else {
            app.theme().error
        };

        let mut lines = vec![
            Line::from(Span::styled(
                " Enter project path:",
                Style::default()
                    .fg(app.theme().text_normal)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                format!("   {}|", app.ui.input_buffer),
                Style::default().fg(app.theme().text_normal),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(body, Style::default().fg(status_color))),
            Line::from(Span::raw("")),
        ];

        if !app.ui.input_completions.is_empty() {
            lines.push(Line::from(Span::styled(
                " Suggestions:",
                Style::default()
                    .fg(app.theme().text_muted)
                    .add_modifier(Modifier::BOLD),
            )));
            let max_show = app.ui.input_completions.len().min(8);
            for i in 0..max_show {
                let is_selected = i == app.ui.input_completion_index;
                let marker = if is_selected { " \u{25B6} " } else { "   " };
                let fg = if is_selected {
                    app.theme().text_normal
                } else {
                    app.theme().text_muted
                };
                let (display, _lockfile_style) = if app.ui.input_completions[i].contains("  [") {
                    // Show lockfile info with different styling
                    let parts: Vec<&str> = app.ui.input_completions[i].splitn(2, "  [").collect();
                    let path = parts[0];
                    let lockfiles = parts.get(1).unwrap_or(&"");
                    (
                        format!("{}  [{}", path, lockfiles),
                        Style::default().fg(app.theme().primary),
                    )
                } else {
                    (app.ui.input_completions[i].clone(), Style::default().fg(fg))
                };
                lines.push(Line::from(vec![Span::styled(
                    format!("{}{}", marker, display),
                    Style::default().fg(fg),
                )]));
            }
            lines.push(Line::from(Span::raw("")));
            if n_completions > 8 {
                lines.push(Line::from(Span::styled(
                    format!("   ... and {} more", n_completions - 8),
                    Style::default().fg(app.theme().text_muted),
                )));
                lines.push(Line::from(Span::raw("")));
            }
        }

        lines.push(Line::from(Span::styled(
            " Tab: autocomplete   \u{2191}\u{2193}: navigate   Enter: confirm   Esc: cancel",
            Style::default().fg(app.theme().text_muted),
        )));

        let block = Block::default()
            .title(" Project Path ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme().primary))
            .bg(app.theme().background);

        let paragraph = Paragraph::new(Text::from(lines)).block(block);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }
}

fn render_package_detail(frame: &mut Frame, area: Rect, app: &App) {
    let tool = match &app.detail.tool {
        Some(t) => t.clone(),
        None => return,
    };

    match app.detail.key.as_deref() {
        Some("security") => render_vulnerabilities(frame, area, &tool, app),
        Some("audit") => render_audit_items(frame, area, &tool, app),
        Some("supply_chain") => render_supply_chain_risks(frame, area, &tool, app),
        _ => render_outdated_detail(frame, area, &tool, app),
    }
}

fn update_type_counts(items: &[crate::scanner::OutdatedItem]) -> (usize, usize, usize, usize) {
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;
    let mut unknown = 0;

    for item in items {
        let cur = item
            .current
            .trim_start_matches(|c: char| !c.is_ascii_digit());
        let lat = item
            .latest
            .trim_start_matches(|c: char| !c.is_ascii_digit());
        let cur_parts: Vec<&str> = cur.split(['.', '-', '+']).collect();
        let lat_parts: Vec<&str> = lat.split(['.', '-', '+']).collect();

        if !cur_parts.is_empty() && !lat_parts.is_empty() && cur_parts[0] != lat_parts[0] {
            major += 1;
        } else if cur_parts.len() >= 2 && lat_parts.len() >= 2 && cur_parts[1] != lat_parts[1] {
            minor += 1;
        } else if cur_parts.len() >= 3 && lat_parts.len() >= 3 && cur_parts[2] != lat_parts[2] {
            patch += 1;
        } else {
            unknown += 1;
        }
    }
    (major, minor, patch, unknown)
}

fn render_outdated_detail_panels(
    frame: &mut Frame,
    area: Rect,
    tool: &str,
    items: &[crate::scanner::OutdatedItem],
    selection: usize,
    app: &App,
    is_horizontal: bool,
) {
    let right_chunks = if is_horizontal {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(1)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let (maj, min, pat, unk) = update_type_counts(items);

    let overview_text = vec![Line::from(vec![
        Span::styled("Major: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", maj),
            Style::default()
                .fg(app.theme().error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Minor: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", min),
            Style::default()
                .fg(app.theme().warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Patch: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", pat),
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Other: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", unk),
            Style::default()
                .fg(app.theme().secondary)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let overview_block = Block::default()
        .borders(Borders::ALL)
        .title(" Update Readiness ")
        .border_style(Style::default().fg(app.theme().secondary));

    let overview_inner = overview_block.inner(right_chunks[0]);
    frame.render_widget(overview_block, right_chunks[0]);

    if overview_inner.width > 0 && overview_inner.height > 0 {
        let metric_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(4)])
            .split(overview_inner);

        frame.render_widget(Paragraph::new(overview_text), metric_chunks[0]);

        let maj_label = format!("Major ({maj})");
        let min_label = format!("Minor ({min})");
        let pat_label = format!("Patch ({pat})");
        let unk_label = format!("Other ({unk})");

        let mut slices = Vec::new();
        if maj > 0 {
            slices.push(PieSlice::new(&maj_label, maj as f64, app.theme().error));
        }
        if min > 0 {
            slices.push(PieSlice::new(&min_label, min as f64, app.theme().warning));
        }
        if pat > 0 {
            slices.push(PieSlice::new(&pat_label, pat as f64, Color::LightCyan));
        }
        if unk > 0 {
            slices.push(PieSlice::new(&unk_label, unk as f64, app.theme().secondary));
        }
        if slices.is_empty() {
            slices.push(PieSlice::new("None", 1.0, app.theme().success));
        }

        let pie = PieChart::new(slices)
            .resolution(Resolution::Braille)
            .show_legend(overview_inner.width >= 30)
            .legend_position(LegendPosition::Right)
            .legend_alignment(LegendAlignment::Center)
            .show_percentages(false);

        frame.render_widget(pie, metric_chunks[1]);
    }

    if let Some(item) = items.get(selection) {
        let update_cmd = match tool {
            "npm" | "bun" => format!("{} install {}@latest", tool, item.name),
            "cargo" => format!("cargo add {}@{}", item.name, item.latest),
            "pip" | "python" => format!("pip install --upgrade {}", item.name),
            _ => format!("Update {} to {}", item.name, item.latest),
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Package: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    &item.name,
                    Style::default()
                        .fg(app.theme().text_normal)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Source: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&item.source, source_style(&item.source, &app.theme())),
            ]),
            Line::from(vec![
                Span::styled("Current: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&item.current, Style::default().fg(app.theme().warning)),
            ]),
            Line::from(vec![
                Span::styled("Latest: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&item.latest, Style::default().fg(app.theme().success)),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&item.size, Style::default().fg(app.theme().secondary)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Quick Update Command:",
                Style::default().fg(app.theme().text_muted),
            )),
            Line::from(Span::styled(
                update_cmd,
                Style::default().fg(app.theme().primary),
            )),
        ];

        let detail_card = Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Selected Package ")
                    .border_style(Style::default().fg(app.theme().primary)),
            );

        frame.render_widget(detail_card, right_chunks[1]);
    } else {
        let empty_card = Paragraph::new("No package selected.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Selected Package "),
        );
        frame.render_widget(empty_card, right_chunks[1]);
    }
}

fn render_outdated_detail(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail.items;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Outdated Packages (0) "))
            .border_style(Style::default().fg(app.theme().success));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  \u{2714} ",
                    Style::default().fg(app.theme().success).bold(),
                ),
                Span::styled(
                    "All packages are completely up to date!",
                    Style::default().fg(app.theme().text_normal),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let sel = i == app.detail.selection;
            let checked = app.detail.checked.contains(&i);
            let cb = if checked { "[x]" } else { "[ ]" };
            let indicator = if sel {
                format!("{cb}\u{25b8}")
            } else {
                format!("{cb} ")
            };
            let mut row = Row::new(vec![
                Cell::from(indicator),
                Cell::from(item.name.as_str()),
                Cell::from(item.source.as_str()).style(source_style(&item.source, &app.theme())),
                Cell::from(item.current.as_str()),
                Cell::from(item.latest.as_str()),
                Cell::from(item.size.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let sub = if !app.detail.message.is_empty() {
        Some(format!("  {}", app.detail.message))
    } else {
        None
    };

    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Outdated Packages ({}) ", items.len()),
            Style::default().fg(app.theme().primary),
            &["", "Package ", "Source ", "Current ", "Latest ", "Size "],
            &rows,
            "outdated",
            sub.as_deref(),
        );

        render_outdated_detail_panels(
            frame,
            chunks[1],
            tool,
            items,
            app.detail.selection,
            app,
            true,
        );
    } else if area.height >= 24 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Outdated Packages ({}) ", items.len()),
            Style::default().fg(app.theme().primary),
            &["", "Package ", "Source ", "Current ", "Latest ", "Size "],
            &rows,
            "outdated",
            sub.as_deref(),
        );

        render_outdated_detail_panels(
            frame,
            chunks[1],
            tool,
            items,
            app.detail.selection,
            app,
            false,
        );
    } else {
        render_item_table(
            frame,
            area,
            app,
            format!(" {tool} — Outdated Packages ({}) ", items.len()),
            Style::default().fg(app.theme().primary),
            &["", "Package ", "Source ", "Current ", "Latest ", "Size "],
            &rows,
            "outdated",
            sub.as_deref(),
        );
    }
}

fn render_vulnerability_detail_panels(
    frame: &mut Frame,
    area: Rect,
    items: &[crate::toolchains::VulnerabilityInfo],
    selection: usize,
    app: &App,
    is_horizontal: bool,
) {
    let right_chunks = if is_horizontal {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(1)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let (crit, high, mod_cnt, other) = severity_counts(items);
    let score = 100_usize
        .saturating_sub(crit * 25 + high * 10 + mod_cnt * 5)
        .clamp(0, 100) as u16;
    let score_color = if score >= 90 {
        app.theme().success
    } else if score >= 70 {
        app.theme().warning
    } else {
        app.theme().error
    };

    let gauge = LineGauge::default()
        .block(Block::default().title(" Security Health Score "))
        .filled_style(Style::default().fg(score_color))
        .unfilled_style(Style::default().fg(app.theme().text_muted))
        .ratio(score as f64 / 100.0);

    let overview_text = vec![Line::from(vec![
        Span::styled("Critical: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", crit),
            Style::default()
                .fg(app.theme().error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" High: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", high),
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Mod: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", mod_cnt),
            Style::default()
                .fg(app.theme().warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Low: ", Style::default().fg(app.theme().text_muted)),
        Span::styled(
            format!("{} ", other),
            Style::default()
                .fg(app.theme().secondary)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let overview_block = Block::default()
        .borders(Borders::ALL)
        .title(" Security Scorecard ")
        .border_style(Style::default().fg(app.theme().secondary));

    let overview_inner = overview_block.inner(right_chunks[0]);
    frame.render_widget(overview_block, right_chunks[0]);

    if overview_inner.width > 0 && overview_inner.height > 0 {
        let metric_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(4),
            ])
            .split(overview_inner);

        frame.render_widget(gauge, metric_chunks[0]);
        frame.render_widget(Paragraph::new(overview_text), metric_chunks[1]);

        let crit_label = format!("Critical ({crit})");
        let high_label = format!("High ({high})");
        let mod_label = format!("Moderate ({mod_cnt})");
        let low_label = format!("Low ({other})");

        let mut slices = Vec::new();
        if crit > 0 {
            slices.push(PieSlice::new(&crit_label, crit as f64, app.theme().error));
        }
        if high > 0 {
            slices.push(PieSlice::new(&high_label, high as f64, Color::LightRed));
        }
        if mod_cnt > 0 {
            slices.push(PieSlice::new(
                &mod_label,
                mod_cnt as f64,
                app.theme().warning,
            ));
        }
        if other > 0 {
            slices.push(PieSlice::new(
                &low_label,
                other as f64,
                app.theme().secondary,
            ));
        }
        if slices.is_empty() {
            slices.push(PieSlice::new("None", 1.0, app.theme().success));
        }

        let pie = PieChart::new(slices)
            .resolution(Resolution::Braille)
            .show_legend(overview_inner.width >= 30)
            .legend_position(LegendPosition::Right)
            .legend_alignment(LegendAlignment::Center)
            .show_percentages(false);

        frame.render_widget(pie, metric_chunks[2]);
    }

    if let Some(vuln) = items.get(selection) {
        let cve = vuln.cve.as_deref().unwrap_or("None");
        let card_border_color = match vuln.severity.to_ascii_lowercase().as_str() {
            "critical" => app.theme().error,
            "high" => Color::LightRed,
            "moderate" | "medium" => app.theme().warning,
            "low" => app.theme().secondary,
            _ => app.theme().text_muted,
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Package: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    &vuln.package,
                    Style::default()
                        .fg(app.theme().text_normal)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Severity: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&vuln.severity, severity_style(&vuln.severity, &app.theme())),
            ]),
            Line::from(vec![
                Span::styled("CVE ID: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(cve, Style::default().fg(app.theme().primary)),
            ]),
            Line::from(vec![
                Span::styled("Patched: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    &vuln.patched_version,
                    Style::default().fg(app.theme().success),
                ),
            ]),
            Line::from(vec![
                Span::styled("Dep Path: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    if vuln.dependency_path.is_empty() {
                        "Direct/Unknown".to_string()
                    } else {
                        vuln.dependency_path.join(" > ")
                    },
                    Style::default().fg(app.theme().primary),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Title / Description:",
                Style::default().fg(app.theme().text_muted),
            )),
            Line::from(Span::styled(
                &vuln.title,
                Style::default().fg(app.theme().text_normal),
            )),
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
}

fn render_vulnerabilities(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail.vulns;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Vulnerabilities (0) "))
            .border_style(Style::default().fg(app.theme().success));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  \u{2714} ",
                    Style::default().fg(app.theme().success).bold(),
                ),
                Span::styled(
                    "No security vulnerabilities detected!",
                    Style::default().fg(app.theme().text_normal),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

    let bottom_msg = if !app.detail.message.is_empty() {
        Some(format!(
            "  {}  |  [E] Export Report  [Esc] Back ",
            app.detail.message
        ))
    } else {
        Some("  [E] Export Report  |  [Esc] Back ".to_string())
    };

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let sel = i == app.detail.selection;
            let cve = v.cve.as_deref().unwrap_or("-");
            let mut row = Row::new(vec![
                Cell::from(v.package.as_str()),
                Cell::from(v.severity.as_str()).style(severity_style(&v.severity, &app.theme())),
                Cell::from(cve),
                Cell::from(v.title.as_str()),
                Cell::from(v.patched_version.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    // Responsive 2-column layout
    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Vulnerabilities ({}) ", items.len()),
            Style::default().fg(app.theme().error),
            &["Package ", "Severity ", "CVE ", "Title ", "Patched In "],
            &rows,
            "security",
            bottom_msg.as_deref(),
        );

        render_vulnerability_detail_panels(
            frame,
            chunks[1],
            items,
            app.detail.selection,
            app,
            true,
        );
    } else if area.height >= 24 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Vulnerabilities ({}) ", items.len()),
            Style::default().fg(app.theme().error),
            &["Package ", "Severity ", "CVE ", "Title ", "Patched In "],
            &rows,
            "security",
            bottom_msg.as_deref(),
        );

        render_vulnerability_detail_panels(
            frame,
            chunks[1],
            items,
            app.detail.selection,
            app,
            false,
        );
    } else {
        // Fallback layout (narrow terminal)
        render_item_table(
            frame,
            area,
            app,
            format!(" {tool} — Vulnerabilities ({}) ", items.len()),
            Style::default().fg(app.theme().error),
            &["Package ", "Severity ", "CVE ", "Title ", "Patched In "],
            &rows,
            "security",
            bottom_msg.as_deref(),
        );
    }
}

fn render_audit_detail_panels(
    frame: &mut Frame,
    area: Rect,
    items: &[crate::toolchains::AuditItem],
    selection: usize,
    app: &App,
    is_horizontal: bool,
) {
    let right_chunks = if is_horizontal {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let alignment_score = 100_usize.saturating_sub(items.len() * 20).clamp(0, 100) as u16;
    let gauge_color = if alignment_score >= 80 {
        app.theme().success
    } else if alignment_score >= 50 {
        app.theme().warning
    } else {
        app.theme().error
    };

    let gauge = LineGauge::default()
        .block(Block::default().title(" System Alignment Score "))
        .filled_style(Style::default().fg(gauge_color))
        .unfilled_style(Style::default().fg(app.theme().text_muted))
        .ratio(alignment_score as f64 / 100.0);

    frame.render_widget(gauge, right_chunks[0]);

    if let Some(audit) = items.get(selection) {
        let lines = vec![
            Line::from(vec![
                Span::styled("Audit Rule: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    &audit.name,
                    Style::default()
                        .fg(app.theme().text_normal)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Current State: ",
                    Style::default().fg(app.theme().text_muted),
                ),
                Span::styled(&audit.current, Style::default().fg(app.theme().warning)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Recommendation / Note:",
                Style::default().fg(app.theme().text_muted),
            )),
            Line::from(Span::styled(
                &audit.note,
                Style::default().fg(app.theme().text_normal),
            )),
        ];

        let detail_card = Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Recommendation Detail ")
                    .border_style(Style::default().fg(app.theme().warning)),
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
}

fn render_audit_items(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail.audits;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Audit Items (0) "))
            .border_style(Style::default().fg(app.theme().success));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  \u{2714} ",
                    Style::default().fg(app.theme().success).bold(),
                ),
                Span::styled(
                    "System and toolchains are aligned! No issues flagged.",
                    Style::default().fg(app.theme().text_normal),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let sel = i == app.detail.selection;
            let mut row = Row::new(vec![
                Cell::from(a.name.as_str()),
                Cell::from(a.current.as_str()),
                Cell::from(a.note.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let bottom_msg = if !app.detail.message.is_empty() {
        Some(format!(
            "  {}  |  [E] Export Report  [Esc] Back ",
            app.detail.message
        ))
    } else {
        Some("  [E] Export Report  |  [Esc] Back ".to_string())
    };

    // Responsive 2-column layout
    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Audit Items ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Name ", "Current ", "Note "],
            &rows,
            "audit",
            bottom_msg.as_deref(),
        );

        render_audit_detail_panels(frame, chunks[1], items, app.detail.selection, app, true);
    } else if area.height >= 24 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Audit Items ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Name ", "Current ", "Note "],
            &rows,
            "audit",
            bottom_msg.as_deref(),
        );

        render_audit_detail_panels(frame, chunks[1], items, app.detail.selection, app, false);
    } else {
        // Fallback layout (narrow terminal)
        render_item_table(
            frame,
            area,
            app,
            format!(" {tool} — Audit Items ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Name ", "Current ", "Note "],
            &rows,
            "audit",
            bottom_msg.as_deref(),
        );
    }
}

fn render_updating(frame: &mut Frame, area: Rect, app: &mut App) {
    let label = if app.ui.update_package_name.is_empty() {
        "Updating packages...".to_string()
    } else if app.ui.update_downloaded_mb > 0.0 {
        format!(
            "Updating {} ({:.2} MB downloaded)...",
            app.ui.update_package_name, app.ui.update_downloaded_mb
        )
    } else {
        format!("Updating {}...", app.ui.update_package_name)
    };

    let throbber = throbber_widgets_tui::Throbber::default()
        .label(label)
        .style(Style::default().fg(app.theme().success))
        .throbber_style(
            Style::default()
                .fg(app.theme().success)
                .add_modifier(Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT)
        .use_type(throbber_widgets_tui::WhichUse::Spin);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Updating ")
        .border_style(Style::default().fg(app.theme().success));

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
            frame.render_stateful_widget(throbber, chunks[0], &mut app.ui.throbber_state);

            let ratio = 1.0 - (0.96_f64).powi(app.ui.progress_counter as i32);
            let pct = (ratio * 100.0).round() as u64;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::NONE))
                .gauge_style(Style::default().fg(app.theme().success))
                .label(Span::styled(
                    format!(" updating... {}% ", pct),
                    Style::default().fg(app.theme().text_normal).bold(),
                ))
                .ratio(ratio);
            frame.render_widget(gauge, chunks[2]);

            if chunks[3].height >= 3 {
                let tip_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme().text_muted))
                    .title(" Loading Tip ")
                    .title_style(Style::default().fg(app.theme().warning).bold());

                let mut tip_lines = vec![Line::from(vec![
                    Span::styled("💡 Tip: ", Style::default().fg(app.theme().warning).bold()),
                    Span::raw("Downloading updates can take a moment depending on your network connection."),
                ])];

                if app.ui.update_downloaded_mb > 0.0 {
                    tip_lines.push(Line::from(vec![
                        Span::styled(
                            "⤓ Progress: ",
                            Style::default().fg(app.theme().primary).bold(),
                        ),
                        Span::styled(
                            format!(
                                "{:.2} MB downloaded so far for {}",
                                app.ui.update_downloaded_mb, app.ui.update_package_name
                            ),
                            Style::default().fg(app.theme().text_normal).bold(),
                        ),
                    ]));
                } else if !app.ui.update_package_name.is_empty() {
                    tip_lines.push(Line::from(vec![
                        Span::styled(
                            "⚡ Active: ",
                            Style::default().fg(app.theme().primary).bold(),
                        ),
                        Span::styled(
                            format!("Installing/updating {}...", app.ui.update_package_name),
                            Style::default().fg(app.theme().text_normal),
                        ),
                    ]));
                }

                let tip_paragraph = Paragraph::new(tip_lines)
                    .block(tip_block)
                    .alignment(Alignment::Left);
                frame.render_widget(tip_paragraph, chunks[3]);
            }
        } else {
            frame.render_stateful_widget(throbber, inner, &mut app.ui.throbber_state);
        }
    }
}

fn render_supply_chain_risks(frame: &mut Frame, area: Rect, tool: &str, app: &App) {
    let items = &app.detail.supply_chains;

    if items.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {tool} — Supply Chain Risks (0) "))
            .border_style(Style::default().fg(app.theme().success));
        let text = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  \u{2714} ",
                    Style::default().fg(app.theme().success).bold(),
                ),
                Span::styled(
                    "No supply chain risks detected! Your dependencies look safe.",
                    Style::default().fg(app.theme().text_normal),
                ),
            ]),
        ]))
        .block(block);
        frame.render_widget(text, area);
        return;
    }

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let sel = i == app.detail.selection;
            let mut row = Row::new(vec![
                Cell::from(r.package.as_str()),
                Cell::from(r.risk_type.as_str()),
                Cell::from(r.description.as_str()),
            ]);
            if sel {
                row = row.style(
                    Style::default()
                        .bg(app.theme().text_muted)
                        .add_modifier(Modifier::BOLD),
                );
            }
            row
        })
        .collect();

    let bottom_msg = if !app.detail.message.is_empty() {
        Some(format!(
            "  {}  |  [E] Export Report  [Esc] Back ",
            app.detail.message
        ))
    } else {
        Some("  [E] Export Report  |  [Esc] Back ".to_string())
    };

    if area.width >= 100 && area.height >= 8 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Supply Chain Risks ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Package ", "Risk Type ", "Description "],
            &rows,
            "supply_chain",
            bottom_msg.as_deref(),
        );

        render_supply_chain_detail_panels(frame, chunks[1], items, app.detail.selection, app, true);
    } else if area.height >= 24 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        render_item_table(
            frame,
            chunks[0],
            app,
            format!(" {tool} — Supply Chain Risks ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Package ", "Risk Type ", "Description "],
            &rows,
            "supply_chain",
            bottom_msg.as_deref(),
        );

        render_supply_chain_detail_panels(
            frame,
            chunks[1],
            items,
            app.detail.selection,
            app,
            false,
        );
    } else {
        // Fallback layout (narrow terminal)
        render_item_table(
            frame,
            area,
            app,
            format!(" {tool} — Supply Chain Risks ({}) ", items.len()),
            Style::default().fg(app.theme().warning),
            &["Package ", "Risk Type ", "Description "],
            &rows,
            "supply_chain",
            bottom_msg.as_deref(),
        );
    }
}

fn render_supply_chain_detail_panels(
    frame: &mut Frame,
    area: Rect,
    items: &[crate::toolchains::SupplyChainRisk],
    selection: usize,
    app: &App,
    is_horizontal: bool,
) {
    let right_chunks = if is_horizontal {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let risk_score = 100_usize.saturating_sub(items.len() * 15).clamp(0, 100) as u16;
    let gauge_color = if risk_score >= 80 {
        app.theme().success
    } else if risk_score >= 50 {
        app.theme().warning
    } else {
        app.theme().error
    };

    let gauge = LineGauge::default()
        .block(Block::default().title(" Supply Chain Health Score "))
        .filled_style(Style::default().fg(gauge_color))
        .unfilled_style(Style::default().fg(app.theme().text_muted))
        .ratio(risk_score as f64 / 100.0);

    frame.render_widget(gauge, right_chunks[0]);

    if let Some(risk) = items.get(selection) {
        let lines = vec![
            Line::from(vec![
                Span::styled("Package: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(
                    &risk.package,
                    Style::default()
                        .fg(app.theme().text_normal)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Risk Type: ", Style::default().fg(app.theme().text_muted)),
                Span::styled(&risk.risk_type, Style::default().fg(app.theme().warning)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Description:",
                Style::default().fg(app.theme().text_muted),
            )),
            Line::from(Span::styled(
                &risk.description,
                Style::default().fg(app.theme().text_normal),
            )),
        ];

        let detail_card = Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Risk Detail ")
                    .border_style(Style::default().fg(app.theme().warning)),
            );

        frame.render_widget(detail_card, right_chunks[1]);
    } else {
        let empty_card = Paragraph::new("No item selected.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Risk Detail "),
        );
        frame.render_widget(empty_card, right_chunks[1]);
    }
}
