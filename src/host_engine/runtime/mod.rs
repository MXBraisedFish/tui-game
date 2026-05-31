// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引用结构体和枚举
use crate::host_engine::core::{
  RuntimeWorld,
  ExitState,
  FrameScheduler
};
use crate::host_engine::services::{
  EngineServices, GameSessionState, KeyInput, InputEvent
};

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

      services.log.info(LogSource::Runtime, format!("[Terminal Resize detected: {}x{}]", width, height));
    }

    let mut consumed_input = false;

    // 若用户摁下ESC则终止运行（但不打断本轮循环）
    if services.input.consume_key(KeyCode::Esc) {
      running = false;
      consumed_input = true;
    } else if services.input.consume_key(KeyCode::Right) {
      services.ui.navigate_next();
      consumed_input = true;
    } else if services.input.consume_key(KeyCode::Left) {
      services.ui.navigate_prev();
      consumed_input = true;
    } else if services.input.consume_key(KeyCode::F(2)) {
      if services.game.state() == GameSessionState::Inactive {
        services.game.start("1");
      } else {
        services.game.stop();
      }

      consumed_input = true;
    }

    // 当前帧未处理队头事件时，弹出一个事件避免阻塞后续输入
    let last_key = if consumed_input {
      None
    } else {
      services.input.next_key()
    };

    update(services, world, frame);
    render(services, world, frame, last_key);

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
fn render(services: &mut EngineServices, world: &mut RuntimeWorld, frame: u64, last_key: Option<KeyInput>) {
  services.render.clear();

  services.render.draw_centered(0, "TUI Game Engine");

  let status = format!(
    "Page: {} | Frame: {} | dt: {:.1}ms",
    services.ui.active_page().name(),
    frame,
    world.clock.delta_time().as_secs_f64() * 1000.0,
  );

  services.render.draw_centered(2, &status);

  services.ui.render_active(&mut services.render);

  let game_status = match services.game.state() {
    GameSessionState::Inactive => "idle",
    GameSessionState::Running => "RUNNING",
    GameSessionState::Paused => "PAUSED",
  };

  let overlay_status = if services.overlay.any_active() {
    "OVERLAY"
  } else {
    "idle"
  };

  let lua_status = match services.lua.eval("return 'ok'") {
    Ok(_) => "Lua:ok",
    Err(_) => "Lua:err",
  };

  let status = format!(
    "Game:{} | Overlay:{} | {} | <- -> Nav | ESC Exit",
    game_status,
    overlay_status,
    lua_status,
  );

  services.render.draw_centered(9, &status);

  let caps = services.terminal.capabilities();
  let capability_text = format!("RGB: {} Unicode: {} Image: {:?}",
  caps.truecolor, caps.unicode, caps.image_protocol);

  services.render.draw_centered(12, &capability_text);

  let logs = services.log.entries();

  let start = logs.len().saturating_sub(5);

  for (i, log) in logs.iter().skip(start).enumerate() {
    let line = format!("[{:?}] {}", log.level, log.message);
    services.render.draw_centered(20 + i, &line);
  }
  
  let terminal = &mut services.terminal;
  let render = &mut services.render;

  if let Some(stdout) = terminal.writer_mut() {
    let _ = render.present(stdout);
  }
}