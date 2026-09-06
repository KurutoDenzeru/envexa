#!/usr/bin/env bash
# Contract tests for toolchains/*.sh — run: bash tests/toolchains_test.sh
# Passes on any host: scanners may return status "skipped".
set -u
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fail() { echo "FAIL: $*" >&2; exit 1; }

valid_result() { # <json> <expected_tool>
	jq -e --arg tool "$2" '(.tool == $tool)
		or ((.tool == "") and .status == "skipped")
		and (.status | IN("ok", "warning", "warn", "error", "skipped"))' <<<"$1" >/dev/null ||
		fail "invalid ScanResult JSON: $1"
}

# --- skipped path: PATH without node/brew forces which() miss ---------------
fakebin=$(mktemp -d)
ln -s "$(command -v jq)" "$fakebin/jq"
stripped_path="$fakebin:/usr/bin:/bin"

out=$(PATH="$stripped_path" bash "$ROOT/toolchains/npm.sh") || fail "npm.sh exited nonzero"
valid_result "$out" ""
[[ $(jq -r '.status' <<<"$out") == "skipped" ]] || fail "npm.sh should skip without node: $out"
[[ $(jq -r '.issues[0]' <<<"$out") == "Node.js not installed" ]] || fail "npm.sh skip reason: $out"

out=$(PATH="$stripped_path" bash "$ROOT/toolchains/brew.sh") || fail "brew.sh exited nonzero"
[[ $(jq -r '.status' <<<"$out") == "skipped" ]] || fail "brew.sh should skip without brew: $out"
[[ $(jq -r '.issues[0]' <<<"$out") == "Homebrew not installed" ]] || fail "brew.sh skip reason: $out"

for t in pnpm yarn bun deno; do
	out=$(PATH="$stripped_path" bash "$ROOT/toolchains/$t.sh") || fail "$t.sh exited nonzero"
	[[ $(jq -r '.status' <<<"$out") == "skipped" ]] || fail "$t.sh should skip: $out"
	[[ $(jq -r '.issues[0]' <<<"$out") == "$t not installed" ]] || fail "$t.sh skip reason: $out"
done

# --- run_cmd timeout: SCAN_TIMEOUT must kill the command ---------------------
# shellcheck source=../toolchains/lib/scan.sh
source "$ROOT/toolchains/lib/scan.sh"
# shellcheck disable=SC2034  # read by run_cmd via sourced lib
SCAN_TIMEOUT=1
out=$(run_cmd sleep 5) && fail "run_cmd should fail on timeout"
[[ -z $out ]] || fail "run_cmd should emit nothing on timeout"
unset SCAN_TIMEOUT

# --- real path: whatever tools exist, output stays contract-valid ------------
for t in npm pnpm yarn bun deno pip gem cargo docker project security audit ci supply_chain; do
	out=$(bash "$ROOT/toolchains/$t.sh") || fail "$t.sh (real PATH) exited nonzero"
	valid_result "$out" "$t"
done

out=$(bash "$ROOT/toolchains/brew.sh") || fail "brew.sh (real PATH) exited nonzero"
valid_result "$out" "brew"

# field names must match the Rust serde contract when present
jq -e 'keys | all(IN("tool", "status", "version", "node_version", "python_version",
	"ruby_version", "rustc_version", "cargo_version", "pnpm_version", "bun_version",
	"deno_version", "installed_count", "outdated_formulae", "outdated_casks",
	"outdated", "outdated_global", "issues", "disk_usage", "project_type",
	"vulnerabilities", "supply_chain_risks", "audit_items"))' <<<"$out" >/dev/null ||
	fail "brew.sh emitted unknown fields: $out"

# version-field wiring: pnpm/bun/deno use their own serde fields, not version
[[ $(jq -r 'has("pnpm_version")' <<<"$(bash "$ROOT/toolchains/pnpm.sh")") == "$(command -v pnpm >/dev/null && echo true || echo false)" ]] ||
	fail "pnpm.sh pnpm_version field wrong"

rm -rf "$fakebin"
echo "toolchains contract tests passed"
