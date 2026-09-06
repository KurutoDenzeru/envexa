#!/usr/bin/env bash
# Port of src/toolchains/brew.rs: fast path reads the Homebrew api JWS cache
# (no brew invocation), fallback shells out to brew like the Rust version.
# Compatible with macOS system bash 3.2 (no assoc arrays — jq does the joins).
set -u
[[ -n ${ENVEXA_DEBUG:-} ]] && set -x
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=lib/scan.sh
source "$SCRIPT_DIR/lib/scan.sh"

RESULT_TOOL="brew"
RESULT_STATUS="ok"

# installed_of prints "name\tlatest" per package dir; latest version dir is the
# lexicographic last, same rule as get_installed in brew.rs.
installed_of() {
	local root=$1 d name latest
	[[ -d $root ]] || return 0
	for d in "$root"/*/; do
		[[ -d $d ]] || continue
		name=$(basename "$d")
		[[ $name == .* ]] && continue
		latest=$(cd "$d" 2>/dev/null && LC_ALL=C ls -1d */ 2>/dev/null |
			sed 's:/*$::' | grep -v '^\.' | LC_ALL=C sort | tail -n 1)
		[[ -n $latest ]] && printf '%s\t%s\n' "$name" "$latest"
	done | LC_ALL=C sort
}

# scan_taps extracts the first `version '…'` per tap .rb file, keyed by stem.
# Single grep pass — a per-file fork loop over ~15k tap files takes minutes.
scan_taps() {
	local taps="$1/Library/Taps" line f stem ver
	[[ -d $taps ]] || return 0
	grep -r -m1 --include='*.rb' --exclude-dir='.*' -E 'version[[:space:]]+' "$taps" 2>/dev/null |
		while IFS= read -r line; do
			f=${line%%:*}
			stem=$(basename "$f" .rb)
			ver=$(printf '%s' "$line" |
				sed -n -E "s/^.*version[[:space:]]+[\"']([^\"']+)[\"'].*/\1/p")
			[[ -n $ver ]] && printf '%s\t%s\n' "$stem" "$ver"
		done
}

# tsv_to_pkgs: TSV name/current/latest -> PackageInfo array (empty -> []).
tsv_to_pkgs() {
	jq -Rsc 'split("\n") | map(select(length > 0)
		| split("\t") | {name: .[0], current: .[1], latest: .[2]})'
}

which_cached brew || {
	scan_skipped "Homebrew not installed"
	exit 0
}

base="/opt/homebrew"
[[ -d $base ]] || base="/usr/local"
[[ -d $base ]] || {
	scan_skipped "Homebrew not installed"
	exit 0
}

cache_file=""
for f in "$HOME/Library/Caches/Homebrew/api/internal"/packages.*.jws.json; do
	[[ -e $f ]] && {
		cache_file=$f
		break
	}
done

if [[ -n $cache_file ]] && payload=$(jq -r '.payload' "$cache_file" 2>/dev/null) &&
	jq -e 'has("formulae")' >/dev/null 2>&1 <<<"$payload"; then
	# Fast path: compare installed dirs against the api cache, no brew run.
	formulae_map=$(printf '%s' "$payload" | jq -c '.formulae
		| with_entries(select(.value.stable_version != null) | .value = .value.stable_version)' 2>/dev/null) || formulae_map="{}"
	casks_map=$(printf '%s' "$payload" | jq -c '.casks
		| with_entries(select(.value.version != null) | .value = .value.version)' 2>/dev/null) || casks_map="{}"
	taps_map=$(scan_taps "$base" | jq -Rsc 'split("\n")
		| map(select(length > 0) | split("\t") | {key: .[0], value: .[1]}) | from_entries')

	inst_f=$(installed_of "$base/Cellar" | jq -Rsc 'split("\n")
		| map(select(length > 0) | split("\t") | {name: .[0], current: .[1]})')
	inst_c=$(installed_of "$base/Caskroom" | jq -Rsc 'split("\n")
		| map(select(length > 0) | split("\t") | {name: .[0], current: .[1]})')

	RESULT_INSTALLED_COUNT=$(jq -n --argjson a "$inst_f" --argjson b "$inst_c" '($a | length) + ($b | length)')

	# Outdated join in jq — mirrors is_outdated in brew.rs.
	join_outdated() { # <cache_map> <installed_json>
		jq -cn --argjson cache "$1" --argjson t "$taps_map" \
			--argjson inst "$2" '
			[$inst[] as $p
			| ($cache[$p.name] // $t[$p.name] // empty) as $latest
			| select(($p.current != $latest)
				and (($p.current | startswith($latest + "_")) | not)
				and ($latest != ":latest") and ($latest != "latest"))
			| {name: $p.name, current: $p.current, latest: $latest}]'
	}
	RESULT_OUTDATED_FORMULAE=$(join_outdated "$formulae_map" "$inst_f")
	RESULT_OUTDATED_CASKS=$(join_outdated "$casks_map" "$inst_c")
	RESULT_VERSION=$(printf '%s' "$payload" | jq -r '.metadata.homebrew_version // empty' 2>/dev/null)
	[[ -z ${RESULT_VERSION} ]] && RESULT_VERSION=$(run_cmd brew --version | awk '{print $2}')
else
	# Fallback path: concurrent in Rust (tokio::join!), sequential here.
	RESULT_VERSION=$(run_cmd brew --version | awk '{print $2}')
	out=$(run_cmd brew outdated --greedy --json 2>/dev/null)
	if [[ -n $out ]]; then
		RESULT_OUTDATED_FORMULAE=$(printf '%s' "$out" | jq -c '[.formulae[]?
			| {name: .name,
				current: (if (.installed_versions | length) > 0 then .installed_versions[0] else "?" end),
				latest: .current_version}]') || RESULT_OUTDATED_FORMULAE="[]"
		RESULT_OUTDATED_CASKS=$(printf '%s' "$out" | jq -c '[.casks[]?
			| {name: .name,
				current: (if (.installed_versions | length) > 0 then .installed_versions[0] else "?" end),
				latest: .current_version}]') || RESULT_OUTDATED_CASKS="[]"
	fi
	list=$(run_cmd brew list --formula --versions 2>/dev/null)
	[[ -n $list ]] && RESULT_INSTALLED_COUNT=$(printf '%s\n' "$list" | grep -c .)
fi

total=$(( $(jq 'length' <<<"${RESULT_OUTDATED_FORMULAE:-[]}") +
	$(jq 'length' <<<"${RESULT_OUTDATED_CASKS:-[]}") ))
if ((total > 0)); then
	RESULT_STATUS="warning"
	RESULT_ISSUES=$(issues_json "$total outdated package(s)")
fi
emit_scan_result
