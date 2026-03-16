#!/bin/bash
set +x
set +v
set -eu

printf '%s\n' '[1] ??' '[2] English'
read -r -p '???? / Select language (1/2): ' CHOICE
if [[ "$CHOICE" == "1" ]]; then
    LANG_CODE='zh-cn'
else
    LANG_CODE='us-en'
fi

ROOT_LINE='========================================'
TO_HTTP='https://github.com/MXBraisedFish/TUI-GAME'
TO_WEB='none'

if [[ "$LANG_CODE" == 'zh-cn' ]]; then
    MSG_START='[??] ???? TUI-GAME...'
    MSG_FETCH='[??] ??? GitHub ????????...'
    MSG_PARSE='[??] ???? macOS ???????...'
    MSG_DL='[??] ???????...'
    MSG_EXTRACT='[??] ???????????...'
    MSG_LANG_INIT='[??] ?????????...'
    MSG_CLEAN='[??] ????????'
    MSG_ASK_PATH='???? tg ????????? PATH ?????(Y/N): '
    MSG_ADD_PATH='[??] ???? shell ??...'
    MSG_PATH_OK='[??] ??? tg ??????????????????? tg?'
    MSG_PATH_SKIP='[??] ?? PATH ???'
    MSG_DONE='[??] TUI-GAME ?????'
    MSG_RUN='[??] ??????? tg ?????'
    MSG_MORE='[??] ??? tg -h ???????'
    ERR_CURL='[??] ??? curl?'
    ERR_PY='[??] ??? python3?'
    ERR_UNZIP='[??] ??? unzip?'
    ERR_FETCH='[??] ?????????????'
    ERR_ASSET='[??] ??? macOS ????'
    ERR_DL='[??] ????????????'
    ERR_EXTRACT='[??] ????????'
    ERR_PATH='[??] PATH ???????????'
    ERR_NO_PATH='[??] ??? PATH ???????????'
    ERR_WHY_PATH='[??] ?? PATH ????????????????'
    MSG_EXIT='[??] ???????'
    ROOT_THANKS='????????????????????????'
    ROOT_ENJOY='????????????:P'
    ROOT_HTTP='?????'
    ROOT_WEB='?????'
else
    MSG_START='[INFO] Starting TUI-GAME installation...'
    MSG_FETCH='[INFO] Fetching latest release information from GitHub...'
    MSG_PARSE='[INFO] Extracting macOS package download URL...'
    MSG_DL='[INFO] Downloading package...'
    MSG_EXTRACT='[INFO] Extracting files to current directory...'
    MSG_LANG_INIT='[INFO] Initializing language preference...'
    MSG_CLEAN='[INFO] Temporary files cleaned up.'
    MSG_ASK_PATH='Do you want to create a tg launcher and add it to PATH? (Y/N): '
    MSG_ADD_PATH='[INFO] Updating shell profile...'
    MSG_PATH_OK='[SUCCESS] tg launcher created. Reopen the terminal to use tg.'
    MSG_PATH_SKIP='[INFO] Skipping PATH registration.'
    MSG_DONE='[SUCCESS] TUI-GAME has been installed.'
    MSG_RUN='[INFO] You can now type tg to start the game.'
    MSG_MORE='[INFO] Or run tg -h to view command details.'
    ERR_CURL='[ERROR] curl was not found.'
    ERR_PY='[ERROR] python3 was not found.'
    ERR_UNZIP='[ERROR] unzip was not found.'
    ERR_FETCH='[ERROR] Failed to download release information. Error code: '
    ERR_ASSET='[ERROR] macOS package asset was not found.'
    ERR_DL='[ERROR] Failed to download the package. Error code: '
    ERR_EXTRACT='[ERROR] Failed to extract the package.'
    ERR_PATH='[WARNING] Failed to update PATH. Please add it manually.'
    ERR_NO_PATH='[WARNING] PATH environment variable not set. Please add it manually.'
    ERR_WHY_PATH='[WARNING] Adding the PATH environment variable allows you to use quick commands in the future.'
    MSG_EXIT='[INFO] Press any key to exit.'
    ROOT_THANKS='Thanks for downloading and playing! If you enjoy it, please give my repository a star.'
    ROOT_ENJOY='Enjoy your entertainment in the terminal. :P'
    ROOT_HTTP='Repository URL: '
    ROOT_WEB='Official website URL: '
fi

