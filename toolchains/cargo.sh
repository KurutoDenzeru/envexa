#!/usr/bin/env bash
# Port of src/toolchains/cargo.rs: rustc/cargo versions + cargo-outdated in
# the configured project path.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="cargo"
RESULT_STATUS="ok"

which_cached rustc || {
	scan_skipped "Rust not installed"
	exit 0
}

RESULT_RUSTC_VERSION=$(run_cmd rustc --version)

if which_cached cargo; then
	RESULT_CARGO_VERSION=$(run_cmd cargo --version)
fi

if ! which_cached cargo-outdated; then
	RESULT_ISSUES=$(issues_json "cargo-outdated not installed (run: cargo install cargo-outdated)")
	emit_scan_result
	exit 0
fi

project=$(get_project_path)
out=$(cd "$project" 2>/dev/null && run_cmd cargo-outdated --format=json 2>/dev/null)
if [[ -n $out ]]; then
	RESULT_OUTDATED=$(printf '%s' "$out" | jq -c '[.dependencies[]?
		| {name: .name, current: (.project_version // "?"), latest: (.latest_version // "?")}]') || RESULT_OUTDATED="[]"
fi

if (( $(jq 'length' <<<"${RESULT_OUTDATED:-[]}") > 0 )); then
	RESULT_STATUS="warning"
fi
emit_scan_result
