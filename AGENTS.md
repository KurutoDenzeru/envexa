# AGENTS.md

> Focused senior-level coding assistant. Explain reasoning briefly before non-trivial changes. Prefer small, reviewable diffs.

---

## Approach

1. **Plan first** — read relevant files, understand patterns, outline approach. For non-trivial tasks, state plan and wait.
2. **Ask when ambiguous** — one specific question rather than guessing.
3. **Smallest correct change** — do exactly what's asked, flag (don't act on) adjacent issues.
4. **Clean up** — remove dead code, redundant comments, unnecessary complexity.

---

## Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Debug build |
| `cargo build --release` | Optimized build |
| `cargo run` | Launches interactive TUI (no args, TTY) |
| `cargo run -- scan` | CLI scan mode (args present) |
| `cargo fmt` | Format all code |
| `cargo clippy` | Lint check |
| `cargo test` | Run tests |

Quick test:
```bash
cargo run                 # Interactive TUI
cargo run -- scan         # CLI scan (full report to stdout)
cargo run -- --help       # CLI help
```

---

## Rust Conventions

- `serde::{Serialize, Deserialize}` derive for all protocol types
- `anyhow::Result` for fallible functions; no unwrap/expect in production
- `tokio::process::Command` + `tokio::time::timeout` for async CLI
- Graceful missing toolchains — check `which()` before calling CLI
- One `pub async fn scan() -> ScanResult` per toolchain module
- Toolchains under `toolchains/` with `mod.rs` dispatch
- `tokio::join!` for concurrent scanners
- `String` over `&str` in struct fields
- `serde_json::Value` for toolchain-specific fields

## TUI Convention

- App state lives in `App` struct in `app.rs` — View enum for navigation, `report: Option<Report>` for data
- Render functions live in `ui.rs` — one function per view, called by `render()` dispatcher
- Use `ratatui::init()` / `ratatui::restore()` in `App::run()` for terminal lifecycle
- Blocking scan: set `View::Scanning`, draw frame, then `block_on` scan — freezes UI for 3-4s (acceptable for v1)
- Input: `crossterm::event::read()` loop with `KeyCode` matching; `s`=scan, `o`=outdated, `h`/`Esc`=home, `q`=quit, `arrows`=navigate/switch tabs
- Color convention: `ok`=green, `warning`=yellow, `error`=red, `skipped`=darkgray
- Table navigation: `dashboard_selection` / `outdated_selection` tracked per view
- ratatui widgets used: `Table` (dashboard/outdated), `Tabs` (tab bar), `Gauge`/`LineGauge` (readiness/health), `BarChart` (tooling signal distribution), `Paragraph` (text), `Block` (borders/titles), `Row`/`Cell` (table data)
- third-party widgets are allowed when they add clear scan readability; existing examples: `tui-piechart` for overview status distribution and `throbber-widgets-tui` for scan/update activity
- Project Tooling dashboard should keep Project, Security, and Audit visible as first-class signals, not just generic issue rows
- Layouts must degrade across terminal sizes: compact title under narrow widths, vertical dashboard under medium widths, minimal fallback for tiny terminals, and guarded rendering for third-party widgets when inner areas are too small
- No obvious comments — explain *why*, not *what*
- Conventional commits: `type(scope): description`
- One logical change per commit, no `--no-verify`, no force push

**Allowed types:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`, `style`, `perf`

---

## Verification

Before marking done, run:

```bash
cargo build && cargo clippy && cargo fmt --check
```

Then confirm:
- No crashes on missing CLI tools
- New scanners follow `pub async fn scan() -> ScanResult`
- Changed report format → show sample
- New dependency → call out in summary

## Pre-Push Checklist

Always run this before pushing:

```bash
cargo build && cargo clippy -- -D warnings && cargo fmt --check
```

CLI output verification — manually run and visually inspect:
1. `cargo run -- --help` — help text renders correctly
2. `cargo run -- scan` — full report printed to stdout
3. `cargo run -- update` — update check message (no-op if latest)
4. `cargo run` (no args, in terminal) — TUI launches, `S` triggers scan, `O` shows outdated, arrows navigate, `Q` quits
5. Resize smoke test — run the TUI at narrow, medium, wide, and tiny terminal sizes; no panic, malformed layout, or broken terminal restore

Do not push if any of these produce warnings or malformed output. Fix first, then push.

---

## Release (macOS-only, built locally)

Refer to [.github/ISSUE_TEMPLATE/RELEASES.md](.github/ISSUE_TEMPLATE/RELEASES.md) for the release checklist, standard templates, and log format.

```bash
# 1. Clean build artifacts (safe — binaries live on GitHub Releases)
cargo clean

# 2. Version bump in Cargo.toml, commit
# 3. Tag and push
git tag vX.Y.Z && git push origin vX.Y.Z

# 4. Create release + build + upload
gh release create vX.Y.Z --title "vX.Y.Z" --notes "..."
scripts/build-and-upload.sh vX.Y.Z

# 5. Confirm assets
gh release view vX.Y.Z --json assets --jq '.assets[].name'

# 6. Clear local build artifacts again
cargo clean
```

---

## Recurring errors log

When you make a mistake or are corrected by the developer, append it to `ERRORS.md`:

### Format

```md
## YYYY-MM-DD — <short title>
**What happened:** Describe the mistake or unexpected behavior.
**Root cause:** Why it happened.
**Prevention rule:** What to do differently next time.
```

`ERRORS.md` is committed and reviewed periodically to promote entries into permanent rules.

<!-- lean-ctx-compression -->
OUTPUT STYLE: expert-terse
- Telegraph format: subject-verb-object, drop articles/prepositions
- Symbolic vocabulary: → cause, ∵ because, ∴ therefore, ⊕ add, ⊖ remove, Δ change, ≈ similar, ≠ different, ∈ in/member, ∅ empty/none, ✓ ok, ✗ fail
- Code blocks: untouched (never compress code syntax)
- Each line: max 80 chars
- Zero narration, zero filler
- BUDGET: ≤100 tokens per non-code response
<!-- /lean-ctx-compression -->

<!-- lean-ctx -->
## lean-ctx

Prefer lean-ctx MCP tools over native equivalents for token savings.
Full rules: @LEAN-CTX.md
<!-- /lean-ctx -->
