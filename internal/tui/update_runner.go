package tui

import (
	"fmt"
	"os/exec"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

// updateArgs maps a scanner source to its per-package upgrade command — the
// minimal counterpart of the Rust update runner (src/tui/app.rs update flow).
func updateArgs(source, name string) ([]string, error) {
	switch source {
	case "brew":
		return []string{"upgrade", name}, nil
	case "npm":
		return []string{"install", "-g", name + "@latest"}, nil
	case "gem":
		return []string{"update", name}, nil
	case "pip", "pip3":
		return []string{"pip3", "install", "--upgrade", name}, nil
	case "cargo":
		return []string{"install", name}, nil
	default:
		return nil, fmt.Errorf("no update runner for %q", source)
	}
}

// updateCmd runs the package upgrade; the TUI shows an indeterminate Updating
// view, so only the failure tail is captured for the message.
func updateCmd(source, name string) tea.Cmd {
	return func() tea.Msg {
		args, err := updateArgs(source, name)
		if err != nil {
			return updateDoneMsg{errMsg: err.Error()}
		}
		cmd := exec.Command(args[0], args[1:]...)
		if out, err := cmd.CombinedOutput(); err != nil {
			lines := strings.Split(strings.TrimSpace(string(out)), "\n")
			if len(lines) > 3 {
				lines = lines[len(lines)-3:]
			}
			msg := strings.Join(lines, " | ")
			if msg == "" {
				msg = err.Error()
			}
			return updateDoneMsg{errMsg: fmt.Sprintf("%s update failed: %s", name, msg)}
		}
		return updateDoneMsg{}
	}
}
