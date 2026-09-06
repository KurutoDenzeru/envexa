#!/usr/bin/env bash
# Port of src/toolchains/npm.rs: node/npm versions + npm outdated -g --json.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="npm"
RESULT_STATUS="ok"

which_cached node || {
	scan_skipped "Node.js not installed"
	exit 0
}

if which_cached npm; then
	RESULT_NODE_VERSION=$(run_cmd node --version)
	RESULT_VERSION=$(run_cmd npm --version)
	out=$(run_cmd npm outdated -g --json 2>/dev/null)
	if [[ -n $out ]]; then
		RESULT_OUTDATED_GLOBAL=$(printf '%s' "$out" | jq -r 'to_entries[]
			| "\(.key)\t\(.value.current // "?")\t\(.value.latest // "?")"' 2>/dev/null |
			json_array_pkgs) || RESULT_OUTDATED_GLOBAL="[]"
	fi
else
	RESULT_NODE_VERSION=$(run_cmd node --version)
fi

if (( $(jq 'length' <<<"${RESULT_OUTDATED_GLOBAL:-[]}") > 0 )); then
	RESULT_STATUS="warning"
fi
emit_scan_result
