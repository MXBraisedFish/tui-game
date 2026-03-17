@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"
set "LANG_CODE=us-en"
set "LANG_FILE=!SCRIPT_DIR!\assets\bash_lang\us-en.json"

if exist "!SCRIPT_DIR!\tui-game-data\language_pref.txt" (
    set /p LANG_CODE=<"!SCRIPT_DIR!\tui-game-data\language_pref.txt"
)
if exist "!SCRIPT_DIR!\assets\bash_lang\!LANG_CODE!.json" (
    set "LANG_FILE=!SCRIPT_DIR!\assets\bash_lang\!LANG_CODE!.json"
)

set "MAIN_BIN=!SCRIPT_DIR!\tui-game.exe"

if "%~1"=="" (
    if not exist "!MAIN_BIN!" (
        call :msg script.error.main_missing "Main game binary not found."
        exit /b 1
    )
    "!MAIN_BIN!"
    exit /b %errorlevel%
)

set "ARG=%~1"
if /I "!ARG!"=="-v" goto run_version
if /I "!ARG!"=="-version" goto run_version
if /I "!ARG!"=="-h" goto show_help
if /I "!ARG!"=="-help" goto show_help
if /I "!ARG!"=="-p" goto show_path
if /I "!ARG!"=="-path" goto show_path

call :msg script.error.unknown_arg "Unknown argument: {arg}" "!ARG!"
call :msg script.hint.try_help "Try 'tg -h' for usage."
exit /b 1

:run_version
if not exist "!MAIN_BIN!" (
    call :msg script.error.main_missing "Main game binary not found."
    exit /b 1
)

set "CUR_VER="
for /f "usebackq delims=" %%I in (`"!MAIN_BIN!" --runtime-version`) do (
    set "CUR_VER=%%I"
    goto got_cur_ver
)
:got_cur_ver
if not defined CUR_VER set "CUR_VER=v0.0.0"
call :msg version.current "Current version: {version}" "!CUR_VER!"

set "LATEST_VER="
for /f "usebackq delims=" %%I in (`powershell -NoProfile -ExecutionPolicy Bypass -Command "try { $r = Invoke-RestMethod -Uri 'https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest' -TimeoutSec 10 -Headers @{ 'User-Agent'='tui-game-tg' }; if ($r.tag_name) { $r.tag_name } } catch { '' }"`) do (
    set "LATEST_VER=%%I"
)

if not defined LATEST_VER (
    call :msg version.check_failed "Failed to check the latest release."
    exit /b 0
)

call :msg version.latest "Latest release: {version}" "!LATEST_VER!"

set "HAS_UPDATE=0"
for /f "usebackq delims=" %%I in (`powershell -NoProfile -ExecutionPolicy Bypass -Command "function Parse([string]$v){ $vv=$v.Trim(); if($vv.StartsWith('v') -or $vv.StartsWith('V')){ $vv=$vv.Substring(1) }; $parts=$vv.Split('.'); [int[]]@(($parts | ForEach-Object { if($_ -match '^\d+$'){ [int]$_ } else {0} })) }; $c=Parse('!CUR_VER!'); $l=Parse('!LATEST_VER!'); $n=[Math]::Max($c.Length,$l.Length); for($i=0;$i -lt $n;$i++){ $cv=if($i -lt $c.Length){$c[$i]}else{0}; $lv=if($i -lt $l.Length){$l[$i]}else{0}; if($lv -gt $cv){ '1'; exit 0 }; if($lv -lt $cv){ '0'; exit 0 } }; '0'"`) do (
    set "HAS_UPDATE=%%I"
)

if "!HAS_UPDATE!"=="1" (
    call :msg version.update_available "Update available."
) else (
    call :msg version.up_to_date "Already up to date."
)
exit /b 0

:show_help
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$file='%LANG_FILE%';" ^
    "$defaults = @(" ^
    "  @('script.help.header','Usage: tg [option]')," ^
    "  @('script.help.run','  tg                 Start the game.')," ^
    "  @('script.help.version','  tg -v              Show current version and check latest release.')," ^
    "  @('script.help.help','  tg -h              Show this help message.')," ^
    "  @('script.help.path','  tg -p              Show the installation path.')," ^
    "  @('script.help.footer','Long options: -version / -help / -path')" ^
    ");" ^
    "try {" ^
    "  $json = $null;" ^
    "  if (Test-Path $file) { $json = Get-Content -Raw -Encoding UTF8 $file | ConvertFrom-Json }" ^
    "  $lines = foreach ($entry in $defaults) { $key = $entry[0]; $fallback = $entry[1]; if ($null -ne $json -and $json.PSObject.Properties[$key]) { [string]$json.PSObject.Properties[$key].Value } else { $fallback } };" ^
    "  [Console]::Write(($lines -join [Environment]::NewLine))" ^
    "} catch {" ^
    "  [Console]::Write((($defaults | ForEach-Object { $_[1] }) -join [Environment]::NewLine))" ^
    "}"
exit /b 0

:show_path
call :msg script.path "Install path: {path}" "!SCRIPT_DIR!"
exit /b 0

:msg
set "KEY=%~1"
set "FALLBACK=%~2"
set "ARG_REPL=%~3"
for /f "usebackq delims=" %%I in (`powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$key='%KEY%';" ^
    "$fallback='%FALLBACK%';" ^
    "$arg='%ARG_REPL%';" ^
    "$path='%SCRIPT_DIR%';" ^
    "$file='%LANG_FILE%';" ^
    "try {" ^
    "  $text = $null;" ^
    "  if (Test-Path $file) { $json = Get-Content -Raw -Encoding UTF8 $file | ConvertFrom-Json; $prop = $json.PSObject.Properties[$key]; if ($prop) { $text = [string]$prop.Value } }" ^
    "  if ([string]::IsNullOrWhiteSpace($text)) { $text = $fallback }" ^
    "  $text = $text.Replace('{arg}', $arg).Replace('{path}', $path).Replace('{version}', $arg);" ^
    "  Write-Output $text" ^
    "} catch { Write-Output $fallback }"`) do (
    echo %%I
)
exit /b 0
