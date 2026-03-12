#!/bin/bash
set +x
set +v
set -u

echo "[WARNING] This will delete all game data, including saves and records."
echo "[WARNING] Deletion is permanent and cannot be restored."
echo "Continue? (Y/N):"
read -r CONTINUE_UNINSTALL
if [[ ! "$CONTINUE_UNINSTALL" =~ ^[Yy]$ ]]; then
    echo "[INFO] Uninstall cancelled."
    echo "[INFO] Press any key to exit."
    read -n1 -r
    rm -f "$0"
    exit 0
fi

SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SOURCE" ]; do
    DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
    TARGET="$(readlink "$SOURCE")"
    if [[ "$TARGET" != /* ]]; then
        SOURCE="$DIR/$TARGET"
    else
        SOURCE="$TARGET"
    fi
done
SCRIPT_DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

INTEGRATION_CLEAN_OK=0

delete_file() {
    local file="$1"
    if [ -f "$file" ]; then
        rm -f "$file" >/dev/null 2>&1 || true
    fi
}

delete_dir() {
    local dir="$1"
    if [ -d "$dir" ]; then
        rm -rf "$dir" >/dev/null 2>&1 || true
    fi
}

echo "Use system cleanup for environment variable? (Y/N):"
read -r CLEAN_ENV
if [[ "$CLEAN_ENV" =~ ^[Yy]$ ]]; then
    echo "[INFO] Cleaning system integration..."
    clean_failed=0
    expected="$SCRIPT_DIR/tg.sh"

    for link in "/usr/local/bin/tg" "$HOME/bin/tg" "$HOME/.local/bin/tg"; do
        if [ -L "$link" ]; then
            target="$(readlink "$link" 2>/dev/null || true)"
            if [ -z "$target" ]; then
                continue
            fi
            if [[ "$target" != /* ]]; then
                target="$(cd -P "$(dirname "$link")" && pwd)/$target"
            fi
            target_dir="$(cd -P "$(dirname "$target")" 2>/dev/null && pwd || true)"
            if [ -z "$target_dir" ]; then
                continue
            fi
            resolved="$target_dir/$(basename "$target")"
            if [ "$resolved" = "$expected" ]; then
                rm -f "$link" >/dev/null 2>&1 || clean_failed=1
            fi
        fi
    done

    app_link="$HOME/Applications/TUI-GAME"
    if [ -L "$app_link" ]; then
        rm -f "$app_link" >/dev/null 2>&1 || clean_failed=1
    fi

    if [ "$clean_failed" -eq 0 ]; then
        INTEGRATION_CLEAN_OK=1
        echo "[INFO] Environment variable cleanup completed."
    else
        echo "[WARNING] Failed to clean environment variable automatically."
    fi
else
    echo "[INFO] Environment variable cleanup skipped."
fi

delete_file "$SCRIPT_DIR/tg.sh"
delete_file "$SCRIPT_DIR/version.sh"
delete_file "$SCRIPT_DIR/tui-game"
delete_dir "$SCRIPT_DIR/assets"
delete_dir "$SCRIPT_DIR/scripts"
delete_dir "$SCRIPT_DIR/tui-game-data"

if [ "$INTEGRATION_CLEAN_OK" -ne 1 ]; then
    echo "[WARNING] Current environment variable was not cleaned. Manual cleanup is recommended."
fi

echo
echo "Bye bye."
echo
echo "[INFO] Press any key to exit."
read -n1 -r
rm -f "$0"
exit 0
