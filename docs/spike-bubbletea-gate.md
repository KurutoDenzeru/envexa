# Spike — Bubbletea gate (Phase 0, issue #34)

**Date:** 2026-09-06 · **Branch:** `feat/tui-bubbletea-migration`
**Question:** can Go `charmbracelet/bubbletea` faithfully host the envexa TUI (dashboard + outdated + spinner + resize degradation) so Rust/ratatui can be sunset?
**Verdict:** ✅ **Go** — proceed to Phase 1 (Bash scanners) and Phase 2 (bubbletea TUI port). Abort criteria not triggered.

## Prototype

| File | Covers |
|---|---|
| `cmd/envexa/main.go` | TTY branch (`no args` → TUI), non-TTY exits with guidance — mirrors `src/main.rs` |
| `internal/tui/model.go` | `Model` = Rust `App` (View enum, selections, report, spinner, gauges); `simulateScan()` defers report so throbber path runs |
| `internal/tui/update.go` | `s`/`o`/`h`/`Esc`/`q`/arrows, tab switching, row selection — key parity with `src/tui/app.rs` |
| `internal/tui/view.go` | tabs, readiness/health gauges, status distribution, tables, guarded rendering (compact tabs <60, chart hidden <80, minimal fallback <40×12) |
| `internal/tui/styles.go` | color convention: ok=green, warn=yellow, error=red, skipped=darkgray (mirrors `theme.rs`) |
| `internal/tui/data.go` | mock `Report`/`ScanResult`/`OutdatedItem` with Rust JSON field names — Phase 1 bash scanners feed this unchanged |
| `internal/tui/bench_test.go` | frame-cost benchmarks + resize-guard assertions |

Widget mapping proven: `bubbles/table` (dashboard/outdated), `bubbles/spinner` Meter (throbber), `bubbles/progress` (Gauge/LineGauge), lipgloss tabs + bar distribution (tui-piechart fallback per issue #34 map).

## Frame cost

`go test -bench . -run ^$ ./internal/tui` (Apple M1 Max, 2000 iterations):

| Benchmark | ns/op | % of 16ms frame budget |
|---|---:|---:|
| Dashboard wide (140×40) | 101,058 | 0.63% |
| Dashboard narrow (70×30) | 80,614 | 0.50% |
| Outdated wide (140×40) | 49,435 | 0.31% |

One `View()` call renders the full frame; bubbletea diffs and repaints on tick. **No ratatui baseline was measured in this spike** — equivalent ratatui widget trees are string-builders of the same order, and scan I/O (seconds) dominates end-to-end latency by 4 orders of magnitude, so the gate decision does not hinge on this number. Phase 2 parity harness re-measures both runtimes on identical data.

## Resize matrix

Guards verified by `TestViewFallbacks`: tiny (<40 wide or <12 high) → minimal fallback text; narrow (<60) → single-letter tabs; medium (<80) → chart hidden, gauges+table kept; wide → full dashboard. Manual resize smoke pending on a real TTY during Phase 2 (spike ran headless; interactive demo recorded then).

## Gaps to Phase 2 (not spike scope)

- `cobra` CLI (`scan`/`serve`/`update`/`daemon`) — entrypoint is TUI-only today.
- Real data feed: bash scanner bridge (`internal/scanner` exec `toolchains/*.sh`) replaces `simulateScan()`.
- Full view set: vulnerabilities, toolchains, logs, settings; theme.rs parity details; update-runner progress.
- `mpsc`/`AppEvent` bridge semantics already map 1:1 onto `tea.Msg`/`tea.Cmd` (proven by `scanDoneMsg`/`spinner.TickMsg`).

## Recommendation

Phase 1 + Phase 2 proceed on this branch. Ratatui sunset stays gated on the parity harness per issue #34's sunset gate; if the Phase 2 port cannot match the resize matrix, abort criteria close this issue as "kept Rust" (hybrid end state).
