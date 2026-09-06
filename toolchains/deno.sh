#!/usr/bin/env bash
# shellcheck disable=SC2034  # RESULT_* globals are consumed by emit_scan_result in lib/scan.sh
# Port of src/toolchains/deno.rs: deno version only.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="deno"
RESULT_STATUS="ok"

which_cached deno || {
	scan_skipped "deno not installed"
	exit 0
}

RESULT_DENO_VERSION=$(run_cmd deno --version)
emit_scan_result
