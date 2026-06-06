// 引入标准线程库
use std::thread;
use std::time::Duration;

// 引入运行时输入处理
mod input;

use input::handle_runtime_keyboard;

// 引用结构体和枚举
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld};
use crate::host_engine::services::{
  EngineServices,
  Key,
  KeyBinding,
  KeyPattern,
  LogSource,
};

// 运行函数
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  // 启用终端模式
  services.terminal.enter(&mut services.log);

  // 启动全局键盘监听器
  services.input.start_key_listener();

  // 临时测试动作表：后续由正式动作表解析器替换
  services.input.load_key_bindings(vec![
    KeyBinding {
      pattern: KeyPattern::Single(Key::Esc),
      action: "quit".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::Fn(1)),
      action: "overlay.push".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::Fn(2)),
      action: "overlay.pop".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Combo(Key::LeftCtrl, Key::S),
      action: "save".to_string(),
    },

    KeyBinding {
      pattern: KeyPattern::Single(Key::S),
      action: "single.s".to_string(),
    },
  ]);

  let mut last_input_action = String::from("InputAction: <none>");

  // 构建一个帧循环
  let mut scheduler = FrameScheduler::new();

  // 开始循环
  while world.session.is_running() {
    // 获取帧信息
    let frame = scheduler.begin_frame();

    // 更新帧时间
    world.clock.tick();

    // 输入帧：开始帧 → 轮询按键 → 处理键盘动作
    services.input.begin_frame();
    services.input.poll();

    // TODO(input-debug):
    // Remove this temporary action-event log after the producer-consumer
    // action event channel is connected.
    services.input.dispatch_action_events();

    while let Some(event) = services.input.next_action_event() {
      last_input_action = format!(
        "InputAction: type={:?} action={} state={:?}",
        event.event_type,
        event.action,
        event.state,
      );

      services.log.info(LogSource::Runtime, last_input_action.clone());
    }

    handle_runtime_keyboard(services, world);

    update(services, world, frame);
    render(services, world, frame, &last_input_action);

    // 请求帧更新：由 CanvasService 封装终端交互
    if let Err(err) = services.canvas.request_frame_update(&mut services.terminal) {
      services.log.warn(
        LogSource::Runtime,
        format!("[Render] Frame update failed: {}", err),
      );
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
// 每帧调用 begin_frame() 后，仅清除并重绘需要更新的区域。
// 绘图通过 RenderService 的文本 API 完成，不直接操作画布。
//
// 注意：此函数只负责绘图调用，不处理终端 I/O。
// 帧提交由主循环中的 request_frame_update() 统一完成。
fn render(services: &mut EngineServices, world: &mut RuntimeWorld, frame: u64, last_input_action: &str) {
  // 开始新帧（保留模式，不自动清空缓冲区）
  services.canvas.begin_frame();

  // 静态文本：通过 RenderService 绘制
  services.canvas.clear_span(2, 0, 40);
  services
    .render
    .draw_normal_text(&mut services.canvas, 2, 2, "Normal text");

  services.canvas.clear_span(4, 0, 40);
  services.render.draw_rich_text(
    &mut services.canvas,
    &services.rich_text,
    2,
    4,
    "f%<fg:red>Forced rich text</fg>",
  );

  services.canvas.clear_span(6, 0, 40);
  services.render.draw_text(
    &mut services.canvas,
    &services.rich_text,
    2,
    6,
    "Auto normal text",
  );

  services.canvas.clear_span(8, 0, 40);
  services.render.draw_text(
    &mut services.canvas,
    &services.rich_text,
    2,
    8,
    "f%Auto <bold>rich</bold> text",
  );

  // 调试指示器：脏区间计数
  let dirty_info = format!("Dirty spans: {}", services.canvas.dirty_spans().len());
  services.canvas.clear_span(14, 0, dirty_info.len() as u16);
  services
    .render
    .draw_normal_text(&mut services.canvas, 0, 14, &dirty_info);

  // 临时调试：运行时状态检查（后续由正式 UI 渲染替换）
  let state_info = format!(
    "Runtime: {:?} | Focus: {:?} | Context: {:?} | Surface: {:?} | UI: {:?} | Overlays: {}",
    world.session.runtime_state(),
    world.session.focus_state(),
    world.session.execution_context(),
    world.session.host_surface(),
    world.session.current_ui_node(),
    world.session.overlay_depth(),
  );
  services.canvas.clear_span(16, 0, state_info.len() as u16);
  services
    .render
    .draw_normal_text(&mut services.canvas, 0, 16, &state_info);

  // 临时调试：动作映射结果（后续由正式 UI 替换）
  services.canvas.clear_span(18, 0, 100);
  services
    .render
    .draw_normal_text(&mut services.canvas, 0, 18, last_input_action);
}
