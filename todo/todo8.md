# 步骤 8：RenderService — 基础画布绘制 + 呈现

## 目标

给 `RenderService` 实现简单的双缓冲画布和帧呈现。
绘制静态占位屏幕以证明渲染管线能跑通。

## 期望输出

终端进入交替屏幕后，每帧清屏并显示：
```
  TUI Game Engine
  Frame: 1  dt: 0.0ms
  Frame: 2  dt: 1000.0ms
  ...
```

## 操作

### 8.1 实现 `src/app/services/render.rs`

```rust
use std::io::{self, stdout, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;

pub struct RenderService {
    width: u16,
    height: u16,
    lines: Vec<String>,
}

impl RenderService {
    pub fn new() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        Self {
            width,
            height,
            lines: Vec::new(),
        }
    }

    /// 清空后台缓冲区。
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// 绘制一行文本。
    pub fn draw_text(&mut self, text: &str) {
        self.lines.push(text.to_string());
    }

    /// 居中绘制文本。
    pub fn draw_centered(&mut self, row: usize, text: &str) {
        let text_width = text.chars().count() as u16;
        let col = if text_width < self.width {
            (self.width - text_width) / 2
        } else {
            0
        };
        while self.lines.len() <= row {
            self.lines.push(String::new());
        }
        let padding = " ".repeat(col as usize);
        self.lines[row] = format!("{}{}", padding, text);
    }

    /// 将缓冲区刷新到终端。
    pub fn present(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.queue(Clear(ClearType::All))?;

        for (row, line) in self.lines.iter().enumerate() {
            if row >= self.height as usize {
                break;
            }
            stdout.queue(MoveTo(0, row as u16))?;
            stdout.queue(Print(line.as_str()))?;
        }

        stdout.flush()?;
        Ok(())
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}
```

### 8.2 更新 `src/app/runtime.rs` 的 render 函数

```rust
fn render(services: &mut EngineServices, world: &RuntimeWorld, frame: u64) {
    services.render.clear();

    // 标题
    services.render.draw_centered(0, "TUI Game Engine");

    // 帧信息
    let info = format!(
        "Frame: {}  dt: {:.1}ms  elapsed: {:.1}s",
        frame,
        world.clock.dt().as_secs_f64() * 1000.0,
        world.clock.elapsed_since_epoch().as_secs_f64(),
    );
    services.render.draw_centered(2, &info);

    // 底部提示
    let hint = "[Step 8: Render pipeline active]";
    services.render.draw_centered(services.render.size().1 as usize - 1, hint);

    let _ = services.render.present();
}
```

## 验证

```bash
cargo build
cargo run
```

每帧清屏并绘制标题 + 帧计数器。5 帧后终端恢复。
