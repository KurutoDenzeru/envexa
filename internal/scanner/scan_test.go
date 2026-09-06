package scanner

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestScanMergesScripts(t *testing.T) {
	dir := t.TempDir()
	good := `#!/usr/bin/env bash
jq -n '{tool:"npm", status:"warning", version:"11.4.2", outdated_global:[{name:"typescript",current:"5.9.2",latest:"5.9.3"}], issues:["1 outdated package(s)"]}'`
	other := `#!/usr/bin/env bash
printf '%s' '{"tool":"brew","status":"ok"}'`
	broken := `#!/usr/bin/env bash
echo 'not json'`
	for name, body := range map[string]string{"npm.sh": good, "brew.sh": other, "cargo.sh": broken} {
		if err := os.WriteFile(filepath.Join(dir, name), []byte(body), 0o755); err != nil {
			t.Fatal(err)
		}
	}
	report := Scan(dir, 30*time.Second)

	if got := len(report.Results); got != 3 {
		t.Fatalf("want 3 results, got %d", got)
	}
	npm := report.Results["npm"]
	if npm.Status != "warning" || npm.Version != "11.4.2" || len(npm.OutdatedGlobal) != 1 ||
		npm.OutdatedGlobal[0].Name != "typescript" {
		t.Fatalf("npm scan result wrong: %+v", npm)
	}
	if report.Results["brew"].Status != "ok" {
		t.Fatalf("brew scan result wrong: %+v", report.Results["brew"])
	}
	if cargo := report.Results["cargo"]; cargo.Status != "error" || cargo.Tool != "cargo" {
		t.Fatalf("broken script should be status error with tool name: %+v", cargo)
	}
	if report.Timestamp == "" {
		t.Fatal("timestamp missing")
	}
}

// TestScanRealToolchains runs the actual toolchains/*.sh in this repo. It must
// pass on any host: scanners return status "skipped" when tools are missing.
func TestScanRealToolchains(t *testing.T) {
	report := Scan("../../toolchains", 60*time.Second)
	if len(report.Results) < 2 {
		t.Fatalf("expected brew+npm scanners, got %d results", len(report.Results))
	}
	for tool, r := range report.Results {
		// Skipped scanners report tool:"" (Rust ScanResult::skipped) but the
		// map key stays the scanner name, like scan_all_with.
		if r.Tool != tool && !(r.Tool == "" && r.Status == "skipped") {
			t.Fatalf("tool %q reported as %q", tool, r.Tool)
		}
		switch r.Status {
		case "ok", "warning", "warn", "error", "skipped":
		default:
			t.Fatalf("tool %q: invalid status %q", tool, r.Status)
		}
	}
}
