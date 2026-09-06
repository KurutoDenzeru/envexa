// Package server is the Go port of src/server.rs: the web dashboard API
// (8 endpoints, byte-compatible JSON shapes) plus static SPA serving.
package server

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"mime"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"sort"
	"strings"
	"syscall"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/config"
	"github.com/KurutoDenzeru/envexa/internal/scanner"
)

var Version = "3.0.0-alpha"

// Serve runs the HTTP server; blocks until interrupted.
func Serve(port int, dist string) error {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/scan", handleScan)
	mux.HandleFunc("GET /api/logs", handleLogs)
	mux.HandleFunc("GET /api/project", handleProjectGet)
	mux.HandleFunc("PUT /api/project", handleProjectSet)
	mux.HandleFunc("GET /api/project/dirs", handleProjectDirs)
	mux.HandleFunc("PUT /api/project/favorite", handleProjectFavorite)
	mux.HandleFunc("GET /api/config", handleConfigGet)
	mux.HandleFunc("PUT /api/config", handleConfigPut)
	mux.HandleFunc("GET /api/update/check", handleUpdateCheck)
	mux.HandleFunc("GET /api/version", handleVersion)
	mux.HandleFunc("/", staticHandler(dist))

	srv := &http.Server{Addr: fmt.Sprintf(":%d", port), Handler: mux}
	errCh := make(chan error, 1)
	go func() { errCh <- srv.ListenAndServe() }()
	fmt.Printf("Web Dashboard serving at http://localhost:%d\n", port)
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	select {
	case err := <-errCh:
		return err
	case <-stop:
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return srv.Shutdown(ctx)
	}
}

func handleScan(w http.ResponseWriter, r *http.Request) {
	force := r.URL.Query().Get("force") == "true"
	now := time.Now()
	logs := config.ReadLogs()

	if force {
		logs = append(logs, config.LogPair{Time: now, Message: "INFO: Web API scan forced, bypassing cache [system]"})
	} else if entry := config.ReadCache(); entry != nil {
		if !config.CacheExpired(entry) {
			logs = append(logs, config.LogPair{Time: now, Message: "INFO: Web API scan cache hit, returning cached report [system]"})
			_ = config.WriteLogs(logs)
			writeJSON(w, entry.Report)
			return
		}
		logs = append(logs, config.LogPair{Time: now, Message: "INFO: Web API scan cache expired, running fresh scan [system]"})
	} else {
		logs = append(logs, config.LogPair{Time: now, Message: "INFO: Web API scan cache miss, running fresh scan [system]"})
	}
	logs = append(logs, config.LogPair{Time: now, Message: "INFO: Running multi-language scan engine... [system]"})

	cfg := config.LoadConfig()
	rep := scanner.Scan(toolchainsDir(), time.Duration(cfg.ScanTimeoutSecs)*time.Second)
	if cfg.EnabledScanners != nil && len(*cfg.EnabledScanners) > 0 {
		enabled := map[string]bool{}
		for _, name := range *cfg.EnabledScanners {
			enabled[name] = true
		}
		for name := range rep.Results {
			if !enabled[name] {
				delete(rep.Results, name)
			}
		}
	}
	rep.Timestamp = time.Now().Format("2006-01-02T15:04:05")

	logs = append(logs, config.LogPair{Time: time.Now(), Message: "INFO: Web API scan completed successfully [system]"})
	_ = config.WriteLogs(logs)

	if err := config.WriteCache(rep, cfg.CacheTTLMinutes); err != nil {
		logs = config.ReadLogs()
		logs = append(logs, config.LogPair{Time: time.Now(),
			Message: fmt.Sprintf("ERROR: Failed to write scan cache: %s [system]", err)})
		_ = config.WriteLogs(logs)
	}
	writeJSON(w, rep)
}

