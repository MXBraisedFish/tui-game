use crate::host_engine::core::{
  ExitState,
  FrameScheduler,
  RuntimeWorld,
  set_crash_phase,
};

use crate::host_engine::core::state_machine::UiNodeKind;

use crate::host_engine::services::{
  EngineServices,
  InputActionEvent,
  translate_action_map,
};

use crate::host_engine::ui::{
  HomeUi,
  HomeUiCommand,
};

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  services.terminal.enter(&mut services.log);
  services.input.start_key_listener();

  let mut scheduler = FrameScheduler::new(60);

  // ── 顶层状态转换：Boot → Init → Runtime ──
  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());

  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  // ── 创建 HomeUi 并加载动作表 ──
  let mut home_ui = HomeUi::init();
  load_home_action_map(services);

  // ── 主循环 ──
  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();

    services.input.begin_frame();
    services.input.poll();
    services.input.dispatch_action_events();

    services.canvas.begin_frame();
    services.canvas.clear();

    route_input_events(services, world, &mut home_ui);

    if world.is_stopped() {
      break;
    }

    route_update(world, &mut home_ui);

    if world.is_stopped() {
      break;
    }

    route_render(services, world, &home_ui);

    let _ = services.canvas.present(&mut services.terminal);

    scheduler.wait_for_next_frame();
  }

  ExitState::new()
}

// ── 辅助函数 ──

fn load_home_action_map(services: &mut EngineServices) {
  let bindings =
    translate_action_map(&HomeUi::action_map())
      .expect("failed to translate HomeUi action map");

  services.input.load_key_bindings(bindings);
}

fn route_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
) {
  while let Some(event) = services.input.next_action_event() {
    route_input_event(&event, world, home_ui);

    if world.is_stopped() {
      break;
    }
  }
}

fn route_input_event(
  event: &InputActionEvent,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.handle_event(event) {
        apply_home_command(command, world);
      }
    }

    _ => {}
  }
}

fn route_update(
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      let dt = world.clock.delta_time();

      if let Some(command) = home_ui.update(dt) {
        apply_home_command(command, world);
      }
    }

    _ => {}
  }
}

fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &HomeUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      home_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
      );
    }

    _ => {}
  }
}

fn apply_home_command(
  command: HomeUiCommand,
  world: &mut RuntimeWorld,
) {
  match command {
    HomeUiCommand::Exit => {
      world.state.enter_shutdown();
      set_crash_phase(world.state.crash_phase());

      world.state.enter_stopped();
      set_crash_phase(world.state.crash_phase());
    }

    HomeUiCommand::StartGame => {}
    HomeUiCommand::ContinueGame => {}
    HomeUiCommand::OpenSettings => {}
    HomeUiCommand::OpenAbout => {}
  }
}
