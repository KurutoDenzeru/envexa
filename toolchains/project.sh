#!/usr/bin/env bash
# shellcheck disable=SC2034  # RESULT_* globals are consumed by emit_scan_result in lib/scan.sh
# Port of src/toolchains/project.rs: detect project type from lockfiles, then
# gather outdated packages with the matching package manager.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="project"
RESULT_STATUS="ok"

project=$(get_project_path)

exists() { [[ -e "$project/$1" ]]; }

# detect_project_type — same precedence as project.rs.
if exists pnpm-lock.yaml || exists pnpm-lock.yml; then
	pm="pnpm"
elif exists yarn.lock; then
	pm="yarn"
elif exists bun.lockb || exists bun.lock; then
	pm="bun"
elif exists deno.json || exists deno.jsonc; then
	pm="deno"
elif exists package.json; then
	pm="npm"
elif exists Cargo.toml; then
	pm="cargo"
elif exists poetry.lock; then
	pm="poetry"
elif exists Pipfile || exists Pipfile.lock; then
	pm="pipenv"
elif exists requirements.txt; then
	pm="requirements"
elif exists go.mod; then
	pm="go"
elif exists build.gradle || exists build.gradle.kts; then
	pm="gradle"
elif exists pom.xml; then
	pm="maven"
elif exists composer.json; then
	pm="composer"
else
	scan_skipped "no project lockfile found in current directory"
	exit 0
fi

RESULT_PROJECT_TYPE=$pm

# npm-style: JSON map {pkg: {current, latest}} -> PackageInfo TSV. Rust skips
# empty/non-{ output and the synthetic "error" key.
map_outdated() { # <json-out> <filter_error:1|0>
	[[ -z $1 ]] || [[ $1 != \{* ]] && return 0
	local sel="."
	(( $2 )) && sel='select(.key != "error")'
	printf '%s' "$1" | jq -r "to_entries[] | $sel | \"\(.key)\t\(.value.current // "?")\t\(.value.latest // "?")\"" 2>/dev/null
}

# array-style: [{name, current, latest}] with "?" defaults.
array_outdated() { # <json-out>
	[[ -z $1 ]] && return 0
	printf '%s' "$1" | jq -r 'if type == "array" then .[] else empty end
		| "\(.name // "?")\t\(.current // "?")\t\(.latest // "?")"' 2>/dev/null
}

pkgs=""

case "$pm" in
npm)
	which_cached npm && pkgs+=$(map_outdated "$(run_cmd_in "$project" npm outdated --json 2>/dev/null)" 1)
	;;
pnpm)
	which_cached pnpm && pkgs+=$(map_outdated "$(run_cmd_in "$project" pnpm outdated --json 2>/dev/null)" 0)
	;;
yarn)
	# yarn prints NDJSON; keep lines where name/current/latest all exist.
	if which_cached yarn; then
		out=$(run_cmd_in "$project" yarn outdated --json 2>/dev/null)
		pkgs+=$(printf '%s\n' "$out" | jq -rR 'fromjson? |
			select(.name != null and .current != null and .latest != null)
			| "\(.name)\t\(.current)\t\(.latest)"' 2>/dev/null)
	fi
	;;
bun)
	which_cached bun && pkgs+=$(array_outdated "$(run_cmd_in "$project" bun outdated --format=json 2>/dev/null)")
	;;
deno)
	which_cached deno && pkgs+=$(array_outdated "$(run_cmd_in "$project" deno outdated --json 2>/dev/null)")
	;;
cargo)
	if which_cached cargo-outdated; then
		out=$(run_cmd_in "$project" cargo outdated --format json 2>/dev/null)
		pkgs+=$(printf '%s' "$out" | jq -r '
			(if type == "array" then . else (.dependencies // []) end)[]
			| "\(.name // "?")\t\(.project // "?")\t\(.latest // "?")"' 2>/dev/null)
	fi
	;;
poetry)
	if which_cached poetry; then
		out=$(run_cmd_in "$project" poetry show --outdated --format=json 2>/dev/null)
		if printf '%s' "$out" | jq -e 'type == "array"' >/dev/null 2>&1; then
			pkgs+=$(printf '%s' "$out" | jq -r '.[] | "\(.name // "?")\t\(.version // "?")\t\(.latest // "?")"' 2>/dev/null)
		else
			# Fallback text table; current/latest must look version-like.
			pkgs+=$(printf '%s\n' "$out" | awk 'NF >= 3 && ($2 ~ /^[0-9]/ || $2 ~ /\./) && ($3 ~ /^[0-9]/ || $3 ~ /\./) {print $1"\t"$2"\t"$3}')
		fi
	fi
	;;
pipenv)
	if which_cached pipenv; then
		out=$(run_cmd_in "$project" pipenv run pip list --outdated --format=json 2>/dev/null)
		pkgs+=$(printf '%s' "$out" | jq -r 'if type == "array" then .[] else empty end
			| "\(.name // "?")\t\(.version // "?")\t\(.latest_version // "?")"' 2>/dev/null)
	fi
	;;
requirements)
	# Prefer project venv pip, else pip3/pip — like pip_venv_outdated.
	pip_cmd=""
	if [[ -x "$project/.venv/bin/pip" ]]; then
		pip_cmd="$project/.venv/bin/pip"
	elif [[ -x "$project/venv/bin/pip" ]]; then
		pip_cmd="$project/venv/bin/pip"
	elif which_cached pip3; then
		pip_cmd="pip3"
	elif which_cached pip; then
		pip_cmd="pip"
	fi
	if [[ -n $pip_cmd ]]; then
		out=$(run_cmd_in "$project" "$pip_cmd" list --outdated --format=json 2>/dev/null)
		pkgs+=$(printf '%s' "$out" | jq -r 'if type == "array" then .[] else empty end
			| "\(.name // "?")\t\(.version // "?")\t\(.latest_version // "?")"' 2>/dev/null)
	fi
	;;
go)
	if which_cached go; then
		out=$(run_cmd_in "$project" go list -u -m -json all 2>/dev/null)
		pkgs+=$(printf '%s' "$out" | jq -r 'select(.Update != null)
			| "\(.Path // "?")\t\(.Version // "?")\t\(.Update.Version // "?")"' 2>/dev/null)
	fi
	;;
composer)
	if which_cached composer; then
		out=$(run_cmd_in "$project" composer outdated --format=json --direct 2>/dev/null)
		pkgs+=$(printf '%s' "$out" | jq -r '.installed[]? | "\(.name // "?")\t\(.version // "?")\t(.latest // "?")"' 2>/dev/null)
	fi
	;;
gradle)
	RESULT_ISSUES=$(issues_json "Gradle dependency check requires ben-manes/gradle-versions-plugin (skipped)")
	;;
maven)
	RESULT_ISSUES=$(issues_json "Maven outdated checking requires manual parsing (skipped)")
	;;
esac

RESULT_OUTDATED=$(printf '%s' "$pkgs" | json_array_pkgs)
n=$(jq 'length' <<<"${RESULT_OUTDATED:-[]}")
if ((n > 0)); then
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(jq -cn --argjson a "${RESULT_ISSUES:-[]}" --arg m "$n outdated package(s)" '$a + [$m]')
fi
emit_scan_result
