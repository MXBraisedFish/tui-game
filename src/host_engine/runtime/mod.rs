// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引用结构体和枚举
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld};
use crate::host_engine::services::CanvasStyle;
use crate::host_engine::services::{EngineServices, GameSessionState, InputEvent, KeyInput};

// 引用按键枚举
use crossterm::event::KeyCode;

// 临时日志
use super::services::{LogEntry, LogLevel, LogService, LogSource, format_log_entry};

// 运行函数
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  // 启用终端模式
  services.terminal.enter(&mut services.log);

  // 构建一个帧循环
  let mut scheduler = FrameScheduler::new();
  let mut running = true;

  // 开始循环
  while running {
    // 获取帧信息
    let frame = scheduler.begin_frame();

    // 更新帧时间
    world.clock.tick();

    // 更新按键事件队列
    services.input.poll();

    if let Some((width, height)) = services.input.consume_resize() {
      services.render.resize(width, height);
      services.ui.on_resize(width, height);

      services.canvas.resize(width, height);

      services.log.info(
        LogSource::Runtime,
        format!("[Terminal Resize detected: {}x{}]", width, height),
      );
    }

    let mut consumed_input = false;

    // 若用户摁下ESC则终止运行（但不打断本轮循环）
    if services.input.consume_key(KeyCode::Esc) {
      running = false;
      consumed_input = true;
    }

    // 当前帧未处理队头事件时，弹出一个事件避免阻塞后续输入
    if consumed_input {
      None
    } else {
      services.input.next_key()
    };

    update(services, world, frame);
    render(services, world, frame);

    thread::sleep(Duration::from_millis(16));
  }

  // 返回退出信息块
  ExitState::new()
}

// 更新函数
fn update(services: &mut EngineServices, world: &mut RuntimeWorld, frame: u64) {
  services.game.update();
  services.overlay.update();
}

// 绘制函数
// 先clear
// 然后使用draw_centered入栈
// 使用复制
fn render(services: &mut EngineServices, world: &mut RuntimeWorld, frame: u64) {
  services.canvas.clear();

  services
    .canvas
    .write_text(0, 0, "ABC中文😀", CanvasStyle::default());

  let canvas_line = services.canvas.line_as_string(0);

  services
    .render
    .draw_centered(22, &format!("Canvas line: {}", canvas_line.trim_end()));

  let terminal = &mut services.terminal;
  let render = &mut services.render;

  if let Some(stdout) = terminal.writer_mut() {
    let _ = render.present(stdout);
  }
}
