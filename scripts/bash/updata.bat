@echo off
setlocal enabledelayedexpansion

set "INSTALL_DIR=%~1"
set "ASSET_URL=%~2"
set "ASSET_NAME=%~3"
set "LATEST_VERSION=%~4"

if "%INSTALL_DIR%"=="" exit /b 1
if "%ASSET_URL%"=="" exit /b 1
if "%LATEST_VERSION%"=="" exit /b 1

set "TEMP_FILE=%temp%\tui-game-updata-%random%"
if /I "%ASSET_NAME:~-4%"==".zip" (
    set "TEMP_FILE=%TEMP_FILE%.zip"
) else (
    set "TEMP_FILE=%TEMP_FILE%.pkg"
)

where curl >nul 2>nul || exit /b 1
curl -L --fail --silent --show-error -o "%TEMP_FILE%" "%ASSET_URL%" || exit /b 1

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$ErrorActionPreference='Stop';" ^
    "$installDir = [System.IO.Path]::GetFullPath('%INSTALL_DIR%');" ^
    "$tempFile = [System.IO.Path]::GetFullPath('%TEMP_FILE%');" ^
    "if ($tempFile.ToLower().EndsWith('.zip')) { Expand-Archive -Path $tempFile -DestinationPath $installDir -Force } else { throw 'Unsupported package format.' }" ^
    "$cachePath = Join-Path $installDir 'tui-game-data\updater_cache.json';" ^
    "$cacheDir = Split-Path $cachePath -Parent;" ^
    "New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null;" ^
    "Set-Content -Path $cachePath -Value ('\"' + '%LATEST_VERSION%' + '\"') -Encoding UTF8"
if errorlevel 1 exit /b 1

del /f /q "%TEMP_FILE%" >nul 2>&1
exit /b 0
