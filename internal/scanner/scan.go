// Package scanner is the Go bridge over the bash toolchain scanners — the
// counterpart of Rust scan_all_with (src/toolchains/mod.rs): run every
// toolchains/*.sh concurrently, timeout each, merge ScanResult JSON. Results
// are keyed by scanner name (script base), not result.tool — skipped results
// carry tool:"" exactly like ScanResult::skipped in Rust.
package scanner

import (
	"context"
	"encoding/json"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/report"
)

const defaultTimeout = 30 * time.Second

// Scan runs every *.sh directly under toolchainsDir (lib/ excluded) and merges
// their JSON into a Report. Scripts that fail or time out are recorded as
// status "error" results instead of aborting the scan — a missing CLI tool
// must never crash the report.
func Scan(toolchainsDir string, timeout time.Duration) report.Report {
	scripts, _ := filepath.Glob(filepath.Join(toolchainsDir, "*.sh"))
	results := make([]report.ScanResult, len(scripts))
	names := make([]string, len(scripts))
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	var wg sync.WaitGroup
	for i, script := range scripts {
		wg.Add(1)
		go func(i int, script string) {
			defer wg.Done()
			names[i] = strings.TrimSuffix(filepath.Base(script), ".sh")
			c, cancel := context.WithTimeout(ctx, defaultTimeout)
			defer cancel()
			out, err := exec.CommandContext(c, "bash", script).Output()
			var r report.ScanResult
			if err == nil {
				err = json.Unmarshal(out, &r)
			}
			// tool:"" is only legitimate for skipped results.
			if err != nil || (r.Tool == "" && r.Status != "skipped") {
				r = report.ScanResult{Tool: names[i], Status: "error", Issues: []string{"scanner failed"}}
			}
			results[i] = r
		}(i, script)
	}
	wg.Wait()

	reportData := report.Report{
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Results:   make(map[string]report.ScanResult, len(results)),
	}
	for i, r := range results {
		name := names[i]
		if r.Tool != "" {
			name = r.Tool
		}
		reportData.Results[name] = r
		// Flat outdated list for the TUI outdated view; size is unknown at
		// scan time. Covers npm's global list, brew's formulae+casks split,
		// and the plain per-project `outdated` arrays.
		flatten := func(pkgs []report.PackageInfo) {
			for _, p := range pkgs {
				reportData.Outdated = append(reportData.Outdated, report.OutdatedItem{
					Source: name, Name: p.Name, Current: p.Current, Latest: p.Latest,
				})
			}
		}
		flatten(r.OutdatedGlobal)
		flatten(r.OutdatedFormulae)
		flatten(r.OutdatedCasks)
		flatten(r.Outdated)
	}
	return reportData
}
