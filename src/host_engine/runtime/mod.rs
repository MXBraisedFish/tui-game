// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引用结构体和枚举
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld};
use crate::host_engine::services::{
  CanvasStyle, EngineServices, LogSource, TerminalColor, TextColor,
};

// 引用按键枚举
use crossterm::event::KeyCode;

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

    // 临时的响应尺寸变化
    if let Some((width, height)) = services.input.consume_resize() {
      services.canvas.resize(width, height);
      services.ui.on_resize(width, height);

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

    // 请求帧更新：由 CanvasService 封装终端交互
    if let Err(err) = services.canvas.request_frame_update(&mut services.terminal) {
      services.log.warn(LogSource::Runtime, format!("[Render] Frame update failed: {}", err));
    }

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

// 绘制函数（保留模式）
//
// 不再每帧 clear() 清空全部缓冲区。
// 每帧调用 begin_frame() 后，仅对需要更新的行调用 clear_row()，
// 脏区间机制自动记录被修改的区域，diff 渲染时只重绘这些区域。
//
// 注意：此函数只负责绘图调用，不处理终端 I/O。
// 帧提交由主循环中的 request_frame_update() 统一完成。
fn render(services: &mut EngineServices, _world: &mut RuntimeWorld, frame: u64) {
  // 开始新帧（保留模式，不自动清空缓冲区）
  services.canvas.begin_frame();

  // 动态内容：每隔 60 帧切换显示文本，测试宽字符清理
  let wide_test = if frame % 120 < 60 {
    "Wide cleanup: 中文😀"
  } else {
    "Wide cleanup: ABCD"
  };

  // 仅清除文本实际占用的水平区间，而非整行
  let (canvas_width, _) = services.canvas.size();
  let centered_x_12 = canvas_width
    .saturating_sub(wide_test.len() as u16)
    .saturating_div(2);
  services
    .canvas
    .clear_span(12, centered_x_12, centered_x_12.saturating_add(wide_test.len() as u16));
  services
    .canvas
    .write_text(centered_x_12, 12, wide_test, CanvasStyle::default());

  // 临时调试指示器：显示当前脏区间数量
  let dirty_info = format!("Dirty spans: {}", services.canvas.dirty_spans().len());
  let centered_x_14 = canvas_width
    .saturating_sub(dirty_info.len() as u16)
    .saturating_div(2);
  services
    .canvas
    .clear_span(14, centered_x_14, centered_x_14.saturating_add(dirty_info.len() as u16));
  services.canvas.write_text(
    centered_x_14,
    14,
    &dirty_info,
    CanvasStyle::default(),
  );
}
