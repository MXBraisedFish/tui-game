# 步骤 7：TerminalService — 交替屏幕 + raw 模式

## 目标

把终端设置（交替屏幕 + raw 模式）从旧 `host_engine::runtime::terminal`
提取到新结构中。

## 背景

旧 `runtime::terminal::enter()` 启用交替屏幕和 raw 模式，
返回一个 RAII 守卫在 drop 时恢复。这是纯粹的终端关注点。
我们创建 `TerminalService` 包装此行为。

注意：内部仍然使用旧 crossterm 代码 — 我们只是把所有权移到正确
的服务槽位。

## 操作

### 7.1 确认 crossterm 依赖

检查 `Cargo.toml` 是否有 `crossterm`。旧代码已经在用，应该已存在。

### 7.2 创建 `src/app/services/terminal.rs`

```rust
use std::io::{self, stdout, Stdout};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

pub struct TerminalService {
    /// RAII 守卫：drop 时恢复终端。
    guard: Option<TerminalGuard>,
}

struct TerminalGuard {
    _stdout: Stdout,
}

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        Ok(Self { _stdout: stdout })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

impl TerminalService {
    /// 进入交替屏幕 + raw 模式。
    pub fn enter(&mut self) {
        match TerminalGuard::enter() {
            Ok(guard) => {
                self.guard = Some(guard);
                println!("[Terminal] Alternate screen enabled.");
            }
            Err(e) => {
                eprintln!("[Terminal] Failed to enter terminal mode: {}", e);
            }
        }
    }

    /// 显式退出（通常由 Drop 自动处理）。
    pub fn exit(&mut self) {
        self.guard = None;
        println!("[Terminal] Terminal restored.");
    }
}

impl Default for TerminalService {
    fn default() -> Self {
        Self { guard: None }
    }
}
```

### 7.3 更新 `src/app/services/mod.rs`

添加 `terminal` 模块和字段：

```rust
mod terminal;
pub use terminal::TerminalService;

pub struct EngineServices {
    pub terminal: TerminalService,
    pub package: PackageService,
    // ... 其余不变
}
```

### 7.4 更新 `src/app/runtime.rs`

在运行时开始进入终端，结束时退出：

```rust
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) {
    services.terminal.enter();

    let mut scheduler = FrameScheduler::new(5);

    while let Some(frame_index) = scheduler.begin_frame() {
        world.clock.tick();
        update(services, world, frame_index);
        render(services, world, frame_index);
    }

    services.terminal.exit();
}
```

## 验证

```bash
cargo build
cargo run
```

终端应进入交替屏幕模式（清屏、无滚动回溯），显示 5 帧消息后恢复。
