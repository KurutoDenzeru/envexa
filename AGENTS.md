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

## TUI Convention (Ink on Bun, `tui/`)

- Ink app (TypeScript + React on Bun): entry `tui/cli.tsx` renders `<App>` with `{ alternateScreen: true }`
- App state lives in the `App` component in `cli.tsx`; views are components under `tui/views/` (Dashboard / Outdated / PackageDetail / Updating / Logs / Settings), shared chrome under `tui/components/`
- Scan bridge `tui/lib/scan.ts` runs every `toolchains/*.sh` concurrently (30s timeout each) and merges `ScanResult` JSON — same contract as the Go bridge; config/logs/settings editor in `tui/lib/config.ts`, update runner in `tui/lib/update.ts`, shared formatting in `tui/lib/format.ts`
- Color convention (`tui/theme.ts`): `ok`=green, `warn`=yellow, `error`=red, `skipped`=gray, accent `#5fffd7`
- Keys: `s`=scan, `o`=outdated, `l`=logs, `c`=settings, `h`/`Esc`=home, `q`=quit, `←→`=switch views, `↑↓`=navigate, `/`=filter, Enter=detail, `y`=update confirm, Tab=dashboard sub-tab (overview/vulns/toolchains)
- Project Tooling dashboard keeps Project, Security, and Audit visible as first-class signals
- Layouts degrade across terminal sizes: compact title under narrow widths, vertical dashboard under ~100 cols, pie hidden when the panel is too small, minimal fallback for tiny terminals
- Dev: `bun run tui/cli.tsx` from the repo root (scanners resolve via `./toolchains`); tests `cd tui && bun test`; typecheck `bun run typecheck`
- Release binary: `bun build --compile cli.tsx` → `envexa-tui`; the Go `envexa` binary execs it on no-args + TTY (resolves `ENVEXA_TUI_BIN`, sibling of the executable, then PATH)
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

Rust runtime (until the issue #34 sunset gate removes it):

```bash
cargo build && cargo clippy -- -D warnings && cargo fmt --check
```

Go + Bash runtime (transition, issue #34):

```bash
go build ./... && go vet ./... && go test ./... && gofmt -l cmd internal
shellcheck -S warning toolchains/*.sh toolchains/lib/scan.sh tests/toolchains_test.sh
bash tests/toolchains_test.sh
bash tests/parity.sh   # sunset gate harness (requires cargo; skips verdict with ENVEXA_PARITY_SKIP_RUST=1)
(cd frontend && bun run typecheck && bun run build)
```

Ink TUI (issue #36):

```bash
(cd tui && bun install --frozen-lockfile && bun test && bun run typecheck)
bun build --compile tui/cli.tsx --outfile /tmp/envexa-tui   # packaging smoke
```

CLI output verification — manually run and visually inspect:
1. `cargo run -- --help` and `go run ./cmd/envexa --help` — help text renders correctly
2. `cargo run -- scan` and `go run ./cmd/envexa scan` — full report printed to stdout
3. `cargo run -- update` and `go run ./cmd/envexa update` — update check message
4. `bun run tui/cli.tsx` (repo root, in terminal) — TUI launches, `s` triggers scan, `o` shows outdated, arrows navigate, `q` quits; `go run ./cmd/envexa` execs the built `envexa-tui` when present
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
OUTPUT STYLE: dense
- Each statement = one atomic fact line
- Use abbreviations: fn, cfg, impl, deps, req, res, ctx, err, ret
- Diff lines only (+/-/~), never repeat unchanged code
- Symbols: → (causes), + (adds), − (removes), ~ (modifies), ∴ (therefore)
- No narration, no filler, no hedging
- BUDGET: ≤200 tokens per response unless code block required
<!-- /lean-ctx-compression -->

<!-- lean-ctx -->
## lean-ctx

Prefer lean-ctx MCP tools over native equivalents for token savings.
Full rules: @LEAN-CTX.md
<!-- /lean-ctx -->
