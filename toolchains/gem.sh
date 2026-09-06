#!/usr/bin/env bash
# Port of src/toolchains/gem.rs: ruby version + gem outdated line parsing.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="gem"
RESULT_STATUS="ok"

which_cached ruby || {
	scan_skipped "Ruby not installed"
	exit 0
}

if ! which_cached gem; then
	RESULT_RUBY_VERSION=$(run_cmd ruby --version)
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(issues_json "gem CLI not found")
	emit_scan_result
	exit 0
fi

RESULT_RUBY_VERSION=$(run_cmd ruby --version)
out=$(run_cmd gem outdated 2>/dev/null)
if [[ -n $out ]]; then
	# gem outdated lines: "name (current < latest)" — mirrors gem_outdated_re;
	# missing latest falls back to "?" like the Rust capture.
	tsv=$(printf '%s\n' "$out" | sed -n -E 's/^([^[:space:]]+)[[:space:]]+\(([^[:space:]]+)[[:space:]]*([<>]?[[:space:]]*)?([^)]*)\)/\1\t\2\t\4/p')
	pkgs=""
	while IFS=$'\t' read -r name cur latest; do
		[[ -z $name ]] && continue
		[[ -z $latest ]] && latest="?"
		pkgs+="$name"$'\t'"$cur"$'\t'"$latest"$'\n'
	done <<<"$tsv"
	RESULT_OUTDATED=$(printf '%s' "$pkgs" | json_array_pkgs)
fi

n=$(jq 'length' <<<"${RESULT_OUTDATED:-[]}")
if ((n > 0)); then
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(issues_json "$n outdated gem(s)")
fi
emit_scan_result
