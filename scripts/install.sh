#!/usr/bin/env bash
set -euo pipefail

VERSION="${ENVEXA_VERSION:-v1.0.0}"
REPO="KurutoDenzeru/envexa"

die() {
    echo "Error: $*" >&2
    exit 1
}

detect_asset_name() {
    local os arch ext
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"
    ext=""

    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64 | arm64) arch="aarch64" ;;
        *) die "unsupported architecture: $arch" ;;
    esac

    case "$os" in
        darwin) os="macos" ;;
        linux) os="linux" ;;
        mingw* | msys* | cygwin*) os="windows"; ext=".exe" ;;
        *) die "unsupported OS: $os" ;;
    esac

    echo "envexa-${arch}-${os}${ext}"
}

detect_bin_name() {
    case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
        mingw* | msys* | cygwin*) echo "envexa.exe" ;;
        *) echo "envexa" ;;
    esac
}

main() {
    local asset_name bin_name url install_dir bin_path

    asset_name="$(detect_asset_name)" || die "cannot detect platform"
    bin_name="$(detect_bin_name)"
    url="https://github.com/${REPO}/releases/download/${VERSION}/${asset_name}"
    install_dir="${ENVEXA_INSTALL_DIR:-${HOME}/.local/bin}"
    bin_path="${install_dir}/${bin_name}"

    if [[ -f "$bin_path" ]]; then
        echo "envexa is already installed at ${bin_path}"
        echo "Re-run to upgrade, or remove it first."
        exit 0
    fi

    mkdir -p "$install_dir"

    echo "Downloading envexa ${VERSION} for ${asset_name}..."
    curl -fsSL "$url" -o "$bin_path" || die "download failed (url: $url)"

    chmod +x "$bin_path" 2>/dev/null || true

    echo ""
    echo "envexa ${VERSION} installed to ${bin_path}"
    echo ""
    echo "Make sure ${install_dir} is in your PATH."
    echo "Run '${bin_name}' to start."
}

main
