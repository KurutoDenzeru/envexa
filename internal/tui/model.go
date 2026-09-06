package tui

import (
	"os"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	tea "github.com/charmbracelet/bubbletea"

	"github.com/KurutoDenzeru/envexa/internal/scanner"
)

// View mirrors the Rust App View enum (src/tui/app.rs) minus PackageDetail/
// Updating, which arrive with the update-runner phase.
type View int

const (
	ViewDashboard View = iota
	ViewOutdated
	ViewLogs
	ViewSettings
)

// Dashboard sub-tabs, cycled with Tab/Left/Right: overview, vulnerabilities,
// toolchains — keeps Project/Security/Audit signals first-class like ui.rs.
const (
	dashOverview = iota
	dashVulns
	dashToolchains
)

type scanDoneMsg struct{ report Report }
type tickMsg time.Time

type Model struct {
	view      View
	dashTab   int
	width     int
	height    int
	report    Report
	scanning  bool
	spinner   spinner.Model
	ready     progress.Model
	health    progress.Model
	dashSel   int // dashboard_selection
	outSel    int // outdated_selection
	dashTable table.Model
	vulnTable table.Model
	toolTable table.Model
	outTable  table.Model
}

func NewModel() Model {
	sp := spinner.New(spinner.WithSpinner(spinner.Meter))
	return Model{
		view:    ViewDashboard,
		spinner: sp,
		ready:   progress.New(progress.WithDefaultGradient()),
		health:  progress.New(progress.WithDefaultGradient()),
		dashTable: table.New(
			table.WithColumns([]table.Column{
				{Title: "Tool", Width: 12},
				{Title: "Status", Width: 10},
				{Title: "Version", Width: 12},
				{Title: "Notes", Width: 28},
			}),
			table.WithFocused(true),
			table.WithHeight(8),
		),
		vulnTable: table.New(
			table.WithColumns([]table.Column{
				{Title: "Package", Width: 20},
				{Title: "Severity", Width: 10},
				{Title: "Fixed", Width: 12},
				{Title: "Title", Width: 40},
			}),
			table.WithFocused(true),
			table.WithHeight(8),
		),
		toolTable: table.New(
			table.WithColumns([]table.Column{
				{Title: "Toolchain", Width: 14},
				{Title: "Status", Width: 10},
				{Title: "Version", Width: 20},
				{Title: "Installed", Width: 10},
			}),
			table.WithFocused(true),
			table.WithHeight(8),
		),
		outTable: table.New(
			table.WithColumns([]table.Column{
				{Title: "Source", Width: 10},
				{Title: "Package", Width: 18},
				{Title: "Current", Width: 12},
				{Title: "Latest", Width: 12},
				{Title: "Size", Width: 10},
			}),
			table.WithFocused(true),
			table.WithHeight(8),
		),
	}
}

func (m Model) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, scanCmd())
}

// scanCmd runs the real bash scanners via the Go bridge — the replacement for
// the Rust scan_all_with call path. Runs async; the spinner keeps ticking.
func scanCmd() tea.Cmd {
	dir := os.Getenv("ENVEXA_TOOLCHAINS_DIR")
	if dir == "" {
		dir = "toolchains"
	}
	return func() tea.Msg {
		return scanDoneMsg{report: scanner.Scan(dir, 60*time.Second)}
	}
}
