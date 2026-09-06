package tui

import (
	"fmt"
	"math"
	"os"
	"strings"
	"time"

	"github.com/charmbracelet/lipgloss"

	"github.com/KurutoDenzeru/envexa/internal/cli"
)

// Dashboard chrome mirrors the Ratatui original (src/tui/ui.rs): ASCII logo,
// project-path subtitle, tab bar, key hints, status line, grouped toolchain
// panels, readiness panel, and footer.

const logoArt = `███████╗███╗   ██╗██╗   ██╗███████╗██╗  ██╗ █████╗
██╔════╝████╗  ██║██║   ██║██╔════╝╚██╗██╔╝██╔══██╗
█████╗ ██╔██╗ ██║██║   ██║█████╗   ╚███╔╝ ███████║
██╔══╝ ██║╚██╗██║╚██╗ ██╔╝██╔══╝   ██╔██╗ ██╔══██║
███████╗██║ ╚████║ ╚████╔╝ ███████╗██╔╝ ██╗██║  ██║
╚══════╝╚═╝  ╚═══╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝`

var (
	logoStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("86"))
	accentStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("86"))
	selectedRow = lipgloss.NewStyle().Background(lipgloss.Color("6")).Foreground(lipgloss.Color("0"))
	panelTitle  = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
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

// tabsBar renders the top navigation with the active view highlighted.
func (m Model) tabsBar() string {
	names := []struct {
		v View
		n string
	}{{ViewDashboard, "Dashboard"}, {ViewOutdated, "Outdated"}, {ViewLogs, "Logs"}, {ViewSettings, "Settings"}}
	parts := []string{}
	for _, t := range names {
		if t.v == m.view || (t.v == ViewDashboard && m.view == ViewPackageDetail) {
			parts = append(parts, accentStyle.Bold(true).Render(t.n))
		} else {
			parts = append(parts, dimStyle.Render(t.n))
		}
	}
	sep := dimStyle.Render(" │ ")
	bar := strings.Join(parts, sep)
	if m.width < 60 {
		bar = strings.Join([]string{parts[0][:1], parts[1][:1], parts[2][:1], parts[3][:1]}, " ")
		// first rune of each styled name, keep active accent
		bar = ""
		for i, t := range names {
			r := []rune(t.n)[0]
			if t.v == m.view {
				bar += accentStyle.Bold(true).Render(string(r))
			} else {
				bar += dimStyle.Render(string(r))
			}
			if i < len(names)-1 {
				bar += " "
			}
		}
	}
	return bar
}

// hints renders the colored key-binding line.
func (m Model) hints() string {
	key := func(k, label string) string {
		return accentStyle.Render("["+k+"]") + label + "  "
	}
	if m.width < 70 {
		return dimStyle.Render("[S]can [O]utd [L]ogs [C]fg [Q]uit")
	}
	return key("S", "can") + key("O", "utdated") + key("L", "ogs") + key("C", "onfig") +
		dimStyle.Render("←→ tabs  ↑↓ nav  ") + accentStyle.Render("[Q]") + dimStyle.Render("uit")
}

// statusLine renders the health percentage, colored status dots, outdated
// total, and relative scan age — the "64% ● 9 ● 3 ● 0 ● 2 ● 119 outdated" bar.
func (m Model) statusLine() string {
	ok, warn, errC, skip := m.statusCounts()
	total := ok + warn + errC + skip
	if total == 0 {
		return ""
	}
	health := (ok + warn + skip) * 100 / total
	dot := func(n int, st string) string {
		return statusStyle(st).Render("●") + fmt.Sprintf(" %d ", n)
	}
	return fmt.Sprintf("%s%%  ", accentStyle.Bold(true).Render(fmt.Sprintf("%d", health))) +
		dot(ok, "ok") + dot(warn, "warn") + dot(errC, "error") + dot(skip, "skipped") +
		statusStyle("warn").Render("●") + fmt.Sprintf(" %d outdated  ", len(m.report.Outdated)) +
		dimStyle.Render("⏱ "+m.scanAge())
}

