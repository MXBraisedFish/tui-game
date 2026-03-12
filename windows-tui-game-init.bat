@echo off
setlocal enabledelayedexpansion

echo [1] 中文
echo [2] English
set /p CHOICE="Select language / 选择语言 (1/2): "
if "%CHOICE%"=="1" (
    set "LANG_CODE=zh-cn"
) else (
    set "LANG_CODE=us-en"
)

if /I "%LANG_CODE%"=="zh-cn" (
    set "MSG_START=[信息] 开始安装 TUI-GAME..."
    set "MSG_FETCH=[信息] 正在从 GitHub 获取最新版本信息..."
    set "MSG_PARSE=[信息] 正在解析 Windows 安装包下载链接..."
    set "MSG_DL=[信息] 正在下载安装包..."
    set "MSG_EXTRACT=[信息] 正在解压文件到当前目录..."
    set "MSG_LANG_INIT=[信息] 正在初始化语言设置..."
    set "MSG_CLEAN=[信息] 已清理临时文件。"
    set "MSG_ASK_PATH=是否将安装目录加入 PATH 环境变量？(Y/N): "
    set "MSG_ADD_PATH=[信息] 正在写入用户 PATH..."
    set "MSG_PATH_OK=[成功] 已写入用户 PATH，重新打开终端后可直接使用 tg。"
    set "MSG_PATH_SKIP=[信息] 跳过 PATH 注册。"
    set "MSG_DONE=[成功] TUI-GAME 安装完成。"
    set "MSG_RUN=[信息] 你现在可以输入 tg 启动游戏。"
    set "ERR_CURL=[错误] 未找到 curl。"
    set "ERR_PS=[错误] 未找到 PowerShell。"
    set "ERR_FETCH=[错误] 下载版本信息失败。"
    set "ERR_ASSET=[错误] 未找到 Windows 安装包。"
    set "ERR_DL=[错误] 下载安装包失败。"
    set "ERR_EXTRACT=[错误] 解压安装包失败。"
    set "ERR_PATH=[警告] PATH 写入失败，请手动添加。"
    set "MSG_EXIT=[信息] 按任意键退出并删除安装脚本。"
) else (
    set "MSG_START=[INFO] Starting TUI-GAME installation..."
    set "MSG_FETCH=[INFO] Fetching latest release information from GitHub..."
    set "MSG_PARSE=[INFO] Extracting Windows package download URL..."
    set "MSG_DL=[INFO] Downloading package..."
    set "MSG_EXTRACT=[INFO] Extracting files to current directory..."
    set "MSG_LANG_INIT=[INFO] Initializing language preference..."
    set "MSG_CLEAN=[INFO] Temporary files cleaned up."
    set "MSG_ASK_PATH=Do you want to add the installation folder to PATH? (Y/N): "
    set "MSG_ADD_PATH=[INFO] Updating user PATH..."
    set "MSG_PATH_OK=[SUCCESS] User PATH updated. Reopen the terminal to use tg."
    set "MSG_PATH_SKIP=[INFO] Skipping PATH registration."
    set "MSG_DONE=[SUCCESS] TUI-GAME has been installed."
    set "MSG_RUN=[INFO] You can now type tg to start the game."
    set "ERR_CURL=[ERROR] curl was not found."
    set "ERR_PS=[ERROR] PowerShell was not found."
    set "ERR_FETCH=[ERROR] Failed to download release information."
    set "ERR_ASSET=[ERROR] Windows package asset was not found."
    set "ERR_DL=[ERROR] Failed to download the package."
    set "ERR_EXTRACT=[ERROR] Failed to extract the package."
    set "ERR_PATH=[WARNING] Failed to update PATH. Please add it manually."
    set "MSG_EXIT=[INFO] Press any key to exit and delete this installer."
)

echo %MSG_START%

where curl >nul 2>nul || (echo %ERR_CURL% & pause & exit /b 1)
where powershell >nul 2>nul || (echo %ERR_PS% & pause & exit /b 1)

set "API_URL=https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
set "TEMP_JSON=%temp%\tui_game_init_%RANDOM%.json"
set "TEMP_ZIP=%temp%\tui_game_%RANDOM%.zip"

echo %MSG_FETCH%
curl -s -L -o "%TEMP_JSON%" "%API_URL%"
if errorlevel 1 (
    echo %ERR_FETCH%
    pause
    exit /b 1
)

echo %MSG_PARSE%
set "DOWNLOAD_URL="
for /f "usebackq delims=" %%i in (`powershell -NoProfile -ExecutionPolicy Bypass -Command "$json = Get-Content -Raw '%TEMP_JSON%' | ConvertFrom-Json; $asset = $json.assets | Where-Object { $_.name -eq 'tui-game-windows.zip' } | Select-Object -First 1; if ($asset) { $asset.browser_download_url }"`) do (
    set "DOWNLOAD_URL=%%i"
)
if "!DOWNLOAD_URL!"=="" (
    echo %ERR_ASSET%
    del /f /q "%TEMP_JSON%" >nul 2>&1
    pause
    exit /b 1
)

echo %MSG_DL%
curl -s -L -o "%TEMP_ZIP%" "!DOWNLOAD_URL!"
if errorlevel 1 (
    echo %ERR_DL%
    del /f /q "%TEMP_JSON%" >nul 2>&1
    pause
    exit /b 1
)

echo %MSG_EXTRACT%
powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -Path '%TEMP_ZIP%' -DestinationPath '%CD%' -Force"
if errorlevel 1 (
    echo %ERR_EXTRACT%
    del /f /q "%TEMP_JSON%" "%TEMP_ZIP%" >nul 2>&1
    pause
    exit /b 1
)

echo %MSG_LANG_INIT%
if not exist "%CD%\tui-game-data" mkdir "%CD%\tui-game-data"
> "%CD%\tui-game-data\language_pref.txt" echo %LANG_CODE%

del /f /q "%TEMP_JSON%" "%TEMP_ZIP%" >nul 2>&1
echo %MSG_CLEAN%

echo.
set /p ADD_PATH="%MSG_ASK_PATH%"
if /I "!ADD_PATH!"=="Y" (
    set "INSTALL_DIR=%CD%"
    echo %MSG_ADD_PATH%
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "$target=[System.IO.Path]::GetFullPath($env:CD).TrimEnd('\');" ^
        "$userPath=[Environment]::GetEnvironmentVariable('Path','User');" ^
        "$parts=@(); if(-not [string]::IsNullOrWhiteSpace($userPath)){ $parts=$userPath -split ';' | Where-Object { $_ } }" ^
        "$exists=$false; foreach($p in $parts){ try { $full=[System.IO.Path]::GetFullPath($p).TrimEnd('\') } catch { $full=$p.TrimEnd('\') }; if($full -eq $target){ $exists=$true; break } }" ^
        "if(-not $exists){ $newPath=if([string]::IsNullOrWhiteSpace($userPath)){$target}else{$userPath.TrimEnd(';') + ';' + $target}; [Environment]::SetEnvironmentVariable('Path',$newPath,'User') }"
    if errorlevel 1 (
        echo %ERR_PATH%
    ) else (
        echo %MSG_PATH_OK%
    )
) else (
    echo %MSG_PATH_SKIP%
)

echo.
echo %MSG_DONE%
echo %MSG_RUN%
echo.
echo %MSG_EXIT%
pause >nul

set "SELF_BAT=%~f0"
start "" /b cmd /c "ping 127.0.0.1 -n 2 >nul & del /f /q ""%SELF_BAT%"" >nul 2>&1"
exit /b 0
