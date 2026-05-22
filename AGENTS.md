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
| `cargo run -- status` | CLI mode (args present) |
| `cargo fmt` | Format all code |
| `cargo clippy` | Lint check |
| `cargo test` | Run tests |

Quick test:
```bash
cargo run -- scan brew    # CLI scan
cargo run                 # Interactive TUI
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
- Input: `crossterm::event::read()` loop with `KeyCode` matching; `s`=scan, `o`=outdated, `h`=home, `q`=quit, `Esc`=back, `jk/arrows`=navigate
- Color convention: `ok`=green, `warning`=yellow, `error`=red, `skipped`=darkgray
- Table navigation: `dashboard_selection` / `outdated_selection` tracked per view
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
2. `cargo run -- scan brew` — scan output is readable
3. `cargo run -- status` — status table is aligned
4. `cargo run -- outdated` — outdated table is correct
5. `cargo run -- info` — info displays
6. `cargo run` (no args, in terminal) — TUI launches, can navigate with arrows, `S` triggers scan, `O` shows outdated, `Q` quits

Do not push if any of these produce warnings or malformed output. Fix first, then push.

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
