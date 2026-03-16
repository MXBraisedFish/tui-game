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
set "EXTRACT_DIR=%temp%\tui-game-updata-extract-%random%"
if /I "%ASSET_NAME:~-4%"==".zip" (
    set "TEMP_FILE=%TEMP_FILE%.zip"
) else (
    set "TEMP_FILE=%TEMP_FILE%.pkg"
)

where curl >nul 2>nul || exit /b 1
curl -L --fail --silent --show-error -o "%TEMP_FILE%" "%ASSET_URL%" || exit /b 1

if exist "%EXTRACT_DIR%" rmdir /s /q "%EXTRACT_DIR%" >nul 2>&1
mkdir "%EXTRACT_DIR%" >nul 2>&1 || exit /b 1

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$ErrorActionPreference='Stop';" ^
    "$installDir = [System.IO.Path]::GetFullPath('%INSTALL_DIR%');" ^
    "$tempFile = [System.IO.Path]::GetFullPath('%TEMP_FILE%');" ^
    "$extractDir = [System.IO.Path]::GetFullPath('%EXTRACT_DIR%');" ^
    "$payloadRoot = $extractDir;" ^
    "function Replace-Dir([string]$src, [string]$dst) { if (Test-Path -LiteralPath $src) { if (Test-Path -LiteralPath $dst) { Remove-Item -LiteralPath $dst -Recurse -Force }; New-Item -ItemType Directory -Force -Path (Split-Path -Parent $dst) | Out-Null; Copy-Item -LiteralPath $src -Destination $dst -Recurse -Force } }" ^
    "if ($tempFile.ToLower().EndsWith('.zip')) { Expand-Archive -Path $tempFile -DestinationPath $extractDir -Force } else { throw 'Unsupported package format.' }" ^
    "$directMain = Join-Path $extractDir 'tui-game.exe';" ^
    "if (-not (Test-Path -LiteralPath $directMain)) { $children = @(Get-ChildItem -LiteralPath $extractDir -Force); if ($children.Count -eq 1 -and $children[0].PSIsContainer) { $payloadRoot = $children[0].FullName } }" ^
    "$mainSrc = Join-Path $payloadRoot 'tui-game.exe';" ^
    "$mainDst = Join-Path $installDir 'tui-game.exe';" ^
    "if (Test-Path -LiteralPath $mainSrc) { Copy-Item -LiteralPath $mainSrc -Destination $mainDst -Force }" ^
    "if (-not (Test-Path -LiteralPath $mainSrc) -and -not (Test-Path -LiteralPath (Join-Path $payloadRoot 'assets')) -and -not (Test-Path -LiteralPath (Join-Path $payloadRoot 'scripts'))) { throw 'Unsupported package layout.' }" ^
    "Replace-Dir (Join-Path $payloadRoot 'assets\\lang') (Join-Path $installDir 'assets\\lang');" ^
    "Replace-Dir (Join-Path $payloadRoot 'assets\\wordle') (Join-Path $installDir 'assets\\wordle');" ^
    "Replace-Dir (Join-Path $payloadRoot 'scripts\\game') (Join-Path $installDir 'scripts\\game');" ^
    "Replace-Dir (Join-Path $payloadRoot 'scripts\\text_function') (Join-Path $installDir 'scripts\\text_function');" ^
    "$cachePath = Join-Path $installDir 'tui-game-data\updater_cache.json';" ^
    "$cacheDir = Split-Path $cachePath -Parent;" ^
    "New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null;" ^
    "Set-Content -Path $cachePath -Value ('\"' + '%LATEST_VERSION%' + '\"') -Encoding UTF8"
if errorlevel 1 (
    rmdir /s /q "%EXTRACT_DIR%" >nul 2>&1
    del /f /q "%TEMP_FILE%" >nul 2>&1
    exit /b 1
)

rmdir /s /q "%EXTRACT_DIR%" >nul 2>&1
del /f /q "%TEMP_FILE%" >nul 2>&1
exit /b 0
