#!/bin/sh
set -e

REPO="justinasRm/heyorace"
BIN_NAME="heyo"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

os=$(uname -s)
arch=$(uname -m)

case "$os" in
  Darwin)
    case "$arch" in
      arm64) asset="heyo-macos-arm64" ;;
      x86_64) asset="heyo-macos-x86_64" ;;
      *) echo "Unsupported macOS arch: $arch" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $os" >&2
    exit 1
    ;;
esac

url="https://github.com/${REPO}/releases/latest/download/${asset}.tar.gz"
tmp=$(mktemp -d)

echo "Downloading ${asset}..."
curl -fsSL "$url" -o "$tmp/${asset}.tar.gz"
tar -xzf "$tmp/${asset}.tar.gz" -C "$tmp"

echo "Installing to ${INSTALL_DIR}/${BIN_NAME} (may prompt for sudo)..."
if [ -w "$INSTALL_DIR" ]; then
  mv "$tmp/${asset}" "$INSTALL_DIR/$BIN_NAME"
else
  sudo mv "$tmp/${asset}" "$INSTALL_DIR/$BIN_NAME"
fi
chmod +x "$INSTALL_DIR/$BIN_NAME"

rm -rf "$tmp"
echo "Installed! Run '${BIN_NAME}'."
