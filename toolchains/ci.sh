#!/usr/bin/env bash
# Port of src/toolchains/ci.rs: scan .github/workflows for pinned GitHub Actions
# below the known-latest major version.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="ci"
RESULT_STATUS="ok"

project=$(get_project_path)
workflows="$project/.github/workflows"

if [[ ! -d $workflows ]]; then
	scan_skipped "No .github/workflows directory found"
	exit 0
fi

# LATEST_ACTIONS from ci.rs — case statement instead of a map (bash 3.2).
latest_of() {
	case "$1" in
	"actions/checkout") echo v4 ;;
	"actions/setup-node") echo v4 ;;
	"actions/setup-python") echo v5 ;;
	"actions/setup-go") echo v5 ;;
	"actions/setup-java") echo v4 ;;
	"actions/cache") echo v4 ;;
	"actions/upload-artifact") echo v4 ;;
	"actions/download-artifact") echo v4 ;;
	"actions/github-script") echo v7 ;;
	"actions/stale") echo v9 ;;
	"docker/setup-buildx-action") echo v3 ;;
	"docker/login-action") echo v3 ;;
	"docker/build-push-action") echo v5 ;;
	"docker/setup-qemu-action") echo v3 ;;
	"codecov/codecov-action") echo v4 ;;
	"dtinth/setup-github-actions-caching") echo v1 ;;
	esac
}

uses_re='uses:[[:space:]]+[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[vV]?[0-9]+(\.[0-9]+)*'

pkgs=""
while IFS= read -r -d '' f; do
	while IFS= read -r m; do
		action=$(printf '%s' "$m" | sed -E 's/^uses:[[:space:]]+([^@]+)@.*/\1/')
		version=$(printf '%s' "$m" | sed -E 's/.*@([vV]?[0-9]+(\.[0-9]+)*)$/\1/')
		latest=$(latest_of "$action")
		[[ -z $latest ]] && continue
		c=${version#[vV]}
		l=${latest#[vV]}
		case $c in *.*|*[!0-9]*) continue ;; esac
		case $l in *.*|*[!0-9]*) continue ;; esac
		if ((c < l)) && [[ ! -v _CI_SEEN_${action} ]]; then
			eval "_CI_SEEN_$(printf '%s' "$action" | tr -c 'a-zA-Z0-9' '_')=1"
			pkgs+="$action"$'\t'"$version"$'\t'"$latest"$'\n'
		fi
	done < <(grep -oE "$uses_re" "$f" 2>/dev/null)
done < <(find "$workflows" -type f \( -name '*.yml' -o -name '*.yaml' \) -print0 2>/dev/null)

RESULT_OUTDATED=$(printf '%s' "$pkgs" | json_array_pkgs)
n=$(jq 'length' <<<"${RESULT_OUTDATED:-[]}")
if ((n > 0)); then
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(issues_json "$n outdated GitHub Action(s) found")
fi
emit_scan_result
