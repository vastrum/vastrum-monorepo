#!/usr/bin/env bash
set -e

echo "Installing vastrum-cli..."

REPO="vastrum/vastrum-monorepo"
BINARY_NAME="vastrum-cli"
INSTALL_DIR="${VASTRUM_INSTALL_DIR:-$HOME/.vastrum/bin}"

# Detect OS
case "$(uname -s)" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      echo "Error: Unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

# Detect architecture
case "$(uname -m)" in
    x86_64)        arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             echo "Error: Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="${arch}-${os}"

# Determine version
version="${VASTRUM_VERSION:-${1:-}}"
if [ -z "$version" ]; then
    echo "Fetching latest version..."
    if command -v curl >/dev/null 2>&1; then
        response="$(curl -fsSL --retry 3 "https://api.github.com/repos/${REPO}/releases/latest")"
    elif command -v wget >/dev/null 2>&1; then
        response="$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest")"
    else
        echo "Error: Neither curl nor wget found" >&2; exit 1
    fi
    version="$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | sed 's/.*"tag_name": *"//;s/"//')"
    if [ -z "$version" ]; then
        echo "Error: Could not fetch latest version from GitHub" >&2; exit 1
    fi
fi
case "$version" in
    v*) ;;
    *)  version="v${version}" ;;
esac

echo "Installing ${BINARY_NAME} ${version} for ${target}..."

url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}.tar.gz"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

# Download binary
if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 "$url" -o "${tmpdir}/${BINARY_NAME}-${target}.tar.gz" \
        || { echo "Error: Failed to download ${BINARY_NAME} ${version} for ${target}. Check that the version exists at https://github.com/${REPO}/releases" >&2; exit 1; }
elif command -v wget >/dev/null 2>&1; then
    wget -qO "${tmpdir}/${BINARY_NAME}-${target}.tar.gz" "$url" \
        || { echo "Error: Failed to download ${BINARY_NAME} ${version} for ${target}. Check that the version exists at https://github.com/${REPO}/releases" >&2; exit 1; }
fi

# Extract and install
mkdir -p "$INSTALL_DIR"
tar xzf "${tmpdir}/${BINARY_NAME}-${target}.tar.gz" -C "$INSTALL_DIR"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "Installed ${BINARY_NAME} ${version} to ${INSTALL_DIR}/${BINARY_NAME}"

# Detect shell and profile file
case $SHELL in
    */zsh)
        PROFILE="${ZDOTDIR-"$HOME"}/.zshenv"
        PREF_SHELL=zsh
        ;;
    */bash)
        PROFILE="$HOME/.bashrc"
        PREF_SHELL=bash
        ;;
    */fish)
        PROFILE="$HOME/.config/fish/config.fish"
        PREF_SHELL=fish
        ;;
    */ash)
        PROFILE="$HOME/.profile"
        PREF_SHELL=ash
        ;;
    *)
        echo "Could not detect shell, manually add ${INSTALL_DIR} to your PATH."
        PROFILE=""
        PREF_SHELL=""
        ;;
esac

# Auto-add to PATH if not already present
if [ -n "${PROFILE:-}" ]; then
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            if [ "${PREF_SHELL:-}" = "fish" ]; then
                echo >> "$PROFILE" && echo "fish_add_path -a \"$INSTALL_DIR\"" >> "$PROFILE"
            else
                echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$PROFILE"
            fi
            echo "Added ${INSTALL_DIR} to PATH in ${PROFILE}"
            echo "Run 'source ${PROFILE}' or start a new terminal session to use ${BINARY_NAME}."
            ;;
    esac
fi

# Shadow check — warn if another binary on PATH would take precedence
existing="$(command -v "$BINARY_NAME" 2>/dev/null || true)"
if [ -n "$existing" ] && [ "$existing" != "${INSTALL_DIR}/${BINARY_NAME}" ]; then
    echo "Warning: Another ${BINARY_NAME} exists at ${existing} and will take precedence" >&2
    echo "  Remove it with:"
    echo "    rm ${existing}"
fi
