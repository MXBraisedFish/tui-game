# test branch

This branch is used for **unstable experiments with new features**. Most test features are experimented with and archived separately on this branch.

## Notes

- Experimental code may be rewritten, reverted, or discarded at any time.
- Experimental code is not guaranteed to run stably. Some content may be abandoned after only an initial attempt, but will still be archived.
- There is no guarantee that any experimental code will eventually be merged into the actual project once it stabilizes.
- The code on this branch is not part of the actual project code.

---

# test 分支

本分支用于**新功能的不稳定实验**，多数测试功能会在此分支单独实验和留档。

## 说明

- 实验代码可能随时被重写、回退或丢弃。
- 实验代码并不保证可以稳定运行，部分内容可能仅在初次尝试后就被弃用，但仍会留档。
- 所有实验性代码并不保证在稳定后一定会并入实际工程当中。
- 该分支代码不属于实际工程代码的一部分。

## 统一 Rust 实验环境

本分支保留独立历史，整个活动开发区使用一个 Cargo workspace：

```text
Cargo.toml / Cargo.lock / rust-toolchain.toml
experiments/
  terminal-raster/   终端自截图合成：字体、字格、颜色与系统后端对照
  image-compare/     背景标尺校准、固定尺寸裁剪、逐行误差与热图
profiles/           可复用的实验参数，不携带系统字体文件
tools/xtask/        dev 依赖与编译器版本审计
docs/               基线、实测结论与小型证据文件
archive/logo_animation/  原有 Python logo 实验原样归档
```

Rust 编译器锁定为 dev 当前使用的 1.93.0，edition 2024 / resolver 3。外部依赖版本集中在根 manifest，按 dev 实际 Cargo.lock 精确锁定；子项目仅继承 workspace。根唯一 lock 固定传递依赖，禁止子项目各自升级。未来 dev 升级后须显式同步基线再验证。

```powershell
cargo run -p xtask --locked                           # 对照归档基线
cargo run -p xtask --locked -- --dev E:\Code\tui-game # 只读检查当前 dev 锁文件
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

开始渲染实验：

```powershell
./experiments/terminal-raster/run.ps1 -OutputDirectory E:\Code\tg-test\b7-render-lab\my-run -DevWorktree E:\Code\tui-game
```

运行后打开输出目录 `index.html`，按原始像素叠加比较终端与合成图。详细使用见 [终端实验说明](experiments/terminal-raster/README.md)，结论和后续工作见 [B7 实测报告](docs/B7_REPORT.md)。Windows 原生后端和窗口捕获仅在 Windows 使用；软件探针和图像比较为可移植 Rust 代码，本轮尚未在 Linux/macOS 验证。
