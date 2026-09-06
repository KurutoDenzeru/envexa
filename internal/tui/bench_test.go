package tui

import (
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
)

// Frame-cost benchmark for the Phase 0 spike gate memo: a frame is one View()
// call; budget is 16ms (60fps). Rust ratatui baseline: same metric measured
// against the current TUI before sunset (see docs/spike-bubbletea-gate.md).

func benchModel(width, height int) Model {
	m := NewModel()
	m.width, m.height = width, height
	m.report = mockReport()
	m.syncTables()
	return m
}

func BenchmarkViewDashboardWide(b *testing.B) {
	m := benchModel(140, 40)
	for b.Loop() {
		_ = m.View()
	}
}

func BenchmarkViewDashboardNarrow(b *testing.B) {
	m := benchModel(70, 30)
	for b.Loop() {
		_ = m.View()
	}
}

func BenchmarkViewOutdatedWide(b *testing.B) {
	m := benchModel(140, 40)
	m.view = ViewOutdated
	for b.Loop() {
		_ = m.View()
	}
}

func TestViewFallbacks(t *testing.T) {
	tiny := benchModel(30, 10)
	if got := tiny.View(); !contains(got, "terminal too small") {
		t.Fatalf("tiny terminal not guarded: %q", got)
	}
	mid := benchModel(70, 30)
	if got := mid.View(); !contains(got, "PASS (6)") || !contains(got, "Overview") {
		t.Fatalf("mid dashboard missing overview panel: %q", got)
	}
	if got := mid.View(); contains(got, "╗") { // logoArt box corner, absent from other chrome
		t.Fatal("logo should be hidden under width 100")
	}
	wide := benchModel(100, 40)
	if got := wide.View(); !contains(got, "╗") || !contains(got, "System & Runtime") ||
		!contains(got, "Crafted by Kuruto Denzeru") {
		t.Fatalf("wide dashboard missing logo, groups, or footer: %q", got)
	}
	scanning := benchModel(100, 40)
	scanning.scanning = true
	if got := scanning.View(); !contains(got, "Scanning") {
		t.Fatalf("scanning view missing spinner state: %q", got)
	}
}

func TestNewViews(t *testing.T) {
	t.Setenv("ENVEXA_DATA_DIR", t.TempDir()) // isolate logs/config reads

	m := benchModel(100, 40)
	m.report = mockReport()
	m.syncTables()

	m.dashTab = dashVulns
	if got := m.View(); !contains(got, "Package") || !contains(got, "Severity") {
		t.Fatalf("vulns tab missing table headers: %q", got)
	}
	m.dashTab = dashToolchains
	if got := m.View(); !contains(got, "Toolchain") {
		t.Fatalf("toolchains tab missing table header: %q", got)
	}
	m.dashTab = dashOverview

	m.view = ViewLogs
	if got := m.View(); !contains(got, "no logs yet") {
		t.Fatalf("logs view missing empty state: %q", got)
	}
	m.view = ViewSettings
	if got := m.View(); !contains(got, "scan_timeout_secs") || !contains(got, "theme") {
		t.Fatalf("settings view missing config fields: %q", got)
	}
}

func TestUpdateViewsAndRunner(t *testing.T) {
	t.Setenv("ENVEXA_DATA_DIR", t.TempDir())

	m := benchModel(100, 40)
	m.report = mockReport()
	m.syncTables()

	// Enter on outdated opens PackageDetail with the selected row.
	m.view = ViewOutdated
	m.outSel = 0
	next, _ := m.Update(tea.KeyMsg{Type: tea.KeyEnter})
	dm := next.(Model)
	if dm.view != ViewPackageDetail || dm.detail.name != "typescript" || dm.detail.source != "npm" {
		t.Fatalf("enter did not open package detail: %+v %+v", dm.view, dm.detail)
	}
	if got := dm.View(); !contains(got, "typescript") || !contains(got, "y update") {
		t.Fatalf("detail view missing fields: %q", got)
	}

	// y in detail starts the update: Updating view blocks, spinner runs.
	next, cmd := dm.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{'y'}})
	um := next.(Model)
	if um.view != ViewUpdating || !um.updating || !contains(um.updateMsg, "typescript") {
		t.Fatalf("y did not start update: %+v %q", um.view, um.updateMsg)
	}
	if cmd == nil {
		t.Fatal("update command missing")
	}

	// updateDoneMsg returns to outdated and rescans.
	next, cmd2 := um.Update(updateDoneMsg{})
	fm := next.(Model)
	if fm.view != ViewOutdated || fm.updating || !fm.scanning {
		t.Fatalf("updateDoneMsg did not return to outdated+rescan: %+v %+v %+v", fm.view, fm.updating, fm.scanning)
	}
	if cmd2 == nil {
		t.Fatal("rescan command missing")
	}

	// Unsupported sources are rejected before exec.
	if _, err := updateArgs("docker", "nginx"); err == nil {
		t.Fatal("docker should have no update runner")
	}
	if args, _ := updateArgs("npm", "typescript"); args[0] != "install" || args[1] != "-g" || args[2] != "typescript@latest" {
		t.Fatalf("npm update args wrong: %v", args)
	}
}

