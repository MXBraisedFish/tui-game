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
// 脏行机制自动记录被修改的行，diff 渲染时只重绘这些行。
fn render(services: &mut EngineServices, _world: &mut RuntimeWorld, frame: u64) {
  // 开始新帧（保留模式，不自动清空缓冲区）
  services.canvas.begin_frame();

  // 动态内容：每隔 60 帧切换显示文本，测试宽字符清理
  let wide_test = if frame % 120 < 60 {
    "Wide cleanup: 中文😀"
  } else {
    "Wide cleanup: ABCD"
  };


}