func handleLogs(w http.ResponseWriter, r *http.Request) {
	raw := config.ReadLogs()
	path := config.LogsPath()

	var logs []LogEntry
	if len(raw) == 0 {
		seed := seedLogs()
		_ = config.WriteLogs(seed)
		for _, pair := range seed {
			logs = append(logs, parseLogLine(pair))
		}
	} else {
		for _, pair := range raw {
			logs = append(logs, parseLogLine(pair))
		}
	}
	writeJSON(w, map[string]any{"path": path, "logs": logs})
}

func handleProjectGet(w http.ResponseWriter, r *http.Request) {
	cfg := config.LoadConfig()
	writeJSON(w, projectResponse(cfg))
}

func handleProjectSet(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Path string `json:"path"`
	}
	if !decodeBody(w, r, &req) {
		return
	}
	info, err := os.Stat(req.Path)
	if err != nil || !info.IsDir() {
		http.Error(w, fmt.Sprintf("Path does not exist or is not a directory: %s", req.Path), http.StatusBadRequest)
		return
	}
	cfg := config.LoadConfig()
	resolved := filepath.Clean(req.Path)
	recent := []string{}
	if cfg.RecentProjectPaths != nil {
		recent = cfg.RecentProjectPaths
	}
	filtered := recent[:0]
	for _, p := range recent {
		if p != resolved {
			filtered = append(filtered, p)
		}
	}
	recent = append([]string{resolved}, filtered...)
	if len(recent) > 10 {
		recent = recent[:10]
	}
	cfg.ProjectPath = &resolved
	cfg.RecentProjectPaths = recent
	if err := config.SaveConfig(cfg); err != nil {
		http.Error(w, fmt.Sprintf("Failed to save config: %s", err), http.StatusInternalServerError)
		return
	}
	writeJSON(w, projectResponse(cfg))
}

func handleProjectFavorite(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Path string `json:"path"`
	}
	if !decodeBody(w, r, &req) {
		return
	}
	cfg := config.LoadConfig()
	favs := []string{}
	if cfg.FavoriteProjectPaths != nil {
		favs = cfg.FavoriteProjectPaths
	}
	found := false
	for _, p := range favs {
		if p == req.Path {
			found = true
		}
	}
	if found {
		kept := favs[:0]
		for _, p := range favs {
			if p != req.Path {
				kept = append(kept, p)
			}
		}
		favs = kept
	} else {
		favs = append(favs, req.Path)
	}
	cfg.FavoriteProjectPaths = favs
	if err := config.SaveConfig(cfg); err != nil {
		http.Error(w, fmt.Sprintf("Failed to save config: %s", err), http.StatusInternalServerError)
		return
	}
	writeJSON(w, projectResponse(cfg))
}

func handleProjectDirs(w http.ResponseWriter, r *http.Request) {
	target := r.URL.Query().Get("path")
	if target == "" {
		cwd, err := os.Getwd()
		if err != nil {
			target = ""
		} else {
			target = cwd
		}
	}
	info, err := os.Stat(target)
	if err != nil || !info.IsDir() {
		http.Error(w, fmt.Sprintf("Not a directory: %s", target), http.StatusBadRequest)
		return
	}
	var parent *string
	if dir := filepath.Dir(filepath.Clean(target)); dir != target {
		p := dir
		parent = &p
	}
	entries := []map[string]string{}
	dirEntries, err := os.ReadDir(target)
	if err == nil {
		for _, e := range dirEntries {
			if !e.IsDir() || strings.HasPrefix(e.Name(), ".") {
				continue
			}
			entries = append(entries, map[string]string{
				"name":      e.Name(),
				"full_path": filepath.Join(target, e.Name()),
			})
		}
	}
	sort.Slice(entries, func(i, j int) bool {
		return strings.ToLower(entries[i]["name"]) < strings.ToLower(entries[j]["name"])
	})
	writeJSON(w, map[string]any{"path": target, "parent": parent, "entries": entries})
}

func handleConfigGet(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, config.LoadConfig())
}

