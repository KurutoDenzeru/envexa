## Description
This PR introduces the **Envexa v2 Productivity Engine**, transforming the tool into a comprehensive, proactive developer environment scanner. It brings programmable ignore rules, extensive environment manager and language ecosystem support, automated TUI remediation for security, a new git hygiene scanner, and a background daemon that runs scans automatically and notifies the user via native macOS notifications.

## Related Issue
<!-- Link to the related issue(s) e.g., Fixes #123, Closes #456 -->

## Type of Change
- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [x] ✨ New feature (non-breaking change that adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to change)
- [ ] 📝 Documentation update
- [ ] 🎨 Style/UI update
- [ ] ♻️ Refactor (no functional changes)
- [ ] 🧪 Test update
- [ ] 🔧 Configuration change

## Changes Made
- **Programmable Rules Engine & JSON output:** Added `--format json` support in the CLI and parsing logic for `.envexaignore` to exclude specific packages, toolchains, or CVEs globally across the repository.
- **Expanded Ecosystems & Environment Managers:** Added ecosystem checks for Go (`govulncheck`), Java/JVM (Gradle/Maven), and PHP (`composer`). Implemented detection and validation for `nvm`, `fnm`, `pyenv`, `asdf`, and `mise` against system config files (`.node-version`, `.nvmrc`, `.python-version`, `.tool-versions`, `mise.toml`).
- **Git Hygiene Scanner:** Built a new `git.rs` toolchain scanner that catches branches stale for >30 days, lingering uncommitted changes, and overly large `.git` directory sizes.
- **Automated TUI Remediation:** Mapped the `F` key inside the TUI Security detail view to attempt automated fixes like `npm audit fix` or `cargo update`, displaying a spinner while resolving.
- **Background Daemon:** Implemented `envexa daemon --interval <seconds>` to periodically scan in the background and dispatch native macOS notifications for any detected outdated packages or health issues.

## Screenshots/Recordings
<!-- If applicable, add screenshots or recordings to demonstrate the changes -->

| Before | After |
|--------|-------|
| N/A | (Add screenshots of the daemon macOS notification and the new TUI 'F' fix view here) |

## Testing
- [x] I have tested this locally
- [ ] I have added/updated unit tests
- [ ] I have added/updated integration tests

## Checklist
- [x] My code follows the project's coding standards
- [x] I have performed a self-review of my code
- [ ] I have commented my code where necessary
- [ ] I have updated the documentation accordingly
- [x] My changes generate no new warnings or errors
- [ ] I have checked for accessibility compliance
- [ ] I have verified responsive design (if applicable)

## Additional Notes
All of these features successfully pass `cargo clippy -- -D warnings` and have been merged onto `feat/v2-productivity`. To test the background daemon locally, use `cargo run -- daemon --interval 3600`.
