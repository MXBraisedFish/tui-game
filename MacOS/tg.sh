#!/bin/bash
set +x
set +v
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
LANG_CODE="us-en"
if [ -f "$SCRIPT_DIR/tui-game-data/language_pref.txt" ]; then
    LANG_CODE=$(tr -d '\r\n' < "$SCRIPT_DIR/tui-game-data/language_pref.txt")
fi
LANG_FILE="$SCRIPT_DIR/assets/bash_lang/$LANG_CODE.json"
[ -f "$LANG_FILE" ] || LANG_FILE="$SCRIPT_DIR/assets/bash_lang/us-en.json"

msg() {
    key="$1"
    fallback="$2"
    arg="${3:-}"
    line=$(grep -m1 -E "^[[:space:]]*\"$key\"[[:space:]]*:" "$LANG_FILE" 2>/dev/null || true)
    if [ -n "$line" ]; then
        text=$(printf '%s' "$line" | sed -E 's/^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*"(.*)"[[:space:]]*,?[[:space:]]*$/\1/')
        text=${text//\\\"/\"}
        text=${text//\\n/$'\n'}
    else
        text="$fallback"
    fi
    text=${text//\{arg\}/$arg}
    text=${text//\{path\}/$SCRIPT_DIR}
    printf '%s\n' "$text"
}

MAIN_BIN="$SCRIPT_DIR/tui-game"
VERSION_BIN="$SCRIPT_DIR/version"
UPDATA_BIN="$SCRIPT_DIR/updata"
REMOVE_BIN="$SCRIPT_DIR/remove"

if [ $# -eq 0 ]; then
    [ -x "$MAIN_BIN" ] || { msg "script.error.main_missing" "Main game binary not found."; exit 1; }
    exec "$MAIN_BIN"
fi

case "$1" in
    -v|-V|-version)
        [ -x "$VERSION_BIN" ] || { msg "script.error.version_missing" "Version helper binary not found."; exit 1; }
        exec "$VERSION_BIN"
        ;;
    -u|-U|-updata)
        [ -x "$UPDATA_BIN" ] || { msg "script.error.updata_missing" "Update helper binary not found."; exit 1; }
        exec "$UPDATA_BIN"
        ;;
    -r|-R|-remove)
        [ -x "$REMOVE_BIN" ] || { msg "script.error.remove_missing" "Remove helper binary not found."; exit 1; }
        exec "$REMOVE_BIN"
        ;;
    -h|-H|-help)
        help_text=$(
            printf '%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s' \
                "$(msg "script.help.header" "Usage: tg [option]")" \
                "$(msg "script.help.run" "  tg                 Start the game.")" \
                "$(msg "script.help.version" "  tg -v              Show current version and check latest release.")" \
                "$(msg "script.help.update" "  tg -u              Check and install updates if available.")" \
                "$(msg "script.help.help" "  tg -h              Show this help message.")" \
                "$(msg "script.help.remove" "  tg -r              Uninstall the game.")" \
                "$(msg "script.help.path" "  tg -p              Show the installation path.")" \
                "$(msg "script.help.footer" "Long options: -version / -updata / -help / -remove / -path")"
        )
        printf '%s\n' "$help_text"
        ;;
    -p|-P|-path)
        msg "script.path" "Install path: {path}"
        ;;
    *)
        msg "script.error.unknown_arg" "Unknown argument: {arg}" "$1"
        msg "script.hint.try_help" "Try 'tg -h' for usage."
        exit 1
        ;;
esac

