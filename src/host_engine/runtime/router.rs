use super::*;
use crate::host_engine::services::{
  MouseEvent, SystemEvent, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub(super) fn current_objects_mut<'a>(
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

pub(super) fn deactivate_hidden_pools(
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
  safe_mode_warning_ui: &mut SafeModeWarningUi,
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
  if world.state.current_overlay_kind() != Some(OverlayKind::SafeModeWarning) {
    services
      .text_input
      .deactivate_pool(safe_mode_warning_ui.objects_mut());
    services
      .hit_area
      .deactivate(safe_mode_warning_ui.objects_mut());
  }
}

pub(super) fn route_text_input_events(
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

pub(super) fn route_input_events(
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
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if world.state.current_overlay_kind().is_some() {
    match world.state.current_overlay_kind() {
      Some(OverlayKind::WindowSizeWarning) => {
        route_window_size_overlay_events(services, world, window_size_ui);
      }
      Some(OverlayKind::SafeModeWarning) => {
        route_safe_mode_warning_overlay_events(
          services,
          world,
          game_package_ui,
          safe_mode_warning_ui,
        );
      }
      _ => {}
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

pub(super) fn route_update(
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
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if world.state.current_overlay_kind().is_some() {
    if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
      language_loading_ui.update(&services.time, world.clock.delta_time());
    } else if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
      safe_mode_warning_ui.update(world.clock.delta_time());
    }
    return;
  }

  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.update(world.clock.delta_time()) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      let _ = settings_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::LanguageSelect) => {
      let _ = language_select_ui
        .as_mut()
        .and_then(|ui| ui.update(world.clock.delta_time()));
    }
    Some(UiNodeKind::Mods) => {
      let _ = mods_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::GamePackage) => {
      let _ = game_package_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      let _ = screensaver_package_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::TerminalCheck) => {
      if let Some(command) = terminal_check_ui.update(world.clock.delta_time()) {
        apply_terminal_check_command(command, terminal_check_ui, services, world);
      }
    }
    Some(UiNodeKind::InputDemo) => input_demo_ui.update(),
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

fn route_window_size_overlay_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  window_size_ui: &mut WindowSizeWarningUi,
) {
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
}

fn route_safe_mode_warning_overlay_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  game_package_ui: &mut GamePackageUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
) {
  while services.input.next_action_event().is_some() {}
  if let Some(command) = safe_mode_warning_ui.handle_raw_key_events(&mut services.input) {
    safe_mode_warning_ui.start();
    apply_safe_mode_warning_command(command, game_package_ui, services, world);
    return;
  }
  for sys_event in services.input.drain_system_events() {
    match sys_event {
      SystemEvent::Mouse(mouse) => {
        services.hit_area.route_mouse_event(
          safe_mode_warning_ui.objects_mut(),
          &mut services.text_input,
          &services.canvas,
          mouse,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        services
          .hit_area
          .focus_lost(safe_mode_warning_ui.objects_mut());
      }
      _ => {}
    }
    while let Some(event) = safe_mode_warning_ui.objects_mut().pop_event() {
      if let Some(command) = safe_mode_warning_ui.handle_event(&event) {
        safe_mode_warning_ui.start();
        apply_safe_mode_warning_command(command, game_package_ui, services, world);
        return;
      }
    }
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
  if world.state.current_ui_kind() == Some(UiNodeKind::TerminalCheck) {
    if let Some(command) = terminal_check_ui.handle_mouse_event(event, positions) {
      apply_terminal_check_command(command, terminal_check_ui, services, world);
    }
  }
}
