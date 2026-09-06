package tui

import (
	"fmt"
	"sort"
	"strings"

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
		return m, nil

	case scanDoneMsg:
		m.scanning = false
		m.report = msg.report
		m.syncTables()
		return m, nil

	case spinner.TickMsg:
		if m.scanning {
			sp, cmd := m.spinner.Update(msg)
			m.spinner = sp
			return m, cmd
		}
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "q", "ctrl+c":
			return m, tea.Quit
		case "s":
			m.scanning = true
			return m, tea.Batch(m.spinner.Tick, scanCmd())
		case "o":
			m.view = ViewOutdated
			return m, nil
		case "l":
			m.view = ViewLogs
			return m, nil
		case "c":
			m.view = ViewSettings
			return m, nil
		case "h", "esc":
			m.view = ViewDashboard
			return m, nil
		case "tab":
			if m.view == ViewDashboard {
				m.dashTab = (m.dashTab + 1) % 3
			}
			return m, nil
		case "left":
			if m.view == ViewDashboard {
				m.dashTab = (m.dashTab + 2) % 3
				return m, nil
			}
			m.view = ViewDashboard
			return m, nil
		case "right":
			if m.view == ViewDashboard {
				m.dashTab = (m.dashTab + 1) % 3
				return m, nil
			}
			m.view = ViewOutdated
			return m, nil
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
				m.dashTable, cmd = m.dashTable.Update(msg)
				m.dashSel = m.dashTable.Cursor()
			}
			return m, cmd
		}
	}
	return m, nil
}

// syncTables reloads bubbletea table rows from the report — equivalent to the
// Rust renderers rebuilding Rows each frame.
func (m *Model) syncTables() {
	tools := make([]string, 0, len(m.report.Results))
	for name := range m.report.Results {
		tools = append(tools, name)
	}
	sort.Strings(tools)

	dashRows := []table.Row{}
	toolRows := []table.Row{}
	vulns := []VulnerabilityInfo{}
	for _, name := range tools {
		r := m.report.Results[name]
		notes := strings.Join(r.Issues, "; ")
		dashRows = append(dashRows, table.Row{name, r.Status, r.Version, notes})
		toolRows = append(toolRows, table.Row{name, r.Status, displayVersion(r), installedCount(r)})
		if name == "security" {
			vulns = r.Vulnerabilities
		}
	}
	m.dashTable.SetRows(dashRows)
	m.dashTable.SetHeight(min(8, len(dashRows)+1))
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
