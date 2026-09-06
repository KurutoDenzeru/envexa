#!/usr/bin/env bash
# shellcheck disable=SC2034  # RESULT_* globals are consumed by emit_scan_result in lib/scan.sh
# Port of src/toolchains/yarn.rs: yarn version only.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="yarn"
RESULT_STATUS="ok"

which_cached yarn || {
	scan_skipped "yarn not installed"
	exit 0
}

RESULT_VERSION=$(run_cmd yarn --version)
emit_scan_result
