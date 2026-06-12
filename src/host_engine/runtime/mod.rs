use crate::host_engine::core::state_machine::UiNodeState;
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld, set_crash_phase};

use crate::host_engine::core::state_machine::UiNodeKind;

use crate::host_engine::services::{
  DetectionResult, EngineServices, InputActionEvent, MouseEvent, SystemEvent,
  TerminalDetector, translate_action_map,
};

use crate::host_engine::ui::{
  HomeLayout, HomeUi, HomeUiCommand, LanguageSelectCommand, LanguageSelectLayout, LanguageSelectUi,
  SettingsLayout, SettingsUi, SettingsUiCommand, TerminalCheckCommand, TerminalCheckLayout,
  TerminalCheckUi,
};

pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  services.terminal.enter(&mut services.log);

  // 图片协议自动检测：必须在 crossterm 事件监听启动前完成（stdin 尚未被占用）
  let detection = match services.terminal.writer_mut() {
    Some(stdout) => TerminalDetector::detect_in_terminal(stdout),
    None => DetectionResult::default(),
  };

  services.input.start_key_listener();
  services.input.start_system_listener();

  let mut scheduler = FrameScheduler::new(60);

  // ── 顶层状态转换：Boot → Init → Runtime ──
  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());

  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  // ── 创建 UI ──
  let registry = services.i18n.language_registry().to_vec();
  let mut home_ui = HomeUi::init();
  let mut settings_ui = SettingsUi::init();
  let mut language_select_ui = if registry.is_empty() {
    None
  } else {
    Some(LanguageSelectUi::init(registry))
  };
  let mut terminal_check_ui = TerminalCheckUi::init(&detection);

  // 初始 UI 节点
  // 1) 无语言 → LanguageSelect
  // 2) 终端能力不完整 → TerminalCheck
  // 3) 否则 → Home（默认已在树中）
  if services.storage.read_language_code().is_none() && language_select_ui.is_some() {
    world.state.enter_ui_node(UiNodeState::language_select());
  } else if !services.storage.is_terminal_profile_complete() {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }

  // ── 主循环 ──
  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();

    services.input.begin_frame();
    services.input.poll();
    services.input.dispatch_action_events();

    // resize 事件：更新画布尺寸并标记强制重绘
    services.input.poll_resize_events(|w, h| {
      services.canvas.resize(w, h);
      services.canvas.request_render();
    });

    services.canvas.begin_frame();
    services.canvas.clear();

    // 按当前 UI 节点加载对应的 action map
    match world.state.current_ui_kind() {
      Some(UiNodeKind::Home) => load_home_action_map(services),
      Some(UiNodeKind::Settings) => load_settings_action_map(services),
      Some(UiNodeKind::LanguageSelect) => load_language_select_action_map(services),
      Some(UiNodeKind::TerminalCheck) => load_terminal_check_action_map(services),
      _ => {}
    }

    route_input_events(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
    );

    if world.is_stopped() {
      break;
    }

    route_update(
      world,
      &mut home_ui,
      &mut settings_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
    );

    if world.is_stopped() {
      break;
    }

    route_render(
      services,
      world,
      &home_ui,
      &settings_ui,
      language_select_ui.as_ref(),
      &terminal_check_ui,
    );

    let _ = services.canvas.present(&mut services.terminal);

    scheduler.wait_for_next_frame();
  }

  ExitState::new()
}

// ── 辅助函数 ──

