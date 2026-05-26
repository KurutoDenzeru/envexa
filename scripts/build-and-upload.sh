#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-$(git describe --tags --abbrev=0 2>/dev/null || true)}"

if [ -z "$TAG" ]; then
    echo "Error: not on a tag and no tag argument provided."
    echo "Usage: $0 [<tag>]"
    exit 1
fi

echo "==> Building for aarch64-apple-darwin..."
cargo build --release --target aarch64-apple-darwin

echo "==> Building for x86_64-apple-darwin..."
cargo build --release --target x86_64-apple-darwin

echo "==> Uploading aarch64-apple-darwin to $TAG..."
cp target/aarch64-apple-darwin/release/envexa target/aarch64-apple-darwin/release/envexa-aarch64-macos
gh release upload "$TAG" \
    target/aarch64-apple-darwin/release/envexa-aarch64-macos \
    --clobber

echo "==> Uploading x86_64-apple-darwin to $TAG..."
cp target/x86_64-apple-darwin/release/envexa target/x86_64-apple-darwin/release/envexa-x86_64-macos
gh release upload "$TAG" \
    target/x86_64-apple-darwin/release/envexa-x86_64-macos \
    --clobber

echo "==> Done. Both macOS binaries uploaded to $TAG"
