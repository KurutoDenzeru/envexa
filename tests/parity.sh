#!/usr/bin/env bash
# tests/parity.sh — issue #34 sunset gate harness.
#
# Runs the Rust and Go runtimes against the live host and compares their scan
# reports after order-normalization (Rust HashMap iteration order is
# nondeterministic; array CONTENT must match, order must not matter).
#
# Usage: bash tests/parity.sh
# Env:
#   ENVEXA_RUST_BIN / ENVEXA_GO_BIN  prebuilt binaries (skip building)
#   ENVEXA_PARITY_SKIP_RUST=1        Go-only run (no gate verdict; CI smoke)
#
# Gate: exit 0 = parity. Any diff fails the sunset gate.
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"

OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT
GO_BIN="${ENVEXA_GO_BIN:-$OUT/envexa-go}"
RUST_BIN="${ENVEXA_RUST_BIN:-$OUT/envexa-rust}"

echo "==> Building Go runtime..."
if [[ -n "${ENVEXA_GO_BIN:-}" ]]; then
    cp "$ENVEXA_GO_BIN" "$GO_BIN"
else
    go build -o "$GO_BIN" ./cmd/envexa
fi

norm() { # canonicalize: drop timestamp, sort HashMap-ordered arrays
    jq -S '
    def norm:
      if type == "object" then with_entries(.value |= norm)
      elif type == "array" then
        (if length > 0 and (.[0] | type) == "object"
            and ((.[0].name? != null) or (.[0].package? != null))
         then sort_by(.name // .package)
         else . end)
      else . end;
    del(.timestamp) | norm'
}

if [[ "${ENVEXA_PARITY_SKIP_RUST:-0}" == "1" ]]; then
    echo "==> ENVEXA_PARITY_SKIP_RUST=1 — Go-only run, no parity verdict."
    "$GO_BIN" scan --format json | jq -S 'del(.timestamp)' > "$OUT/go.json"
    echo "Go report: $(jq '.results | length' "$OUT/go.json") toolchains — OK"
    exit 0
fi

echo "==> Building Rust runtime..."
if [[ -n "${ENVEXA_RUST_BIN:-}" ]]; then
    cp "$ENVEXA_RUST_BIN" "$RUST_BIN"
else
    cargo build --quiet && cp target/debug/envexa "$RUST_BIN"
fi

echo "==> Scanning with the Rust runtime..."
"$RUST_BIN" scan --format json > "$OUT/rust-raw.json"

echo "==> Scanning with the Go + Bash runtime..."
"$GO_BIN" scan --format json > "$OUT/go-raw.json"

norm < "$OUT/rust-raw.json" > "$OUT/rust.json"
norm < "$OUT/go-raw.json" > "$OUT/go.json"

if diff -u "$OUT/rust.json" "$OUT/go.json" > "$OUT/parity.diff"; then
    echo "==> PARITY OK: $(jq '.results | length' "$OUT/rust.json") toolchains, \
$(jq '[.results[] | ((.outdated // []) + (.outdated_global // []) + (.outdated_formulae // []) + (.outdated_casks // [])) | length] | add' "$OUT/rust.json") outdated entries."
    exit 0
fi

echo "==> PARITY FAILED — report diff (first 60 lines):"
head -60 "$OUT/parity.diff"
echo "..."
echo "Full diff: $OUT/parity.diff"
exit 1
