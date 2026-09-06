// Command envexa — cobra entrypoint mirroring src/core/cli.rs: no args + TTY
// runs the bubbletea TUI, subcommands run the CLI.
package main

import (
	"fmt"
	"os"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/spf13/cobra"

	"github.com/KurutoDenzeru/envexa/internal/cli"
	"github.com/KurutoDenzeru/envexa/internal/tui"
)

// Version is overridden at release time via -ldflags "-X ...cli.Version=...".

func main() {
	root := &cobra.Command{
		Use:     "envexa",
		Short:   "Dependency environment scanner",
		Version: cli.Version,
		RunE: func(cmd *cobra.Command, args []string) error {
			if isTTY() {
				p := tea.NewProgram(tui.NewModel(), tea.WithAltScreen())
				_, err := p.Run()
				return err
			}
			return cmd.Help()
		},
	}

	scanCmd := &cobra.Command{
		Use:   "scan",
		Short: "Full health scan (CLI output, for scripting)",
		RunE: func(cmd *cobra.Command, args []string) error {
			format, _ := cmd.Flags().GetString("format")
			return cli.RunScan(format, os.Stdout)
		},
	}
	scanCmd.Flags().StringP("format", "f", "markdown", "output format (markdown|json)")
	scanCmd.Flags().Uint64("ttl", 7, "cache TTL in minutes")
	root.AddCommand(scanCmd)

	serveCmd := &cobra.Command{
		Use:   "serve",
		Short: "Serve the web dashboard",
		RunE: func(cmd *cobra.Command, args []string) error {
			port, _ := cmd.Flags().GetInt("port")
			dist, _ := cmd.Flags().GetString("dist")
			return cli.Serve(port, dist)
		},
	}
	serveCmd.Flags().Int("port", 8080, "port to listen on")
	serveCmd.Flags().String("dist", "frontend/dist", "static frontend directory")
	root.AddCommand(serveCmd)

	updateCmd := &cobra.Command{
		Use:   "update",
		Short: "Check for a newer envexa release",
		RunE: func(cmd *cobra.Command, args []string) error {
			return cli.CheckUpdate(os.Stdout)
		},
	}
	root.AddCommand(updateCmd)

	daemonCmd := &cobra.Command{
		Use:   "daemon",
		Short: "Run scans on an interval until stopped",
		RunE: func(cmd *cobra.Command, args []string) error {
			secs, _ := cmd.Flags().GetUint64("interval")
			if secs == 0 {
				secs = 300
			}
			return cli.RunDaemon(time.Duration(secs)*time.Second, os.Stdout)
		},
	}
	daemonCmd.Flags().Uint64("interval", 300, "seconds between scans")
	root.AddCommand(daemonCmd)

	if err := root.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "envexa: %v\n", err)
		os.Exit(1)
	}
}

func isTTY() bool {
	info, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}
