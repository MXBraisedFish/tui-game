use crate::host_engine::core::state_machine::{MainHostState, OverlayKind, UiNodeState};
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld, set_crash_phase};

use crate::host_engine::core::state_machine::{HostState, UiNodeKind};

use crate::host_engine::services::{
  EngineServices, InputActionEvent, MouseButton, MouseEvent, MouseEventKind, SystemEvent,
  UiObjectPoolOwner, translate_action_map,
};

use crate::host_engine::ui::{
  self, HomeLayout, HomeUi, HomeUiCommand, InputDemoCommand, InputDemoUi, LanguageSelectCommand,
  LanguageSelectLayout, LanguageSelectUi, ModsCommand, ModsLayout, ModsUi, SettingsLayout,
  SettingsUi, SettingsUiCommand, TerminalCheckCommand, TerminalCheckLayout, TerminalCheckUi,
  WindowSizeWarningCommand,
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

  // ── 创建 UI ──
  let registry = services.i18n.language_registry().to_vec();
  let mut home_ui = HomeUi::init();
  let mut settings_ui = SettingsUi::init();
  let mut language_select_ui = if registry.is_empty() {
    None
  } else {
    Some(LanguageSelectUi::init(
      registry,
      &services.storage,
      &mut services.log,
    ))
  };
  let mut terminal_check_ui = TerminalCheckUi::init();
  let mut mods_ui = ModsUi::init();
  let mut input_demo_ui = InputDemoUi::init(&services.text_input);

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

    // resize 事件：更新画布尺寸并标记强制重绘
    services.input.poll_resize_events(|w, h| {
      services.canvas.resize(w, h);
      services.canvas.request_render();
      services.presenter.request_render();
    });

    services.canvas.begin_frame();
    services.canvas.clear();

    // 窗口尺寸检查与覆盖层管理
    manage_window_size_overlay(services, world);

    // 输入所有权：覆盖层 > 文本输入 > 当前页面 action map
    if world.state.current_overlay_kind().is_some() {
      load_window_size_action_map(services);
      services.input.dispatch_action_events();
      route_input_events(
        services,
        world,
        &mut home_ui,
        &mut settings_ui,
        language_select_ui.as_mut(),
        &mut terminal_check_ui,
        &mut mods_ui,
        &mut input_demo_ui,
      );
    } else if services.text_input.is_active() {
      route_text_input_events(
        services,
        world,
        &mut home_ui,
        &mut settings_ui,
        language_select_ui.as_mut(),
        &mut terminal_check_ui,
        &mut mods_ui,
        &mut input_demo_ui,
      );
    } else {
      match world.state.current_ui_kind() {
        Some(UiNodeKind::Home) => load_home_action_map(services),
        Some(UiNodeKind::Settings) => load_settings_action_map(services),
        Some(UiNodeKind::LanguageSelect) => load_language_select_action_map(services),
        Some(UiNodeKind::Mods) => load_mods_action_map(services),
        Some(UiNodeKind::TerminalCheck) => load_terminal_check_action_map(services),
        Some(UiNodeKind::InputDemo) => load_input_demo_action_map(services),
        _ => {}
      }
      services.input.dispatch_action_events();
      route_input_events(
        services,
        world,
        &mut home_ui,
        &mut settings_ui,
        language_select_ui.as_mut(),
        &mut terminal_check_ui,
        &mut mods_ui,
        &mut input_demo_ui,
      );
    }

    if world.is_stopped() {
      break;
    }

    route_update(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut input_demo_ui,
    );

    if world.is_stopped() {
      break;
    }

    let input_cursor = route_render(
      services,
      world,
      &home_ui,
      &settings_ui,
      language_select_ui.as_ref(),
      &terminal_check_ui,
      &mods_ui,
      &input_demo_ui,
    );

    let text_force_redraw = services.canvas.take_render_requested();
    let composed = services.compositor.compose(&services.canvas);
    let _ = services.presenter.present(
      &composed,
      &mut services.terminal,
      text_force_redraw,
      input_cursor,
    );

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

fn load_mods_action_map(services: &mut EngineServices) {
  let bindings =
    translate_action_map(&ModsUi::action_map()).expect("failed to translate ModsUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_terminal_check_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&TerminalCheckUi::action_map())
    .expect("failed to translate TerminalCheckUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_window_size_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&ui::window_size_warning::action_map())
    .expect("failed to translate window_size action map");

  services.input.load_key_bindings(bindings);
}

fn load_input_demo_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&InputDemoUi::action_map())
    .expect("failed to translate InputDemoUi action map");
  services.input.load_key_bindings(bindings);
}

fn route_text_input_events(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  input_demo_ui: &mut InputDemoUi,
) {
  for event in services.input.drain_system_events() {
    if let SystemEvent::TerminalKey(key) = event {
      let objects = match world.state.current_ui_kind() {
        Some(UiNodeKind::Home) => Some(home_ui.objects_mut()),
        Some(UiNodeKind::Settings) => Some(settings_ui.objects_mut()),
        Some(UiNodeKind::LanguageSelect) => language_select_ui
          .as_deref_mut()
          .map(UiObjectPoolOwner::objects_mut),
        Some(UiNodeKind::Mods) => Some(mods_ui.objects_mut()),
        Some(UiNodeKind::TerminalCheck) => Some(terminal_check_ui.objects_mut()),
        Some(UiNodeKind::InputDemo) => Some(input_demo_ui.objects_mut()),
        _ => None,
      };
      if let Some(objects) = objects {
        services.text_input.route_terminal_key(objects, key);
      }
    }
  }
}

fn route_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  input_demo_ui: &mut InputDemoUi,
) {
  // 覆盖层输入优先
  if world.state.current_overlay_kind().is_some() {
    while let Some(event) = services.input.next_action_event() {
      if let Some(cmd) = ui::window_size_warning::handle_event(&event) {
        apply_window_size_command(cmd, world);
      }
      if world.is_stopped() {
        break;
      }
    }
    // 鼠标右键等同于 Esc
    for sys_event in services.input.drain_system_events() {
      if let SystemEvent::Mouse(me) = sys_event {
        if me.kind == MouseEventKind::Press && me.button == Some(MouseButton::Right) {
          apply_window_size_command(WindowSizeWarningCommand::Exit, world);
        }
      }
      if world.is_stopped() {
        break;
      }
    }
    return;
  }

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
      mods_ui,
      input_demo_ui,
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
    Some(UiNodeKind::Mods) => {
      let positions = mods_ui.compute_positions(&services.layout, &services.i18n);
      for sys_event in services.input.drain_system_events() {
        if let SystemEvent::Mouse(me) = sys_event {
          route_mods_mouse_event(&me, &positions, world, mods_ui);
          if world.is_stopped() {
            break;
          }
        }
      }
    }
    Some(UiNodeKind::TerminalCheck) => {
      let positions = terminal_check_ui.compute_positions(&services.layout, &services.i18n);
      for sys_event in services.input.drain_system_events() {
        if let SystemEvent::Mouse(me) = sys_event {
          route_terminal_check_mouse_event(&me, &positions, services, world, terminal_check_ui);
          if world.is_stopped() {
            break;
          }
        }
      }
    }
    Some(UiNodeKind::InputDemo) => {
      let _ = services.input.drain_system_events();
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
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  input_demo_ui: &mut InputDemoUi,
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
    Some(UiNodeKind::Mods) => {
      if let Some(command) = mods_ui.handle_event(event) {
        apply_mods_command(command, world);
      }
    }
    Some(UiNodeKind::TerminalCheck) => {
      if let Some(command) = terminal_check_ui.handle_event(event) {
        apply_terminal_check_command(command, terminal_check_ui, services, world);
      }
    }
    Some(UiNodeKind::InputDemo) => {
      if let Some(command) = input_demo_ui.handle_event(event) {
        apply_input_demo_command(command, input_demo_ui, services, world);
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

fn route_mods_mouse_event(
  event: &MouseEvent,
  positions: &ModsLayout,
  world: &mut RuntimeWorld,
  mods_ui: &mut ModsUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Mods) => {
      if let Some(command) = mods_ui.handle_mouse_event(event, positions) {
        apply_mods_command(command, world);
      }
    }
    _ => {}
  }
}

fn route_terminal_check_mouse_event(
  event: &MouseEvent,
  positions: &TerminalCheckLayout,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  terminal_check_ui: &mut TerminalCheckUi,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::TerminalCheck) => {
      if let Some(command) = terminal_check_ui.handle_mouse_event(event, positions) {
        apply_terminal_check_command(command, terminal_check_ui, services, world);
      }
    }
    _ => {}
  }
}

