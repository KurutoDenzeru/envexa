# AGENTS.md

> You are a focused, senior-level coding assistant. Explain your reasoning briefly before making non-trivial changes. Never modify critical files without explicit instruction. Prefer small, reviewable diffs over sweeping rewrites.

---

## How to approach a task

1. **Plan before you act.** Read the relevant files, understand the existing patterns, and outline your approach before writing any code. For non-trivial tasks, state your plan and wait for confirmation.
2. **Ask, don't assume.** If the task is ambiguous — unclear scope, missing context, or conflicting requirements — stop and ask a single, specific question rather than guessing and proceeding.
3. **Prefer the smallest correct change.** Resist the urge to refactor adjacent code. Do exactly what was asked, and flag (but don't act on) anything else you notice.
4. **Clean up after yourself.** Once the task is complete and verified, scan your own changes for anything that can be made cleaner, shorter, or clearer without changing behavior. Remove dead code, redundant comments, and unnecessary complexity before calling it done.

---

## Setup commands

- Build: `cargo build`
- Build optimized: `cargo build --release`
- Run MCP server: `cargo run`
- Quick test (brew scan): `printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"brew"}}}\n' | cargo run | python3 -c "import sys,json; [print(json.loads(l)['result']['content'][0]['text']) for l in sys.stdin if json.loads(l).get('result',{}).get('content')]"`

---

## Code style

Follows standard Rust conventions (`cargo fmt` + `cargo clippy`). Write clean, idiomatic Rust:

- Use `serde::{Serialize, Deserialize}` derive for all protocol types
- Use `anyhow::Result` for fallible functions; avoid unwrap/expect in production paths
- Use `tokio::process::Command` with `tokio::time::timeout` for async CLI execution
- Handle missing toolchains gracefully — check `which()` before calling CLI tools
- Keep toolchain scanners lean: one `pub async fn scan() -> ScanResult` per module
- All scanner modules live under `toolchains/` with `mod.rs` handling dispatch
- Use `tokio::join!` to run independent scanners concurrently
- Prefer `String` over `&str` in struct fields for owned data
- Use `serde_json::Value` for toolchain-specific fields that vary by scanner
- No comments on obvious code — explain *why*, not *what*

---

## Verification (before marking a task done)

Before calling any task done, run this command — do not mark it done if it exits with errors:

```bash
cargo build && printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"envexa_scan","arguments":{"chain":"all"}}}\n' | cargo run > /dev/null && echo "PASS"
```

Then confirm:

- No toolchain scanner crashes on missing CLI tools (brew/npm/pip/gem/cargo/docker)
- New scanner modules follow the `pub async fn scan() -> ScanResult` pattern
- New tools are registered in both `server.rs` and the README MCP Tools table
- If you changed the report format: show a sample of the new output
- If you added a dependency to `Cargo.toml`: call it out explicitly in your summary

---

## Slash commands

When the user types a `/` command in chat (e.g. `/scan`, `/status`), use the `envexa_cmd` tool to handle it. Do NOT call individual scan/outdated tools unless the user explicitly asks.

Available via `envexa_cmd`:
- `/scan [chain]` → full health scan
- `/outdated [chain]` → outdated packages only
- `/status` → quick dashboard
- `/upgrade <tool>` → upgrade a toolchain
- `/report` → latest cached report
- `/help` → show all commands

The MCP prompts (`/envexa:scan`, `/envexa:status`, `/envexa:outdated`) also appear in the `/` menu as an alternative.

## MCP tool patterns

- Each tool is registered in `server.rs` as a `ToolDescription` with name, description, and input_schema
- All tool descriptions start with "Envexa — "
- Tools return `Value::String(markdown)` — human-readable markdown
- Keep handler implementations under ~50 LOC; compose by calling `scanner.rs` helpers
- Use `tokio::task::block_in_place` + `Handle::current().block_on` inside tool handlers to run async scan code synchronously
- New single-toolchain scanners (e.g. `envexa_brew_status`) follow the `scan_single()` pattern

---

## Commit rules

- Format: `type(scope): description` (conventional commits)
- One logical change per commit
- No `--no-verify` flag
- No force push

**Allowed types:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`, `style`, `perf`

---

## Scope restrictions

- **Never** force push
- **Never** delete branches without confirmation
- **Never** modify files outside the project directory
- **Never** touch `Cargo.toml`, `Cargo.lock`, CI/workflow files, or `.gitignore` without explicit instruction

---

## Recurring errors log

When you make a mistake or are corrected by the developer, **do not edit this file**. Instead:

1. Offer to append the mistake to `ERRORS.md` before moving on (create it if it doesn't exist)
2. Use this format:

```md
## YYYY-MM-DD — <short title>
**What happened:** Describe the mistake or unexpected behavior.
**Root cause:** Why it happened.
**Prevention rule:** What to do differently next time.
```

`ERRORS.md` is committed to git and reviewed periodically to promote entries into permanent rules in this file or the linter config.
