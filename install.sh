#!/bin/sh
set -eu

REPO="centy-io/installer"
BINARY_NAME="centy-installer"

main() {
    detect_os
    detect_arch
    target="${ARCH}-${OS}"

    version="${1:-}"
    if [ -z "$version" ]; then
        version=$(fetch_latest_version)
    fi

    # Ensure v prefix
    case "$version" in
        v*) ;;
        *) version="v${version}" ;;
    esac

    url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}"

    echo "Installing centy via ${BINARY_NAME} ${version} (${target})..."

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    binary="${tmpdir}/${BINARY_NAME}"

    if ! curl -fsSL "$url" -o "$binary"; then
        echo "Error: failed to download ${BINARY_NAME} ${version} for ${target}" >&2
        echo "URL: ${url}" >&2
        exit 1
    fi

    chmod +x "$binary"
    "$binary"
}

detect_os() {
    OS=$(uname -s)
    case "$OS" in
        Darwin) OS="apple-darwin" ;;
        Linux) OS="unknown-linux-gnu" ;;
        *)
            echo "Error: unsupported OS: ${OS}" >&2
            exit 1
            ;;
    esac
}

detect_arch() {
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64 | amd64) ARCH="x86_64" ;;
        aarch64 | arm64) ARCH="aarch64" ;;
        *)
            echo "Error: unsupported architecture: ${ARCH}" >&2
            exit 1
            ;;
    esac
}

fetch_latest_version() {
    if ! version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'); then
        echo "Error: failed to fetch latest version" >&2
        exit 1
    fi

    if [ -z "$version" ]; then
        echo "Error: no releases found" >&2
        exit 1
    fi

    echo "$version"
}

main "$@"