fn route_update(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  input_demo_ui: &mut InputDemoUi,
) {
  // 覆盖层无逐帧逻辑
  if world.state.current_overlay_kind().is_some() {
    return;
  }

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
    Some(UiNodeKind::Mods) => {
      let dt = world.clock.delta_time();
      let _ = mods_ui.update(dt);
    }
    Some(UiNodeKind::TerminalCheck) => {
      let dt = world.clock.delta_time();
      if let Some(command) = terminal_check_ui.update(dt) {
        apply_terminal_check_command(command, terminal_check_ui, services, world);
      }
    }
    Some(UiNodeKind::InputDemo) => {
      input_demo_ui.update(&mut services.text_input);
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
  mods_ui: &ModsUi,
  input_demo_ui: &InputDemoUi,
) -> Option<(u16, u16)> {
  // 覆盖层渲染优先
  if let Some(OverlayKind::WindowSizeWarning) = world.state.current_overlay_kind() {
    let runtime = world.state.runtime().unwrap();
    let overlay = runtime.overlays().top().unwrap();
    let req_w = overlay.render.required_width;
    let req_h = overlay.render.required_height;
    let term = services.layout.get_terminal_size();

    ui::window_size_warning::render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      req_w,
      req_h,
      term.width,
      term.height,
      world.state.is_host_mode(),
    );
    return None;
  }

  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      home_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
      None
    }
    Some(UiNodeKind::Settings) => {
      settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
      None
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
      None
    }
    Some(UiNodeKind::Mods) => {
      mods_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
      None
    }
    Some(UiNodeKind::TerminalCheck) => {
      terminal_check_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
      None
    }
    Some(UiNodeKind::InputDemo) => input_demo_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.text_input,
    ),
    _ => None,
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
    HomeUiCommand::OpenAbout => {
      world.state.enter_ui_node(UiNodeState::input_demo());
    }
  }
}

