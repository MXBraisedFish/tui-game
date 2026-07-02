use crate::host_engine::core::state_machine::{MainHostState, OverlayKind, UiNodeState};
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld, set_crash_phase};

use crate::host_engine::core::state_machine::{HostState, UiNodeKind};

use crate::host_engine::services::{
  EngineServices, HostAreaKind, MouseEvent, PackageEvent, Rect, SystemEvent, UiEvent, UiObjectPool,
  UiObjectPoolOwner, translate_action_map,
};

use crate::host_engine::ui::{
  GamePackageCommand, GamePackageUi, HomeUi, HomeUiCommand, InputDemoCommand, InputDemoUi,
  LanguageLoadingUi, LanguageSelectCommand, LanguageSelectUi, ModsCommand, ModsUi,
  ScreensaverPackageCommand, ScreensaverPackageUi, SettingsUi, SettingsUiCommand,
  TerminalCheckCommand, TerminalCheckLayout, TerminalCheckUi, WindowSizeWarningCommand,
  WindowSizeWarningUi,
};

#[derive(Default)]
struct LanguageLoadingRuntime {
  active: bool,
  pending_language: Option<String>,
  enter_terminal_check_after_finish: bool,
}

/// 运行引擎主循环：初始化 UI 并循环处理输入、更新与渲染，直到退出
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  services.terminal.enter(&mut services.log);

  services.input.start_key_listener();
  services.input.start_system_listener();

  let mut scheduler = FrameScheduler::new(60);

  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());

  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  let registry = services.i18n.language_registry().to_vec();
  let mut home_ui = HomeUi::init(&services.hit_area);
  let mut settings_ui = SettingsUi::init(&services.hit_area);
  let mut language_select_ui = if registry.is_empty() {
    None
  } else {
    Some(LanguageSelectUi::init(
      registry,
      &services.storage,
      &mut services.log,
      &services.hit_area,
    ))
  };
  let mut terminal_check_ui = TerminalCheckUi::init();
  let mut mods_ui = ModsUi::init(&services.hit_area);
  let mut game_package_ui = GamePackageUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
  let mut screensaver_package_ui = ScreensaverPackageUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
  let mut input_demo_ui =
    InputDemoUi::init(&services.hit_area, &services.slice, &services.scroll_box);
  let mut window_size_ui = WindowSizeWarningUi::init(&services.hit_area);
  let mut language_loading_ui = LanguageLoadingUi::init(&services.progress_bar);
  let mut language_loading = LanguageLoadingRuntime::default();

  if services.storage.read_language_code().is_none() && language_select_ui.is_some() {
    world.state.enter_ui_node(UiNodeState::language_select());
  } else if !services.storage.is_terminal_profile_complete() {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }

  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();

    services.input.begin_frame();
    services.input.poll();
    let package_events = services.package.poll_events(&mut services.log);
    apply_language_loading_package_events(
      &package_events,
      &mut language_loading,
      &mut language_loading_ui,
      services,
      world,
    );

    services.input.poll_resize_events(|w, h| {
      services.layout.resize_physical(w, h);
      services.canvas.resize(w, h);
      services.canvas.request_render();
      services.presenter.request_render();
    });

    services.canvas.begin_frame(&services.layout);

    manage_window_size_overlay(services, world);
    deactivate_hidden_pools(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut window_size_ui,
    );

    if world.state.current_overlay_kind() == Some(OverlayKind::WindowSizeWarning) {
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
        &mut game_package_ui,
        &mut screensaver_package_ui,
        &mut input_demo_ui,
        &mut window_size_ui,
        &mut language_loading_ui,
        &mut language_loading,
      );
    } else if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
      while services.input.next_action_event().is_some() {}
      services.input.clear();
      let _ = services.input.drain_system_events();
    } else if services.text_input.is_active() {
      route_text_input_events(
        services,
        world,
        &mut home_ui,
        &mut settings_ui,
        language_select_ui.as_mut(),
        &mut terminal_check_ui,
        &mut mods_ui,
        &mut game_package_ui,
        &mut screensaver_package_ui,
        &mut input_demo_ui,
        &mut language_loading_ui,
        &mut language_loading,
      );
    } else {
      match world.state.current_ui_kind() {
        Some(UiNodeKind::Home) => load_home_action_map(services),
        Some(UiNodeKind::Settings) => load_settings_action_map(services),
        Some(UiNodeKind::LanguageSelect) => load_language_select_action_map(services),
        Some(UiNodeKind::Mods) => load_mods_action_map(services),
        Some(UiNodeKind::GamePackage) => load_game_package_action_map(services),
        Some(UiNodeKind::ScreensaverPackage) => load_screensaver_package_action_map(services),
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
        &mut game_package_ui,
        &mut screensaver_package_ui,
        &mut input_demo_ui,
        &mut window_size_ui,
        &mut language_loading_ui,
        &mut language_loading,
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
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut language_loading_ui,
      &mut language_loading,
    );

    if world.is_stopped() {
      break;
    }

    let input_cursor = route_render(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut window_size_ui,
      &mut language_loading_ui,
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

fn load_game_package_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&GamePackageUi::action_map())
    .expect("failed to translate GamePackageUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_screensaver_package_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&ScreensaverPackageUi::action_map())
    .expect("failed to translate ScreensaverPackageUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_terminal_check_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&TerminalCheckUi::action_map())
    .expect("failed to translate TerminalCheckUi action map");

  services.input.load_key_bindings(bindings);
}

fn load_window_size_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&WindowSizeWarningUi::action_map())
    .expect("failed to translate window_size action map");

  services.input.load_key_bindings(bindings);
}

