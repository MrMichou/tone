#!/bin/sh
set -eu

REPO="MrMichou/tone"
INSTALL_DIR="${TONE_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    detect_platform
    get_version "$@"
    download_and_install
}

detect_platform() {
    OS=$(uname -s)
    case "$OS" in
        Linux)  OS_TARGET="unknown-linux-gnu" ;;
        Darwin) OS_TARGET="apple-darwin" ;;
        *)      error "Unsupported OS: $OS" ;;
    esac

    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64)   ARCH_TARGET="x86_64" ;;
        aarch64|arm64)  ARCH_TARGET="aarch64" ;;
        *)              error "Unsupported architecture: $ARCH" ;;
    esac

    TARGET="${ARCH_TARGET}-${OS_TARGET}"
    echo "Detected platform: ${TARGET}"
}

get_version() {
    if [ $# -ge 1 ]; then
        VERSION="$1"
    else
        echo "Fetching latest version..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' \
            | sed -E 's/.*"([^"]+)".*/\1/')
    fi
    echo "Version: ${VERSION}"
}

download_and_install() {
    URL="https://github.com/${REPO}/releases/download/${VERSION}/tone-${TARGET}.tar.gz"
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/checksums.txt"

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    echo "Downloading tone ${VERSION} for ${TARGET}..."
    curl -fsSL "$URL" -o "$TMPDIR/tone.tar.gz"
    curl -fsSL "$CHECKSUM_URL" -o "$TMPDIR/checksums.txt"

    verify_checksum
    extract_and_install
}

verify_checksum() {
    EXPECTED=$(grep "tone-${TARGET}.tar.gz" "$TMPDIR/checksums.txt" | cut -d' ' -f1)

    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "$TMPDIR/tone.tar.gz" | cut -d' ' -f1)
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL=$(shasum -a 256 "$TMPDIR/tone.tar.gz" | cut -d' ' -f1)
    else
        echo "Warning: cannot verify checksum (no sha256sum or shasum found)"
        return
    fi

    if [ "$EXPECTED" != "$ACTUAL" ]; then
        error "Checksum mismatch! Expected: ${EXPECTED}, Got: ${ACTUAL}"
    fi
    echo "Checksum verified."
}

extract_and_install() {
    tar -xzf "$TMPDIR/tone.tar.gz" -C "$TMPDIR"
    mkdir -p "$INSTALL_DIR"
    mv "$TMPDIR/tone" "$INSTALL_DIR/tone"
    chmod +x "$INSTALL_DIR/tone"

    echo "tone ${VERSION} installed to ${INSTALL_DIR}/tone"

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *) echo "Note: Add ${INSTALL_DIR} to your PATH to use tone" ;;
    esac
}

error() {
    echo "Error: $1" >&2
    exit 1
}

main "$@"
