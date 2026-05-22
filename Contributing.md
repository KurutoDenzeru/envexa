# Contributing

## Development

1. Fork this repository.

2. Clone the repository.

```bash
git clone https://github.com/KurutoDenzeru/envexa.git
```

3. Build the project.

```bash
cargo build
```

4. Run the TUI (no arguments).

```bash
cargo run
```

5. Run a full CLI scan.

```bash
cargo run -- scan
```

## Project Layout

```
src/
├── main.rs           # Entry — no args = TUI, args = CLI
├── app.rs            # App state, event loop, scan dispatch
├── ui.rs             # ratatui render functions
├── cli.rs            # CLI subcommands (scan, update)
├── config.rs         # File-backed cache (~/.envexa/cache.json)
├── scanner.rs        # Scan orchestration + formatting
└── toolchains/
    ├── mod.rs        # ScanResult type + concurrent scan_all()
    ├── brew.rs / npm.rs / pip.rs / gem.rs / cargo.rs / docker.rs
    ├── pnpm.rs / yarn.rs / bun.rs / deno.rs
```

## Commit Convention

Before you create a Pull Request, please check whether your commits comply with
the commit conventions used in this repository.

When you create a commit we kindly ask you to follow the convention
`category: message` in your commit message while using one of
the following categories:

- `feat / feature`: all changes that introduce completely new code or new
  features
- `fix`: changes that fix a bug (ideally you will additionally reference an
  issue if present)
- `refactor`: any code related change that is not a fix nor a feature
- `docs`: changing existing or creating new documentation (i.e. README)
- `build`: all changes regarding the build of the software, changes to
  dependencies or the addition of new dependencies
- `test`: all changes regarding tests (adding new tests or changing existing
  ones)
- `ci`: all changes regarding the configuration of continuous integration (i.e.
  github actions, ci system)
- `chore`: all changes to the repository that do not fit into any of the above
  categories

  e.g. `feat: add new toolchain scanner for pnpm`

If you are interested in the detailed specification you can visit
https://www.conventionalcommits.org/ or check out the
[Angular Commit Message Guidelines](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines).

## Testing

Build and test the full scan:

```bash
cargo build && cargo run -- scan > /dev/null && echo "PASS"
```

Run clippy and format checks:

```bash
cargo clippy -- -D warnings && cargo fmt --check
```

Please ensure the project compiles, passes clippy, and runs without errors when submitting a pull request.

## Release (maintainers)

Envexa is macOS-only and built locally (no CI). Release process:

```bash
# 1. Bump version in Cargo.toml, commit
# 2. Tag and push
git tag vX.Y.Z && git push origin vX.Y.Z

# 3. Create GitHub release
gh release create vX.Y.Z --title "vX.Y.Z" --notes "## What Changed

..."

# 4. Build both architectures + upload
scripts/build-and-upload.sh vX.Y.Z

# 5. Verify
gh release view vX.Y.Z --json assets --jq '.assets[].name'

# 6. Clean up build artifacts (reclaim ~2 GB)
cargo clean
```

Builds two binaries: `envexa-aarch64-macos` (Apple Silicon) and `envexa-x86_64-macos` (Intel).
