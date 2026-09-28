# test 分支实验工作区

本分支与 dev/main 历史独立。工作仅在 test worktree 内进行，不切换其他工作区、不提交 dev 的未提交内容。

## Rust 环境

- 全分支只有根 Cargo workspace、Cargo.lock、rust-toolchain.toml 和 target/。实验位于 `experiments/<name>/`，共享工具位于 `tools/`。
- 子项目只能用 `dependency.workspace = true`；外部依赖的版本和特性集中在根 `[workspace.dependencies]`，版本必须取自 dev 的锁文件。不得独立运行 cargo update 或创建子 workspace/锁文件。
- `docs/dev-baseline.json` 记录复制时的 dev HEAD、工作区文件摘要、编译器和实际依赖版本。当前基线包含 dev 未提交的 B7 实现，不能将它说成该 HEAD 的已提交状态。
- 修改依赖后运行 `cargo run -p xtask --locked -- --dev <dev-worktree>`；该命令只读取 dev 的 Cargo.lock，不修改 dev。没有 dev checkout 时运行不带参数的 xtask，对照归档基线。
- dev 升级时显式更新根依赖和基线记录，重新生成唯一锁文件并运行校验；不要静默接受版本漂移。
- `archive/` 是原有非 Rust 实验的原样归档，不是第二套活动开发环境。新增实验使用 Rust，不要求将历史 Python 代码改写。

## 终端渲染实验

- 使用当前用户确认的终端配置，不修改用户终端设置。字体 face/字重、DPI、实际字格、基线、默认色与调色板必须记录。
- 用背景色标尺测格子，不能用文字墨迹边界推断字格。图像比对不得自动缩放或偷偷平移消除误差。
- 捕获唯一标题的实验窗口，不捕获整个桌面；窗口创建／退出、raw mode、光标与 alternate screen 必须清理。复用 `run.ps1`。
- 输出到显式指定的测试目录（本机 `E:\Code\tg-test\`）；不依赖当前目录寻字体、不复制或分发系统字体文件。系统字体缺失应明确报错。
- 区分字体回退、字重模拟、形状塑形、格子定位、颜色与抗锯齿。软件隔离探针不是 dev 完整栅格器；DirectWrite TextLayout 也不是 Windows Terminal Atlas。
- 固定图片结果与数值结论一起保存。空白占多数时全图平均误差会误导，必须单列文字行、有效像素、调色板和已知失败项。
- 本实验获得用户授权查看终端截图；不要继承 dev 历史文档中已被用户撤销的“禁止读图”限制。

## 最小验证

`cargo fmt --all -- --check`、`cargo test --workspace --locked`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、xtask 依赖审计；渲染路径变更后再执行终端实测。只在结果证实后向 dev 建议迁移，不自动合并。
