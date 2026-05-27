# 步骤 5：添加 EngineClock，接入帧调度器

## 目标

用 `RuntimeWorld` 持有的专用 `EngineClock` 替代散落的 `std::time::Instant`。
时钟提供每帧的权威时间源。

## 背景

每个游戏引擎需要单一时间真源。后续时钟可以支持时间缩放、暂停、
固定时间步长。目前只包装 `Instant`。

## 操作

### 5.1 创建 `src/app/clock.rs`

```rust
use std::time::{Duration, Instant};

/// 引擎的权威时间源。
pub struct EngineClock {
    /// 引擎启动时刻（boot 完成时）。
    epoch: Instant,
    /// 当前帧开始时刻。
    frame_start: Instant,
    /// 距上一帧的时间间隔。
    dt: Duration,
}

impl EngineClock {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            epoch: now,
            frame_start: now,
            dt: Duration::ZERO,
        }
    }

    /// 标记新帧开始，计算 dt。
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.dt = now.duration_since(self.frame_start);
        self.frame_start = now;
    }

    /// 自引擎启动以来的时间。
    pub fn elapsed_since_epoch(&self) -> Duration {
        self.epoch.elapsed()
    }

    /// 当前帧的 delta time。
    pub fn dt(&self) -> Duration {
        self.dt
    }
}
```

### 5.2 更新 `src/app/world.rs`

```rust
use super::clock::EngineClock;

pub struct RuntimeWorld {
    pub clock: EngineClock,
}

impl RuntimeWorld {
    pub fn new() -> Self {
        Self {
            clock: EngineClock::new(),
        }
    }
}
```

### 5.3 更新 `src/app/frame.rs`

从 `FrameScheduler` 移除 `Instant::now()`。调度器只跟踪帧计数和最大帧数。
时钟在运行时循环中显式 tick。

```rust
pub struct FrameScheduler {
    max_frames: u64,
    current_frame: u64,
}

impl FrameScheduler {
    pub fn new(max_frames: u64) -> Self {
        Self { max_frames, current_frame: 0 }
    }

    pub fn begin_frame(&mut self) -> Option<u64> {
        if self.max_frames > 0 && self.current_frame >= self.max_frames {
            return None;
        }
        self.current_frame += 1;
        Some(self.current_frame)
    }
}
```

### 5.4 更新 `src/app/mod.rs`

添加 `clock` 模块。

### 5.5 更新 `src/app/runtime.rs`

```rust
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) {
    let mut scheduler = FrameScheduler::new(5);

    while let Some(frame_index) = scheduler.begin_frame() {
        world.clock.tick();
        update(services, world, frame_index);
        render(services, world, frame_index);
    }
}

fn update(_s: &mut EngineServices, _w: &mut RuntimeWorld, _frame: u64) {}

fn render(_s: &EngineServices, w: &RuntimeWorld, frame: u64) {
    println!(
        "[Frame {:>3}] dt={:>6.1}ms  elapsed={:>8.1}ms",
        frame,
        w.clock.dt().as_secs_f64() * 1000.0,
        w.clock.elapsed_since_epoch().as_secs_f64() * 1000.0,
    );
}
```

## 验证

```bash
cargo build
cargo run
```

打印 5 帧带 dt 和 elapsed 时间。首帧 dt 接近 0，后续帧 dt 约等于帧间隔。