fn load_input_demo_action_map(services: &mut EngineServices) {
  let bindings = translate_action_map(&InputDemoUi::action_map())
    .expect("failed to translate InputDemoUi action map");
  services.input.load_key_bindings(bindings);
}

fn current_objects_mut<'a>(
  world: &RuntimeWorld,
  home_ui: &'a mut HomeUi,
  settings_ui: &'a mut SettingsUi,
  language_select_ui: Option<&'a mut LanguageSelectUi>,
  terminal_check_ui: &'a mut TerminalCheckUi,
  mods_ui: &'a mut ModsUi,
  game_package_ui: &'a mut GamePackageUi,
  screensaver_package_ui: &'a mut ScreensaverPackageUi,
  input_demo_ui: &'a mut InputDemoUi,
) -> Option<&'a mut UiObjectPool> {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => Some(home_ui.objects_mut()),
    Some(UiNodeKind::Settings) => Some(settings_ui.objects_mut()),
    Some(UiNodeKind::LanguageSelect) => language_select_ui.map(UiObjectPoolOwner::objects_mut),
    Some(UiNodeKind::TerminalCheck) => Some(terminal_check_ui.objects_mut()),
    Some(UiNodeKind::Mods) => Some(mods_ui.objects_mut()),
    Some(UiNodeKind::GamePackage) => Some(game_package_ui.objects_mut()),
    Some(UiNodeKind::ScreensaverPackage) => Some(screensaver_package_ui.objects_mut()),
    Some(UiNodeKind::InputDemo) => Some(input_demo_ui.objects_mut()),
    _ => None,
  }
}

