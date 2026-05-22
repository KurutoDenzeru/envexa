## 2026-05-22 — macOS CI cost inefficiency
**What happened:** Initial release workflow used 5 CI jobs including 2 on macos-latest, which are the slowest and most expensive runners (billed per-minute even on public repos via macOS runner quotas).
**Root cause:** Assumed all targets should be built in CI without considering local macOS capability.
**Prevention rule:** Build macOS binaries locally (fast, free) and use CI only for Linux. Always evaluate whether each target can be built locally before adding CI jobs.