append_path_export() {
    local profile_file="$1"
    local launcher_dir="$2"
    if [[ ! -f "$profile_file" ]]; then
        : > "$profile_file"
    fi
    if ! grep -Fqs "$launcher_dir" "$profile_file" 2>/dev/null; then
        {
            printf '\n# TUI-GAME launcher\n'
            printf 'export PATH="%s:$PATH"\n' "$launcher_dir"
        } >> "$profile_file"
    fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LAUNCHER_DIR="$HOME/.local/bin"
LAUNCHER_PATH="$LAUNCHER_DIR/tg"
PROFILE_FILES=("$HOME/.zprofile" "$HOME/.zshrc" "$HOME/.profile")
cd "$SCRIPT_DIR"

echo "$MSG_START"
command -v curl >/dev/null 2>&1 || { echo "$ERR_CURL"; read -n1 -r; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "$ERR_PY"; read -n1 -r; exit 1; }
command -v unzip >/dev/null 2>&1 || { echo "$ERR_UNZIP"; read -n1 -r; exit 1; }

echo "$MSG_LANG_INIT"
mkdir -p "$SCRIPT_DIR/tui-game-data"
printf '%s\n' "$LANG_CODE" > "$SCRIPT_DIR/tui-game-data/language_pref.txt"

API_URL='https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest'
TEMP_JSON="$(mktemp)"
TEMP_TGZ="$(mktemp).zip"

echo "$MSG_FETCH"
if ! curl -fsSL -o "$TEMP_JSON" "$API_URL"; then
    CURL_CODE=$?
    echo "${ERR_FETCH}${CURL_CODE}"
    rm -f "$TEMP_JSON"
    read -n1 -r
    exit 1
fi

echo "$MSG_PARSE"
DOWNLOAD_URL=$(python3 - <<PY
import json
url = ''
with open(r"$TEMP_JSON", 'r', encoding='utf-8') as f:
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
if ! curl -fsSL -o "$TEMP_TGZ" "$DOWNLOAD_URL"; then
    CURL_CODE=$?
    echo "${ERR_DL}${CURL_CODE}"
    rm -f "$TEMP_JSON" "$TEMP_TGZ"
    read -n1 -r
    exit 1
fi

echo "$MSG_EXTRACT"
if ! unzip -oq "$TEMP_TGZ" -d "$SCRIPT_DIR"; then
    echo "$ERR_EXTRACT"
    rm -f "$TEMP_JSON" "$TEMP_TGZ"
    read -n1 -r
    exit 1
fi

chmod +x \
    "$SCRIPT_DIR/tui-game" \
    "$SCRIPT_DIR/version" \
    "$SCRIPT_DIR/updata" \
    "$SCRIPT_DIR/remove" \
    "$SCRIPT_DIR/tg.sh" \
    "$SCRIPT_DIR/delete-tui-game.sh" \
    "$SCRIPT_DIR"/*.sh \
    "$SCRIPT_DIR/scripts/bash"/*.sh 2>/dev/null || true

rm -f "$TEMP_JSON" "$TEMP_TGZ"
echo "$MSG_CLEAN"

echo
read -r -p "$MSG_ASK_PATH" ADD_PATH
if [[ "$ADD_PATH" =~ ^[Yy]$ ]]; then
    echo "$MSG_ADD_PATH"
    mkdir -p "$LAUNCHER_DIR"
    ln -sf "$SCRIPT_DIR/tg.sh" "$LAUNCHER_PATH"
    PATH_OK=1
    for profile_file in "${PROFILE_FILES[@]}"; do
        append_path_export "$profile_file" "$LAUNCHER_DIR" || PATH_OK=0
    done
    if [[ -L "$LAUNCHER_PATH" && "$PATH_OK" -eq 1 ]]; then
        echo "$MSG_PATH_OK"
    else
        echo "$ERR_PATH"
        ADD_PATH='N'
    fi
else
    echo "$MSG_PATH_SKIP"
fi

echo
echo "$MSG_DONE"
echo
echo "$ROOT_LINE"
echo "$ROOT_THANKS"
echo "$ROOT_ENJOY"
echo "${ROOT_HTTP}${TO_HTTP}"
if [[ "$TO_WEB" != 'none' ]]; then
    echo "${ROOT_WEB}${TO_WEB}"
fi
echo "$ROOT_LINE"
echo
if [[ ! "$ADD_PATH" =~ ^[Yy]$ ]]; then
    echo "$ERR_NO_PATH"
    echo "$ERR_WHY_PATH"
    echo
fi
echo "$MSG_RUN"
echo "$MSG_MORE"
echo
echo "$MSG_EXIT"
read -n1 -r
rm -f "$0"
exit 0
