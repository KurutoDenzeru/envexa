// Package config is the Go port of src/core/config.rs: user config, scan
// cache, and log storage under the envexa data dir.
package config

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/report"
)

// UserConfig mirrors the serde field names of core::config::UserConfig.
type UserConfig struct {
	CacheTTLMinutes      uint64    `json:"cache_ttl_minutes"`
	ProjectPath          *string   `json:"project_path"`
	RecentProjectPaths   []string  `json:"recent_project_paths"`
	FavoriteProjectPaths []string  `json:"favorite_project_paths"`
	AutoScanOnStartup    bool      `json:"auto_scan_on_startup"`
	Theme                string    `json:"theme"`
	VerboseLogs          bool      `json:"verbose_logs"`
	ScanTimeoutSecs      uint64    `json:"scan_timeout_secs"`
	DaemonIntervalSecs   uint64    `json:"daemon_interval_secs"`
	ExportFormat         string    `json:"export_format"`
	EnabledScanners      *[]string `json:"enabled_scanners"`
	LogRetentionDays     uint64    `json:"log_retention_days"`
}

// Default mirrors the Rust Default impl and serde field defaults.
func Default() UserConfig {
	cfg := UserConfig{
		CacheTTLMinutes:    15,
		Theme:              "default",
		ScanTimeoutSecs:    30,
		DaemonIntervalSecs: 14400,
		ExportFormat:       "markdown",
		LogRetentionDays:   7,
	}
	normalize(&cfg)
	return cfg
}

// normalize keeps Vec fields serializing as [] instead of null.
func normalize(cfg *UserConfig) {
	if cfg.RecentProjectPaths == nil {
		cfg.RecentProjectPaths = []string{}
	}
	if cfg.FavoriteProjectPaths == nil {
		cfg.FavoriteProjectPaths = []string{}
	}
}

// Dir mirrors core::config::dir(): XDG_DATA_HOME, ~/.local/share/envexa, ~/.envexa.
func Dir() string {
	if xdg := os.Getenv("XDG_DATA_HOME"); xdg != "" {
		return filepath.Join(xdg, "envexa")
	}
	if home := homeDir(); home != "" {
		return filepath.Join(home, ".local", "share", "envexa")
	}
	return filepath.Join(".", ".envexa")
}

func homeDir() string {
	home, err := os.UserHomeDir()
	if err != nil {
		return "."
	}
	return home
}

func configPath() string { return filepath.Join(Dir(), "config.json") }
func cachePath() string  { return filepath.Join(Dir(), "cache.json") }
func LogsPath() string   { return filepath.Join(Dir(), "logs.json") }

// LoadConfig mirrors load_config: any parse failure falls back to Default
// (required serde fields missing make the whole parse fail in Rust).
func LoadConfig() UserConfig {
	raw, err := os.ReadFile(configPath())
	if err != nil {
		return Default()
	}
	var cfg UserConfig
	if json.Unmarshal(raw, &cfg) != nil {
		return Default()
	}
	normalize(&cfg)
	return cfg
}

func SaveConfig(cfg UserConfig) error {
	if err := os.MkdirAll(Dir(), 0o755); err != nil {
		return err
	}
	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(configPath(), data, 0o644)
}

// CacheEntry mirrors core::config::CacheEntry.
type CacheEntry struct {
	Report     report.Report `json:"report"`
	CachedAt   string        `json:"cached_at"`
	TTLMinutes uint64        `json:"ttl_minutes"`
}

func ReadCache() *CacheEntry {
	raw, err := os.ReadFile(cachePath())
	if err != nil {
		return nil
	}
	var entry CacheEntry
	if json.Unmarshal(raw, &entry) != nil {
		return nil
	}
	return &entry
}

// WriteCache stamps cached_at like the Rust "%Y-%m-%dT%H:%M:%S" local format.
func WriteCache(rep report.Report, ttlMinutes uint64) error {
	if err := os.MkdirAll(Dir(), 0o755); err != nil {
		return err
	}
	entry := CacheEntry{
		Report:     rep,
		CachedAt:   time.Now().Format("2006-01-02T15:04:05"),
		TTLMinutes: ttlMinutes,
	}
	data, err := json.MarshalIndent(entry, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(cachePath(), data, 0o644)
}

// CacheExpired parses cached_at; unparseable stamps count as expired.
func CacheExpired(entry *CacheEntry) bool {
	cached, err := time.ParseInLocation("2006-01-02T15:04:05", entry.CachedAt, time.Local)
	if err != nil {
		return true
	}
	return time.Since(cached) > time.Duration(entry.TTLMinutes)*time.Minute
}

// LogPair is one [timestamp, message] entry of logs.json.
type LogPair struct {
	Time    time.Time
	Message string
}

func ReadLogs() []LogPair {
	raw, err := os.ReadFile(LogsPath())
	if err != nil {
		return nil
	}
	var tuples [][2]string
	if json.Unmarshal(raw, &tuples) != nil {
		return nil
	}
	logs := make([]LogPair, 0, len(tuples))
	for _, t := range tuples {
		ts, err := time.Parse(time.RFC3339, t[0])
		if err != nil {
			continue
		}
		logs = append(logs, LogPair{Time: ts, Message: t[1]})
	}
	return logs
}

func WriteLogs(logs []LogPair) error {
	if err := os.MkdirAll(Dir(), 0o755); err != nil {
		return err
	}
	tuples := make([][2]string, 0, len(logs))
	for _, l := range logs {
		tuples = append(tuples, [2]string{l.Time.Format(time.RFC3339), l.Message})
	}
	data, err := json.Marshal(tuples)
	if err != nil {
		return err
	}
	return os.WriteFile(LogsPath(), data, 0o644)
}
