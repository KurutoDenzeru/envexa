## 2026-05-22 — macOS CI cost inefficiency
**What happened:** Initial release workflow used 5 CI jobs including 2 on macos-latest, which are the slowest and most expensive runners (billed per-minute even on public repos via macOS runner quotas).
**Root cause:** Assumed all targets should be built in CI without considering local macOS capability.
**Prevention rule:** Build macOS binaries locally (fast, free) and use CI only for Linux. Always evaluate whether each target can be built locally before adding CI jobs.
## 2026-09-13 — bun add from a dir without package.json
**What happened:** Running `bun add ink` inside `tui/` before `tui/package.json` existed resolved upward to the root `envexa-root` package.json, modifying it and creating a root `bun.lock`/`node_modules`.
**Root cause:** Bun always walks up to the nearest package.json; I assumed it would create one in the cwd.
**Prevention rule:** Create the package.json first, then `bun add`; verify `git status` after dependency installs to catch parent-manifest pollution.

## 2026-09-13 — git commit --amend landed on the wrong commit
**What happened:** Fixing `scripts/build-and-upload.sh` after five sequential commits, `git commit --amend` folded the fix into the docs commit (HEAD) instead of the packaging commit.
**Root cause:** Amending without checking which commit is HEAD.
**Prevention rule:** Never amend more than one commit after the target; either fix forward or verify `git log -1` is the intended commit first.