func (m Model) scanAge() string {
	ts, err := time.Parse(time.RFC3339, m.report.Timestamp)
	if err != nil {
		if ts, err = time.Parse("2006-01-02T15:04:05", m.report.Timestamp); err != nil {
			return "—"
		}
	}
	d := time.Since(ts)
	switch {
	case d < time.Minute:
		return "just now"
	case d < time.Hour:
		return fmt.Sprintf("%dm ago", int(d.Minutes()))
	default:
		return fmt.Sprintf("%dh ago", int(d.Hours()))
	}
}

func (m Model) statusCounts() (ok, warn, errC, skip int) {
	for _, r := range m.report.Results {
		switch r.Status {
		case "ok":
			ok++
		case "warn", "warning":
			warn++
		case "error":
			errC++
		default:
			skip++
		}
	}
	return
}

func (m Model) viewScanning() string {
	title := titleStyle.Render("envexa")
	if m.width >= 60 {
		title = titleStyle.Render("envexa — dependency environment scanner")
	}
	return fmt.Sprintf("%s\n\n%s Scanning toolchains…\n\n%s",
		title, m.spinner.View(), dimStyle.Render("s rescan · h home · q quit"))
}

// viewDashboard assembles header + two-column body + footer like ui.rs.
func (m Model) viewDashboard() string {
	var b strings.Builder
	b.WriteString(m.dashboardHeader())

	// Guarded rendering: below the minimal threshold skip tables and charts
	// entirely (mirrors ui.rs tiny-terminal fallback).
	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	if m.report.Timestamp == "" {
		b.WriteString(dimStyle.Render("no report yet — press s to scan") + "\n")
		b.WriteString(m.hints())
		return b.String()
	}

	switch m.dashTab {
	case dashVulns:
		b.WriteString(panel("Vulnerabilities", m.width-2,
			strings.Split(strings.TrimRight(m.vulnTable.View(), "\n"), "\n")) + "\n")
		if sec, ok := m.report.Results["security"]; ok {
			b.WriteString(dimStyle.Render(strings.Join(sec.Issues, " · ")) + "\n")
		}
	case dashToolchains:
		b.WriteString(panel("All toolchains", m.width-2,
			strings.Split(strings.TrimRight(m.toolTable.View(), "\n"), "\n")) + "\n")
	default:
		b.WriteString(m.dashboardBody())
	}
	b.WriteString(m.footer())
	return b.String()
}

func (m Model) dashboardHeader() string {
	var b strings.Builder
	if m.width >= 100 && m.height >= 22 {
		b.WriteString(logoStyle.Render(logoArt) + "\n")
		b.WriteString(lipgloss.NewStyle().Width(m.width).Align(lipgloss.Center).
			Render(dimStyle.Render(projectPath())) + "\n")
	}
	b.WriteString(m.tabsBar() + "\n")
	b.WriteString(m.hints() + "\n")
	if m.report.Timestamp != "" {
		b.WriteString(m.statusLine() + "\n")
	}
	return b.String()
}

func projectPath() string {
	if p := readConfig().ProjectPath; p != "" {
		return p
	}
	cwd, err := os.Getwd()
	if err != nil {
		return "—"
	}
	return cwd
}

// dashboardBody lays out the left column (Overview pie, Project Tooling) and
// the right column (grouped toolchain panels) side by side; below ~100 cols
// the panels stack vertically with the pie hidden.
func (m Model) dashboardBody() string {
	leftW := 38
	rightW := m.width - leftW - 2
	wide := m.width >= 100 && rightW > 40
	if !wide {
		leftW = m.width - 2
		rightW = m.width - 2
	}

	overview := panel("Overview", leftW, m.overviewLines(leftW))
	tooling := panel("Project Tooling", leftW, m.toolingLines(leftW))
	left := lipgloss.JoinVertical(lipgloss.Left, overview, tooling)

	right := lipgloss.JoinVertical(lipgloss.Left, m.groupPanels(rightW)...)

	if wide {
		return lipgloss.JoinHorizontal(lipgloss.Top, left, " ", right) + "\n"
	}
	return left + "\n" + right + "\n"
}

