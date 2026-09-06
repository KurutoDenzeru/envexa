#!/usr/bin/env bash
# shellcheck disable=SC2034  # RESULT_* globals are consumed by emit_scan_result in lib/scan.sh
# Port of src/toolchains/docker.rs: version + 10s-capped daemon info probe.
# Unlike the Rust run_cmd path, docker info checks status.success() — try_cmd
# preserves that.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="docker"
RESULT_STATUS="ok"

which_cached docker || {
	scan_skipped "Docker not installed"
	exit 0
}

RESULT_VERSION=$(run_cmd docker --version)
info=$(try_cmd 10 docker info --format '{{json .}}')
if ((CMD_RC != 0)); then
	RESULT_STATUS="error"
	RESULT_ISSUES=$(issues_json "Docker daemon not running")
	emit_scan_result
	exit 0
fi

driver=$(printf '%s' "$info" | jq -r '.Driver // empty' 2>/dev/null)
if [[ -n $driver ]]; then
	RESULT_DISK_USAGE=$(jq -cn --arg d "$driver" '{driver: $d}')
fi
emit_scan_result
