#!/bin/bash
set +x
set +v
set -u
echo "[INFO] Starting update process..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || { echo "[ERROR] Failed to change to script directory."; exit 1; }

if ! command -v curl >/dev/null 2>&1; then
    echo "[ERROR] curl is required but not found."
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
    echo "[ERROR] python3 is required but not found."
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
    echo "[ERROR] tar is required but not found."
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 1: Fetch latest release info
echo "[INFO] Fetching latest release information from GitHub..."
API_URL="https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
TEMP_JSON=$(mktemp)

if ! curl -s -L -o "$TEMP_JSON" "$API_URL"; then
    echo "[ERROR] Failed to download release information. Check your internet connection."
    rm -f "$TEMP_JSON"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 2: Extract Linux asset download URL
echo "[INFO] Extracting download URL for Linux package..."
ASSET_NAME="tui-game-linux.tar.gz"
DOWNLOAD_URL=$(python3 -c "
import sys, json
try:
    with open('$TEMP_JSON') as f:
        data = json.load(f)
    for asset in data.get('assets', []):
        if asset.get('name') == '$ASSET_NAME':
            print(asset.get('browser_download_url', ''))
            break
except Exception:
    pass
")

if [ -z "$DOWNLOAD_URL" ]; then
    echo "[ERROR] Could not find Linux asset '$ASSET_NAME' in the latest release."
    rm -f "$TEMP_JSON"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi
echo "[INFO] Download URL: $DOWNLOAD_URL"
rm -f "$TEMP_JSON"

# Step 3: Download the package
echo "[INFO] Downloading update package..."
TEMP_TGZ=$(mktemp).tar.gz
if ! curl -s -L -o "$TEMP_TGZ" "$DOWNLOAD_URL"; then
    echo "[ERROR] Failed to download update package."
    rm -f "$TEMP_TGZ"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 4: Extract to current directory, overwriting
echo "[INFO] Extracting update to current directory (overwriting files)..."
if ! tar -xzf "$TEMP_TGZ" -C "$SCRIPT_DIR"; then
    echo "[ERROR] Failed to extract update package."
    rm -f "$TEMP_TGZ"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 5: Clean up temporary files
rm -f "$TEMP_TGZ"
echo "[INFO] Temporary files cleaned up."

# Step 6: Ensure extracted files have execute permissions (binary + helper scripts)
chmod +x "$SCRIPT_DIR/tui-game" "$SCRIPT_DIR"/*.sh 2>/dev/null || true

echo "[SUCCESS] Update completed successfully!"
echo "[INFO] Press any key to exit."
read -n1 -r
exit 0
