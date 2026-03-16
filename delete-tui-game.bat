@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul

set "INSTALL_DIR=%~dp0"
if "!INSTALL_DIR:~-1!"=="\" set "INSTALL_DIR=!INSTALL_DIR:~0,-1!"

set "LANG_CODE=us-en"
if exist "%INSTALL_DIR%\tui-game-data\language_pref.txt" (
    set /p LANG_CODE=<"%INSTALL_DIR%\tui-game-data\language_pref.txt"
)

set "MSG_CONFIRM1=This will uninstall TUI-GAME. Continue? [Y/N]"
set "MSG_MODE=Choose uninstall mode: [1] Keep saves  [2] Delete all data"
set "MSG_CONFIRM2=Confirm uninstall in mode ""{mode}""? [Y/N]"
set "MSG_PATH=Clean PATH registration as well? [Y/N]"
set "MSG_PATH_WARN=If you modified the PATH entry manually, please remove it yourself to avoid affecting other environment settings."
set "MSG_PATH_KEEP=Keeping PATH registration."
set "MSG_PATH_DONE=PATH registration cleaned."
set "MSG_PATH_FAIL=PATH registration cleanup failed. Please remove it manually."
set "MSG_MODE_KEEP=Keep saves"
set "MSG_MODE_FULL=Delete all data"
set "MSG_CANCELLED=Uninstall cancelled."
set "MSG_START=Starting uninstall..."
set "MSG_DONE=Uninstall finished."
set "MSG_PRESS_KEY=Press any key to finish and remove the uninstaller."

if /I "%LANG_CODE%"=="zh-cn" (
    set "MSG_CONFIRM1=这将卸载 TUI-GAME，是否继续？ [Y/N]"
    set "MSG_MODE=选择卸载模式：[1] 保留存档  [2] 删除全部数据"
    set "MSG_CONFIRM2=确认以“{mode}”模式卸载？ [Y/N]"
    set "MSG_PATH=是否同时清理 PATH 注册？ [Y/N]"
    set "MSG_PATH_WARN=如果你手动修改过 PATH 条目，请自行删除，避免影响其它环境配置。"
    set "MSG_PATH_KEEP=保留 PATH 注册。"
    set "MSG_PATH_DONE=PATH 注册已清理。"
    set "MSG_PATH_FAIL=PATH 注册清理失败，请手动移除。"
    set "MSG_MODE_KEEP=保留存档"
    set "MSG_MODE_FULL=删除全部数据"
    set "MSG_CANCELLED=已取消卸载。"
    set "MSG_START=正在启动卸载程序……"
    set "MSG_DONE=卸载完成。"
    set "MSG_PRESS_KEY=按任意键完成卸载并移除卸载程序。"
)

set /p CONFIRM1=%MSG_CONFIRM1% 
if /I not "%CONFIRM1%"=="Y" if /I not "%CONFIRM1%"=="y" (
    echo %MSG_CANCELLED%
    exit /b 0
)

echo %MSG_MODE%
set /p MODE_INPUT=^> 
if "%MODE_INPUT%"=="1" (
    set "DELETE_DATA=0"
    set "MODE_TEXT=%MSG_MODE_KEEP%"
) else if "%MODE_INPUT%"=="2" (
    set "DELETE_DATA=1"
    set "MODE_TEXT=%MSG_MODE_FULL%"
) else (
    echo %MSG_CANCELLED%
    exit /b 0
)

set "CONFIRM2_MSG=!MSG_CONFIRM2:{mode}=%MODE_TEXT%!"
set /p CONFIRM2=!CONFIRM2_MSG! 
if /I not "%CONFIRM2%"=="Y" if /I not "%CONFIRM2%"=="y" (
    echo %MSG_CANCELLED%
    exit /b 0
)

echo %MSG_PATH_WARN%
set /p CLEAN_PATH_ANSWER=%MSG_PATH% 
if /I "%CLEAN_PATH_ANSWER%"=="Y" (
    set "CLEAN_PATH=1"
) else if /I "%CLEAN_PATH_ANSWER%"=="y" (
    set "CLEAN_PATH=1"
) else (
    set "CLEAN_PATH=0"
)

echo %MSG_START%
ping 127.0.0.1 -n 2 >nul

if "%CLEAN_PATH%"=="1" (
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ^
        "$ErrorActionPreference='Stop';" ^
        "$target = [System.IO.Path]::GetFullPath('%INSTALL_DIR%').TrimEnd('\');" ^
        "$userPath = [Environment]::GetEnvironmentVariable('Path','User');" ^
        "if ([string]::IsNullOrWhiteSpace($userPath)) { exit 0 };" ^
        "$parts = $userPath -split ';' | Where-Object { $_ };" ^
        "$filtered = foreach ($p in $parts) { try { $full = [System.IO.Path]::GetFullPath($p).TrimEnd('\') } catch { $full = $p.TrimEnd('\') }; if ($full -ine $target) { $p } };" ^
        "[Environment]::SetEnvironmentVariable('Path', ($filtered -join ';'), 'User')" >nul 2>&1
    if errorlevel 1 (
        echo %MSG_PATH_FAIL%
    ) else (
        echo %MSG_PATH_DONE%
    )
) else (
    echo %MSG_PATH_KEEP%
)

del /f /q "%INSTALL_DIR%\tg.bat" >nul 2>&1
del /f /q "%INSTALL_DIR%\tg.sh" >nul 2>&1
del /f /q "%INSTALL_DIR%\tui-game.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\tui-game" >nul 2>&1
del /f /q "%INSTALL_DIR%\version.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\version" >nul 2>&1
del /f /q "%INSTALL_DIR%\updata.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\updata" >nul 2>&1
rmdir /s /q "%INSTALL_DIR%\assets" >nul 2>&1
rmdir /s /q "%INSTALL_DIR%\scripts" >nul 2>&1
if "%DELETE_DATA%"=="1" rmdir /s /q "%INSTALL_DIR%\tui-game-data" >nul 2>&1

echo %MSG_DONE%
echo %MSG_PRESS_KEY%
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')"

start "" /b powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command ^
    "Start-Sleep -Seconds 1;" ^
    "$targets = @('%INSTALL_DIR%\remove.exe','%INSTALL_DIR%\remove','%INSTALL_DIR%\delete-tui-game.bat','%INSTALL_DIR%\delete-tui-game.sh');" ^
    "foreach ($target in $targets) { try { Remove-Item -LiteralPath $target -Force -ErrorAction SilentlyContinue } catch {} }"
exit /b 0
