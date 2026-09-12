#!/usr/bin/env bash
# ==============================================================================
# Tagisan (tgs) One-Line Automated Installer
# High-Performance Multi-LLM Swarm, Dialectical Debate & ECC Engine in Rust
# Usage: curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh
# ==============================================================================

set -e

REPO="CharleGutierrez/tagisan"
BIN_NAME="tgs"
INSTALL_DIR="/usr/local/bin"

# Detect non-root execution
if [ "$(id -u)" -ne 0 ]; then
    if [ -d "$HOME/.cargo/bin" ]; then
        INSTALL_DIR="$HOME/.cargo/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
fi

echo "======================================================================"
echo "  🇵🇭 Installing Tagisan (tgs) Multi-LLM Collaboration Engine"
echo "======================================================================"

# Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "Unsupported CPU architecture: $ARCH"
        exit 1
        ;;
esac

case "$OS" in
    linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    darwin)
        TARGET_OS="apple-darwin"
        ;;
    *)
        echo "Unsupported Operating System: $OS"
        exit 1
        ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
echo "Detected platform: ${TARGET}"

# Fetch latest release tag
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

INSTALLED=false

if [ -n "$LATEST_RELEASE" ]; then
    TARBALL_URL="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/tgs-${LATEST_RELEASE}-${TARGET}.tar.gz"
    echo "Attempting prebuilt download from: ${TARBALL_URL}"

    TMP_DIR=$(mktemp -d)
    if curl -sSL --fail "$TARBALL_URL" -o "${TMP_DIR}/tgs.tar.gz" 2>/dev/null; then
        tar -xzf "${TMP_DIR}/tgs.tar.gz" -C "${TMP_DIR}"
        if [ -f "${TMP_DIR}/tgs" ]; then
            chmod +x "${TMP_DIR}/tgs"
            cp "${TMP_DIR}/tgs" "${INSTALL_DIR}/${BIN_NAME}"
            rm -rf "${TMP_DIR}"
            INSTALLED=true
            echo "Successfully installed prebuilt binary to ${INSTALL_DIR}/${BIN_NAME}"
        fi
    fi
fi

# Fallback to cargo install if prebuilt binary is not found
if [ "$INSTALLED" = false ]; then
    if command -v cargo >/dev/null 2>&1; then
        echo "Prebuilt release binary not found. Compiling via Cargo from source..."
        cargo install --git "https://github.com/${REPO}.git" --bin tgs
        INSTALLED=true
    else
        echo "Error: Prebuilt binary unavailable and Cargo is not installed."
        echo "Please install Rust (https://rustup.rs) or download a release binary from:"
        echo "https://github.com/${REPO}/releases"
        exit 1
    fi
fi

# Verification
if command -v "$BIN_NAME" >/dev/null 2>&1 || [ -f "${INSTALL_DIR}/${BIN_NAME}" ]; then
    echo "======================================================================"
    echo "  ✅ Tagisan (tgs) successfully installed!"
    echo "  Run 'tgs --help' to begin."
    echo "  Run 'tgs debate \"Postgres vs MongoDB\" --local' for local debate."
    echo "======================================================================"
else
    echo "Installed to ${INSTALL_DIR}/${BIN_NAME}. Please ensure ${INSTALL_DIR} is in your PATH."
fi