func contains(s, sub string) bool { return strings.Contains(s, sub) }

func TestViewCycling(t *testing.T) {
	t.Setenv("ENVEXA_DATA_DIR", t.TempDir())
	m := benchModel(100, 40)

	key := func(s string) tea.KeyMsg {
		switch s {
		case "left":
			return tea.KeyMsg{Type: tea.KeyLeft}
		case "right":
			return tea.KeyMsg{Type: tea.KeyRight}
		case "tab":
			return tea.KeyMsg{Type: tea.KeyTab}
		}
		return tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune(s)}
	}

	// right cycles dashboard -> outdated -> logs -> settings -> dashboard
	next, _ := m.Update(key("right"))
	if n := next.(Model); n.view != ViewOutdated {
		t.Fatalf("right from dashboard: %v", n.view)
	}
	next, _ = next.Update(key("right"))
	if n := next.(Model); n.view != ViewLogs {
		t.Fatalf("right from outdated: %v", n.view)
	}
	next, _ = next.Update(key("right"))
	next, _ = next.Update(key("left"))
	if n := next.(Model); n.view != ViewLogs {
		t.Fatalf("left from settings: %v", n.view)
	}

	// tab stays on dashboard and cycles sub-tabs
	next, _ = m.Update(key("tab"))
	n := next.(Model)
	if n.view != ViewDashboard || n.dashTab != dashVulns {
		t.Fatalf("tab should cycle dashboard sub-tabs: %v %v", n.view, n.dashTab)
	}

	// left/right on dashboard also cycles sub-tab state untouched
	next, _ = m.Update(key("right"))
	if n := next.(Model); n.view != ViewOutdated {
		t.Fatalf("right should leave dashboard: %v", n.view)
	}
}

func TestOutdatedSearch(t *testing.T) {
	t.Setenv("ENVEXA_DATA_DIR", t.TempDir())
	m := benchModel(100, 40)
	m.report = mockReport()
	m.syncTables()
	m.view = ViewOutdated

	// "/" activates the filter; typing narrows rows; selection maps back.
	next, _ := m.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("/")})
	sm := next.(Model)
	if !sm.searchActive {
		t.Fatal("/ did not activate search")
	}
	next, _ = sm.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("v")})
	next, _ = next.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("i")})
	sm = next.(Model)
	if got := len(sm.filtered); got != 1 || sm.report.Outdated[sm.filtered[0]].Name != "vite" {
		t.Fatalf("filter 'vi' should match vite only: %v", sm.filtered)
	}
	if sm.fpos != 0 {
		t.Fatalf("cursor should reset to first match: %d", sm.fpos)
	}
	if got := sm.View(); !contains(got, "/ vi") || !contains(got, "vite") || contains(got, "ripgrep") {
		t.Fatalf("filtered view wrong: %q", got)
	}

	// esc with a query clears it; esc again closes the filter.
	next, _ = sm.Update(tea.KeyMsg{Type: tea.KeyEsc})
	next, _ = next.Update(tea.KeyMsg{Type: tea.KeyEsc})
	sm = next.(Model)
	if sm.searchActive || len(sm.filtered) != len(sm.report.Outdated) {
		t.Fatalf("esc should clear then close filter: %v %v", sm.searchActive, len(sm.filtered))
	}

	// enter on a filtered row opens the right package detail.
	sm.view = ViewOutdated
	next, _ = sm.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("/")})
	next, _ = next.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("da")})
	sm = next.(Model)
	next, _ = sm.Update(tea.KeyMsg{Type: tea.KeyEnter})
	dm := next.(Model)
	if dm.view != ViewPackageDetail || dm.detail.name != "dax" {
		t.Fatalf("enter after filter should open dax detail: %+v %+v", dm.view, dm.detail)
	}
}

func TestFilterIndices(t *testing.T) {
	items := []OutdatedItem{
		{Source: "npm", Name: "typescript"},
		{Source: "brew", Name: "ripgrep"},
		{Source: "npm", Name: "vite"},
	}
	if got := filterIndices(items, "npm"); len(got) != 2 {
		t.Fatalf("source filter: %v", got)
	}
	if got := filterIndices(items, "grep"); len(got) != 1 || got[0] != 1 {
		t.Fatalf("case-insensitive name filter: %v", got)
	}
	if got := filterIndices(items, ""); len(got) != 3 {
		t.Fatalf("empty query keeps all: %v", got)
	}
}
