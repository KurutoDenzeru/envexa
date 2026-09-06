package cli

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

// TestRunScanRunsRealScanners exercises the CLI against the repo's real
// toolchains/*.sh; passes on any host because scanners may skip.
func TestRunScanRunsRealScanners(t *testing.T) {
	var buf bytes.Buffer
	if err := RunScan("json", &buf); err != nil {
		t.Fatal(err)
	}
	var rep map[string]any
	if err := json.Unmarshal(buf.Bytes(), &rep); err != nil {
		t.Fatalf("scan --format json invalid: %v", err)
	}
	if rep["timestamp"] == nil || rep["results"] == nil {
		t.Fatalf("report missing timestamp/results: %v", rep)
	}

	buf.Reset()
	if err := RunScan("markdown", &buf); err != nil {
		t.Fatal(err)
	}
	out := buf.String()
	if !strings.Contains(out, "| Tool | Status |") {
		t.Fatalf("markdown missing table header: %q", out[:min(200, len(out))])
	}

	if err := RunScan("yaml", &buf); err == nil {
		t.Fatal("unknown format should error")
	}
}
