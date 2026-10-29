# 部署与运行环境

上线目录以实际可执行文件所在目录为根：

```text
[root]
├── tg.exe          # Windows executable; tg on Linux/macOS
├── system/         # System configuration and optional tools
├── assets/         # Official assets
├── scripts/        # Official scripts
└── data/           # Runtime data, created when needed
```

运行不需要 Rust、Lua 或 GCC 开发环境。操作系统、架构必须与构建目标匹配，运行目录需要允许创建运行数据。将可执行文件和上述随包目录放在一起；不要单独移动可执行文件。改变命令行工作目录不会改变资源与配置的根目录。

## Windows GNU 构建

在仓库根目录运行：

```powershell
cargo build --release --workspace --locked
powershell -NoProfile -ExecutionPolicy Bypass -File tools/check_windows_runtime.ps1 -Executable target/release/tg.exe
```

视频编码依赖 OpenH264，其 C++ 代码原先使 `tg.exe` 动态导入 `libstdc++-6.dll`，并间接需要 `libgcc_s_seh-1.dll`、`libwinpthread-1.dll`。开发机的 MSYS2 PATH 掩盖了这个缺失，其他电脑会在进入程序之前报错。

`.cargo/config.toml` 对 `x86_64-pc-windows-gnu` 禁用依赖构建器自动添加的动态 C++ 标准库；视频服务的 `build.rs` 从当前 C++ 编译器查询 `libstdc++.a` 并明确静态链接。开发机需要提供这份静态库。请从仓库根目录使用该配置构建；独立复制视频 crate 时也需要带上相同构建配置。运行机无需安装这些 GCC 运行库。

检查脚本直接读取 PE 导入表，不依赖 objdump 或开发环境。它拒绝上述已知运行库和其他未获 Windows 系统库规则认可的直接导入；新增依赖需要复核其传递依赖。静态导入检查不能替代目标操作系统上的实际运行验收。

## system 目录

附加环境配置统一放在 `system/`，由存储服务提供部署路径与受限文件解析接口。`resolve_system_file` 接收相对路径，拒绝绝对路径、父目录越界、无效文件以及解析后逃出 system 根目录的文件链接。

FFmpeg 优先使用 `system/ffmpeg.exe`，保留原部署根目录、旧版 `data/cache/ffmpeg/`、其他兼容位置和系统 PATH 查找。外部工具自身也应携带完整运行依赖；分发前应检查其依赖。FFmpeg 是可选工具，基础启动与内置 OpenH264 视频编码不要求安装它。

目录名称和路径可以包含中文、空格。`system/` 允许目录链接，读取时以链接指向的实际目录作为边界；跨电脑打包应复制实际文件，不能依赖只在开发机存在的链接目标。不将开发机绝对路径写入产品配置。

## 验证

```powershell
cargo test -p tg-service-storage -p tg-service-ffmpeg -p tg-service-video --locked
cargo run -p tg-service-storage --example storage_smoke --locked
cargo run -p tg-service-ffmpeg --example ffmpeg_smoke --locked -- E:/path/to/deployment
cargo run -p tg-service-video --example video_smoke --locked -- E:/path/to/deployment
cargo test --bin tg host_engine::app::services::tests --locked
cargo fmt --all --check
```

发布时还应在没有开发工具 PATH 的环境中启动，确认从其他工作目录也能找到随包资源；目标平台实机验证仍然必要。

视频冒烟入口需要部署根目录中的 `assets/fonts/mmo.ttf`，会使用内置 OpenH264 实际导出并读取 MP4，不依赖 FFmpeg。Windows 存储目录链接测试使用系统自带 PowerShell 创建目录联接，无需开发者模式或管理员权限。
