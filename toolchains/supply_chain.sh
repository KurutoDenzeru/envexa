#!/usr/bin/env bash
# Port of src/toolchains/supply_chain.rs: flag node_modules packages that run
# install scripts or are marked deprecated.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="supply_chain"
RESULT_STATUS="ok"

project=$(get_project_path)
pkg_json="$project/package.json"

risks=""
if [[ -f $pkg_json ]]; then
	while IFS= read -r pkg; do
		[[ -z $pkg ]] && continue
		mod_json="$project/node_modules/$pkg/package.json"
		[[ -f $mod_json ]] || continue
		if jq -e '(.scripts // {}) | has("postinstall") or has("preinstall") or has("install")' \
			"$mod_json" >/dev/null 2>&1; then
			risks+=$(jq -cn --arg p "$pkg" '{package: $p, risk_type: "Install Script",
				description: "Package executes scripts automatically on install"}')$'\n'
		fi
		if jq -e 'has("deprecated")' "$mod_json" >/dev/null 2>&1; then
			risks+=$(jq -cn --arg p "$pkg" '{package: $p, risk_type: "Deprecated",
				description: "Package is marked as deprecated by its author"}')$'\n'
		fi
	done < <(jq -r '.dependencies // {} | keys[]' "$pkg_json" 2>/dev/null)
fi

RESULT_SUPPLY_CHAIN_RISKS=$(printf '%s' "$risks" | jq -Rsc 'split("\n") | map(select(length > 0) | fromjson)')
n=$(jq 'length' <<<"${RESULT_SUPPLY_CHAIN_RISKS:-[]}")
if ((n > 0)); then
	if ((n <= 3)); then
		RESULT_STATUS="warning"
	else
		RESULT_STATUS="error"
	fi
	RESULT_ISSUES=$(issues_json "$n supply chain risk(s) detected")
fi
emit_scan_result