fn apply_input_demo_command(
  command: InputDemoCommand,
  input_demo_ui: &mut InputDemoUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    InputDemoCommand::SelectPrevious => input_demo_ui.select_previous(),
    InputDemoCommand::SelectNext => input_demo_ui.select_next(),
    InputDemoCommand::FocusInput => input_demo_ui.focus(&mut services.text_input),
    InputDemoCommand::Back => {
      world.state.pop_ui_node();
    }
  }
}

fn apply_settings_command(command: SettingsUiCommand, world: &mut RuntimeWorld) {
  match command {
    SettingsUiCommand::Back => {
      world.state.pop_ui_node();
    }
    SettingsUiCommand::OpenLanguageSelect => {
      world.state.enter_ui_node(UiNodeState::language_select());
    }
    SettingsUiCommand::OpenMods => {
      world.state.enter_ui_node(UiNodeState::mods());
    }
  }
}

fn apply_mods_command(command: ModsCommand, world: &mut RuntimeWorld) {
  match command {
    ModsCommand::Back => {
      world.state.pop_ui_node();
    }
    ModsCommand::OpenGame => {}
    ModsCommand::OpenScreensaver => {}
  }
}

fn apply_language_select_command(
  command: LanguageSelectCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    LanguageSelectCommand::Confirm(code) => {
      let _ = services.storage.write_language_code(&code);
      services
        .i18n
        .load_runtime_language(&services.storage, &mut services.log, &code);
      services.i18n.set_current_language(code);
      // 不退出，留在语言页面让用户看到效果
    }
    LanguageSelectCommand::Back => {
      world.state.pop_ui_node();
      if !services.storage.is_terminal_profile_complete() {
        world.state.enter_ui_node(UiNodeState::terminal_check());
      }
    }
  }
}

fn apply_terminal_check_command(
  command: TerminalCheckCommand,
  terminal_check_ui: &mut TerminalCheckUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    TerminalCheckCommand::Next => {
      terminal_check_ui.persist_current_step(&mut services.storage);
      terminal_check_ui.advance_step();
    }
    TerminalCheckCommand::Done { mouse } => {
      let _ = services.storage.update_terminal_profile(|p| {
        p.mouse = Some(mouse);
      });
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

// ── 窗口尺寸覆盖层 ──

/// 每帧调用：当终端过小时推入覆盖层，尺寸恢复时自动弹出。
fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  let term = services.layout.get_terminal_size();

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
      // 自动解除：终端尺寸已满足需求
      let runtime = world.state.runtime().unwrap();
      if let Some(overlay) = runtime.overlays().top() {
        let req_w = overlay.render.required_width as u16;
        let req_h = overlay.render.required_height as u16;
        if term.width >= req_w && term.height >= req_h {
          world.state.pop_overlay();
        }
      }
    }
    None => {
      // 无覆盖层时，检查终端尺寸是否达标
      let (min_w, min_h) = get_min_window_size(world);
      if (term.width as u32) < min_w || (term.height as u32) < min_h {
        world.state.push_window_size_overlay(min_w, min_h);
      }
    }
    _ => {} // 其他覆盖层：跳过尺寸检查
  }
}

/// 获取当前模式下的最小窗口尺寸。
/// Host 模式：固定 80×24。
/// Game 模式：占位，后续读取活跃游戏包的 PackageRuntime。
fn get_min_window_size(world: &RuntimeWorld) -> (u32, u32) {
  if world.state.is_host_mode() {
    (80, 24)
  } else {
    // 游戏模式占位：暂用与 Host 相同的默认值
    (80, 24)
  }
}

fn apply_window_size_command(cmd: WindowSizeWarningCommand, world: &mut RuntimeWorld) {
  match cmd {
    WindowSizeWarningCommand::Exit => {
      if world.state.is_host_mode() {
        // Host 模式：退出程序
        world.state.pop_overlay();
        world.state.enter_shutdown();
        set_crash_phase(world.state.crash_phase());
        world.state.enter_stopped();
        set_crash_phase(world.state.crash_phase());
      } else {
        // Game 模式：返回游戏列表（切回 Host）
        world.state.pop_overlay();
        if let Some(runtime) = world.state.runtime_mut() {
          runtime.set_main_host(MainHostState::Host(HostState::new()));
        }
      }
    }
  }
}
