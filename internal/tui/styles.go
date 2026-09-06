package tui

import "github.com/charmbracelet/lipgloss"

// Color convention mirrors src/tui/theme.rs: ok=green, warning=yellow,
// error=red, skipped=darkgray.

var (
	okStyle      = lipgloss.NewStyle().Foreground(lipgloss.Color("2"))
	warningStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("3"))
	errorStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("1"))
	skippedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))

	titleStyle  = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("6"))
	tabActive   = lipgloss.NewStyle().Bold(true).Background(lipgloss.Color("6")).Foreground(lipgloss.Color("0"))
	tabInactive = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
	dimStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("8"))
	borderStyle = lipgloss.NewStyle().Border(lipgloss.RoundedBorder()).BorderForeground(lipgloss.Color("8")).Padding(0, 1)
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
