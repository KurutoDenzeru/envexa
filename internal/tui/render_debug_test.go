package tui

import (
	"fmt"
	"os"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
)

func TestRenderDebug(t *testing.T) {
	out := os.Getenv("ENVEXA_RENDER_OUT")
	if out == "" {
		t.Skip("ENVEXA_RENDER_OUT not set")
	}
	w, h := 120, 36
	if v := os.Getenv("ENVEXA_RENDER_SIZE"); v != "" {
		if _, err := fmt.Sscanf(v, "%dx%d", &w, &h); err != nil {
			t.Fatal(err)
		}
	}
	m := benchModel(w, h)
	nm, _ := m.Update(tea.WindowSizeMsg{Width: w, Height: h})
	mm := nm.(Model)
	_ = os.WriteFile(out, []byte(mm.View()), 0o644)
}