// overviewLines: status legend + dot pie (pie hidden on narrow terminals).
func (m Model) overviewLines(width int) []string {
	ok, warn, errC, skip := m.statusCounts()
	counts := map[string]int{"ok": ok, "warn": warn, "error": errC, "skipped": skip}
	lines := wrapChunks(legendParts(counts), width-2)
	if width-2 >= 22 {
		lines = append(lines, "")
		lines = append(lines, pieChart(counts, ok+warn+errC+skip)...)
	}
	return lines
}

func legendParts(counts map[string]int) []string {
	parts := []string{}
	for _, st := range []string{"ok", "warn", "error", "skipped"} {
		if counts[st] == 0 {
			continue
		}
		parts = append(parts, statusStyle(st).Render(
			fmt.Sprintf("■ %s (%d)", strings.ToUpper(label(st)), counts[st])))
	}
	return parts
}

// wrapChunks joins styled chunks with two spaces, splitting into lines that
// fit width (lipgloss-aware).
func wrapChunks(parts []string, width int) []string {
	var lines []string
	cur := ""
	for _, p := range parts {
		cand := p
		if cur != "" {
			cand = cur + "  " + p
		}
		if lipgloss.Width(cand) > width && cur != "" {
			lines = append(lines, cur)
			cur = p
			continue
		}
		cur = cand
	}
	if cur != "" {
		lines = append(lines, cur)
	}
	return lines
}

func label(st string) string {
	switch st {
	case "ok":
		return "pass"
	case "warn":
		return "warn"
	case "error":
		return "error"
	default:
		return "skip"
	}
}

// pieChart renders the status distribution as a colored dot pie — the
// Go counterpart of tui-piechart in ui.rs.
func pieChart(counts map[string]int, total int) []string {
	if total == 0 {
		return []string{""}
	}
	const R = 4
	order := []string{"ok", "warn", "error", "skipped"}
	rows := []string{}
	for y := -R; y <= R; y++ {
		line := ""
		for x := -R; x <= R; x++ {
			dx := float64(x) * 0.5 // cell aspect correction
			dy := float64(y)
			if dx*dx+dy*dy > float64(R)*float64(R)+0.5 {
				line += "  "
				continue
			}
			deg := math.Mod(math.Atan2(dx, -dy)*180/math.Pi+360, 360)
			acc := 0.0
			st := "skipped"
			for _, s := range order {
				acc += float64(counts[s]) / float64(total) * 360
				if deg < acc {
					st = s
					break
				}
			}
			line += statusStyle(st).Render("•") + " "
		}
		rows = append(rows, strings.TrimRight(line, " "))
	}
	return rows
}

// toolingLines: readiness bar, first-class Project/Security/Audit signals,
// and the severity chips row.
func (m Model) toolingLines(width int) []string {
	ok, _, errC, _ := m.statusCounts()
	total := len(m.report.Results)
	readiness := 0.0
	if total > 0 {
		readiness = float64(ok) / float64(total)
	}
	risk := min(100, errC*15+warnRisk())
	lines := []string{
		m.ready.ViewAs(readiness),
		accentStyle.Render(fmt.Sprintf("readiness %.0f%%", readiness*100)) +
			dimStyle.Render(fmt.Sprintf(" │ risk %d/100", risk)),
		"",
	}
	prow := func(name, st, note string) string {
		return pad(name, 9) + statusStyle(st).Bold(true).Render(pad(strings.ToUpper(label(st)), 6)) + note
	}
	p := m.report.Results["project"]
	lines = append(lines, prow("Project", p.Status, fmt.Sprintf("%s / %d outdated", nonEmpty(p.ProjectType, "—"), len(p.Outdated))))
	s := m.report.Results["security"]
	lines = append(lines, prow("Security", s.Status, fmt.Sprintf("%d vulns", len(s.Vulnerabilities))))
	a := m.report.Results["audit"]
	lines = append(lines, prow("Audit", a.Status, fmt.Sprintf("%d checks flagged", len(a.AuditItems))))
	lines = append(lines, "")
	lines = append(lines, wrapChunks(m.severityChips(), width-2)...)
	return lines
}

