#!/bin/sh
set -e

# Detect OS
OS="$(uname -s)"
if [ "$OS" != "Darwin" ]; then
    echo "Error: This script currently only supports macOS."
    exit 1
fi

# Detect Architecture
ARCH="$(uname -m)"
if [ "$ARCH" = "arm64" ]; then
    ASSET_NAME="bb-darwin-arm64"
elif [ "$ARCH" = "x86_64" ]; then
    ASSET_NAME="bb-darwin-amd64"
else
    echo "Error: Unsupported architecture: $ARCH"
    exit 1
fi

REPO="tmaffia/bbcli"
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET_NAME"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="bb"
INSTALL_PATH="$INSTALL_DIR/$BINARY_NAME"

echo "Downloading $ASSET_NAME from latest release..."
if ! curl -fsSL -o "$BINARY_NAME" "$DOWNLOAD_URL"; then
    echo "Error: Failed to download release asset."
    echo "Please check if a release exists and supports your architecture."
    exit 1
fi

chmod +x "$BINARY_NAME"

echo "Installing to $INSTALL_PATH..."

# Check if we need sudo
if [ -w "$INSTALL_DIR" ]; then
    mv -f "$BINARY_NAME" "$INSTALL_PATH"
else
    echo "Sudo permissions required to install to $INSTALL_DIR"
    sudo mv -f "$BINARY_NAME" "$INSTALL_PATH"
fi

echo "Successfully installed bb to $INSTALL_PATH"
echo "Run 'bb --help' to get started."
