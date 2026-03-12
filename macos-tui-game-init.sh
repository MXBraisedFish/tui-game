#!/bin/bash
set -eu

printf '%s\n' '[1] 中文' '[2] English'
read -r -p 'Select language / 选择语言 (1/2): ' CHOICE
if [[ "$CHOICE" == "1" ]]; then
    LANG_CODE='zh-cn'
else
    LANG_CODE='us-en'
fi

if [[ "$LANG_CODE" == 'zh-cn' ]]; then
    MSG_START='[信息] 开始安装 TUI-GAME...'
    MSG_FETCH='[信息] 正在从 GitHub 获取最新版本信息...'
    MSG_PARSE='[信息] 正在解析 macOS 安装包下载链接...'
    MSG_DL='[信息] 正在下载安装包...'
    MSG_EXTRACT='[信息] 正在解压文件到当前目录...'
    MSG_LANG_INIT='[信息] 正在初始化语言设置...'
    MSG_CLEAN='[信息] 已清理临时文件。'
    MSG_ASK_PATH='是否创建 tg 快捷启动命令？(Y/N): '
    MSG_LINK_OK='[成功] 已创建 tg 快捷启动命令。'
    MSG_LINK_SKIP='[信息] 跳过快捷命令创建。'
    MSG_DONE='[成功] TUI-GAME 安装完成。'
    MSG_RUN='[信息] 你现在可以输入 tg 启动游戏。'
    ERR_CURL='[错误] 未找到 curl。'
    ERR_PY='[错误] 未找到 python3。'
    ERR_UNZIP='[错误] 未找到 unzip。'
    ERR_FETCH='[错误] 下载版本信息失败。'
    ERR_ASSET='[错误] 未找到 macOS 安装包。'
    ERR_DL='[错误] 下载安装包失败。'
    ERR_EXTRACT='[错误] 解压安装包失败。'
    MSG_EXIT='[信息] 按任意键退出并删除安装脚本。'
else
    MSG_START='[INFO] Starting TUI-GAME installation...'
    MSG_FETCH='[INFO] Fetching latest release information from GitHub...'
    MSG_PARSE='[INFO] Extracting macOS package download URL...'
    MSG_DL='[INFO] Downloading package...'
    MSG_EXTRACT='[INFO] Extracting files to current directory...'
    MSG_LANG_INIT='[INFO] Initializing language preference...'
    MSG_CLEAN='[INFO] Temporary files cleaned up.'
    MSG_ASK_PATH='Create a tg launcher command? (Y/N): '
    MSG_LINK_OK='[SUCCESS] tg launcher command created.'
    MSG_LINK_SKIP='[INFO] Skipping launcher creation.'
    MSG_DONE='[SUCCESS] TUI-GAME has been installed.'
    MSG_RUN='[INFO] You can now type tg to start the game.'
    ERR_CURL='[ERROR] curl was not found.'
    ERR_PY='[ERROR] python3 was not found.'
    ERR_UNZIP='[ERROR] unzip was not found.'
    ERR_FETCH='[ERROR] Failed to download release information.'
    ERR_ASSET='[ERROR] macOS package asset was not found.'
    ERR_DL='[ERROR] Failed to download the package.'
    ERR_EXTRACT='[ERROR] Failed to extract the package.'
    MSG_EXIT='[INFO] Press any key to exit and delete this installer.'
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "$MSG_START"
command -v curl >/dev/null 2>&1 || { echo "$ERR_CURL"; read -n1 -r; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "$ERR_PY"; read -n1 -r; exit 1; }
command -v unzip >/dev/null 2>&1 || { echo "$ERR_UNZIP"; read -n1 -r; exit 1; }

API_URL='https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest'
TEMP_JSON="$(mktemp)"
TEMP_ZIP="$(mktemp).zip"

echo "$MSG_FETCH"
curl -fsSL -o "$TEMP_JSON" "$API_URL" || { echo "$ERR_FETCH"; rm -f "$TEMP_JSON"; read -n1 -r; exit 1; }

echo "$MSG_PARSE"
DOWNLOAD_URL=$(python3 - <<PY
import json
url = ''
with open(r'''$TEMP_JSON''', 'r', encoding='utf-8') as f:
    data = json.load(f)
for asset in data.get('assets', []):
    if asset.get('name') == 'tui-game-macos.zip':
        url = asset.get('browser_download_url', '')
        break
print(url)
PY
)
if [[ -z "$DOWNLOAD_URL" ]]; then
    echo "$ERR_ASSET"
    rm -f "$TEMP_JSON"
    read -n1 -r
    exit 1
fi

echo "$MSG_DL"
curl -fsSL -o "$TEMP_ZIP" "$DOWNLOAD_URL" || { echo "$ERR_DL"; rm -f "$TEMP_JSON" "$TEMP_ZIP"; read -n1 -r; exit 1; }

echo "$MSG_EXTRACT"
unzip -oq "$TEMP_ZIP" -d "$SCRIPT_DIR" || { echo "$ERR_EXTRACT"; rm -f "$TEMP_JSON" "$TEMP_ZIP"; read -n1 -r; exit 1; }

echo "$MSG_LANG_INIT"
mkdir -p "$SCRIPT_DIR/tui-game-data"
printf '%s\n' "$LANG_CODE" > "$SCRIPT_DIR/tui-game-data/language_pref.txt"

chmod +x "$SCRIPT_DIR"/tui-game "$SCRIPT_DIR"/version "$SCRIPT_DIR"/updata "$SCRIPT_DIR"/remove "$SCRIPT_DIR"/*.sh "$SCRIPT_DIR"/scripts/bash/*.sh 2>/dev/null || true

rm -f "$TEMP_JSON" "$TEMP_ZIP"
echo "$MSG_CLEAN"

echo
read -r -p "$MSG_ASK_PATH" ADD_PATH
if [[ "$ADD_PATH" =~ ^[Yy]$ ]]; then
    mkdir -p "$HOME/bin"
    ln -sf "$SCRIPT_DIR/tg.sh" "$HOME/bin/tg"
    echo "$MSG_LINK_OK"
else
    echo "$MSG_LINK_SKIP"
fi

echo
echo "$MSG_DONE"
echo "$MSG_RUN"
echo
echo "$MSG_EXIT"
read -n1 -r
rm -f "$0"
exit 0
