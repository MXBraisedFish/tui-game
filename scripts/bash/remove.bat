@echo off
setlocal enabledelayedexpansion

set "INSTALL_DIR=%~1"
set "DELETE_DATA=%~2"
if "%INSTALL_DIR%"=="" exit /b 1
if "%DELETE_DATA%"=="" set "DELETE_DATA=0"

ping 127.0.0.1 -n 2 >nul

del /f /q "%INSTALL_DIR%\tg.bat" >nul 2>&1
rmdir /s /q "%INSTALL_DIR%\assets" >nul 2>&1
rmdir /s /q "%INSTALL_DIR%\scripts" >nul 2>&1
if "%DELETE_DATA%"=="1" rmdir /s /q "%INSTALL_DIR%\tui-game-data" >nul 2>&1

del /f /q "%INSTALL_DIR%\tui-game.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\version.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\updata.exe" >nul 2>&1
del /f /q "%INSTALL_DIR%\remove.exe" >nul 2>&1

start "" /b cmd /c "ping 127.0.0.1 -n 3 >nul & del /f /q \"%~f0\" >nul 2>&1"
exit /b 0
