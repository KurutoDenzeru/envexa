#!/usr/bin/env bash
# Port of src/toolchains/pip.rs: python/pip versions + pip3 list --outdated.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="pip"
RESULT_STATUS="ok"

which_cached python3 || {
	scan_skipped "Python not installed"
	exit 0
}

if which_cached pip3; then
	RESULT_PYTHON_VERSION=$(run_cmd python3 --version)
	RESULT_VERSION=$(run_cmd pip3 --version)
	out=$(run_cmd pip3 list --outdated --format=json 2>/dev/null)
	if [[ -n $out ]]; then
		RESULT_OUTDATED=$(printf '%s' "$out" | jq -c '[.[]?
			| {name: .name, current: .version, latest: .latest_version}]') || RESULT_OUTDATED="[]"
	fi
else
	RESULT_PYTHON_VERSION=$(run_cmd python3 --version)
fi

if (( $(jq 'length' <<<"${RESULT_OUTDATED:-[]}") > 0 )); then
	RESULT_STATUS="warning"
fi
emit_scan_result
