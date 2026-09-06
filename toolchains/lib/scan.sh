#!/usr/bin/env bash
# Shared scanner library — bash counterpart of src/toolchains/mod.rs.
# Provides which_cached, run_cmd (30s timeout), ScanResult JSON emission with
# the same serde field names and skip-if-empty rules as Rust.

: "${SCAN_TIMEOUT:=30}"

# which_cached mirrors the LazyLock<Mutex<HashMap>> which cache in mod.rs.
# Dynamic-var cache (no assoc arrays) so macOS system bash 3.2 works.
which_cached() {
	local key=$1 varname
	varname="_WHICH_$(printf '%s' "$key" | tr -c 'a-zA-Z0-9' '_')"
	if [[ -n "${!varname+x}" ]]; then
		[[ "${!varname}" == 1 ]]
		return
	fi
	if command -v "$key" >/dev/null 2>&1; then
		eval "$varname=1"
		return 0
	fi
	eval "$varname=0"
	return 1
}

# run_cmd prints stdout even when the command exits nonzero (npm outdated
# exits 1 with output) — same as Rust run_cmd which ignores the exit status.
# Only a timeout yields empty output, like the tokio timeout Err path.
run_cmd() {
	local secs="$SCAN_TIMEOUT"
	local tmp rc
	tmp=$(mktemp) || return 1
	if command -v timeout >/dev/null 2>&1; then
		timeout "$secs" "$@" >"$tmp" 2>/dev/null
	elif command -v gtimeout >/dev/null 2>&1; then
		gtimeout "$secs" "$@" >"$tmp" 2>/dev/null
	else
		# macOS without coreutils: perl alarm survives exec.
		perl -e 'alarm shift; exec @ARGV or exit 127' -- "$secs" "$@" >"$tmp" 2>/dev/null
	fi
	rc=$?
	if ((rc == 124 || rc == 137 || rc == 142)); then
		rm -f "$tmp"
		return 1
	fi
	cat "$tmp"
	rm -f "$tmp"
}

# try_cmd honors the exit status (unlike run_cmd, which mirrors Rust's
# status-ignoring behavior). Prints stdout on success, sets CMD_RC, empty on
# timeout/failure. Used where Rust checked `status.success()` (docker info).
CMD_RC=0
try_cmd() { # <secs> <cmd> [args...]
	CMD_RC=0
	local secs="$1"
	shift
	local tmp
	tmp=$(mktemp) || {
		CMD_RC=1
		return 1
	}
	if command -v timeout >/dev/null 2>&1; then
		timeout "$secs" "$@" >"$tmp" 2>/dev/null
	elif command -v gtimeout >/dev/null 2>&1; then
		gtimeout "$secs" "$@" >"$tmp" 2>/dev/null
	else
		perl -e 'alarm shift; exec @ARGV or exit 127' -- "$secs" "$@" >"$tmp" 2>/dev/null
	fi
	CMD_RC=$?
	((CMD_RC != 0)) && {
		rm -f "$tmp"
		return 1
	}
	cat "$tmp"
	rm -f "$tmp"
}

# get_project_path mirrors mod.rs: project_path from config.json, else cwd.
get_project_path() {
	local cfg="$HOME/.local/share/envexa/config.json" p=""
	[[ -f $cfg ]] && p=$(jq -r '.project_path // empty' "$cfg" 2>/dev/null)
	[[ -n $p ]] && [[ $p != "null" ]] && printf '%s' "$p" || printf '%s' "$PWD"
}

# json_array_pkgs: TSV (name\tcurrent\tlatest) on stdin -> PackageInfo JSON array.
json_array_pkgs() {
	jq -Rsc 'split("\n") | map(select(length > 0) | split("\t")
		| {name: .[0], current: .[1], latest: .[2]})'
}

issues_json() {
	jq -cn --arg m "$1" '[$m]'
}

# emit_scan_result prints the ScanResult JSON from RESULT_* globals.
# Omits version/node_version/installed_count when unset and empty arrays —
# mirrors serde skip_serializing_if in mod.rs.
emit_scan_result() {
	jq -n \
		--arg tool "${RESULT_TOOL-}" \
		--arg status "${RESULT_STATUS-}" \
		--arg version "${RESULT_VERSION-}" \
		--arg node_version "${RESULT_NODE_VERSION-}" \
		--arg python_version "${RESULT_PYTHON_VERSION-}" \
		--arg ruby_version "${RESULT_RUBY_VERSION-}" \
		--arg rustc_version "${RESULT_RUSTC_VERSION-}" \
		--arg cargo_version "${RESULT_CARGO_VERSION-}" \
		--arg pnpm_version "${RESULT_PNPM_VERSION-}" \
		--arg bun_version "${RESULT_BUN_VERSION-}" \
		--arg deno_version "${RESULT_DENO_VERSION-}" \
		--argjson installed_count "${RESULT_INSTALLED_COUNT:-null}" \
		--argjson disk_usage "${RESULT_DISK_USAGE:-null}" \
		--argjson outdated_formulae "${RESULT_OUTDATED_FORMULAE:-[]}" \
		--argjson outdated_casks "${RESULT_OUTDATED_CASKS:-[]}" \
		--argjson outdated "${RESULT_OUTDATED:-[]}" \
		--argjson outdated_global "${RESULT_OUTDATED_GLOBAL:-[]}" \
		--argjson issues "${RESULT_ISSUES:-[]}" \
		'{tool: $tool, status: $status}
		+ (if $version != "" then {version: $version} else {} end)
		+ (if $node_version != "" then {node_version: $node_version} else {} end)
		+ (if $python_version != "" then {python_version: $python_version} else {} end)
		+ (if $ruby_version != "" then {ruby_version: $ruby_version} else {} end)
		+ (if $rustc_version != "" then {rustc_version: $rustc_version} else {} end)
		+ (if $cargo_version != "" then {cargo_version: $cargo_version} else {} end)
		+ (if $pnpm_version != "" then {pnpm_version: $pnpm_version} else {} end)
		+ (if $bun_version != "" then {bun_version: $bun_version} else {} end)
		+ (if $deno_version != "" then {deno_version: $deno_version} else {} end)
		+ (if $installed_count != null then {installed_count: $installed_count} else {} end)
		+ (if $disk_usage != null then {disk_usage: $disk_usage} else {} end)
		+ (if ($outdated_formulae | length) > 0 then {outdated_formulae: $outdated_formulae} else {} end)
		+ (if ($outdated_casks | length) > 0 then {outdated_casks: $outdated_casks} else {} end)
		+ (if ($outdated | length) > 0 then {outdated: $outdated} else {} end)
		+ (if ($outdated_global | length) > 0 then {outdated_global: $outdated_global} else {} end)
		+ (if ($issues | length) > 0 then {issues: $issues} else {} end)'
}

# scan_skipped mirrors ScanResult::skipped: empty tool, status "skipped",
# single issue with the reason.
scan_skipped() {
	RESULT_TOOL=""
	RESULT_STATUS="skipped"
	RESULT_ISSUES=$(issues_json "$1")
	emit_scan_result
}
