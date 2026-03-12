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
set "VERSION_BIN=!SCRIPT_DIR!\version.exe"
set "UPDATA_BIN=!SCRIPT_DIR!\updata.exe"
set "REMOVE_BIN=!SCRIPT_DIR!\remove.exe"

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
if /I "!ARG!"=="-u" goto run_updata
if /I "!ARG!"=="-updata" goto run_updata
if /I "!ARG!"=="-h" goto show_help
if /I "!ARG!"=="-help" goto show_help
if /I "!ARG!"=="-r" goto run_remove
if /I "!ARG!"=="-remove" goto run_remove
if /I "!ARG!"=="-p" goto show_path
if /I "!ARG!"=="-path" goto show_path

call :msg script.error.unknown_arg "Unknown argument: {arg}" "!ARG!"
call :msg script.hint.try_help "Try 'tg -h' for usage."
exit /b 1

:run_version
if not exist "!VERSION_BIN!" (
    call :msg script.error.version_missing "Version helper binary not found."
    exit /b 1
)
"!VERSION_BIN!"
exit /b %errorlevel%

:run_updata
if not exist "!UPDATA_BIN!" (
    call :msg script.error.updata_missing "Update helper binary not found."
    exit /b 1
)
"!UPDATA_BIN!"
exit /b %errorlevel%

:run_remove
if not exist "!REMOVE_BIN!" (
    call :msg script.error.remove_missing "Remove helper binary not found."
    exit /b 1
)
"!REMOVE_BIN!"
exit /b %errorlevel%

:show_help
call :msg script.help.header "Usage: tg [option]"
call :msg script.help.run "  tg                 Start the game."
call :msg script.help.version "  tg -v              Show current version and check latest release."
call :msg script.help.update "  tg -u              Check and install updates if available."
call :msg script.help.help "  tg -h              Show this help message."
call :msg script.help.remove "  tg -r              Uninstall the game."
call :msg script.help.path "  tg -p              Show the installation path."
call :msg script.help.footer "Long options: -version / -updata / -help / -remove / -path"
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
    "  $text = $text.Replace('{arg}', $arg).Replace('{path}', $path);" ^
    "  Write-Output $text" ^
    "} catch { Write-Output $fallback }"`) do (
    echo %%I
)
exit /b 0
