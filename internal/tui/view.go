package tui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/lipgloss"
)

func (m Model) View() string {
	if m.scanning {
		return m.viewScanning()
	}
	if m.view == ViewUpdating {
		return m.viewUpdating()
	}
	switch m.view {
	case ViewOutdated:
		return m.viewOutdated()
	case ViewPackageDetail:
		return m.viewPackageDetail()
	case ViewLogs:
		return m.viewLogs()
	case ViewSettings:
		return m.viewSettings()
	default:
		return m.viewDashboard()
	}
}

// viewPackageDetail shows the selected outdated package (Rust detail view);
// `y` confirms the update, Esc returns to the outdated list.
func (m Model) viewPackageDetail() string {
	var b strings.Builder
	b.WriteString(titleStyle.Render("Package detail") + "\n\n")
	rows := [][2]string{
		{"package", m.detail.name},
		{"source", m.detail.source},
		{"current", m.detail.current},
		{"latest", m.detail.latest},
	}
	for _, r := range rows {
		b.WriteString(okStyle.Render(pad(r[0], 10)) + r[1] + "\n")
	}
	if m.updateMsg != "" {
		b.WriteString("\n" + statusStyle("error").Render(m.updateMsg) + "\n")
	}
	b.WriteString("\n" + dimStyle.Render("y update · esc back"))
	return b.String()
}

func (m Model) viewUpdating() string {
	return fmt.Sprintf("%s\n\n%s %s\n\n%s",
		titleStyle.Render("Updating"),
		m.spinner.View(), m.updateMsg,
		dimStyle.Render("q quit"))
}

func tabs(view View, width int) string {
	dash, out := tabActive.Render(" Dashboard "), tabInactive.Render(" Outdated ")
	if view == ViewOutdated {
		dash, out = tabInactive.Render(" Dashboard "), tabActive.Render(" Outdated ")
	}
	bar := lipgloss.JoinHorizontal(lipgloss.Top, dash, out)
	if width < 60 { // compact title under narrow widths, same rule as ui.rs
		bar = lipgloss.JoinHorizontal(lipgloss.Top, tabActive.Render("D"), tabInactive.Render("O"))
		if view == ViewOutdated {
			bar = lipgloss.JoinHorizontal(lipgloss.Top, tabInactive.Render("D"), tabActive.Render("O"))
		}
	}
	return bar
}

func (m Model) viewScanning() string {
	title := titleStyle.Render("envexa")
	if m.width >= 60 {
		title = titleStyle.Render("envexa — dependency environment scanner")
	}
	return fmt.Sprintf("%s\n\n%s Scanning toolchains…\n\n%s",
		title, m.spinner.View(), dimStyle.Render("s rescan · h home · q quit"))
}

func (m Model) viewDashboard() string {
	var b strings.Builder
	b.WriteString(tabs(m.view, m.width) + "\n")

	// Guarded rendering: below the minimal threshold skip tables and charts
	// entirely (mirrors ui.rs tiny-terminal fallback).
	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	if m.report.Timestamp == "" {
		b.WriteString(dimStyle.Render("no report yet — press s to scan") + "\n")
		b.WriteString(dimStyle.Render("s scan · o outdated · l logs · c settings · q quit"))
		return b.String()
	}

	switch m.dashTab {
	case dashVulns:
		b.WriteString(borderStyle.Render(m.vulnTable.View()) + "\n")
		if sec, ok := m.report.Results["security"]; ok {
			b.WriteString(dimStyle.Render(strings.Join(sec.Issues, " · ")) + "\n")
		}
	case dashToolchains:
		b.WriteString(borderStyle.Render(m.toolTable.View()) + "\n")
	default:
		b.WriteString(m.gauges() + "\n")
		if m.width >= 80 { // pie/chart area only when there is room
			b.WriteString(m.distribution() + "\n")
		}
		b.WriteString(borderStyle.Render(m.dashTable.View()) + "\n")
	}
	b.WriteString(dimStyle.Render("s scan · o outdated · l logs · c settings · tab vulns/toolchains · ↑/↓ select · h home · q quit"))
	return b.String()
}

