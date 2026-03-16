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
EXTRACT_DIR="$TMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"

case "$ASSET_NAME" in
  *.tar.gz) TMP_FILE="$TMP_DIR/package.tar.gz" ;;
  *.zip) TMP_FILE="$TMP_DIR/package.zip" ;;
  *) TMP_FILE="$TMP_DIR/package.bin" ;;
esac

curl -L --fail --silent --show-error -o "$TMP_FILE" "$ASSET_URL"

case "$ASSET_NAME" in
  *.tar.gz) tar -xzf "$TMP_FILE" -C "$EXTRACT_DIR" ;;
  *.zip) unzip -oq "$TMP_FILE" -d "$EXTRACT_DIR" ;;
  *) exit 1 ;;
esac

PAYLOAD_ROOT="$EXTRACT_DIR"
if [ ! -f "$PAYLOAD_ROOT/tui-game" ]; then
  set +e
  child_count=$(find "$EXTRACT_DIR" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')
  set -e
  if [ "$child_count" = "1" ]; then
    only_child=$(find "$EXTRACT_DIR" -mindepth 1 -maxdepth 1 | head -n 1)
    if [ -d "$only_child" ]; then
      PAYLOAD_ROOT="$only_child"
    fi
  fi
fi

replace_dir() {
  src_dir="$1"
  dst_dir="$2"
  if [ -d "$src_dir" ]; then
    rm -rf "$dst_dir"
    mkdir -p "$(dirname "$dst_dir")"
    cp -R "$src_dir" "$dst_dir"
  fi
}

if [ -f "$PAYLOAD_ROOT/tui-game" ]; then
  cp "$PAYLOAD_ROOT/tui-game" "$INSTALL_DIR/tui-game"
fi

replace_dir "$PAYLOAD_ROOT/assets/lang" "$INSTALL_DIR/assets/lang"
replace_dir "$PAYLOAD_ROOT/assets/wordle" "$INSTALL_DIR/assets/wordle"
replace_dir "$PAYLOAD_ROOT/scripts/game" "$INSTALL_DIR/scripts/game"
replace_dir "$PAYLOAD_ROOT/scripts/text_function" "$INSTALL_DIR/scripts/text_function"

mkdir -p "$INSTALL_DIR/tui-game-data"
printf '"%s"\n' "$LATEST_VERSION" > "$INSTALL_DIR/tui-game-data/updater_cache.json"

chmod +x "$INSTALL_DIR"/tui-game 2>/dev/null || true
exit 0