func handleConfigPut(w http.ResponseWriter, r *http.Request) {
	var cfg config.UserConfig
	body, err := io.ReadAll(r.Body)
	if err != nil || json.Unmarshal(body, &cfg) != nil {
		http.Error(w, "invalid config body", http.StatusBadRequest)
		return
	}
	if err := config.SaveConfig(cfg); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, config.LoadConfig())
}

func handleUpdateCheck(w http.ResponseWriter, r *http.Request) {
	current := strings.TrimPrefix(Version, "v")
	resp := map[string]any{
		"current_version":  current,
		"latest_version":   current,
		"update_available": false,
		"release_body":     "",
	}
	type release struct {
		TagName string `json:"tag_name"`
		Body    string `json:"body"`
	}
	client := &http.Client{Timeout: 10 * time.Second}
	req, _ := http.NewRequestWithContext(r.Context(), http.MethodGet,
		"https://api.github.com/repos/KurutoDenzeru/envexa/releases/latest", nil)
	req.Header.Set("User-Agent", "envexa")
	if respHTTP, err := client.Do(req); err == nil {
		var rel release
		if json.NewDecoder(respHTTP.Body).Decode(&rel) == nil && rel.TagName != "" {
			latest := strings.TrimPrefix(rel.TagName, "v")
			resp["latest_version"] = latest
			resp["update_available"] = latest != current
			resp["release_body"] = rel.Body
		}
		respHTTP.Body.Close()
	}
	writeJSON(w, resp)
}

func handleVersion(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, map[string]string{"version": Version})
}

func staticHandler(dist string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		name := strings.TrimPrefix(r.URL.Path, "/")
		if name == "" {
			name = "index.html"
		}
		path := filepath.Join(dist, filepath.Clean("/"+name))
		if _, err := os.Stat(path); err != nil {
			// SPA fallback, like static_handler in server.rs.
			path = filepath.Join(dist, "index.html")
			if _, err := os.Stat(path); err != nil {
				http.Error(w, "404 Not Found", http.StatusNotFound)
				return
			}
			w.Header().Set("Content-Type", "text/html")
			http.ServeFile(w, r, path)
			return
		}
		w.Header().Set("Content-Type", mimeType(name))
		http.ServeFile(w, r, path)
	}
}

// --- shared helpers -----------------------------------------------------------

type LogEntry struct {
	Time    string `json:"time"`
	Date    string `json:"date"`
	Level   string `json:"level"`
	Message string `json:"message"`
	Source  string `json:"source"`
}

// parseLogLine mirrors server.rs parse_log_line: "LEVEL: message [source]".
func parseLogLine(pair config.LogPair) LogEntry {
	level := "INFO"
	source := "system"
	message := pair.Message

	for _, p := range []struct{ prefix, name string }{
		{"INFO: ", "INFO"}, {"WARN: ", "WARN"}, {"ERROR: ", "ERROR"}, {"DEBUG: ", "DEBUG"},
	} {
		if strings.HasPrefix(message, p.prefix) {
			level = p.name
			message = message[len(p.prefix):]
			break
		}
	}
	if start := strings.LastIndex(message, "["); start >= 0 {
		if end := strings.LastIndex(message, "]"); start < end {
			source = message[start+1 : end]
			message = strings.TrimRight(message[:start], " ")
			message = strings.TrimLeft(message, " ")
		}
	}
	return LogEntry{
		Time:    pair.Time.Format("15:04:05"),
		Date:    pair.Time.Format("January 02, 2006"),
		Level:   level,
		Message: message,
		Source:  source,
	}
}

