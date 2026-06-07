// 引用结构体和枚举
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld, set_crash_phase};
use crate::host_engine::services::EngineServices;

// 运行函数
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  // 启用终端模式
  services.terminal.enter(&mut services.log);

  // 启动全局键盘监听器
  services.input.start_key_listener();

  // 构建一个帧循环
  let mut scheduler = FrameScheduler::new(60);

  // ── 顶层状态转换：Boot → Init → Runtime ──
  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());

  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  // ── 顶层运行时循环 ──
  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();

    services.canvas.begin_frame();
    services.canvas.clear();

    // TODO(runtime):
    // 1. input events -> state machine handle_event()
    // 2. state machine update(dt)
    // 3. state machine render()

    let _ = services.canvas.present(&mut services.terminal);

    scheduler.wait_for_next_frame();

    // TODO(runtime): 在真实事件路由存在之前的临时停止条件
    world.state.enter_shutdown();
    set_crash_phase(world.state.crash_phase());

    world.state.enter_stopped();
    set_crash_phase(world.state.crash_phase());
  }

  // 返回退出信息块
  ExitState::new()
}
