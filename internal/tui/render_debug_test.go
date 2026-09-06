package tui

import (
	"os"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
)

func TestRenderDebug(t *testing.T) {
	out := os.Getenv("ENVEXA_RENDER_OUT")
	if out == "" {
		t.Skip("ENVEXA_RENDER_OUT not set")
	}
	m := benchModel(120, 36)
	nm, _ := m.Update(tea.WindowSizeMsg{Width: 120, Height: 36})
	mm := nm.(Model)
	_ = os.WriteFile(out, []byte(mm.View()), 0o644)
}
