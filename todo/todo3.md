# 步骤 3：加入帧调度器，建立帧循环

## 目标

把简单的 `for` 循环替换为正式的帧调度器：
`begin_frame → update → render`。

仍然没有真实系统 — 只证明帧边界存在。

## 背景

每个实时引擎都有一个帧循环。这是未来所有系统的插入点：
输入轮询、动作解析、游戏/UI 更新、渲染。

## 期望输出

```
[Boot] Preparing engine...
[Frame   1] dt=...ms  elapsed=...ms
[Frame   2] dt=...ms  elapsed=...ms
[Frame   3] dt=...ms  elapsed=...ms
[Frame   4] dt=...ms  elapsed=...ms
[Frame   5] dt=...ms  elapsed=...ms
[Shutdown] Engine closed.
```

## 操作

### 3.1 创建 `src/app/frame.rs`

```rust
use std::time::{Duration, Instant};

/// 每帧的时间信息。
#[derive(Clone, Copy, Debug)]
pub struct FrameInfo {
    /// 单调递增的帧序号。
    pub index: u64,
    /// 本帧开始的挂钟时间。
    pub started_at: Instant,
    /// 距离上一帧开始的间隔时间。
    pub dt: Duration,
}

/// 管理帧循环生命周期。
pub struct FrameScheduler {
    /// 最大运行帧数（0 = 无限）。
    max_frames: u64,
    current_frame: u64,
    last_frame_start: Instant,
}

impl FrameScheduler {
    pub fn new(max_frames: u64) -> Self {
        Self {
            max_frames,
            current_frame: 0,
            last_frame_start: Instant::now(),
        }
    }

    /// 返回 Some(FrameInfo) 表示应继续运行，None 表示结束。
    pub fn begin_frame(&mut self) -> Option<FrameInfo> {
        if self.max_frames > 0 && self.current_frame >= self.max_frames {
            return None;
        }

        self.current_frame += 1;
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_start);
        self.last_frame_start = now;

        Some(FrameInfo {
            index: self.current_frame,
            started_at: now,
            dt,
        })
    }

    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}
```

### 3.2 更新 `src/app/mod.rs`

添加 `frame` 模块。

### 3.3 更新 `src/app/runtime.rs`

使用调度器：

```rust
use super::frame::{FrameInfo, FrameScheduler};
use super::services::EngineServices;
use super::world::RuntimeWorld;

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) {
    let mut scheduler = FrameScheduler::new(5);

    while let Some(frame) = scheduler.begin_frame() {
        update(services, world, frame);
        render(services, world, frame);
    }
}

fn update(_services: &mut EngineServices, _world: &mut RuntimeWorld, _frame: FrameInfo) {
    // 后续系统从此处插入。
}

fn render(_services: &EngineServices, _world: &RuntimeWorld, frame: FrameInfo) {
    println!(
        "[Frame {:>3}] dt={:>6.1}ms  elapsed={:>8.1}ms",
        frame.index,
        frame.dt.as_secs_f64() * 1000.0,
        frame.started_at.elapsed().as_secs_f64() * 1000.0,
    );
}
```

## 验证

```bash
cargo build
cargo run
```

打印 5 帧，每帧带时间信息。`update`/`render` 分离可见。
