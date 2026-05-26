# Contributing

Thanks for improving Envexa. Keep changes small, focused, and easy to review.

## Setup

```bash
git clone https://github.com/KurutoDenzeru/envexa.git
cd envexa
cargo build
```

Run the app:

```bash
cargo run          # interactive TUI
cargo run -- scan  # CLI report
cargo run -- --help
```

## Project Layout

```text
src/
├── main.rs             # no args = TUI, args = CLI
├── cli.rs              # CLI subcommands
├── config.rs           # cache/config persistence
├── scanner/
│   └── mod.rs          # report formatting and extract helpers
├── tui/
│   ├── app.rs          # state, event loop, scan/update actions
│   └── ui.rs           # ratatui render functions
└── toolchains/
    ├── mod.rs          # ScanResult + scan_all()
    ├── brew.rs / npm.rs / pnpm.rs / yarn.rs / bun.rs / deno.rs
    ├── pip.rs / gem.rs / cargo.rs / docker.rs
    └── project.rs / security.rs / audit.rs / cleanup.rs
```

## Development Rules

- Use `anyhow::Result` for fallible Rust functions.
- Do not use `unwrap` or `expect` in production scanner paths.
- Check missing CLI tools before calling them.
- Add one `pub async fn scan() -> ScanResult` per scanner module.
- Keep `ScanResult` fields serializable with `serde`.
- Keep TUI state in `tui/app.rs` and rendering in `tui/ui.rs`.
- Prefer built-in `ratatui` widgets first; add third-party widgets only when they improve readability or interaction.
- For Project Tooling UI, keep Project, Security, Audit, and Cleanup signals easy to compare.

## TUI Changes

When changing the TUI, verify:

- Dashboard opens with cached data.
- `S` refreshes scan data.
- `O` opens outdated packages.
- Arrow keys navigate rows.
- `/` filters the current view.
- `Enter` opens detail rows with data.
- Read-only details do not show update-only shortcuts.
- `Q` exits cleanly.
- Narrow, medium, wide, and tiny terminal sizes do not panic or leave the terminal in alternate screen mode.

For visual widgets, prefer dense monitoring patterns: gauges for readiness, barcharts for signal distribution, tables for actionable rows, and focused color for status/severity.

## Commit Convention

Use conventional commits:

```text
type(scope): description
```

Allowed types:

- `feat`
- `fix`
- `chore`
- `docs`
- `refactor`
- `test`
- `ci`
- `style`
- `perf`

Examples:

```text
feat(tui): add project tooling readiness panel
docs(readme): refresh usage guide
```

## Verification

Before opening a pull request:

```bash
cargo build
cargo clippy -- -D warnings
cargo fmt --check
```

Also run:

```bash
cargo run -- --help
cargo run -- scan
cargo run -- update
cargo run
```

Do not submit warnings, malformed CLI output, or broken TUI navigation.

## Pull Requests

- Use `.github/PULL_REQUEST_TEMPLATE.md`.
- Link related issues when available.
- Include verification commands and results.
- Note any new dependency.
- Add screenshots or recordings for substantial TUI changes when practical.

## Release (maintainers)

Envexa is macOS-only and built locally. Refer to [.github/ISSUE_TEMPLATE/RELEASES.md](.github/ISSUE_TEMPLATE/RELEASES.md) for the release checklist, standard templates, and log format.

```bash
cargo clean
# bump version in Cargo.toml, commit
git tag vX.Y.Z && git push origin vX.Y.Z
gh release create vX.Y.Z --title "vX.Y.Z" --notes "..."
scripts/build-and-upload.sh vX.Y.Z
gh release view vX.Y.Z --json assets --jq '.assets[].name'
cargo clean
```

Release assets:

- `envexa-aarch64-macos`
- `envexa-x86_64-macos`