// 将非活跃 UI 对应的对象池反激活，确保只有当前界面响应点击和输入
fn deactivate_hidden_pools(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
) {
  let active = world
    .state
    .current_overlay_kind()
    .is_none()
    .then(|| world.state.current_ui_kind())
    .flatten();
  let mut deactivate = |kind, pool: &mut UiObjectPool| {
    if active != Some(kind) {
      services.text_input.deactivate_pool(pool);
      services.hit_area.deactivate(pool);
    }
  };
  deactivate(UiNodeKind::Home, home_ui.objects_mut());
  deactivate(UiNodeKind::Settings, settings_ui.objects_mut());
  if let Some(ui) = language_select_ui {
    deactivate(UiNodeKind::LanguageSelect, ui.objects_mut());
  }
  deactivate(UiNodeKind::TerminalCheck, terminal_check_ui.objects_mut());
  deactivate(UiNodeKind::Mods, mods_ui.objects_mut());
  deactivate(UiNodeKind::GamePackage, game_package_ui.objects_mut());
  deactivate(
    UiNodeKind::ScreensaverPackage,
    screensaver_package_ui.objects_mut(),
  );
  deactivate(UiNodeKind::InputDemo, input_demo_ui.objects_mut());
  if world.state.current_overlay_kind() != Some(OverlayKind::WindowSizeWarning) {
    services
      .text_input
      .deactivate_pool(window_size_ui.objects_mut());
    services.hit_area.deactivate(window_size_ui.objects_mut());
  }
}

fn route_component_mouse(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  event: MouseEvent,
) -> bool {
  let Some(pool) = current_objects_mut(
    world,
    home_ui,
    settings_ui,
    language_select_ui,
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
  ) else {
    return false;
  };
  if services
    .scroll_box
    .route_mouse_event(pool, &services.canvas, &services.layout, event)
  {
    services.canvas.request_render();
    return true;
  }
  services
    .hit_area
    .route_mouse_event(pool, &mut services.text_input, &services.canvas, event)
}

fn route_mouse_and_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  event: MouseEvent,
) -> bool {
  let consumed = route_component_mouse(
    services,
    world,
    home_ui,
    settings_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
    event,
  );
  route_component_events(
    services,
    world,
    home_ui,
    settings_ui,
    language_select_ui,
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
    language_loading_ui,
    language_loading,
  );
  consumed
}

fn route_component_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  loop {
    let event = current_objects_mut(
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
    )
    .and_then(UiObjectPool::pop_event);
    let Some(event) = event else { break };
    route_input_event(
      &event,
      services,
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      language_loading_ui,
      language_loading,
    );
    if world.is_stopped() {
      break;
    }
  }
}

