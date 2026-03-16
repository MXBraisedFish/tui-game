#!/bin/sh
set +x
set +v
set -eu

INSTALL_DIR=${1:-}
[ -n "$INSTALL_DIR" ] || exit 1
ROOT_SCRIPT="$INSTALL_DIR/delete-tui-game.sh"
[ -f "$ROOT_SCRIPT" ] || exit 1
exec bash "$ROOT_SCRIPT"
