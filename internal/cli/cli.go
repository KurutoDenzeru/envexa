// Package cli holds the envexa subcommand logic (scan/serve/update/daemon),
// kept out of the cobra wiring so it stays testable.
package cli

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/signal"
	"sort"
	"strings"
	"syscall"
	"time"

	"github.com/KurutoDenzeru/envexa/internal/report"
	"github.com/KurutoDenzeru/envexa/internal/scanner"
)

// Version is overridden at release time via -ldflags "-X ...cli.Version=...".
var Version = "3.0.0-alpha"

// RunScan executes all bash scanners and renders the report.
func RunScan(format string, out io.Writer) error {
	rep := scanner.Scan(scanner.Dir(), 60*time.Second)
	switch format {
	case "json":
		enc := json.NewEncoder(out)
		enc.SetIndent("", "  ")
		return enc.Encode(rep)
	case "markdown":
		_, _ = fmt.Fprint(out, renderMarkdown(rep))
		return nil
	default:
		return fmt.Errorf("unknown format %q (want markdown|json|sarif)", format)
	}
}

func renderMarkdown(rep report.Report) string {
	var b strings.Builder
	fmt.Fprintf(&b, "# envexa report — %s\n\n", rep.Timestamp)
	tools := make([]string, 0, len(rep.Results))
	for name := range rep.Results {
		tools = append(tools, name)
	}
	sort.Strings(tools)
	b.WriteString("| Tool | Status | Version | Notes |\n|---|---|---|---|\n")
	for _, name := range tools {
		r := rep.Results[name]
		fmt.Fprintf(&b, "| %s | %s | %s | %s |\n", name, r.Status, r.Version,
			strings.Join(r.Issues, "; "))
	}
	if n := len(rep.Outdated); n > 0 {
		fmt.Fprintf(&b, "\n## Outdated (%d)\n\n| Source | Package | Current | Latest |\n|---|---|---|---|\n", n)
		for _, o := range rep.Outdated {
			fmt.Fprintf(&b, "| %s | %s | %s | %s |\n", o.Source, o.Name, o.Current, o.Latest)
		}
	}
	if sec, ok := rep.Results["security"]; ok && len(sec.Vulnerabilities) > 0 {
		fmt.Fprintf(&b, "\n## Vulnerabilities (%d)\n\n", len(sec.Vulnerabilities))
		for _, v := range sec.Vulnerabilities {
			fmt.Fprintf(&b, "- **%s** [%s] %s (fixed: %s)\n", v.Package, v.Severity, v.Title, v.PatchedVersion)
		}
	}
	return b.String()
}

// CheckUpdate compares Version against the latest GitHub release tag.
func CheckUpdate(out io.Writer) error {
	const repo = "KurutoDenzeru/envexa"
	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Get("https://api.github.com/repos/" + repo + "/releases/latest")
	if err != nil {
		return err
	}
	defer func() { _ = resp.Body.Close() }()
	var body struct {
		TagName string `json:"tag_name"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		return err
	}
	latest := strings.TrimPrefix(body.TagName, "v")
	current := strings.TrimPrefix(Version, "v")
	if latest == "" {
		_, _ = fmt.Fprintln(out, "envexa: no releases found")
		return nil
	}
	if latest == current {
		_, _ = fmt.Fprintf(out, "envexa %s is up to date (latest %s)\n", current, body.TagName)
	} else {
		_, _ = fmt.Fprintf(out, "envexa %s -> %s available (run the installer to update)\n", current, body.TagName)
	}
	return nil
}

// RunDaemon re-scans on an interval until interrupted.
func RunDaemon(interval time.Duration, out io.Writer) error {
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	run := func() {
		rep := scanner.Scan(scanner.Dir(), 60*time.Second)
		_, _ = fmt.Fprintf(out, "%s scan: %d toolchains checked\n", time.Now().Format(time.RFC3339), len(rep.Results))
	}
	run()
	for {
		select {
		case <-stop:
			return nil
		case <-ticker.C:
			run()
		}
	}
}
