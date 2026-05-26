# Releases

## [2.0.0] - 2025-05-26

### Summary
🔄 Major version with breaking changes. See [Migration](#migration-v1--v2).

### ⚠️ Breaking Changes
- Removed deprecated `legacy_mode` flag. Update config files.
- Changed `timeout` default from 30s to 10s.
- Reorganized CLI commands: `--old-format` → `--new-format`.

### ✨ Features
- New streaming support for batch operations
- Built-in request deduplication (configurable)
- Improved error reporting with stack traces

### 🐛 Fixes
- Memory leak in background workers (#456)
- Incorrect handling of edge case inputs (#459)
- Race condition in concurrent operations (#461)

### 📦 Dependencies
| Dependency | Old | New | Required |
|-----------|-----|-----|----------|
| dependency-a | 1.0.x | 2.0.x | Yes |
| dependency-b | 3.x | 4.x | No |

### 🔗 Related
- PR: #480 (main refactor)
- Issue: #450 (original request)
- Docs: [Upgrade Guide](./UPGRADE.md)

### Migration (v1 → v2)

**Step 1: Update configuration**
```
# ❌ Old
legacy_mode: true

# ✅ New
use_new_format: true
timeout: 10
```

**Step 2: Update CLI calls**
```bash
# ❌ Old
tool --old-format input.txt

# ✅ New
tool --new-format input.txt
```

---

## [1.9.2] - 2025-05-19

### Summary
🔧 Performance improvements and bug fixes. No breaking changes.

### 🐛 Fixes
- Reduced memory footprint by 20% (#443)
- Fixed edge case in parsing logic (#448)

### 📈 Performance
- ~15% faster processing on large datasets
- Improved cleanup of temporary resources

---

## [1.9.1] - 2025-05-10

### Summary
🔧 Hotfix for critical production issue in 1.9.0.

### 🐛 Fixes
- Critical: Data corruption in v1.9.0 under specific conditions (#440)
- Verbose logging in normal mode (#442)

---

## [1.9.0] - 2025-05-05

### Summary
✨ New features and improved stability.

### ✨ Features
- Async task support with retry logic
- Better error messages and diagnostics
- New CLI option for batch processing

### ⚠️ Breaking Changes
- Dropped support for legacy format. Minimum requirement: new format only.

### 🐛 Fixes
- Handling of special characters in input (#425)
- Incorrect state cleanup in long-running tasks (#430)

---

## [1.8.0] - 2025-04-15

### Summary
📚 Documentation updates and minor improvements.

### ✨ Features
- New documentation site with examples
- Improved API consistency
- Better validation of user input

### 🐛 Fixes
- Validation warnings in strict mode (#410)

---

## Release Notes Template

Use this for new releases:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Summary
[1-2 lines. Include icon. If breaking: reference migration guide.]

### ⚠️ Breaking Changes
[List only if present. Include migration path with before/after examples.]

### ✨ Features
[Bullet list. Be specific about what changed.]

### 🐛 Fixes
[Link to issues: #123]

### 📈 Performance
[If applicable. Quantify where possible.]

### 📦 Dependencies
[Table if updates required.]

### 🔗 Related
[Links to PRs, docs, issues.]
```

### Emoji System
- ⚠️ Breaking Changes
- ✨ Features
- 🐛 Bug Fixes
- 📈 Performance
- 📚 Documentation
- 🔗 Links/Related
- 🔄 Major Refactor
- 🧹 Cleanup/Chores

### Principles
- Signal-dense: Each line carries weight
- Pragmatic: Include only what matters
- Concise: No narrative; use structure
- Parseable: Clear hierarchy for agents
- Linkable: Real detail lives in PRs/issues, not here
