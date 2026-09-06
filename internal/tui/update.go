package tui

import (
	"fmt"
	"sort"
	"strings"

	"github.com/KurutoDenzeru/envexa/internal/config"

	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
	"github.com/charmbracelet/lipgloss"

	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	tea "github.com/charmbracelet/bubbletea"
)

// Key map mirrors src/tui/app.rs: s=scan, o=outdated, h/Esc=home, q=quit,
// arrows navigate rows / switch tabs.

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width, m.height = msg.Width, msg.Height
		// Readiness bar lives in the left panel: 34 cols wide, or full width
		// when the dashboard stacks vertically.
		if m.width >= 100 {
			m.ready.Width = 34
			m.health.Width = 34
		} else {
			m.ready.Width = max(10, m.width-8)
			m.health.Width = max(10, m.width-8)
		}
		if m.view == ViewLogs {
			m.syncLogsVP()
		}
		return m, nil

	case scanDoneMsg:
		m.scanning = false
		m.report = msg.report
		m.syncTables()
		return m, nil

	case updateDoneMsg:
		m.updating = false
		if msg.errMsg != "" {
			m.updateMsg = msg.errMsg
			m.view = ViewPackageDetail
			return m, nil
		}
		m.updateMsg = ""
		m.view = ViewOutdated
		m.scanning = true
		return m, tea.Batch(m.spinner.Tick, scanCmd())

	case spinner.TickMsg:
		if m.scanning || m.updating {
			sp, cmd := m.spinner.Update(msg)
			m.spinner = sp
			return m, cmd
		}
		return m, nil

	case tea.KeyMsg:
		// Updating blocks input like the Rust Scanning/Updating guard.
		if m.view == ViewUpdating {
			switch msg.String() {
			case "q", "ctrl+c":
				return m, tea.Quit
			}
			return m, nil
		}
		if m.view == ViewSettings {
			recents := len(m.editCfg.RecentProjectPaths)
			switch msg.String() {
			case "up":
				m.setSel = max(0, m.setSel-1)
			case "down":
				m.setSel = min(len(settingsFields)+recents-1, m.setSel+1)
			case "left":
				m.adjustSettings(-1)
			case "right":
				m.adjustSettings(1)
			case "enter":
				if !m.adjustSettings(1) {
					m.selectRecent()
				}
			case "esc", "h":
				return *m.gotoView(ViewDashboard), nil
			case "q", "ctrl+c":
				return m, tea.Quit
			}
			return m, nil
		}
		if m.searchActive && m.view == ViewOutdated {
			switch msg.String() {
			case "esc":
				if m.searchInput.Value() == "" {
					m.searchActive = false
					m.searchInput.Blur()
				} else {
					m.searchInput.SetValue("")
				}
				m.refilter()
				return m, nil
			case "enter":
				if idx, ok := m.selectedOutdated(); ok {
					o := m.report.Outdated[idx]
					m.detail = detail{source: o.Source, name: o.Name, current: o.Current, latest: o.Latest}
					m.view = ViewPackageDetail
				}
				return m, nil
			}
			var cmd tea.Cmd
			m.searchInput, cmd = m.searchInput.Update(msg)
			if m.searchInput.Value() != m.lastQuery() {
				m.refilter()
			}
			return m, cmd
		}
		switch msg.String() {
		case "q", "ctrl+c":
			return m, tea.Quit
		case "s":
			m.scanning = true
			return m, tea.Batch(m.spinner.Tick, scanCmd())
		case "o":
			return *m.gotoView(ViewOutdated), nil
		case "/":
			if m.view == ViewOutdated {
				m.searchActive = true
				m.searchInput.Focus()
				m.refilter()
				return m, textinput.Blink
			}
			return m, nil
		case "l":
			return *m.gotoView(ViewLogs), nil
		case "c":
			return *m.gotoView(ViewSettings), nil
		case "esc":
			if m.view == ViewPackageDetail {
				m.view = ViewOutdated
				return m, nil
			}
			m.view = ViewDashboard
			return m, nil
		case "h":
			m.view = ViewDashboard
			return m, nil
		case "enter":
			if m.view != ViewOutdated {
				return m, nil
			}
			if idx, ok := m.selectedOutdated(); ok {
				o := m.report.Outdated[idx]
				m.detail = detail{source: o.Source, name: o.Name, current: o.Current, latest: o.Latest}
				m.view = ViewPackageDetail
			}
			return m, nil
		case "y":
			if m.view == ViewPackageDetail && !m.updating {
				if _, err := updateArgs(m.detail.source, m.detail.name); err != nil {
					m.updateMsg = err.Error()
					return m, nil
				}
				m.updating = true
				m.view = ViewUpdating
				m.updateMsg = fmt.Sprintf("Updating %s (%s → %s) via %s…",
					m.detail.name, m.detail.current, m.detail.latest, m.detail.source)
				return m, tea.Batch(m.spinner.Tick, updateCmd(m.detail.source, m.detail.name))
			}
			return m, nil
		case "tab":
			// Dashboard sub-tabs (overview / vulnerabilities / toolchains).
			if m.view == ViewDashboard {
				m.dashTab = (m.dashTab + 1) % 3
			}
			return m, nil
		case "left", "right":
			order := []View{ViewDashboard, ViewOutdated, ViewLogs, ViewSettings}
			idx := 0
			for i, v := range order {
				if v == m.view {
					idx = i
					break
				}
			}
			if msg.String() == "right" {
				idx = (idx + 1) % len(order)
			} else {
				idx = (idx + len(order) - 1) % len(order)
			}
			return *m.gotoView(order[idx]), nil
		case "up", "down":
			var cmd tea.Cmd
			switch {
			case m.view == ViewOutdated:
				m.outTable, cmd = m.outTable.Update(msg)
				m.outSel = m.outTable.Cursor()
			case m.view == ViewDashboard && m.dashTab == dashVulns:
				m.vulnTable, cmd = m.vulnTable.Update(msg)
				m.dashSel = m.vulnTable.Cursor()
			case m.view == ViewDashboard && m.dashTab == dashToolchains:
				m.toolTable, cmd = m.toolTable.Update(msg)
				m.dashSel = m.toolTable.Cursor()
			case m.view == ViewDashboard:
				// Shared cursor across the grouped dashboard rows.
				if msg.String() == "up" {
					m.dashSel = max(0, m.dashSel-1)
				} else {
					m.dashSel = min(m.dashRows-1, m.dashSel+1)
				}
			case m.view == ViewLogs:
				vp, c := m.logsVP.Update(msg)
				m.logsVP = vp
				return m, c
			}
			return m, cmd
		}
	}
	return m, nil
}