fn load_home_action_map(services: &mut EngineServices) {
  let bindings =
    translate_action_map(&HomeUi::action_map()).expect("failed to translate HomeUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_settings_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&SettingsUi::action_map())
    .expect("failed to translate SettingsUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_language_select_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&LanguageSelectUi::action_map())
    .expect("failed to translate LanguageSelectUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_terminal_check_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&TerminalCheckUi::action_map())
    .expect("failed to translate TerminalCheckUi action map");

  services.input.load_key_bindings(bindings);
}

fn route_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
) {
  // 键盘事件
  while let Some(event) = services.input.next_action_event() {
    route_input_event(
      &event,
      services,
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
    );

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
          route_mouse_event(
            &me,
            &positions,
            world,
            home_ui,
            settings_ui,
            terminal_check_ui,
          );
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
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(ui) = language_select_ui.as_ref() {
        let positions = ui.compute_positions(&services.layout);
        for sys_event in services.input.drain_system_events() {
          if let SystemEvent::Mouse(me) = sys_event {
            if let Some(ui_mut) = language_select_ui.as_mut() {
              route_language_select_mouse_event(&me, &positions, services, world, ui_mut);
            }
            if world.is_stopped() {
              break;
            }
          }
        }
      }
    }
    Some(UiNodeKind::TerminalCheck) => {
      let positions = terminal_check_ui.compute_positions(&services.layout, &services.i18n);
      for sys_event in services.input.drain_system_events() {
        if let SystemEvent::Mouse(me) = sys_event {
          route_terminal_check_mouse_event(&me, &positions, world, terminal_check_ui);
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
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
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
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(ui) = language_select_ui {
        if let Some(command) = ui.handle_event(event) {
          apply_language_select_command(command, services, world);
        }
      }
    }
    Some(UiNodeKind::TerminalCheck) => {
      if let Some(command) = terminal_check_ui.handle_event(event) {
        apply_terminal_check_command(command, terminal_check_ui, world);
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
  _settings_ui: &mut SettingsUi,
  _terminal_check_ui: &mut TerminalCheckUi,
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

fn route_language_select_mouse_event(
  event: &MouseEvent,
  positions: &LanguageSelectLayout,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  language_select_ui: &mut LanguageSelectUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(command) = language_select_ui.handle_mouse_event(event, positions) {
        apply_language_select_command(command, services, world);
      }
    }
    _ => {}
  }
}

fn route_terminal_check_mouse_event(
  event: &MouseEvent,
  positions: &TerminalCheckLayout,
  world: &mut RuntimeWorld,
  terminal_check_ui: &mut TerminalCheckUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::TerminalCheck) => {
      if let Some(command) = terminal_check_ui.handle_mouse_event(event, positions) {
        apply_terminal_check_command(command, terminal_check_ui, world);
      }
    }
    _ => {}
  }
}

fn route_update(
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
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
    Some(UiNodeKind::LanguageSelect) => {
      let dt = world.clock.delta_time();
      let _ = language_select_ui.as_mut().and_then(|ui| ui.update(dt));
    }
    Some(UiNodeKind::TerminalCheck) => {
      let dt = world.clock.delta_time();
      if let Some(command) = terminal_check_ui.update(dt) {
        apply_terminal_check_command(command, terminal_check_ui, world);
      }
    }
    _ => {}
  }
}

fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &HomeUi,
  settings_ui: &SettingsUi,
  language_select_ui: Option<&LanguageSelectUi>,
  terminal_check_ui: &TerminalCheckUi,
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
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(ui) = language_select_ui {
        ui.render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
        );
      }
    }
    Some(UiNodeKind::TerminalCheck) => {
      terminal_check_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
    }
    _ => {}
  }
}

fn apply_home_command(command: HomeUiCommand, world: &mut RuntimeWorld) {
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

fn apply_settings_command(command: SettingsUiCommand, world: &mut RuntimeWorld) {
  match command {
    SettingsUiCommand::Back => {
      world.state.pop_ui_node();
    }
  }
}

fn apply_language_select_command(
  command: LanguageSelectCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    LanguageSelectCommand::Confirm(code) => {
      // 持久化语言
      let _ = services.storage.write_language_code(&code);
      // 重新加载运行时文本
      services
        .i18n
        .load_runtime_language(&services.storage, &mut services.log, &code);
      services.i18n.set_current_language(code);
      // 返回上一级
      world.state.pop_ui_node();
      // 检查终端能力 → 决定下一步
      if !services.storage.is_terminal_profile_complete() {
        world.state.enter_ui_node(UiNodeState::terminal_check());
      }
    }
    LanguageSelectCommand::Exit => {
      world.state.enter_shutdown();
      set_crash_phase(world.state.crash_phase());
      world.state.enter_stopped();
      set_crash_phase(world.state.crash_phase());
    }
  }
}

fn apply_terminal_check_command(
  command: TerminalCheckCommand,
  terminal_check_ui: &mut TerminalCheckUi,
  world: &mut RuntimeWorld,
) {
  match command {
    TerminalCheckCommand::Next => {
      terminal_check_ui.advance_step();
    }
    TerminalCheckCommand::Done => {
      world.state.pop_ui_node();
    }
    TerminalCheckCommand::Exit => {
      world.state.enter_shutdown();
      set_crash_phase(world.state.crash_phase());
      world.state.enter_stopped();
      set_crash_phase(world.state.crash_phase());
    }
  }
}
