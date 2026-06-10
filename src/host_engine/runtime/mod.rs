use crate::host_engine::core::{
  ExitState,
  FrameScheduler,
  RuntimeWorld,
  set_crash_phase,
};
use crate::host_engine::core::state_machine::UiNodeState;

use crate::host_engine::core::state_machine::UiNodeKind;

use crate::host_engine::services::{
  EngineServices,
  InputActionEvent,
  MouseEvent,
  SystemEvent,
  translate_action_map,
};

use crate::host_engine::ui::{
  HomeLayout,
  HomeUi,
  HomeUiCommand,
  SettingsLayout,
  SettingsUi,
  SettingsUiCommand,
};

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  services.terminal.enter(&mut services.log);
  services.input.start_key_listener();
  services.input.start_system_listener();

  let mut scheduler = FrameScheduler::new(60);

  // ── 顶层状态转换：Boot → Init → Runtime ──
  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());

  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  // ── 创建 UI 并加载动作表 ──
  let mut home_ui = HomeUi::init();
  let mut settings_ui = SettingsUi::init();
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

    // 按当前 UI 节点加载对应的 action map
    match world.state.current_ui_kind() {
      Some(UiNodeKind::Home) => load_home_action_map(services),
      Some(UiNodeKind::Settings) => load_settings_action_map(services),
      _ => {}
    }

    route_input_events(services, world, &mut home_ui, &mut settings_ui);

    if world.is_stopped() {
      break;
    }

    route_update(world, &mut home_ui, &mut settings_ui);

    if world.is_stopped() {
      break;
    }

    route_render(services, world, &home_ui, &settings_ui);

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

fn load_settings_action_map(services: &mut EngineServices) {
  let bindings =
    translate_action_map(&SettingsUi::action_map())
      .expect("failed to translate SettingsUi action map");

  services.input.load_key_bindings(bindings);
}

fn route_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
) {
  // 键盘事件
  while let Some(event) = services.input.next_action_event() {
    route_input_event(&event, world, home_ui, settings_ui);

    if world.is_stopped() {
      break;
    }
  }

  // 鼠标事件
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      let positions = home_ui.compute_positions(&services.layout, &services.i18n);
      for sys_event in services.input.drain_system_events() {
        if let SystemEvent::Mouse(me) = sys_event {
          route_mouse_event(&me, &positions, world, home_ui, settings_ui);
          if world.is_stopped() {
            break;
          }
        }
      }
    }
    Some(UiNodeKind::Settings) => {
      let positions = settings_ui.compute_positions(&services.layout, &services.i18n);
      for sys_event in services.input.drain_system_events() {
        if let SystemEvent::Mouse(me) = sys_event {
          route_settings_mouse_event(&me, &positions, world, settings_ui);
          if world.is_stopped() {
            break;
          }
        }
      }
    }
    _ => {
      let _ = services.input.drain_system_events();
    }
  }
}

fn route_input_event(
  event: &InputActionEvent,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.handle_event(event) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      if let Some(command) = settings_ui.handle_event(event) {
        apply_settings_command(command, world);
      }
    }
    _ => {}
  }
}

fn route_mouse_event(
  event: &MouseEvent,
  positions: &HomeLayout,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.handle_mouse_event(event, positions) {
        apply_home_command(command, world);
      }
    }
    _ => {}
  }
}

fn route_settings_mouse_event(
  event: &MouseEvent,
  positions: &SettingsLayout,
  world: &mut RuntimeWorld,
  settings_ui: &mut SettingsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Settings) => {
      if let Some(command) = settings_ui.handle_mouse_event(event, positions) {
        apply_settings_command(command, world);
      }
    }
    _ => {}
  }
}

fn route_update(
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      let dt = world.clock.delta_time();
      if let Some(command) = home_ui.update(dt) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      let dt = world.clock.delta_time();
      let _ = settings_ui.update(dt);
    }
    _ => {}
  }
}

fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &HomeUi,
  settings_ui: &SettingsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      home_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
    }
    Some(UiNodeKind::Settings) => {
      settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
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
    HomeUiCommand::OpenSettings => {
      world.state.enter_ui_node(UiNodeState::settings());
    }
    HomeUiCommand::OpenAbout => {}
  }
}

fn apply_settings_command(
  command: SettingsUiCommand,
  world: &mut RuntimeWorld,
) {
  match command {
    SettingsUiCommand::Back => {
      world.state.pop_ui_node();
    }
  }
}
