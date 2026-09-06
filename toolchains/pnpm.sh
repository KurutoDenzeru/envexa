#!/usr/bin/env bash
# Port of src/toolchains/pnpm.rs: node + pnpm versions only.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="pnpm"
RESULT_STATUS="ok"

which_cached pnpm || {
	scan_skipped "pnpm not installed"
	exit 0
}

RESULT_NODE_VERSION=$(run_cmd node --version)
RESULT_PNPM_VERSION=$(run_cmd pnpm --version)
emit_scan_result
