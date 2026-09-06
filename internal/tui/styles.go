package tui

import "github.com/charmbracelet/lipgloss"

// Color convention mirrors src/tui/theme.rs: ok=green, warning=yellow,
// error=red, skipped=darkgray.

var (
	okStyle      = lipgloss.NewStyle().Foreground(lipgloss.Color("2"))
	warningStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("3"))
	errorStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("1"))
	skippedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))

	titleStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("6"))
	dimStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
)

func statusStyle(status string) lipgloss.Style {
	switch status {
	case "ok":
		return okStyle
	case "warn", "warning":
		return warningStyle
	case "error":
		return errorStyle
	default:
		return skippedStyle
	}
}
