#!/bin/sh
set +x
set +v
set -eu

INSTALL_DIR=${1:-}
ASSET_URL=${2:-}
ASSET_NAME=${3:-}
LATEST_VERSION=${4:-}

[ -n "$INSTALL_DIR" ] || exit 1
[ -n "$ASSET_URL" ] || exit 1
[ -n "$LATEST_VERSION" ] || exit 1

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

case "$ASSET_NAME" in
  *.tar.gz) TMP_FILE="$TMP_DIR/package.tar.gz" ;;
  *.zip) TMP_FILE="$TMP_DIR/package.zip" ;;
  *) TMP_FILE="$TMP_DIR/package.bin" ;;
esac

curl -L --fail --silent --show-error -o "$TMP_FILE" "$ASSET_URL"

case "$ASSET_NAME" in
  *.tar.gz) tar -xzf "$TMP_FILE" -C "$INSTALL_DIR" ;;
  *.zip) unzip -oq "$TMP_FILE" -d "$INSTALL_DIR" ;;
  *) exit 1 ;;
esac

mkdir -p "$INSTALL_DIR/tui-game-data"
printf '"%s"\n' "$LATEST_VERSION" > "$INSTALL_DIR/tui-game-data/updater_cache.json"

chmod +x "$INSTALL_DIR"/*.sh "$INSTALL_DIR"/tui-game "$INSTALL_DIR"/version "$INSTALL_DIR"/updata "$INSTALL_DIR"/remove 2>/dev/null || true
exit 0