// syncTables reloads bubbletea table rows from the report — equivalent to the
// Rust renderers rebuilding Rows each frame.
// gotoView switches top-level views and refreshes view-local state (the logs
// viewport reloads so it picks up new entries and the current size).
func (m *Model) gotoView(v View) *Model {
	m.view = v
	if v == ViewLogs {
		m.syncLogsVP()
	}
	if v == ViewSettings {
		m.syncSettings()
	}
	return m
}

// syncSettings reloads the config for the settings editor and resets the
// cursor — the editor edits a live copy and persists on every change.
func (m *Model) syncSettings() {
	m.editCfg = config.LoadConfig()
	m.setSel = 0
}

// adjustSettings applies a direction to the selected field and saves; returns
// false when the cursor is on a non-adjustable field (recents).
func (m *Model) adjustSettings(dir int) bool {
	if m.setSel >= len(settingsFields) {
		return false
	}
	if f := settingsFields[m.setSel]; f.adjust != nil {
		f.adjust(&m.editCfg, dir)
		_ = config.SaveConfig(m.editCfg)
	}
	return true
}

// selectRecent points project_path at the highlighted recent entry.
func (m *Model) selectRecent() bool {
	idx := m.setSel - len(settingsFields)
	recents := m.editCfg.RecentProjectPaths
	if idx < 0 || idx >= len(recents) {
		return false
	}
	path := recents[idx]
	m.editCfg.ProjectPath = &path
	return config.SaveConfig(m.editCfg) == nil
}

// syncLogsVP fills the bubbles/viewport with the full log history — the old
// TUI scrolled these; 20 lines visible is the viewport's job now.
func (m *Model) syncLogsVP() {
	lines := formatLogLines(readLogs())
	if len(lines) == 0 {
		lines = append(lines, dimStyle.Render("no logs yet — run a scan"))
	}
	w := max(20, m.width-2)
	h := max(4, m.height-9)
	m.logsVP = viewport.New(w, h)
	m.logsVP.SetContent(strings.Join(lines, "\n"))
	m.logsVP.GotoBottom()
}

// formatLogLines renders entries web-dashboard style (colored level, readable
// time, source tag) and collapses consecutive duplicates with a ×N marker.
func formatLogLines(logs []config.LogPair) []string {
	var out []string
	i := 0
	for i < len(logs) {
		e := config.ParseLogLine(logs[i])
		j := i + 1
		for j < len(logs) {
			n := config.ParseLogLine(logs[j])
			if n.Level == e.Level && n.Message == e.Message && n.Source == e.Source {
				j++
				continue
			}
			break
		}
		out = append(out, logLine(logs[i], e, j-i))
		i = j
	}
	return out
}

