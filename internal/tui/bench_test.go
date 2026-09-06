package tui

import (
	"strings"
	"testing"
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
	if got := mid.View(); contains(got, "ok 6") { // distribution legend, not the progress gauges
		t.Fatalf("distribution chart should be hidden under width 80: %q", got)
	}
	wide := benchModel(100, 40)
	if got := wide.View(); !contains(got, "ok 6") {
		t.Fatal("distribution chart missing on wide terminal")
	}
	scanning := benchModel(100, 40)
	scanning.scanning = true
	if got := scanning.View(); !contains(got, "Scanning") {
		t.Fatalf("scanning view missing spinner state: %q", got)
	}
}

func contains(s, sub string) bool { return strings.Contains(s, sub) }