// seedLogs mirrors api_logs' fake 7-day history used when logs.json is empty.
func seedLogs() []config.LogPair {
	type offsetPair struct {
		off time.Duration
		msg string
	}
	now := time.Now()
	out := []config.LogPair{}
	day := 24 * time.Hour
	scanCycle := []offsetPair{
		{0, "INFO: Envexa daemon started — daily scan initiated [system]"},
		{45 * time.Second, "INFO: Detected Node.js project. Scanning package.json... [node]"},
		{75 * time.Second, "INFO: Detected Rust project. Scanning Cargo.toml... [rust]"},
		{110 * time.Second, "INFO: Detected Python project. Scanning requirements.txt... [python]"},
		{140 * time.Second, "INFO: Scan completed. Generated report. [system]"},
	}
	todayTimeline := []offsetPair{
		{-10 * time.Minute, "INFO: Starting Envexa scanner engine... [system]"},
		{-9 * time.Minute, "INFO: Detected Node.js project. Scanning package.json... [node]"},
		{-8 * time.Minute, "WARN: Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21) [node]"},
		{-7 * time.Minute, "INFO: Detected Rust project. Scanning Cargo.toml... [rust]"},
		{-6 * time.Minute, "ERROR: Security vulnerability found in 'regex' crate: CVE-2022-24713 [rust]"},
		{-5 * time.Minute, "INFO: Detected Python project. Scanning requirements.txt... [python]"},
		{-4 * time.Minute, "INFO: Scan completed successfully. Generated report. [system]"},
		{-3 * time.Minute, "DEBUG: Cleaning up temporary files... [system]"},
		{-1 * time.Minute, "INFO: Web API server listening on port 8080 [system]"},
	}
	for dayOffset := 6; dayOffset >= 0; dayOffset-- {
		base := now.Add(-time.Duration(dayOffset) * day)
		if dayOffset > 0 {
			dayBase := time.Date(base.Year(), base.Month(), base.Day(), 9, 15, 0, 0, base.Location())
			for _, p := range scanCycle {
				out = append(out, config.LogPair{Time: dayBase.Add(p.off), Message: p.msg})
			}
			if dayOffset%3 == 0 {
				out = append(out, config.LogPair{Time: dayBase.Add(90 * time.Second),
					Message: "WARN: Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21) [node]"})
			}
			if dayOffset == 2 {
				out = append(out, config.LogPair{Time: dayBase.Add(100 * time.Second),
					Message: "ERROR: Security vulnerability found in 'regex' crate: CVE-2022-24713 [rust]"})
			}
		} else {
			for _, p := range todayTimeline {
				out = append(out, config.LogPair{Time: now.Add(p.off), Message: p.msg})
			}
		}
	}
	return out
}

func projectResponse(cfg config.UserConfig) map[string]any {
	current := ""
	if cfg.ProjectPath != nil && *cfg.ProjectPath != "" {
		current = *cfg.ProjectPath
	} else if cwd, err := os.Getwd(); err == nil {
		current = cwd
	}
	recent := cfg.RecentProjectPaths
	if recent == nil {
		recent = []string{}
	}
	favs := cfg.FavoriteProjectPaths
	if favs == nil {
		favs = []string{}
	}
	return map[string]any{"current": current, "recent": recent, "favorites": favs}
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(v)
}

func decodeBody(w http.ResponseWriter, r *http.Request, v any) bool {
	body, err := io.ReadAll(r.Body)
	if err != nil || json.Unmarshal(body, v) != nil {
		http.Error(w, "invalid JSON body", http.StatusBadRequest)
		return false
	}
	return true
}

func mimeType(name string) string {
	// mime_guess never appends charset — strip Go's suffix for byte parity.
	m := mime.TypeByExtension(filepath.Ext(name))
	if m != "" {
		if i := strings.Index(m, ";"); i >= 0 {
			return strings.TrimSpace(m[:i])
		}
		return m
	}
	switch {
	case strings.HasSuffix(name, ".js"), strings.HasSuffix(name, ".mjs"):
		return "text/javascript"
	case strings.HasSuffix(name, ".css"):
		return "text/css"
	case strings.HasSuffix(name, ".svg"):
		return "image/svg+xml"
	default:
		return "application/octet-stream"
	}
}

func toolchainsDir() string {
	if d := os.Getenv("ENVEXA_TOOLCHAINS_DIR"); d != "" {
		return d
	}
	return "toolchains"
}
