#!/usr/bin/env bash
# Contract tests for toolchains/*.sh — run: bash tests/toolchains_test.sh
# Passes on any host: scanners may return status "skipped".
set -u
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fail() { echo "FAIL: $*" >&2; exit 1; }

valid_result() { # <json> <expected_tool>
	jq -e --arg tool "$2" '(.tool == $tool or ($tool == "" and .tool == ""))
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

# --- run_cmd timeout: SCAN_TIMEOUT must kill the command ---------------------
# shellcheck source=../toolchains/lib/scan.sh
source "$ROOT/toolchains/lib/scan.sh"
SCAN_TIMEOUT=1
out=$(run_cmd sleep 5) && fail "run_cmd should fail on timeout"
[[ -z $out ]] || fail "run_cmd should emit nothing on timeout"
unset SCAN_TIMEOUT

# --- real path: whatever tools exist, output stays contract-valid ------------
out=$(bash "$ROOT/toolchains/npm.sh") || fail "npm.sh (real PATH) exited nonzero"
valid_result "$out" "npm"

out=$(bash "$ROOT/toolchains/brew.sh") || fail "brew.sh (real PATH) exited nonzero"
valid_result "$out" "brew"

# field names must match the Rust serde contract when present
jq -e 'keys | all(IN("tool", "status", "version", "node_version", "installed_count",
	"outdated_formulae", "outdated_casks", "outdated", "outdated_global", "issues"))' <<<"$out" >/dev/null ||
	fail "brew.sh emitted unknown fields: $out"

rm -rf "$fakebin"
echo "toolchains contract tests passed"