fn route_text_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  for event in services.input.drain_system_events() {
    match event {
      SystemEvent::TerminalKey(key) => {
        if let Some(objects) = current_objects_mut(
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
        ) {
          services
            .text_input
            .route_terminal_key(objects, &mut services.clipboard, key);
        }
      }
      SystemEvent::Mouse(mouse) => {
        route_component_mouse(
          services,
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
          mouse,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        if let Some(objects) = current_objects_mut(
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
        ) {
          services.hit_area.focus_lost(objects);
        }
      }
      _ => {}
    }
    route_component_events(
      services,
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      language_loading_ui,
      language_loading,
    );
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
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if world.state.current_overlay_kind().is_some() {
    while let Some(event) = services.input.next_action_event() {
      if let Some(cmd) = window_size_ui.handle_event(&UiEvent::Action(event)) {
        apply_window_size_command(cmd, world);
      }
      if world.is_stopped() {
        break;
      }
    }
    for sys_event in services.input.drain_system_events() {
      match sys_event {
        SystemEvent::Mouse(mouse) => {
          services.hit_area.route_mouse_event(
            window_size_ui.objects_mut(),
            &mut services.text_input,
            &services.canvas,
            mouse,
          );
        }
        SystemEvent::Focus(focus) if !focus.gained => {
          services.hit_area.focus_lost(window_size_ui.objects_mut());
        }
        _ => {}
      }
      while let Some(event) = window_size_ui.objects_mut().pop_event() {
        if let Some(command) = window_size_ui.handle_event(&event) {
          apply_window_size_command(command, world);
        }
      }
      if world.is_stopped() {
        break;
      }
    }
    return;
  }

  while let Some(event) = services.input.next_action_event() {
    route_input_event(
      &UiEvent::Action(event),
      services,
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      language_loading_ui,
      language_loading,
    );
    route_component_events(
      services,
      world,
      home_ui,
      settings_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      language_loading_ui,
      language_loading,
    );

    if world.is_stopped() {
      break;
    }
  }

  let terminal_positions = (world.state.current_ui_kind() == Some(UiNodeKind::TerminalCheck))
    .then(|| terminal_check_ui.compute_positions(&services.layout, &services.i18n));
  for event in services.input.drain_system_events() {
    match event {
      SystemEvent::Mouse(mouse) => {
        let consumed = route_mouse_and_events(
          services,
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
          language_loading_ui,
          language_loading,
          mouse,
        );
        if !consumed {
          if let Some(positions) = terminal_positions.as_ref() {
            route_terminal_check_mouse_event(&mouse, positions, services, world, terminal_check_ui);
          }
        }
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        if let Some(pool) = current_objects_mut(
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
        ) {
          services.hit_area.focus_lost(pool);
        }
        route_component_events(
          services,
          world,
          home_ui,
          settings_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
          language_loading_ui,
          language_loading,
        );
      }
      _ => {}
    }
    if world.is_stopped() {
      break;
    }
  }
}

fn route_input_event(
  event: &UiEvent,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.handle_event(event) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      if let Some(command) = settings_ui.handle_event(event) {
        apply_settings_command(command, settings_ui, services, world);
      }
    }
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(ui) = language_select_ui.as_deref_mut() {
        if let Some(command) = ui.handle_event(event) {
          apply_language_select_command(
            command,
            ui,
            services,
            world,
            language_loading_ui,
            language_loading,
          );
        }
      }
    }
    Some(UiNodeKind::Mods) => {
      if let Some(command) = mods_ui.handle_event(event) {
        apply_mods_command(command, mods_ui, services, world);
      }
    }
    Some(UiNodeKind::GamePackage) => {
      if let Some(command) = game_package_ui.handle_event(event) {
        apply_game_package_command(command, game_package_ui, services, world);
      }
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      if let Some(command) = screensaver_package_ui.handle_event(event) {
        apply_screensaver_package_command(command, screensaver_package_ui, services, world);
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
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if world.state.current_overlay_kind().is_some() {
    if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
      language_loading_ui.update(world.clock.delta_time());
    }
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
    Some(UiNodeKind::GamePackage) => {
      let dt = world.clock.delta_time();
      let _ = game_package_ui.update(dt);
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      let dt = world.clock.delta_time();
      let _ = screensaver_package_ui.update(dt);
    }
    Some(UiNodeKind::TerminalCheck) => {
      let dt = world.clock.delta_time();
      if let Some(command) = terminal_check_ui.update(dt) {
        apply_terminal_check_command(command, terminal_check_ui, services, world);
      }
    }
    Some(UiNodeKind::InputDemo) => {
      input_demo_ui.update();
    }
    _ => {}
  }
  route_component_events(
    services,
    world,
    home_ui,
    settings_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
    language_loading_ui,
    language_loading,
  );
}

fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
) -> Option<(u16, u16)> {
  if let Some(OverlayKind::WindowSizeWarning) = world.state.current_overlay_kind() {
    apply_host_viewport(services);
    let runtime = world.state.runtime().unwrap();
    let overlay = runtime.overlays().top().unwrap();
    let req_w = overlay.render.required_width;
    let req_h = overlay.render.required_height;
    let term = services.layout.physical_size();

    window_size_ui.objects_mut().begin_render();
    services
      .canvas
      .prepare(window_size_ui.objects(), &services.layout);
    window_size_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
      req_w,
      req_h,
      term.width,
      term.height,
      world.state.is_host_mode(),
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
    apply_host_viewport(services);
    language_loading_ui.objects_mut().begin_render();
    services
      .canvas
      .prepare(language_loading_ui.objects(), &services.layout);
    language_loading_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.progress_bar,
    );
    return None;
  }

  apply_host_viewport(services);

  if let Some(objects) = current_objects_mut(
    world,
    home_ui,
    settings_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
  ) {
    objects.begin_render();
    services.canvas.prepare(objects, &services.layout);
  }

  let input_cursor = match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      home_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
      None
    }
    Some(UiNodeKind::Settings) => {
      settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
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
          &services.hit_area,
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
        &services.hit_area,
      );
      None
    }
    Some(UiNodeKind::GamePackage) => {
      game_package_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        &services.package,
        &mut services.image,
      );
      None
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      screensaver_package_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        &services.package,
        &mut services.image,
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
    Some(UiNodeKind::InputDemo) => {
      input_demo_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.hit_area,
        &services.scroll_box,
      );
      None
    }
    _ => None,
  };
  input_cursor
}

