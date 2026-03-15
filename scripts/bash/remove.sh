#!/bin/sh
set +x
set +v
set -eu

INSTALL_DIR=${1:-}
[ -n "$INSTALL_DIR" ] || exit 1

LANG_CODE="us-en"
if [ -f "$INSTALL_DIR/tui-game-data/language_pref.txt" ]; then
  LANG_CODE=$(cat "$INSTALL_DIR/tui-game-data/language_pref.txt" 2>/dev/null || printf '%s' "us-en")
fi

if [ "$LANG_CODE" = "zh-cn" ]; then
  MSG_CONFIRM1="这将卸载 TUI-GAME，是否继续？ [y/N]"
  MSG_MODE="选择卸载模式：[1] 保留存档  [2] 删除全部数据"
  MSG_CONFIRM2="确认以“%s”模式卸载？ [y/N]"
  MSG_MODE_KEEP="保留存档"
  MSG_MODE_FULL="删除全部数据"
  MSG_CANCELLED="已取消卸载。"
  MSG_START="正在启动卸载程序……"
  MSG_DONE="卸载任务已启动。"
else
  MSG_CONFIRM1="This will uninstall TUI-GAME. Continue? [y/N]"
  MSG_MODE="Choose uninstall mode: [1] Keep saves  [2] Delete all data"
  MSG_CONFIRM2="Confirm uninstall in mode \"%s\"? [y/N]"
  MSG_MODE_KEEP="Keep saves"
  MSG_MODE_FULL="Delete all data"
  MSG_CANCELLED="Uninstall cancelled."
  MSG_START="Starting uninstall..."
  MSG_DONE="Uninstall task started."
fi

printf '%s ' "$MSG_CONFIRM1"
read -r confirm1
case "$confirm1" in
  y|Y) ;;
  *) printf '%s\n' "$MSG_CANCELLED"; exit 0 ;;
esac

printf '%s\n> ' "$MSG_MODE"
read -r mode
case "$mode" in
  1) DELETE_DATA=0; MODE_TEXT="$MSG_MODE_KEEP" ;;
  2) DELETE_DATA=1; MODE_TEXT="$MSG_MODE_FULL" ;;
  *) printf '%s\n' "$MSG_CANCELLED"; exit 0 ;;
esac

printf "$MSG_CONFIRM2 " "$MODE_TEXT"
read -r confirm2
case "$confirm2" in
  y|Y) ;;
  *) printf '%s\n' "$MSG_CANCELLED"; exit 0 ;;
esac

printf '%s\n' "$MSG_START"

HELPER=$(mktemp "${TMPDIR:-/tmp}/tui-game-remove-XXXXXX.sh")
cat >"$HELPER" <<EOF
#!/bin/sh
sleep 1
rm -f "$INSTALL_DIR/tg.sh" "$INSTALL_DIR/tui-game" "$INSTALL_DIR/version" "$INSTALL_DIR/updata" "$INSTALL_DIR/remove" 2>/dev/null || true
rm -rf "$INSTALL_DIR/assets" "$INSTALL_DIR/scripts" 2>/dev/null || true
EOF
if [ "$DELETE_DATA" = "1" ]; then
  cat >>"$HELPER" <<EOF
rm -rf "$INSTALL_DIR/tui-game-data" 2>/dev/null || true
EOF
fi
cat >>"$HELPER" <<EOF
rm -f "$HELPER" 2>/dev/null || true
exit 0
EOF

chmod +x "$HELPER"
nohup sh "$HELPER" >/dev/null 2>&1 &

printf '%s\n' "$MSG_DONE"
exit 0
