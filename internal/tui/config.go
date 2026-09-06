package tui

import (
	"encoding/json"
	"os"
	"path/filepath"
)

// Reads the on-disk state written by the Rust runtime (~/.local/share/envexa)
// so Logs and Settings views show real user data during the transition.

func dataDir() string {
	if d := os.Getenv("ENVEXA_DATA_DIR"); d != "" {
		return d
	}
	if xdg := os.Getenv("XDG_DATA_HOME"); xdg != "" {
		return filepath.Join(xdg, "envexa")
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return filepath.Join(".", ".envexa")
	}
	return filepath.Join(home, ".local", "share", "envexa")
}

// readLogs mirrors core::config::read_logs: logs.json is [[timestamp, msg]].
func readLogs() [][2]string {
	path := filepath.Join(dataDir(), "logs.json")
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil
	}
	var logs [][2]string
	if json.Unmarshal(raw, &logs) != nil {
		return nil
	}
	return logs
}

// userConfig mirrors core::config::UserConfig fields shown in Settings.
type userConfig struct {
	Theme              string   `json:"theme"`
	ScanTimeoutSecs    uint64   `json:"scan_timeout_secs"`
	DaemonIntervalSecs uint64   `json:"daemon_interval_secs"`
	ExportFormat       string   `json:"export_format"`
	LogRetentionDays   uint64   `json:"log_retention_days"`
	ProjectPath        string   `json:"project_path"`
	RecentPaths        []string `json:"recent_project_paths"`
}

func readConfig() userConfig {
	path := filepath.Join(dataDir(), "config.json")
	raw, err := os.ReadFile(path)
	cfg := userConfig{Theme: "default", ScanTimeoutSecs: 30, DaemonIntervalSecs: 300,
		ExportFormat: "markdown", LogRetentionDays: 30}
	if err != nil {
		return cfg
	}
	_ = json.Unmarshal(raw, &cfg)
	return cfg
}
