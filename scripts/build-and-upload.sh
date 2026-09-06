#!/usr/bin/env bash
# Build the Go-era release artifacts and upload them to a GitHub release:
#   envexa-v<version>-<os>-<arch>.tar.gz  (envexa binary + toolchains/*.sh + frontend/dist)
#   + .sha256 sidecars
set -euo pipefail

TAG="${1:-$(git describe --tags --abbrev=0 2>/dev/null || true)}"

if [ -z "$TAG" ]; then
    echo "Error: not on a tag and no tag argument provided."
    echo "Usage: $0 [<tag>]"
    exit 1
fi
VERSION="${TAG#v}"

echo "==> Building frontend..."
(cd frontend && bun run build)

OUT="target/release-envexa"
rm -rf "$OUT"
mkdir -p "$OUT"

build() { # <goos> <goarch> <os-name> <arch-name>
    local goos="$1" goarch="$2" osname="$3" archname="$4"
    local name="envexa-v${VERSION}-${osname}-${archname}"
    echo "==> Building ${name}.tar.gz..."
    local dir="${OUT}/${name}"
    mkdir -p "$dir/toolchains/lib" "$dir/frontend"

    GOOS="$goos" GOARCH="$goarch" \
        go build -ldflags "-s -w -X github.com/KurutoDenzeru/envexa/internal/cli.Version=${VERSION}" \
        -o "$dir/envexa" ./cmd/envexa

    cp toolchains/*.sh "$dir/toolchains/"
    cp toolchains/lib/scan.sh "$dir/toolchains/lib/"
    cp -R frontend/dist "$dir/frontend/dist"
    cp LICENSE "$dir/" 2>/dev/null || true

    (cd "$dir" && tar -czf "../${name}.tar.gz" .)
    (cd "$OUT" && shasum -a 256 "${name}.tar.gz" > "${name}.tar.gz.sha256")
}

build darwin arm64 darwin arm64
build darwin amd64 darwin x64
build linux arm64 linux arm64
build linux amd64 linux x64

echo "==> Uploading artifacts to $TAG..."
gh release upload "$TAG" \
    "$OUT"/envexa-v"${VERSION}"-*.tar.gz \
    "$OUT"/envexa-v"${VERSION}"-*.tar.gz.sha256 \
    --clobber

echo "==> Done. Artifacts uploaded to $TAG"
