#!/bin/bash
set -e
TOOL="envelope-encrypt"
REPO="oobujieshi/alibabacloud-kms-skills-cli"
CACHE="${HOME}/.cache/alibabacloud-kms-skills"
mkdir -p "$CACHE"

case "$(uname -s)" in
  Linux)  OS="linux" ;; Darwin) OS="darwin" ;;
  CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
  *) echo "Unsupported OS" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ARCH="amd64" ;; aarch64|arm64) ARCH="arm64" ;;
  *) echo "Unsupported arch" >&2; exit 1 ;;
esac
[ "$OS" = "windows" ] && SUFFIX="windows-amd64.exe" || SUFFIX="${OS}-${ARCH}"

BIN="$CACHE/${TOOL}-${SUFFIX}"
if [ ! -f "$BIN" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/${TOOL}-${SUFFIX}"
  curl -sL "$URL" -o "$BIN" || wget -q "$URL" -O "$BIN"
  chmod +x "$BIN" 2>/dev/null || true
fi
exec "$BIN" "$@"
