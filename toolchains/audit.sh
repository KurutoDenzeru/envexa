#!/usr/bin/env bash
# Port of src/toolchains/audit.rs: version pairing checks (node/npm, python/pip,
# brew age, cargo vs rustc, bun age) and env-manager file checks.
set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="audit"
RESULT_STATUS="ok"

# major_of mirrors parse_semver_parts().first: first numeric token anywhere in
# the string ("Python 3.13.5" -> 3, "v26.8.1" -> 26).
major_of() {
	printf '%s' "$1" | grep -oE '[0-9]+' | head -n 1
}

audit_items=""

add_item() { # <name> <current> <note>
	jq -cn --arg name "$1" --arg current "$2" --arg note "$3" \
		'{name: $name, current: $current, note: $note}'
}

# --- node vs npm pairing -----------------------------------------------------
if node_ver=$(run_cmd node --version) && npm_ver=$(run_cmd npm --version); then
	node_major=$(major_of "$node_ver")
	npm_major=$(major_of "$npm_ver")
	if [[ -n $node_major && -n $npm_major ]]; then
		if ((node_major >= 20)); then
			expected=10
		elif ((node_major >= 18)); then
			expected=9
		elif ((node_major >= 16)); then
			expected=8
		else
			expected=6
		fi
		if ((npm_major < expected)); then
			audit_items+="$(add_item "npm (vs Node)" "node v${node_major} + npm v${npm_major}" "npm v${expected}+ expected with Node v${node_major}")"$'\n'
		fi
	fi
fi

# --- python vs pip pairing ----------------------------------------------------
if py_ver=$(run_cmd python3 --version) && pip_ver=$(run_cmd pip3 --version); then
	py_major=$(major_of "$py_ver")
	pip_num=$(printf '%s' "$pip_ver" | awk '{print $2}')
	pip_major=$(major_of "$pip_num")
	if [[ -n $py_major && -n $pip_major ]] && ((py_major >= 12 && pip_major < 24)); then
		audit_items+="$(add_item "pip (vs Python)" "Python v${py_major} + pip v${pip_major}" "pip v24+ recommended with Python v${py_major}")"$'\n'
	fi
fi

# --- brew age ------------------------------------------------------------------
if brew_ver=$(run_cmd brew --version); then
	ver=$(printf '%s' "$brew_ver" | awk '{print $2}')
	m=$(major_of "$ver")
	if [[ -n $m ]] && ((m < 4)); then
		audit_items+="$(add_item "Homebrew" "v${ver}" "v4+ recommended (run \`brew update\`)")"$'\n'
	fi
fi

# --- cargo vs rustc -------------------------------------------------------------
if rustc_ver=$(run_cmd rustc --version) && cargo_ver=$(run_cmd cargo --version); then
	rc_ver=$(printf '%s' "$rustc_ver" | awk '{print $2}')
	c_ver=$(printf '%s' "$cargo_ver" | awk '{print $2}')
	rc_major=$(major_of "$rc_ver")
	c_major=$(major_of "$c_ver")
	if [[ -n $rc_major && -n $c_major ]]; then
		diff=$((rc_major > c_major ? rc_major - c_major : c_major - rc_major))
		if ((diff > 1)); then
			audit_items+="$(add_item "rustc vs Cargo" "rustc v${rc_ver}, cargo v${c_ver}" "versions should track within 1 major")"$'\n'
		fi
	fi
fi

# --- bun age ---------------------------------------------------------------------
if bun_ver=$(run_cmd bun --version); then
	m=$(major_of "$bun_ver")
	if [[ -n $m ]] && ((m < 1)); then
		audit_items+="$(add_item "Bun" "v${bun_ver}" "v1+ recommended")"$'\n'
	fi
fi

# --- env managers (.nvmrc/.node-version/.tool-versions/mise.toml) --------------
project=$(get_project_path)
expected_node="" node_src=""
expected_python="" py_src=""
expected_java="" java_src=""

if [[ -f "$project/.nvmrc" ]]; then
	expected_node=$(tr -d '[:space:]' <"$project/.nvmrc")
	node_src="nvmrc"
elif [[ -f "$project/.node-version" ]]; then
	expected_node=$(tr -d '[:space:]' <"$project/.node-version")
	node_src="node-version"
fi

if [[ -f "$project/.tool-versions" ]]; then
	while read -r tool ver; do
		[[ -n $ver ]] || continue
		case "$tool" in
		nodejs) [[ -z $expected_node ]] && expected_node=$ver && node_src="tool-versions" ;;
		python) [[ -z $expected_python ]] && expected_python=$ver && py_src="tool-versions" ;;
		java) [[ -z $expected_java ]] && expected_java=$ver && java_src="tool-versions" ;;
		esac
	done <"$project/.tool-versions"
fi

if [[ -f "$project/mise.toml" ]]; then
	in_tools=0
	while IFS= read -r line; do
		line=${line#"${line%%[![:space:]]*}"}
		if [[ $line == "[tools]" ]]; then
			in_tools=1
			continue
		elif [[ $line == \[* ]]; then
			in_tools=0
		fi
		if ((in_tools)) && [[ $line == *=* ]]; then
			k=${line%%=*}
			k=${k#"${k%%[![:space:]]*}"}
			k=${k%"${k##*[![:space:]]}"}
			v=${line#*=}
			v=${v#"${v%%[![:space:]]*}"}
			v=${v%"${v##*[![:space:]]}"}
			while [[ $v == \"* || $v == \'* ]]; do v=${v#?}; done
			while [[ $v == *\" || $v == *\' ]]; do v=${v%?}; done
			case "$k" in
			node) [[ -z $expected_node ]] && expected_node=$v && node_src="mise.toml" ;;
			python) [[ -z $expected_python ]] && expected_python=$v && py_src="mise.toml" ;;
			java) [[ -z $expected_java ]] && expected_java=$v && java_src="mise.toml" ;;
			esac
		fi
	done <"$project/mise.toml"
fi

if [[ -n $expected_node ]] && node_cur=$(run_cmd node --version); then
	current=${node_cur#v}
	expected_clean=${expected_node#v}
	[[ $current == ${expected_clean}* ]] ||
		audit_items+="$(add_item "Node Environment" "node v${current}" "expected v${expected_clean} from .${node_src}")"$'\n'
fi

if [[ -z $expected_python && -f "$project/.python-version" ]]; then
	expected_python=$(tr -d '[:space:]' <"$project/.python-version")
	py_src="python-version"
fi
if [[ -n $expected_python ]] && py_cur=$(run_cmd python3 --version); then
	current=$(printf '%s' "$py_cur" | awk '{print $2}')
	[[ -n $current ]] || current="0"
	[[ $current == ${expected_python}* ]] ||
		audit_items+="$(add_item "Python Environment" "python v${current}" "expected v${expected_python} from .${py_src}")"$'\n'
fi

if [[ -n $expected_java ]] && java_out=$(run_cmd java -version); then
	current=$(printf '%s\n' "$java_out" | head -n 1 | awk -F'"' '{print $2}')
	[[ -n $current ]] || current="0"
	[[ $current == ${expected_java}* ]] ||
		audit_items+="$(add_item "Java Environment" "java v${current}" "expected v${expected_java} from .${java_src}")"$'\n'
fi

RESULT_AUDIT_ITEMS=$(printf '%s' "$audit_items" | jq -Rsc 'split("\n") | map(select(length > 0) | fromjson)')
n=$(jq 'length' <<<"${RESULT_AUDIT_ITEMS:-[]}")
if ((n > 0)); then
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(issues_json "$n audit item(s)")
fi
emit_scan_result
