#!/usr/bin/env bash
# Go-era installer: downloads the envexa-v<version>-<os>-<arch>.tar.gz release
# artifact (binary + bash scanners + web dashboard dist) and installs it:
#   envexa            -> ~/.local/bin
#   toolchains/       -> ~/.local/share/envexa/toolchains
#   frontend/dist     -> ~/.local/share/envexa/frontend/dist
set -euo pipefail

REPO="KurutoDenzeru/envexa"

die() {
    echo "Error: $*" >&2
    exit 1
}

require() {
    command -v "$1" >/dev/null 2>&1 || die "$1 is required but not installed"
}

require curl
require jq
require tar
command -v bash >/dev/null 2>&1 || die "bash is required"
if ! command -v go >/dev/null 2>&1; then
    echo "Note: go not found — fine, the release ships a prebuilt binary."
fi

fetch_latest_tag() {
    curl -fsSL -H "User-Agent: envexa" \
        "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | jq -r '.tag_name // empty'
}

detect_asset_name() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$arch" in
        x86_64) arch="x64" ;;
        aarch64 | arm64) arch="arm64" ;;
        *) die "unsupported architecture: $arch" ;;
    esac

    case "$os" in
        darwin | linux) ;;
        *) die "unsupported OS: $os" ;;
    esac

    echo "envexa-${version#v}-${os}-${arch}.tar.gz"
}

main() {
    local version asset_name url tmp install_dir bin_path share_dir

    version="${ENVEXA_VERSION:-$(fetch_latest_tag)}"
    [[ -n "$version" ]] || die "could not determine latest release tag"

    asset_name="$(detect_asset_name)" || die "cannot detect platform"
    url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"

    install_dir="${ENVEXA_INSTALL_DIR:-${HOME}/.local/bin}"
    share_dir="${ENVEXA_SHARE_DIR:-${XDG_DATA_HOME:-${HOME}/.local/share}/envexa}"
    bin_path="${install_dir}/envexa"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    echo "Downloading envexa ${version} (${asset_name})..."
    curl -fsSL "$url" -o "${tmp}/${asset_name}" \
        || die "download failed (url: $url)"

    echo "Verifying checksum..."
    curl -fsSL "${url}.sha256" -o "${tmp}/checksum" \
        || die "checksum download failed"
    (cd "$tmp" && shasum -a 256 -c checksum) >/dev/null \
        || die "checksum verification failed"

    mkdir -p "$install_dir" "${share_dir}/frontend"
    tar -xzf "${tmp}/${asset_name}" -C "$tmp"
    mv "${tmp}/envexa" "$bin_path"
    chmod +x "$bin_path"
    mkdir -p "${share_dir}/toolchains"
    cp "${tmp}/toolchains/"*.sh "${share_dir}/toolchains/"
    mkdir -p "${share_dir}/toolchains/lib"
    cp "${tmp}/toolchains/lib/scan.sh" "${share_dir}/toolchains/lib/"
    cp -R "${tmp}/frontend/dist" "${share_dir}/frontend/dist"

    echo ""
    echo "envexa ${version} installed:"
    echo "  binary:     ${bin_path}"
    echo "  scanners:   ${share_dir}/toolchains"
    echo "  dashboard:  ${share_dir}/frontend/dist"
    echo ""
    echo "Make sure ${install_dir} is in your PATH."
    echo "Run 'envexa' to start, 'envexa serve' for the web dashboard."
}

main
