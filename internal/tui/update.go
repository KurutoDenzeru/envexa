package tui

import (
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
			return m, tea.Batch(m.spinner.Tick, simulateScan())
		case "o":
			m.view = ViewOutdated
			return m, nil
		case "h", "esc":
			m.view = ViewDashboard
			return m, nil
		case "left":
			m.view = ViewDashboard
			return m, nil
		case "right":
			m.view = ViewOutdated
			return m, nil
		case "up", "down":
			var cmd tea.Cmd
			switch m.view {
			case ViewDashboard:
				m.dashTable, cmd = m.dashTable.Update(msg)
				m.dashSel = m.dashTable.Cursor()
			case ViewOutdated:
				m.outTable, cmd = m.outTable.Update(msg)
				m.outSel = m.outTable.Cursor()
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
	for _, name := range tools {
		r := m.report.Results[name]
		notes := strings.Join(r.Issues, "; ")
		dashRows = append(dashRows, table.Row{name, r.Status, r.Version, notes})
	}
	m.dashTable.SetRows(dashRows)
	m.dashTable.SetHeight(min(8, len(dashRows)+1))

	outRows := []table.Row{}
	for _, o := range m.report.Outdated {
		outRows = append(outRows, table.Row{o.Source, o.Name, o.Current, o.Latest, o.Size})
	}
	m.outTable.SetRows(outRows)
	m.outTable.SetHeight(min(8, len(outRows)+1))
}
