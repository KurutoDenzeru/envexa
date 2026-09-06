// Package scanner is the Go bridge over the bash toolchain scanners — the
// counterpart of Rust scan_all_with (src/toolchains/mod.rs): run every
// toolchains/*.sh concurrently, 30s timeout each, merge ScanResult JSON.
package scanner

import (
	"context"
	"encoding/json"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/tui"
)

const defaultTimeout = 30 * time.Second

// Scan runs every *.sh directly under toolchainsDir (lib/ excluded) and merges
// their JSON into a Report. Scripts that fail or time out are recorded as
// status "error" results instead of aborting the scan — a missing CLI tool
// must never crash the report.
func Scan(toolchainsDir string, timeout time.Duration) tui.Report {
	scripts, _ := filepath.Glob(filepath.Join(toolchainsDir, "*.sh"))
	results := make([]tui.ScanResult, len(scripts))
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	var wg sync.WaitGroup
	for i, script := range scripts {
		wg.Add(1)
		go func(i int, script string) {
			defer wg.Done()
			c, cancel := context.WithTimeout(ctx, defaultTimeout)
			defer cancel()
			out, err := exec.CommandContext(c, "bash", script).Output()
			var r tui.ScanResult
			if err == nil {
				err = json.Unmarshal(out, &r)
			}
			if err != nil || r.Tool == "" {
				name := strings.TrimSuffix(filepath.Base(script), ".sh")
				r = tui.ScanResult{Tool: name, Status: "error", Issues: []string{"scanner failed"}}
			}
			results[i] = r
		}(i, script)
	}
	wg.Wait()

	report := tui.Report{
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Results:   make(map[string]tui.ScanResult, len(results)),
	}
	for _, r := range results {
		report.Results[r.Tool] = r
	}
	return report
}
