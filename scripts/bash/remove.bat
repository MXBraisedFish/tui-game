@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

set "INSTALL_DIR=%~1"
if "%INSTALL_DIR%"=="" exit /b 1

set "LANG_CODE=us-en"
if exist "%INSTALL_DIR%\tui-game-data\language_pref.txt" (
    set /p LANG_CODE=<"%INSTALL_DIR%\tui-game-data\language_pref.txt"
)

set "MSG_CONFIRM1=This will uninstall TUI-GAME. Continue? [y/N]"
set "MSG_MODE=Choose uninstall mode: [1] Keep saves  [2] Delete all data"
set "MSG_CONFIRM2=Confirm uninstall in mode ""{mode}""? [y/N]"
set "MSG_MODE_KEEP=Keep saves"
set "MSG_MODE_FULL=Delete all data"
set "MSG_CANCELLED=Uninstall cancelled."
set "MSG_START=Starting uninstall..."
set "MSG_DONE=Uninstall task started."

if /I "%LANG_CODE%"=="zh-cn" (
    set "MSG_CONFIRM1=这将卸载 TUI-GAME，是否继续？ [y/N]"
    set "MSG_MODE=选择卸载模式：[1] 保留存档  [2] 删除全部数据"
    set "MSG_CONFIRM2=确认以“{mode}”模式卸载？ [y/N]"
    set "MSG_MODE_KEEP=保留存档"
    set "MSG_MODE_FULL=删除全部数据"
    set "MSG_CANCELLED=已取消卸载。"
    set "MSG_START=正在启动卸载程序……"
    set "MSG_DONE=卸载任务已启动。"
)

set /p CONFIRM1=%MSG_CONFIRM1% 
if /I not "%CONFIRM1%"=="y" if /I not "%CONFIRM1%"=="Y" (
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

set "CONFIRM2_MSG=%MSG_CONFIRM2:{mode}=%MODE_TEXT%%"
set /p CONFIRM2=%CONFIRM2_MSG% 
if /I not "%CONFIRM2%"=="y" if /I not "%CONFIRM2%"=="Y" (
    echo %MSG_CANCELLED%
    exit /b 0
)

echo %MSG_START%

set "HELPER=%TEMP%\tui-game-remove-%RANDOM%%RANDOM%.cmd"
> "%HELPER%" echo @echo off
>> "%HELPER%" echo setlocal
>> "%HELPER%" echo ping 127.0.0.1 -n 4 ^>nul
>> "%HELPER%" echo del /f /q "%INSTALL_DIR%\tg.bat" ^>nul 2^>^&1
>> "%HELPER%" echo rmdir /s /q "%INSTALL_DIR%\assets" ^>nul 2^>^&1
>> "%HELPER%" echo rmdir /s /q "%INSTALL_DIR%\scripts" ^>nul 2^>^&1
if "%DELETE_DATA%"=="1" >> "%HELPER%" echo rmdir /s /q "%INSTALL_DIR%\tui-game-data" ^>nul 2^>^&1
>> "%HELPER%" echo del /f /q "%INSTALL_DIR%\tui-game.exe" ^>nul 2^>^&1
>> "%HELPER%" echo del /f /q "%INSTALL_DIR%\version.exe" ^>nul 2^>^&1
>> "%HELPER%" echo del /f /q "%INSTALL_DIR%\updata.exe" ^>nul 2^>^&1
>> "%HELPER%" echo del /f /q "%INSTALL_DIR%\remove.exe" ^>nul 2^>^&1
>> "%HELPER%" echo start "" /b cmd /c "ping 127.0.0.1 -n 3 ^>nul ^& del /f /q ""%HELPER%"" ^>nul 2^>^&1"
>> "%HELPER%" echo exit /b 0

start "" /b cmd /c ""%HELPER%""
echo %MSG_DONE%
exit /b 0
