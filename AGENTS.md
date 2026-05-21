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
| `cargo run` | Run MCP server |
| `cargo fmt` | Format all code |
| `cargo clippy` | Lint check |
| `cargo test` | Run tests |

Quick test (brew scan):
```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize"...' | cargo run | python3 -c "..."
```

---

## Rust & MCP Conventions

- `serde::{Serialize, Deserialize}` derive for all protocol types
- `anyhow::Result` for fallible functions; no unwrap/expect in production
- `tokio::process::Command` + `tokio::time::timeout` for async CLI
- Graceful missing toolchains — check `which()` before calling CLI
- One `pub async fn scan() -> ScanResult` per toolchain module
- Toolchains under `toolchains/` with `mod.rs` dispatch
- `tokio::join!` for concurrent scanners
- `String` over `&str` in struct fields
- `serde_json::Value` for toolchain-specific fields
- No obvious comments — explain *why*, not *what*
- Conventional commits: `type(scope): description`
- One logical change per commit, no `--no-verify`, no force push

**Allowed types:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`, `style`, `perf`

---

## Verification

Before marking done, run:

```bash
cargo build && printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"all"}}}\n' | cargo run > /dev/null && echo "PASS"
```

Then confirm:
- No crashes on missing CLI tools
- New scanners follow `pub async fn scan() -> ScanResult`
- New tools registered in `server.rs` and README
- Changed report format → show sample
- New dependency → call out in summary

---

## MCP Tools

- Registered in `server.rs` as `ToolDescription` with name, description, input_schema
- Descriptions start with `"Envexa — "`
- Return `Value::String(markdown)`
- Handlers under ~50 LOC; compose via `scanner.rs`
- `block_in_place` + `Handle::current().block_on` for async sync calls
- `/` commands: `/scan`, `/outdated`, `/status`, `/upgrade`, `/report`, `/help`

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
