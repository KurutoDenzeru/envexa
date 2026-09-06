package tui

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/config"
)

// Reads/writes the on-disk state (~/.local/share/envexa or XDG equivalent) so
// Logs and Settings views show and edit real user data.

func dataDir() string {
	if d := os.Getenv("ENVEXA_DATA_DIR"); d != "" {
		return d
	}
	return config.Dir()
}

// readLogs mirrors core::config::read_logs: logs.json is [[timestamp, msg]].
func readLogs() []config.LogPair {
	path := filepath.Join(dataDir(), "logs.json")
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil
	}
	var tuples [][2]string
	if json.Unmarshal(raw, &tuples) != nil {
		return nil
	}
	logs := make([]config.LogPair, 0, len(tuples))
	for _, t := range tuples {
		ts, err := time.Parse(time.RFC3339, t[0])
		if err != nil {
			continue
		}
		logs = append(logs, config.LogPair{Time: ts, Message: t[1]})
	}
	return logs
}

// Settings editor state: fields mirror the web dashboard settings page; every
// change persists to config.json immediately (same contract as PUT /api/config).
var settingsFields = []struct {
	name   string
	value  func(config.UserConfig) string
	adjust func(*config.UserConfig, int)
}{
	{"theme",
		func(c config.UserConfig) string { return c.Theme },
		func(c *config.UserConfig, d int) {
			c.Theme = cycleStr(c.Theme, []string{"default", "dark", "light"}, d)
		}},
	{"scan_timeout_secs",
		func(c config.UserConfig) string { return itou(c.ScanTimeoutSecs) + "s" },
		func(c *config.UserConfig, d int) {
			c.ScanTimeoutSecs = clampu64(int64(c.ScanTimeoutSecs)+int64(d)*5, 10, 120)
		}},
	{"daemon_interval_secs",
		func(c config.UserConfig) string { return itou(c.DaemonIntervalSecs) + "s" },
		func(c *config.UserConfig, d int) {
			c.DaemonIntervalSecs = clampu64(int64(c.DaemonIntervalSecs)+int64(d)*300, 300, 86400)
		}},
	{"export_format",
		func(c config.UserConfig) string { return c.ExportFormat },
		func(c *config.UserConfig, d int) {
			c.ExportFormat = cycleStr(c.ExportFormat, []string{"markdown", "json", "sarif"}, d)
		}},
	{"log_retention_days",
		func(c config.UserConfig) string { return itou(c.LogRetentionDays) },
		func(c *config.UserConfig, d int) {
			c.LogRetentionDays = clampu64(int64(c.LogRetentionDays)+int64(d), 0, 90)
		}},
	{"project_path",
		func(c config.UserConfig) string {
			if c.ProjectPath == nil {
				return "(current directory)"
			}
			return *c.ProjectPath
		},
		nil},
}

func cycleStr(cur string, opts []string, dir int) string {
	idx := 0
	for i, o := range opts {
		if o == cur {
			idx = i
			break
		}
	}
	idx = ((idx+dir)%len(opts) + len(opts)) % len(opts)
	return opts[idx]
}

func clampu64(v, lo, hi int64) uint64 {
	if v < lo {
		return uint64(lo)
	}
	if v > hi {
		return uint64(hi)
	}
	return uint64(v)
}