fn apply_host_viewport(services: &mut EngineServices) {
  refresh_host_areas(&mut services.host_objects, services.layout.physical_size());
  apply_developer_viewport(&mut services.layout, &services.host_objects);
}

fn apply_developer_viewport(
  layout: &mut crate::host_engine::services::LayoutService,
  host_objects: &crate::host_engine::services::HostObjectPool,
) {
  if let Some(rect) = host_objects.area_rect(HostAreaKind::DeveloperViewport) {
    layout.set_developer_viewport(rect);
  }
}

fn refresh_host_areas(
  host_objects: &mut crate::host_engine::services::HostObjectPool,
  physical: crate::host_engine::services::Size,
) {
  let top = host_objects.ensure_area(HostAreaKind::TopBar);
  let separator = host_objects.ensure_area(HostAreaKind::Separator);
  let viewport = host_objects.ensure_area(HostAreaKind::DeveloperViewport);
  host_objects.update_area(top, Rect::default(), false);
  host_objects.update_area(separator, Rect::default(), false);
  host_objects.update_area(
    viewport,
    Rect {
      x: 0,
      y: 0,
      width: physical.width,
      height: physical.height,
    },
    true,
  );
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
    InputDemoCommand::Back => {
      input_demo_ui.leave();
      world.state.pop_ui_node();
      reset_input_demo_ui(input_demo_ui, services);
    }
  }
}

fn apply_settings_command(
  command: SettingsUiCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    SettingsUiCommand::Back => {
      world.state.pop_ui_node();
      reset_settings_ui(settings_ui, services);
    }
    SettingsUiCommand::OpenLanguageSelect => {
      world.state.enter_ui_node(UiNodeState::language_select());
    }
    SettingsUiCommand::OpenMods => {
      world.state.enter_ui_node(UiNodeState::mods());
    }
  }
}

fn apply_mods_command(
  command: ModsCommand,
  mods_ui: &mut ModsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ModsCommand::Back => {
      world.state.pop_ui_node();
      reset_mods_ui(mods_ui, services);
    }
    ModsCommand::OpenGame => {
      world.state.enter_ui_node(UiNodeState::game_package());
    }
    ModsCommand::OpenScreensaver => {
      world
        .state
        .enter_ui_node(UiNodeState::screensaver_package());
    }
  }
}

fn apply_game_package_command(
  command: GamePackageCommand,
  game_package_ui: &mut GamePackageUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    GamePackageCommand::Back => {
      world.state.pop_ui_node();
      reset_game_package_ui(game_package_ui, services);
    }
    GamePackageCommand::FocusSearch => {
      game_package_ui.focus_search(&mut services.text_input);
    }
    GamePackageCommand::BlurSearch => {
      game_package_ui.blur_search(&mut services.text_input);
    }
    GamePackageCommand::FocusJump => {
      game_package_ui.focus_jump(&mut services.text_input);
    }
    GamePackageCommand::BlurJump => {
      game_package_ui.blur_jump(&mut services.text_input);
    }
    GamePackageCommand::SubmitJump(value) => {
      game_package_ui.submit_jump(&mut services.text_input, value);
    }
  }
}