func logLine(pair config.LogPair, e config.LogEntry, count int) string {
	var levelStyle lipgloss.Style
	switch e.Level {
	case "WARN":
		levelStyle = warningStyle
	case "ERROR":
		levelStyle = errorStyle
	case "DEBUG":
		levelStyle = dimStyle
	default:
		levelStyle = okStyle
	}
	line := dimStyle.Render(pair.Time.Format("Jan 02 15:04:05")) + "  " +
		levelStyle.Render(pad(e.Level, 5)) + "  " + e.Message
	if e.Source != "system" {
		line += dimStyle.Render("  [" + e.Source + "]")
	}
	if count > 1 {
		line += dimStyle.Render(fmt.Sprintf("  ×%d", count))
	}
	return line
}

// lastQuery returns the query used by the current filter set.
func (m *Model) lastQuery() string {
	return m.query
}

// refilter recomputes the filtered outdated indices for the active query.
func (m *Model) refilter() {
	m.query = m.searchInput.Value()
	m.filtered = filterIndices(m.report.Outdated, m.query)
	m.fpos = min(m.fpos, max(0, len(m.filtered)-1))
}

// selectedOutdated resolves the highlighted row (filter-aware) to an index
// into report.Outdated.
func (m *Model) selectedOutdated() (int, bool) {
	if m.searchActive {
		if len(m.filtered) == 0 || m.fpos >= len(m.filtered) {
			return 0, false
		}
		return m.filtered[m.fpos], true
	}
	if m.outSel < len(m.report.Outdated) {
		return m.outSel, true
	}
	return 0, false
}

// filterIndices returns indices of items matching the query
// (case-insensitive substring over package and source).
func filterIndices(items []OutdatedItem, query string) []int {
	q := strings.ToLower(query)
	out := make([]int, 0, len(items))
	for i, o := range items {
		if q == "" || strings.Contains(strings.ToLower(o.Name), q) ||
			strings.Contains(strings.ToLower(o.Source), q) {
			out = append(out, i)
		}
	}
	return out
}

func (m *Model) syncTables() {
	tools := make([]string, 0, len(m.report.Results))
	for name := range m.report.Results {
		tools = append(tools, name)
	}
	sort.Strings(tools)

	toolRows := []table.Row{}
	vulns := []VulnerabilityInfo{}
	dashRows := 0
	for _, name := range tools {
		r := m.report.Results[name]
		toolRows = append(toolRows, table.Row{name, r.Status, displayVersion(r), installedCount(r)})
		if name == "security" {
			vulns = r.Vulnerabilities
		}
	}
	for _, g := range toolGroups {
		for _, t := range g.tools {
			if _, ok := m.report.Results[t]; ok {
				dashRows++
			}
		}
	}
	m.dashRows = dashRows
	m.refilter()
	m.toolTable.SetRows(toolRows)
	m.toolTable.SetHeight(min(8, len(toolRows)+1))
	m.vulnTable.SetRows(vulnRows(vulns))
	m.vulnTable.SetHeight(min(8, len(vulns)+1))

	outRows := []table.Row{}
	for _, o := range m.report.Outdated {
		outRows = append(outRows, table.Row{o.Source, o.Name, o.Current, o.Latest, o.Size})
	}
	m.outTable.SetRows(outRows)
	m.outTable.SetHeight(min(8, len(outRows)+1))
}

func vulnRows(vulns []VulnerabilityInfo) []table.Row {
	rows := []table.Row{}
	for _, v := range vulns {
		cve := ""
		if v.CVE != nil {
			cve = *v.CVE
		}
		rows = append(rows, table.Row{v.Package, v.Severity, v.PatchedVersion, v.Title + cve})
	}
	return rows
}

// displayVersion picks the first tool-specific version field, like the Rust
// toolchains view.
func displayVersion(r ScanResult) string {
	for _, v := range []string{r.Version, r.NodeVersion, r.PythonVersion, r.RubyVersion,
		r.RustcVersion, r.CargoVersion, r.PnpmVersion, r.BunVersion, r.DenoVersion} {
		if v != "" {
			return v
		}
	}
	return ""
}

func installedCount(r ScanResult) string {
	if r.InstalledCount == nil {
		return ""
	}
	return fmt.Sprintf("%d", *r.InstalledCount)
}
