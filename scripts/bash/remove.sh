#!/bin/sh
set -eu

INSTALL_DIR=${1:-}
DELETE_DATA=${2:-0}
[ -n "$INSTALL_DIR" ] || exit 1

sleep 1
rm -f "$INSTALL_DIR/tg.sh" "$INSTALL_DIR/tui-game" "$INSTALL_DIR/version" "$INSTALL_DIR/updata" "$INSTALL_DIR/remove" 2>/dev/null || true
rm -rf "$INSTALL_DIR/assets" "$INSTALL_DIR/scripts" 2>/dev/null || true
if [ "$DELETE_DATA" = "1" ]; then
  rm -rf "$INSTALL_DIR/tui-game-data" 2>/dev/null || true
fi
rm -f "$0" 2>/dev/null || true
exit 0