func warnRisk() int {
	ok, warn, _, _ := (Model{}).statusCountsSlow()
	_ = ok
	return warn * 5
}

// statusCountsSlow avoids needing a report receiver for the risk formula.
func (m Model) statusCountsSlow() (int, int, int, int) { return m.statusCounts() }

func nonEmpty(s, fallback string) string {
	if s == "" {
		return fallback
	}
	return s
}

// severityChips: Outd/Crit/High/Medi/Othe/Audi counters from the report.
func (m Model) severityChips() []string {
	var crit, high, med, oth int
	if sec, ok := m.report.Results["security"]; ok {
		for _, v := range sec.Vulnerabilities {
			switch v.Severity {
			case "CRITICAL":
				crit++
			case "HIGH":
				high++
			case "MEDIUM":
				med++
			default:
				oth++
			}
		}
	}
	audit := 0
	if a, ok := m.report.Results["audit"]; ok {
		audit = len(a.AuditItems)
	}
	chip := func(name string, n int) string {
		st := "skipped"
		if n > 0 {
			st = "error"
		}
		return statusStyle(st).Render(fmt.Sprintf("%s %d", name, n))
	}
	return []string{
		statusStyle("warn").Render(fmt.Sprintf("Outd %d", len(m.report.Outdated))),
		chip("Crit", crit), chip("High", high), chip("Medi", med), chip("Othe", oth),
		statusStyle("warn").Render(fmt.Sprintf("Audi %d", audit)),
	}
}

// toolchain groups mirror the ui.rs dashboard sections.
var toolGroups = []struct {
	title string
	tools []string
}{
	{"System & Runtime", []string{"brew", "cargo", "docker", "pip", "gem"}},
	{"Web Development", []string{"npm", "pnpm", "yarn", "bun", "deno"}},
	{"Project Tooling", []string{"project", "security", "audit", "ci"}},
}

func rowLabel(tool string) string {
	if tool == "ci" {
		return "CI/CD"
	}
	if tool == "" {
		return tool
	}
	return strings.ToUpper(tool[:1]) + tool[1:]
}

func countOutdated(r ScanResult) int {
	return len(r.Outdated) + len(r.OutdatedGlobal) + len(r.OutdatedFormulae) + len(r.OutdatedCasks)
}

// groupPanels renders the three grouped tables with a shared row cursor.
func (m Model) groupPanels(width int) []string {
	panels := []string{}
	for _, g := range toolGroups {
		lines := []string{tableHeader(width)}
		for _, tool := range g.tools {
			r, ok := m.report.Results[tool]
			if !ok {
				continue
			}
			line := m.tableRow(tool, r, width)
			if m.flattenIndex(tool) == m.dashSel {
				line = selectedRow.Render(stripAnsi(line))
			}
			lines = append(lines, line)
		}
		panels = append(panels, panel(g.title, width, lines))
	}
	return panels
}

func tableHeader(width int) string {
	h := dimStyle.Render(pad("Toolchain", 12) + pad("Status", 9) + pad("Version", 16) + pad("Outd", 5) + "Issues")
	return h
}

func (m Model) tableRow(tool string, r ScanResult, width int) string {
	notes := strings.Join(r.Issues, "; ")
	outd := countOutdated(r)
	outdCell := ""
	if outd > 0 {
		outdCell = fmt.Sprintf("%d", outd)
	}
	line := pad(rowLabel(tool), 12) +
		statusStyle(r.Status).Render(pad(strings.ToUpper(label(r.Status)), 9)) +
		pad(truncate(displayVersion(r), 16), 16) +
		pad(truncate(outdCell, 5), 5) +
		truncate(notes, max(8, width-44))
	return line
}