func (m Model) viewOutdated() string {
	var b strings.Builder
	b.WriteString(tabs(m.view, m.width) + "\n")

	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	if len(m.report.Outdated) > 0 {
		b.WriteString(borderStyle.Render(m.outTable.View()) + "\n")
		b.WriteString(fmt.Sprintf("%d outdated packages\n", len(m.report.Outdated)))
	} else {
		b.WriteString(dimStyle.Render("no report yet — press s to scan") + "\n")
	}
	b.WriteString(dimStyle.Render("s scan · ↑/↓ select · ←/→ tab · h home · q quit"))
	return b.String()
}

// viewLogs shows the scan history from ~/.local/share/envexa/logs.json —
// newest last, like the Rust logs view reads them.
func (m Model) viewLogs() string {
	var b strings.Builder
	b.WriteString(tabs(m.view, m.width) + "\n")

	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	logs := readLogs()
	if len(logs) == 0 {
		b.WriteString(dimStyle.Render("no logs yet — run a scan") + "\n")
	} else {
		n := len(logs)
		start := max(0, n-20)
		for i := start; i < n; i++ {
			b.WriteString(dimStyle.Render(logs[i][0]) + " " + logs[i][1] + "\n")
		}
	}
	b.WriteString(dimStyle.Render("s scan · o outdated · h home · q quit"))
	return b.String()
}

// viewSettings renders the persisted UserConfig plus data paths.
func (m Model) viewSettings() string {
	var b strings.Builder
	b.WriteString(tabs(m.view, m.width) + "\n")

	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	cfg := readConfig()
	dir := dataDir()
	rows := [][2]string{
		{"theme", cfg.Theme},
		{"scan_timeout_secs", itou(cfg.ScanTimeoutSecs)},
		{"daemon_interval_secs", itou(cfg.DaemonIntervalSecs)},
		{"export_format", cfg.ExportFormat},
		{"log_retention_days", itou(cfg.LogRetentionDays)},
		{"project_path", cfg.ProjectPath},
		{"data dir", dir},
	}
	for _, r := range rows {
		b.WriteString(okStyle.Render(pad(r[0], 22)) + r[1] + "\n")
	}
	if len(cfg.RecentPaths) > 0 {
		b.WriteString("\n" + titleStyle.Render("recent projects") + "\n")
		for _, p := range cfg.RecentPaths {
			b.WriteString("  " + dimStyle.Render(p) + "\n")
		}
	}
	b.WriteString(dimStyle.Render("s scan · o outdated · h home · q quit"))
	return b.String()
}

func pad(s string, n int) string {
	for len(s) < n {
		s += " "
	}
	return s
}

func itou(v uint64) string {
	if v == 0 {
		return "—"
	}
	return fmt.Sprintf("%d", v)
}

// gauges renders readiness (% tools ok) and health (% non-error) via
// bubbles/progress — the LineGauge/Gauge equivalents.
func (m Model) gauges() string {
	total := len(m.report.Results)
	if total == 0 {
		return ""
	}
	ok, err := 0, 0
	for _, r := range m.report.Results {
		switch r.Status {
		case "ok":
			ok++
		case "error":
			err++
		}
	}
	readiness := float64(ok) / float64(total)
	health := float64(total-err) / float64(total)
	return fmt.Sprintf("Readiness %s %.0f%%\nHealth    %s %.0f%%",
		m.ready.ViewAs(readiness), readiness*100,
		m.health.ViewAs(health), health*100)
}

// distribution renders the ok/warn/error/skipped split as a colored bar —
// spike stands in for tui-piechart; a bar fallback is the documented target
// under narrow widths.
func (m Model) distribution() string {
	counts := map[string]int{"ok": 0, "warn": 0, "error": 0, "skipped": 0}
	for _, r := range m.report.Results {
		if _, known := counts[r.Status]; known {
			counts[r.Status]++
		} else {
			counts["skipped"]++
		}
	}
	total := len(m.report.Results)
	if total == 0 {
		return ""
	}
	segs := []string{}
	legend := []string{}
	for _, st := range []string{"ok", "warn", "error", "skipped"} {
		n := counts[st]
		if n == 0 {
			continue
		}
		width := n * max(20, m.width-40) / total
		segs = append(segs, statusStyle(st).Render(strings.Repeat("█", width)))
		legend = append(legend, statusStyle(st).Render(fmt.Sprintf("%s %d", st, n)))
	}
	return strings.Join(segs, "") + "\n" + strings.Join(legend, "  ")
}