fn apply_screensaver_package_command(
  command: ScreensaverPackageCommand,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ScreensaverPackageCommand::Back => {
      world.state.pop_ui_node();
      reset_screensaver_package_ui(screensaver_package_ui, services);
    }
    ScreensaverPackageCommand::FocusSearch => {
      screensaver_package_ui.focus_search(&mut services.text_input);
    }
    ScreensaverPackageCommand::BlurSearch => {
      screensaver_package_ui.blur_search(&mut services.text_input);
    }
    ScreensaverPackageCommand::FocusJump => {
      screensaver_package_ui.focus_jump(&mut services.text_input);
    }
    ScreensaverPackageCommand::BlurJump => {
      screensaver_package_ui.blur_jump(&mut services.text_input);
    }
    ScreensaverPackageCommand::SubmitJump(value) => {
      screensaver_package_ui.submit_jump(&mut services.text_input, value);
    }
  }
}

fn apply_language_select_command(
  command: LanguageSelectCommand,
  language_select_ui: &mut LanguageSelectUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  match command {
    LanguageSelectCommand::Confirm(code) => {
      language_loading.pending_language = Some(code);
    }
    LanguageSelectCommand::Back => {
      let pending_language = language_loading.pending_language.take();
      let enter_terminal_check_after_finish = !services.storage.is_terminal_profile_complete();
      world.state.pop_ui_node();
      if let Some(code) = pending_language {
        start_language_loading(
          &code,
          enter_terminal_check_after_finish,
          language_loading,
          language_loading_ui,
          services,
          world,
        );
        reset_language_select_ui(language_select_ui, services);
      } else if enter_terminal_check_after_finish {
        reset_language_select_ui(language_select_ui, services);
        world.state.enter_ui_node(UiNodeState::terminal_check());
      } else {
        reset_language_select_ui(language_select_ui, services);
      }
    }
  }
}

fn start_language_loading(
  code: &str,
  enter_terminal_check_after_finish: bool,
  language_loading: &mut LanguageLoadingRuntime,
  language_loading_ui: &mut LanguageLoadingUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  language_loading.active = true;
  language_loading.enter_terminal_check_after_finish = enter_terminal_check_after_finish;
  language_loading_ui.set_progress(&services.progress_bar, 0.0, 0.0);
  world.state.push_language_loading_overlay();
  let _ = services.storage.write_language_code(code);
  services
    .i18n
    .load_runtime_language(&services.storage, &mut services.log, code);
  language_loading_ui.set_progress(&services.progress_bar, 0.5, 0.5);
  let package_language = services.i18n.current_language().to_string();
  let missing_template = services
    .i18n
    .get_runtime_text("language_warning", "language_warning.missing");
  let requested = services
    .package
    .request_rescan_for_language(&package_language, &missing_template);
  if !requested {
    finish_language_loading(language_loading, language_loading_ui, services, world);
  }
}

fn apply_language_loading_package_events(
  events: &[PackageEvent],
  language_loading: &mut LanguageLoadingRuntime,
  language_loading_ui: &mut LanguageLoadingUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  if !language_loading.active {
    return;
  }

  for event in events {
    match *event {
      PackageEvent::ScanStarted { total } if total == 0 => {
        language_loading_ui.set_progress(&services.progress_bar, 0.5, 1.0);
      }
      PackageEvent::ScanStarted { .. } => {
        language_loading_ui.set_progress(&services.progress_bar, 0.5, 0.5);
      }
      PackageEvent::ScanProgress { scanned, total } => {
        let package_progress = if total == 0 {
          1.0
        } else {
          (scanned as f32 / total as f32).clamp(0.0, 1.0)
        };
        language_loading_ui.set_progress(&services.progress_bar, 0.5, 0.5 + package_progress * 0.5);
      }
      PackageEvent::ScanFinished { .. } => {
        finish_language_loading(language_loading, language_loading_ui, services, world);
      }
      _ => {}
    }
  }
}

fn finish_language_loading(
  language_loading: &mut LanguageLoadingRuntime,
  language_loading_ui: &mut LanguageLoadingUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  if !language_loading.active {
    return;
  }
  language_loading.active = false;
  let enter_terminal_check = language_loading.enter_terminal_check_after_finish;
  language_loading.enter_terminal_check_after_finish = false;
  language_loading_ui.set_progress(&services.progress_bar, 1.0, 1.0);
  let _ = world
    .state
    .remove_overlay_kind(OverlayKind::LanguageLoading);
  if enter_terminal_check {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }
}

