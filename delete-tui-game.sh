#!/bin/bash
set +x
set +v
set -eu

INSTALL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LANG_CODE="us-en"
if [[ -f "$INSTALL_DIR/tui-game-data/language_pref.txt" ]]; then
    LANG_CODE="$(cat "$INSTALL_DIR/tui-game-data/language_pref.txt" 2>/dev/null || printf '%s' 'us-en')"
fi

if [[ "$LANG_CODE" == "zh-cn" ]]; then
    MSG_CONFIRM1='这将卸载 TUI-GAME，是否继续？ [Y/N]'
    MSG_MODE='选择卸载模式：[1] 保留存档  [2] 删除全部数据'
    MSG_CONFIRM2='确认以“%s”模式卸载？ [Y/N]'
    MSG_PATH='是否同时清理 PATH 注册？ [Y/N]'
    MSG_PATH_WARN='如果你手动修改过 PATH 条目，请自行删除，避免影响其它环境配置。'
    MSG_PATH_KEEP='保留 PATH 注册。'
    MSG_PATH_DONE='PATH 注册已清理。'
    MSG_PATH_FAIL='PATH 注册清理失败，请手动移除。'
    MSG_MODE_KEEP='保留存档'
    MSG_MODE_FULL='删除全部数据'
    MSG_CANCELLED='已取消卸载。'
    MSG_START='正在启动卸载程序……'
    MSG_DONE='卸载完成。'
    MSG_PRESS_KEY='按任意键完成卸载并移除卸载程序。'
else
    MSG_CONFIRM1='This will uninstall TUI-GAME. Continue? [Y/N]'
    MSG_MODE='Choose uninstall mode: [1] Keep saves  [2] Delete all data'
    MSG_CONFIRM2='Confirm uninstall in mode "%s"? [Y/N]'
    MSG_PATH='Clean PATH registration as well? [Y/N]'
    MSG_PATH_WARN='If you modified the PATH entry manually, please remove it yourself to avoid affecting other environment settings.'
    MSG_PATH_KEEP='Keeping PATH registration.'
    MSG_PATH_DONE='PATH registration cleaned.'
    MSG_PATH_FAIL='PATH registration cleanup failed. Please remove it manually.'
    MSG_MODE_KEEP='Keep saves'
    MSG_MODE_FULL='Delete all data'
    MSG_CANCELLED='Uninstall cancelled.'
    MSG_START='Starting uninstall...'
    MSG_DONE='Uninstall finished.'
    MSG_PRESS_KEY='Press any key to finish and remove the uninstaller.'
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
    1)
        DELETE_DATA=0
        MODE_TEXT="$MSG_MODE_KEEP"
        ;;
    2)
        DELETE_DATA=1
        MODE_TEXT="$MSG_MODE_FULL"
        ;;
    *)
        printf '%s\n' "$MSG_CANCELLED"
        exit 0
        ;;
esac

printf "$MSG_CONFIRM2 " "$MODE_TEXT"
read -r confirm2
case "$confirm2" in
    y|Y) ;;
    *) printf '%s\n' "$MSG_CANCELLED"; exit 0 ;;
esac

printf '%s\n' "$MSG_PATH_WARN"
printf '%s ' "$MSG_PATH"
read -r clean_path_answer
case "$clean_path_answer" in
    y|Y) CLEAN_PATH=1 ;;
    *) CLEAN_PATH=0 ;;
esac

printf '%s\n' "$MSG_START"
sleep 1

LAUNCHER_DIR="$HOME/.local/bin"
LAUNCHER_PATH="$LAUNCHER_DIR/tg"
PROFILE_FILES=("$HOME/.profile" "$HOME/.bashrc" "$HOME/.zprofile" "$HOME/.zshrc")

if [[ "$CLEAN_PATH" == "1" ]]; then
    rm -f "$LAUNCHER_PATH" 2>/dev/null || true
    cleanup_ok=1
    for profile_file in "${PROFILE_FILES[@]}"; do
        [[ -f "$profile_file" ]] || continue
        tmp_file="${profile_file}.tui-game-remove.$$"
        if ! python3 - <<PY "$profile_file" "$tmp_file" "$LAUNCHER_DIR"
from pathlib import Path
import sys
profile = Path(sys.argv[1])
out = Path(sys.argv[2])
launcher_dir = sys.argv[3]
text = profile.read_text(encoding='utf-8', errors='ignore')
lines = []
for line in text.splitlines():
    if '# TUI-GAME launcher' in line:
        continue
    if launcher_dir in line and 'export PATH=' in line:
        continue
    lines.append(line)
out.write_text('\n'.join(lines) + ('\n' if lines else ''), encoding='utf-8')
PY
        then
            cleanup_ok=0
            rm -f "$tmp_file" 2>/dev/null || true
            continue
        fi
        mv "$tmp_file" "$profile_file" || cleanup_ok=0
    done
    if [[ "$cleanup_ok" == "1" ]]; then
        printf '%s\n' "$MSG_PATH_DONE"
    else
        printf '%s\n' "$MSG_PATH_FAIL"
    fi
else
    rm -f "$LAUNCHER_PATH" 2>/dev/null || true
    printf '%s\n' "$MSG_PATH_KEEP"
fi

rm -f \
    "$INSTALL_DIR/tg.sh" \
    "$INSTALL_DIR/tg.bat" \
    "$INSTALL_DIR/tui-game" \
    "$INSTALL_DIR/tui-game.exe" \
    "$INSTALL_DIR/version" \
    "$INSTALL_DIR/version.exe" \
    "$INSTALL_DIR/updata" \
    "$INSTALL_DIR/updata.exe" 2>/dev/null || true
rm -rf "$INSTALL_DIR/assets" "$INSTALL_DIR/scripts" 2>/dev/null || true
if [[ "$DELETE_DATA" == "1" ]]; then
    rm -rf "$INSTALL_DIR/tui-game-data" 2>/dev/null || true
fi

printf '%s\n' "$MSG_DONE"
printf '%s\n' "$MSG_PRESS_KEY"
read -r -n 1 _
printf '\n'

HELPER="$(mktemp "${TMPDIR:-/tmp}/tui-game-remove-XXXXXX.sh")"
cat >"$HELPER" <<EOF
#!/bin/bash
sleep 1
rm -f "$INSTALL_DIR/remove" "$INSTALL_DIR/remove.exe" 2>/dev/null || true
rm -f "$INSTALL_DIR/delete-tui-game.sh" "$INSTALL_DIR/delete-tui-game.bat" 2>/dev/null || true
rm -f "$HELPER" 2>/dev/null || true
exit 0
EOF
chmod +x "$HELPER"
nohup bash "$HELPER" >/dev/null 2>&1 &

exit 0
