#!/usr/bin/env bash
set -euo pipefail

REPO="KurutoDenzeru/envexa"

die() {
    echo "Error: $*" >&2
    exit 1
}

fetch_latest_tag() {
    curl -fsSL -H "User-Agent: envexa" \
        "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' \
        | cut -d'"' -f4
}

detect_asset_name() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64 | arm64) arch="aarch64" ;;
        *) die "unsupported architecture: $arch" ;;
    esac

    case "$os" in
        darwin) os="macos" ;;
        linux) os="linux" ;;
        *) die "unsupported OS: $os" ;;
    esac

    echo "envexa-${arch}-${os}"
}

main() {
    local version asset_name url install_dir bin_path

    version="${ENVEXA_VERSION:-$(fetch_latest_tag)}"

    asset_name="$(detect_asset_name)" || die "cannot detect platform"
    url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"
    install_dir="${ENVEXA_INSTALL_DIR:-${HOME}/.local/bin}"
    bin_path="${install_dir}/envexa"

    if [[ -f "$bin_path" ]]; then
        if file "$bin_path" | grep -qE 'Mach-O|ELF'; then
            echo "envexa is already installed at ${bin_path}"
            echo "Re-run to upgrade, or remove it first."
            exit 0
        fi
        echo "Warning: existing file is not a valid binary, re-downloading..."
        rm "$bin_path"
    fi

    mkdir -p "$install_dir"

    echo "Downloading envexa ${version} for ${asset_name}..."
    curl -fsSL "$url" -o "$bin_path" || die "download failed (url: $url)"

    if ! file "$bin_path" | grep -qE 'Mach-O|ELF'; then
        rm "$bin_path"
        die "downloaded file is not a valid binary (got HTML/redirect instead of release asset)"
    fi

    chmod +x "$bin_path"

    echo ""
    echo "envexa ${version} installed to ${bin_path}"
    echo ""
    echo "Make sure ${install_dir} is in your PATH."
    echo "Run 'envexa' to start."
}

main