// flattenIndex maps a tool to its position in the shared row cursor.
func (m Model) flattenIndex(tool string) int {
	idx := 0
	for _, g := range toolGroups {
		for _, t := range g.tools {
			if _, ok := m.report.Results[t]; ok {
				if t == tool {
					return idx
				}
				idx++
			}
		}
	}
	return -1
}

func truncate(s string, w int) string {
	if lipgloss.Width(s) <= w {
		return s
	}
	if w <= 1 {
		return "…"
	}
	return string([]rune(s)[:w-1]) + "…"
}

// stripAnsi removes SGR sequences — used before re-styling a selected row.
func stripAnsi(s string) string {
	var b strings.Builder
	inEsc := false
	for _, r := range s {
		if r == '\x1b' {
			inEsc = true
			continue
		}
		if inEsc {
			if r == 'm' {
				inEsc = false
			}
			continue
		}
		b.WriteRune(r)
	}
	return b.String()
}

func (m Model) footer() string {
	return lipgloss.NewStyle().Width(max(0, m.width)).Align(lipgloss.Right).
		Render(dimStyle.Render("⚡ Envexa "+cli.Version+"  ·  Crafted by Kuruto Denzeru")) + "\n"
}

func (m Model) viewOutdated() string {
	var b strings.Builder
	b.WriteString(m.dashboardHeader())

	if m.width < 40 || m.height < 12 {
		b.WriteString(dimStyle.Render("terminal too small — enlarge window") + "\n")
		return b.String()
	}

	if len(m.report.Outdated) > 0 {
		outLines := append([]string{tableHeader(m.width - 4)}, m.outdatedLines()...)
		b.WriteString(panel("Outdated packages", m.width-2, outLines) + "\n")
		fmt.Fprintf(&b, "%d outdated packages\n", len(m.report.Outdated))
	} else {
		b.WriteString(dimStyle.Render("no report yet — press s to scan") + "\n")
	}
	b.WriteString(m.hints())
	return b.String()
}

func (m Model) outdatedLines() []string {
	lines := []string{}
	for i, o := range m.report.Outdated {
		line := pad(o.Source, 12) + pad(truncate(o.Name, 20), 20) +
			pad(o.Current, 12) + pad(o.Latest, 12) + o.Size
		if i == m.outSel {
			line = selectedRow.Render(stripAnsi(line))
		}
		lines = append(lines, line)
	}
	return lines
}

// viewLogs shows the scan history from the envexa data dir — newest last,
// like the Rust logs view reads them.
func (m Model) viewLogs() string {
	var b strings.Builder
	b.WriteString(m.dashboardHeader())

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
	b.WriteString(m.hints())
	return b.String()
}

// viewSettings renders the persisted UserConfig plus data paths.
func (m Model) viewSettings() string {
	var b strings.Builder
	b.WriteString(m.dashboardHeader())

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
	b.WriteString(m.hints())
	return b.String()
}

// panel draws a rounded box with the title set into the top border line —
// the "─ Overview ─╮" style from ui.rs.
func panel(title string, width int, lines []string) string {
	inner := max(4, width-2)
	titleRunes := lipgloss.Width(title)
	dash := strings.Repeat("─", max(0, inner-titleRunes-2))
	head := panelTitle.Render("─ "+title+" ") + dimStyle.Render(dash+"╮")
	body := make([]string, 0, len(lines))
	for _, l := range lines {
		padW := inner - lipgloss.Width(l)
		if padW < 0 {
			l = truncate(l, inner)
			padW = inner - lipgloss.Width(l)
		}
		body = append(body, dimStyle.Render("│")+l+strings.Repeat(" ", max(0, padW))+dimStyle.Render("│"))
	}
	bottom := dimStyle.Render("╰" + strings.Repeat("─", inner) + "╯")
	return head + "\n" + strings.Join(body, "\n") + "\n" + bottom
}

func pad(s string, n int) string {
	w := lipgloss.Width(s)
	for w < n {
		s += " "
		w++
	}
	return s
}

func itou(v uint64) string {
	if v == 0 {
		return "—"
	}
	return fmt.Sprintf("%d", v)
}
