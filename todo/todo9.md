# 步骤 9：InputService — 轮询按键，ESC 退出

## 目标

给 `InputService` 添加按键轮询能力。
用真正的退出条件（按 ESC 退出）替代硬编码的 5 帧限制。

## 期望行为

- 应用启动，显示帧计数器
- 按任意键 → 屏幕上显示键名
- 按 ESC → 退出循环 → 关闭
- 不按键时循环继续（非阻塞轮询）

## 操作

### 9.1 实现 `src/app/services/input.rs`

```rust
use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, poll};

/// 输入系统的最小按键事件。
#[derive(Clone, Debug)]
pub struct KeyInput {
    pub code: KeyCode,
    pub kind: KeyEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

impl From<KeyEvent> for KeyInput {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            kind: match event.kind {
                crossterm::event::KeyEventKind::Press => KeyEventKind::Press,
                crossterm::event::KeyEventKind::Release => KeyEventKind::Release,
                crossterm::event::KeyEventKind::Repeat => KeyEventKind::Repeat,
            },
        }
    }
}

pub struct InputService {
    queue: VecDeque<KeyInput>,
}

impl InputService {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// 非阻塞轮询新输入事件。
    pub fn poll(&mut self) {
        while poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                self.queue.push_back(KeyInput::from(key_event));
            }
        }
    }

    /// 获取下一个待处理按键事件。
    pub fn next_key(&mut self) -> Option<KeyInput> {
        self.queue.pop_front()
    }

    /// 消费指定按键的一次按下事件。
    pub fn consume_key(&mut self, code: KeyCode) -> bool {
        let pos = self.queue.iter().position(
            |k| k.code == code && k.kind == KeyEventKind::Press
        );
        if let Some(idx) = pos {
            self.queue.remove(idx);
            true
        } else {
            false
        }
    }
}
```

### 9.2 更新 `src/app/frame.rs` — 移除 max_frames

```rust
pub struct FrameScheduler {
    current_frame: u64,
}

impl FrameScheduler {
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }

    pub fn begin_frame(&mut self) -> u64 {
        self.current_frame += 1;
        self.current_frame
    }
}
```

### 9.3 更新 `src/app/runtime.rs`

```rust
use crossterm::event::KeyCode;
use std::thread;
use std::time::Duration;

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) {
    services.terminal.enter();

    let mut scheduler = FrameScheduler::new();
    let mut running = true;

    while running {
        let frame_index = scheduler.begin_frame();
        world.clock.tick();

        // 轮询输入
        services.input.poll();

        // 检查退出
        if services.input.consume_key(KeyCode::Esc) {
            running = false;
        }

        // 获取最后按下的键
        let last_key = services.input.next_key();

        update(services, world, frame_index);
        render(services, world, frame_index, last_key);

        // 空闲休眠避免忙等（约 60fps）
        thread::sleep(Duration::from_millis(16));
    }

    services.terminal.exit();
}
```

render 函数中显示按键和退出提示。

## 验证

```bash
cargo build
cargo run
```

- 帧计数器持续更新
- 按任意键 → 键名出现在屏幕上
- 按 ESC → 应用干净退出
- 终端正确恢复
