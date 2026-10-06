## 2026-05-22 — macOS CI cost inefficiency
**What happened:** Initial release workflow used 5 CI jobs including 2 on macos-latest, which are the slowest and most expensive runners (billed per-minute even on public repos via macOS runner quotas).
**Root cause:** Assumed all targets should be built in CI without considering local macOS capability.
**Prevention rule:** Build macOS binaries locally (fast, free) and use CI only for Linux. Always evaluate whether each target can be built locally before adding CI jobs.

## 2026-10-07 — version bump committed without the Cargo.lock sync
**What happened:** Bumped `version` in `Cargo.toml` to 2.13.0, staged only `Cargo.toml`, and tagged and published the release. `Cargo.lock` still recorded `envexa 2.12.0`, so the v2.13.0 tag shipped an inconsistent lockfile.
**Root cause:** `cargo build` had already rewritten `Cargo.lock` in the working tree, but the release step staged a single path instead of the whole diff, so the generated change was left uncommitted and unnoticed.
**Prevention rule:** After any `Cargo.toml` version bump, run `git status` before committing and stage `Cargo.lock` with it. Treat a dirty `Cargo.lock` as a release blocker.
