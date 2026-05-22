# Handoff — CI Workflow Deferred

## What & Why

The `.github/workflows/ci.yml` was hindering local workflow — CI would trigger on every push to `main`, but all testing is done locally during active development.

## Action Taken

- **Original:** `.github/workflows/ci.yml` (removed from active workflows)
- **Backup:** `.github/workflows/ci.yml.bak` (preserved for future reference)

## To Re-enable

When ready to activate CI:

```bash
mv .github/workflows/ci.yml.bak .github/workflows/ci.yml
git add .github/workflows/ci.yml
git commit -m "ci: re-enable GitHub Actions workflow"
git push
```

## What the Workflow Does

The backed-up workflow (`ci.yml.bak`) runs on push/PR to `main`:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo build --release`
