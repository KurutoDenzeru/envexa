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
