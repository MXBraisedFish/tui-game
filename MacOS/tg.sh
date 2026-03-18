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
    text=${text//\{version\}/$arg}
    text=${text//\{path\}/$SCRIPT_DIR}
    printf '%s\n' "$text"
}

parse_ver() {
    v="$1"
    v=${v#v}
    v=${v#V}
    IFS='.' read -r a b c d <<EOF
$v
EOF
    printf '%s %s %s %s\n' "${a:-0}" "${b:-0}" "${c:-0}" "${d:-0}"
}

is_latest_newer() {
    cur="$1"
    latest="$2"
    set -- $(parse_ver "$cur")
    c1=$1; c2=$2; c3=$3; c4=$4
    set -- $(parse_ver "$latest")
    l1=$1; l2=$2; l3=$3; l4=$4
    [ "$l1" -gt "$c1" ] && return 0
    [ "$l1" -lt "$c1" ] && return 1
    [ "$l2" -gt "$c2" ] && return 0
    [ "$l2" -lt "$c2" ] && return 1
    [ "$l3" -gt "$c3" ] && return 0
    [ "$l3" -lt "$c3" ] && return 1
    [ "$l4" -gt "$c4" ] && return 0
    return 1
}

fetch_latest_tag() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -H "User-Agent: tui-game-tg" "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest" \
            | grep -m1 '"tag_name"' \
            | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/'
        return
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -qO- --header="User-Agent: tui-game-tg" "https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest" \
            | grep -m1 '"tag_name"' \
            | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/'
        return
    fi
    return 1
}

MAIN_BIN="$SCRIPT_DIR/tui-game"

if [ $# -eq 0 ]; then
    [ -x "$MAIN_BIN" ] || { msg "script.error.main_missing" "Main game binary not found."; exit 1; }
    "$MAIN_BIN"
    exit $?
fi

case "$1" in
    -v|-V|-version)
        [ -x "$MAIN_BIN" ] || { msg "script.error.main_missing" "Main game binary not found."; exit 1; }
        CUR_VER="$($MAIN_BIN --runtime-version 2>/dev/null || true)"
        [ -n "$CUR_VER" ] || CUR_VER="v0.0.0"
        msg "version.current" "Current version: {version}" "$CUR_VER"

        LATEST_VER="$(fetch_latest_tag 2>/dev/null || true)"
        if [ -z "$LATEST_VER" ]; then
            msg "version.check_failed" "Failed to check the latest release."
            exit 0
        fi

        msg "version.latest" "Latest release: {version}" "$LATEST_VER"
        if is_latest_newer "$CUR_VER" "$LATEST_VER"; then
            msg "version.update_available" "Update available."
        else
            msg "version.up_to_date" "Already up to date."
        fi
        ;;
    -h|-H|-help)
        help_text=$(
            printf '%s\n%s\n%s\n%s\n%s\n%s' \
                "$(msg "script.help.header" "Usage: tg [option]")" \
                "$(msg "script.help.run" "  tg                 Start the game.")" \
                "$(msg "script.help.version" "  tg -v              Show current version and check latest release.")" \
                "$(msg "script.help.help" "  tg -h              Show this help message.")" \
                "$(msg "script.help.path" "  tg -p              Show the installation path.")" \
                "$(msg "script.help.footer" "Long options: -version / -help / -path")"
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
