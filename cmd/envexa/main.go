package main

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"

	"github.com/KurutoDenzeru/envexa/internal/tui"
)

func main() {
	// TTY branch mirrors src/main.rs: no args + TTY → TUI, otherwise CLI.
	if !isTTY() {
		fmt.Println("envexa spike: TUI requires a terminal (CLI subcommands land in Phase 2)")
		os.Exit(2)
	}
	p := tea.NewProgram(tui.NewModel(), tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "envexa: %v\n", err)
		os.Exit(1)
	}
}

func isTTY() bool {
	info, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}
