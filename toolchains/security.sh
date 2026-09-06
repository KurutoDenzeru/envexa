#!/usr/bin/env bash
# shellcheck disable=SC2034  # RESULT_* globals are consumed by emit_scan_result in lib/scan.sh
# Port of src/toolchains/security.rs: aggregate vulnerabilities from npm, pnpm,
# bun, cargo, pip, go, and composer auditors. Each runs only when its lockfile
# and CLI exist, matching the Rust gates.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="security"
RESULT_STATUS="ok"

project=$(get_project_path)
vulns=""

# --- npm/pnpm: shared shape (security.rs parse_vuln) -------------------------
npm_style_audit() { # <cmd> <json-out>
	printf '%s' "$2" | jq -c '.vulnerabilities // {} | to_entries[] | .key as $name | .value as $info |
	{
		package: $name,
		severity: (($info.severity // "unknown") | ascii_upcase),
		title: (($info.via[0] | objects | .title) // "unknown"),
		cve: (([$info.via[0] | objects | .cve? | strings]
			+ [$info.via[]? | strings | select(startswith("CVE-"))]) | first),
		patched_version: (if ($info.fixAvailable | type) == "object"
			then ($info.fixAvailable.version // "?") else "?" end),
		dependency_path: (if ($info.nodes | type) == "array" and ($info.nodes | length) > 0
			and ($info.nodes[0] | type) == "string"
			then [$info.nodes[0] | split("node_modules/")[] | select(length > 0) | sub("/$"; "")]
			else [] end)
	}' 2>/dev/null
}

if which_cached npm &&
	[[ -f "$project/package-lock.json" || -f "$project/package.json" ]]; then
	out=$(run_cmd_in "$project" npm audit --json 2>/dev/null)
	vulns+=$(npm_style_audit npm "$out")
fi

if which_cached pnpm &&
	{ [[ -f "$project/pnpm-lock.yaml" ]] || [[ -f "$project/pnpm-lock.yml" ]]; }; then
	out=$(run_cmd_in "$project" pnpm audit --json 2>/dev/null)
	vulns+=$(npm_style_audit pnpm "$out")
fi

# --- bun: new JSON format (map pkg -> vuln array); text fallback -------------
if which_cached bun &&
	{ [[ -f "$project/bun.lockb" ]] || [[ -f "$project/bun.lock" ]]; }; then
	out=$(run_cmd_in "$project" bun audit --json 2>/dev/null)
	if printf '%s' "$out" | jq -e 'type == "object"' >/dev/null 2>&1; then
		vulns+=$(printf '%s' "$out" | jq -c 'to_entries[] | .key as $pkg | .value[] |
		{
			package: $pkg,
			severity: ((.severity // "unknown") | ascii_upcase),
			title: (.title // "?"),
			cve: ([((.cve // [])[0]) | strings][0]),
			patched_version: "",
			dependency_path: []
		}' 2>/dev/null)
	else
		# Text table: rows after the "Package ... Severity" header.
		vulns+=$(printf '%s\n' "$out" | awk '
			!in_section { if (index($0, "Package") && index($0, "Severity")) in_section = 1; next }
			index($0, "└") || $0 == "" { next }
			{
				line = $0
				sub(/^[[:space:]]+/, "", line)
				n = split(line, parts, " ")
				if (n >= 3 && parts[1] != "")
					print parts[1] "\t" toupper(parts[2]) "\t" parts[3]
			}' 2>/dev/null | jq -Rsc 'split("\n") | map(select(length > 0)
				| split("\t") | {package: .[0], severity: .[1], title: .[2],
					cve: null, patched_version: "", dependency_path: []})')
	fi
fi

# --- cargo-audit --------------------------------------------------------------
if which_cached cargo-audit &&
	{ [[ -f "$project/Cargo.toml" ]] || [[ -f "$project/Cargo.lock" ]]; }; then
	out=$(run_cmd_in "$project" cargo-audit audit --json 2>/dev/null)
	vulns+=$(printf '%s' "$out" | jq -c '.vulnerabilities.list[]? |
	{
		package: (.package.name // "?"),
		severity: ((.advisory.severity // "unknown") | ascii_upcase),
		title: (.advisory.title // "?"),
		cve: ([.advisory.aliases[]? | strings][0]),
		patched_version: (.advisory.patched_versions // "?"),
		dependency_path: []
	}' 2>/dev/null)
fi

# --- pip-audit ----------------------------------------------------------------
if which_cached pip-audit && {
	[[ -f "$project/requirements.txt" ]] || [[ -f "$project/Pipfile" ]] ||
		[[ -f "$project/Pipfile.lock" ]] || [[ -f "$project/poetry.lock" ]]
}; then
	out=$(run_cmd_in "$project" pip-audit --format json --desc --no-deps 2>/dev/null)
	vulns+=$(printf '%s' "$out" | jq -c '.dependencies[]? | .name as $pkg | .vulns[]? |
	{
		package: $pkg,
		severity: ((.severity // "unknown") | ascii_upcase),
		title: (.description // .id // "?"),
		cve: ([.aliases[]? | strings][0]),
		patched_version: (.fixed_version // "?"),
		dependency_path: []
	}' 2>/dev/null)
fi

# --- govulncheck (NDJSON) -------------------------------------------------------
if which_cached govulncheck && [[ -f "$project/go.mod" ]]; then
	out=$(run_cmd_in "$project" govulncheck -json ./... 2>/dev/null)
	vulns+=$(printf '%s\n' "$out" | jq -c 'select(.osv != null) | .osv |
	{
		package: (.id // "?"),
		severity: "HIGH",
		title: (.details // "?"),
		cve: ([(.aliases // [])[] | strings | select(startswith("CVE-"))] | first),
		patched_version: "?",
		dependency_path: []
	}' 2>/dev/null)
fi

# --- composer audit -------------------------------------------------------------
if which_cached composer && [[ -f "$project/composer.json" ]]; then
	out=$(run_cmd_in "$project" composer audit --format=json 2>/dev/null)
	vulns+=$(printf '%s' "$out" | jq -c '.advisories // {} | to_entries[] | .key as $pkg | .value[]? |
	{
		package: $pkg,
		severity: "HIGH",
		title: (.title // "?"),
		cve: ([.cve | strings][0]),
		patched_version: "?",
		dependency_path: []
	}' 2>/dev/null)
fi

# Empty dependency_path is omitted (serde skip_serializing_if on Vec::is_empty).
RESULT_VULNERABILITIES=$(printf '%s' "$vulns" | jq -Rsc 'split("\n")
	| map(select(length > 0) | fromjson)
	| map(if (.dependency_path // []) == [] then del(.dependency_path) else . end)')
n=$(jq 'length' <<<"${RESULT_VULNERABILITIES:-[]}")
if ((n > 0)); then
	if ((n <= 3)); then
		RESULT_STATUS="warning"
	else
		RESULT_STATUS="error"
	fi
	RESULT_ISSUES=$(issues_json "$n vulnerability(ies) found")
fi
emit_scan_result