fn clear_exiting_pool(pool: &mut UiObjectPool, services: &mut EngineServices) {
  let _ = services.text_input.blur(pool);
  services.text_input.deactivate_pool(pool);
  services.hit_area.deactivate(pool);
}

fn reset_settings_ui(ui: &mut SettingsUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = SettingsUi::init(&services.hit_area);
}

fn reset_mods_ui(ui: &mut ModsUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = ModsUi::init(&services.hit_area);
}

fn reset_game_package_ui(ui: &mut GamePackageUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = GamePackageUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
}

fn reset_screensaver_package_ui(ui: &mut ScreensaverPackageUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = ScreensaverPackageUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
}

fn reset_language_select_ui(ui: &mut LanguageSelectUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = LanguageSelectUi::init(
    services.i18n.language_registry().to_vec(),
    &services.storage,
    &mut services.log,
    &services.hit_area,
  );
}

fn reset_input_demo_ui(ui: &mut InputDemoUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = InputDemoUi::init(&services.hit_area, &services.slice, &services.scroll_box);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{HostObjectPool, LayoutService, Size};

  #[test]
  fn host_viewport_uses_full_terminal_by_default() {
    let mut layout = LayoutService::new();
    let mut host_objects = HostObjectPool::new();
    layout.resize_physical(120, 40);

    refresh_host_areas(&mut host_objects, layout.physical_size());
    apply_developer_viewport(&mut layout, &host_objects);

    assert_eq!(host_objects.area_rect(HostAreaKind::TopBar), None);
    assert_eq!(host_objects.area_rect(HostAreaKind::Separator), None);
    assert!(host_objects.is_visible(HostAreaKind::DeveloperViewport));
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 40
      }
    );
    assert_eq!(
      layout.developer_size(),
      Size {
        width: 120,
        height: 40
      }
    );
  }

  #[test]
  fn repeated_host_viewport_refresh_keeps_full_terminal_base() {
    let mut layout = LayoutService::new();
    let mut host_objects = HostObjectPool::new();
    layout.resize_physical(120, 40);
    refresh_host_areas(&mut host_objects, layout.physical_size());
    apply_developer_viewport(&mut layout, &host_objects);

    refresh_host_areas(&mut host_objects, layout.physical_size());
    apply_developer_viewport(&mut layout, &host_objects);

    assert!(!host_objects.is_visible(HostAreaKind::TopBar));
    assert_eq!(host_objects.area_rect(HostAreaKind::Separator), None);
    assert_eq!(
      host_objects.area_width(HostAreaKind::DeveloperViewport),
      Some(120)
    );
    assert_eq!(
      layout.developer_viewport_rect(),
      Rect {
        x: 0,
        y: 0,
        width: 120,
        height: 40
      }
    );
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

fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  let term = services.layout.physical_size();

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
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
      let (min_w, min_h) = get_min_window_size(world);
      if (term.width as u32) < min_w || (term.height as u32) < min_h {
        world.state.push_window_size_overlay(min_w, min_h);
      }
    }
    _ => {}
  }
}

fn get_min_window_size(world: &RuntimeWorld) -> (u32, u32) {
  if world.state.is_host_mode() {
    (95, 24)
  } else {
    (95, 24)
  }
}

fn apply_window_size_command(cmd: WindowSizeWarningCommand, world: &mut RuntimeWorld) {
  match cmd {
    WindowSizeWarningCommand::Exit => {
      if world.state.is_host_mode() {
        world.state.pop_overlay();
        world.state.enter_shutdown();
        set_crash_phase(world.state.crash_phase());
        world.state.enter_stopped();
        set_crash_phase(world.state.crash_phase());
      } else {
        world.state.pop_overlay();
        if let Some(runtime) = world.state.runtime_mut() {
          runtime.set_main_host(MainHostState::Host(HostState::new()));
        }
      }
    }
  }
}
