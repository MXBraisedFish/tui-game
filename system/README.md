# System configuration

此目录存放程序运行所需的附加配置和外部工具，随 `tg.exe`、`assets/`、`scripts/` 一起分发。

- Windows GNU 构建已静态链接已知的 GCC C++ 运行库，无需在此放置 `libstdc++-6.dll`、`libgcc_s_seh-1.dll` 或 `libwinpthread-1.dll`。
- 可选 FFmpeg 放在 `system/ffmpeg.exe`（Linux/macOS 为 `system/ffmpeg`）。程序优先从这里查找；未安装时，依赖 FFmpeg 的功能不可用。
- 配置路径以可执行文件所在的部署根目录为基准，不受启动时工作目录影响。服务通过 `StorageService::system_dir_path` 或 `resolve_system_file` 定位文件。
- `system/` 本身可以是指向有效目录的链接；配置文件解析后必须仍在该目录内。向其他电脑分发时应复制链接指向的实际内容，避免无效链接。

此目录不是 Windows 通用 DLL 搜索目录。新增原生动态依赖时，应另行验证其加载方式及完整的传递依赖，不能只把 DLL 放进这里。
