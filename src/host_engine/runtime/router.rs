use super::*;
use crate::host_engine::services::{
  KeyState, MouseEvent, SystemEvent, TerminalKeyCode, UiEvent, UiObjectPool, UiObjectPoolOwner,
};

pub(super) fn current_objects_mut<'a>(
  world: &RuntimeWorld,
  home_ui: &'a mut HomeUi,
  settings_ui: &'a mut SettingsUi,
  display_settings_ui: &'a mut DisplaySettingsUi,
  screensaver_list_ui: &'a mut ScreensaverListUi,
  security_uis: &'a mut SecurityUis,
  storage_management_ui: &'a mut StorageManagementUi,
  storage_management_clear_ui: &'a mut StorageManagementClearUi,
  storage_management_export_ui: &'a mut StorageManagementExportUi,
  storage_management_view_ui: &'a mut StorageManagementViewUi,
  language_select_ui: Option<&'a mut LanguageSelectUi>,
  terminal_check_ui: &'a mut TerminalCheckUi,
  mods_ui: &'a mut ModsUi,
  game_list_ui: &'a mut GameListUi,
  game_package_ui: &'a mut GamePackageUi,
  screensaver_package_ui: &'a mut ScreensaverPackageUi,
  input_demo_ui: &'a mut InputDemoUi,
) -> Option<&'a mut UiObjectPool> {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => Some(home_ui.objects_mut()),
    Some(UiNodeKind::Settings) => Some(settings_ui.objects_mut()),
    Some(UiNodeKind::DisplaySettings) => Some(display_settings_ui.objects_mut()),
    Some(UiNodeKind::ToolbarCustom) => Some(display_settings_ui.custom_mut().objects_mut()),
    Some(UiNodeKind::ScreensaverList) => Some(screensaver_list_ui.objects_mut()),
    Some(UiNodeKind::ScreenshotRecording) => {
      Some(settings_ui.screenshot_recording_mut().objects_mut())
    }
    Some(UiNodeKind::ScreenshotSettings) => Some(
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .objects_mut(),
    ),
    Some(UiNodeKind::SecuritySettings) => Some(security_uis.settings.objects_mut()),
    Some(UiNodeKind::SecurityDetails) => Some(security_uis.details.objects_mut()),
    Some(UiNodeKind::StorageManagement) => Some(storage_management_ui.objects_mut()),
    Some(UiNodeKind::StorageManagementClear) => Some(storage_management_clear_ui.objects_mut()),
    Some(UiNodeKind::StorageManagementExport) => Some(storage_management_export_ui.objects_mut()),
    Some(UiNodeKind::StorageManagementView) => Some(storage_management_view_ui.objects_mut()),
    Some(UiNodeKind::LanguageSelect) => language_select_ui.map(UiObjectPoolOwner::objects_mut),
    Some(UiNodeKind::TerminalCheck) => Some(terminal_check_ui.objects_mut()),
    Some(UiNodeKind::Mods) => Some(mods_ui.objects_mut()),
    Some(UiNodeKind::GameList) => Some(game_list_ui.objects_mut()),
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  _screenshot_capture_ui: &mut ScreenshotCaptureUi,
  export_loading_ui: &mut ExportLoadingUi,
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
  deactivate(
    UiNodeKind::DisplaySettings,
    display_settings_ui.objects_mut(),
  );
  deactivate(
    UiNodeKind::ToolbarCustom,
    display_settings_ui.custom_mut().objects_mut(),
  );
  deactivate(
    UiNodeKind::ScreensaverList,
    screensaver_list_ui.objects_mut(),
  );
  deactivate(
    UiNodeKind::ScreenshotRecording,
    settings_ui.screenshot_recording_mut().objects_mut(),
  );
  deactivate(
    UiNodeKind::ScreenshotSettings,
    settings_ui
      .screenshot_recording_mut()
      .screenshot_settings_mut()
      .objects_mut(),
  );
  deactivate(
    UiNodeKind::SecuritySettings,
    security_uis.settings.objects_mut(),
  );
  deactivate(
    UiNodeKind::SecurityDetails,
    security_uis.details.objects_mut(),
  );
  deactivate(
    UiNodeKind::StorageManagement,
    storage_management_ui.objects_mut(),
  );
  deactivate(
    UiNodeKind::StorageManagementClear,
    storage_management_clear_ui.objects_mut(),
  );
  deactivate(
    UiNodeKind::StorageManagementExport,
    storage_management_export_ui.objects_mut(),
  );
  deactivate(
    UiNodeKind::StorageManagementView,
    storage_management_view_ui.objects_mut(),
  );
  if let Some(ui) = language_select_ui {
    deactivate(UiNodeKind::LanguageSelect, ui.objects_mut());
  }
  deactivate(UiNodeKind::TerminalCheck, terminal_check_ui.objects_mut());
  deactivate(UiNodeKind::Mods, mods_ui.objects_mut());
  deactivate(UiNodeKind::GameList, game_list_ui.objects_mut());
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
  if world.state.current_overlay_kind() != Some(OverlayKind::ClearWarning) {
    services
      .text_input
      .deactivate_pool(clear_warning_ui.objects_mut());
    services.hit_area.deactivate(clear_warning_ui.objects_mut());
  }
  if world.state.current_overlay_kind() != Some(OverlayKind::ExportSettings) {
    services
      .text_input
      .deactivate_pool(export_settings_ui.objects_mut());
    services
      .hit_area
      .deactivate(export_settings_ui.objects_mut());
  }
  if world.state.current_overlay_kind() != Some(OverlayKind::ExportLoading) {
    services
      .text_input
      .deactivate_pool(export_loading_ui.objects_mut());
    services
      .hit_area
      .deactivate(export_loading_ui.objects_mut());
  }
}

pub(super) fn route_text_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
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
      display_settings_ui,
      screensaver_list_ui,
      security_uis,
      storage_management_ui,
      storage_management_clear_ui,
      storage_management_export_ui,
      storage_management_view_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_list_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      clear_warning_ui,
      export_settings_ui,
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  _screenshot_capture_ui: &mut ScreenshotCaptureUi,
  export_loading_ui: &mut ExportLoadingUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  export_loading: &mut ExportLoadingRuntime,
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
          security_uis,
          game_package_ui,
          safe_mode_warning_ui,
        );
      }
      Some(OverlayKind::ClearWarning) => {
        route_clear_warning_overlay_events(services, world, clear_warning_ui);
      }
      Some(OverlayKind::ExportSettings) => {
        route_export_settings_overlay_events(
          services,
          world,
          export_settings_ui,
          export_loading_ui,
          export_loading,
        );
      }
      _ => {}
    }
    return;
  }

  while let Some(event) = services.input.next_action_event() {
    if handle_host_key_action(event.action.as_str(), event.state, world) {
      if world.is_stopped() {
        break;
      }
      continue;
    }
    route_input_event(
      &UiEvent::Action(event),
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
      screensaver_list_ui,
      security_uis,
      storage_management_ui,
      storage_management_clear_ui,
      storage_management_export_ui,
      storage_management_view_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_list_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      clear_warning_ui,
      export_settings_ui,
      language_loading_ui,
      language_loading,
    );
    route_component_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
      screensaver_list_ui,
      security_uis,
      storage_management_ui,
      storage_management_clear_ui,
      storage_management_export_ui,
      storage_management_view_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_list_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      clear_warning_ui,
      export_settings_ui,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
          clear_warning_ui,
          export_settings_ui,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
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
          display_settings_ui,
          screensaver_list_ui,
          security_uis,
          storage_management_ui,
          storage_management_clear_ui,
          storage_management_export_ui,
          storage_management_view_ui,
          language_select_ui.as_deref_mut(),
          terminal_check_ui,
          mods_ui,
          game_list_ui,
          game_package_ui,
          screensaver_package_ui,
          input_demo_ui,
          clear_warning_ui,
          export_settings_ui,
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  screenshot_capture_ui: &mut ScreenshotCaptureUi,
  export_loading_ui: &mut ExportLoadingUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  _export_loading: &mut ExportLoadingRuntime,
) {
  if world.state.current_overlay_kind().is_some() {
    if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
      language_loading_ui.update(&services.time, world.clock.delta_time());
    } else if world.state.current_overlay_kind() == Some(OverlayKind::ExportLoading) {
      export_loading_ui.update(&services.time, world.clock.delta_time());
    } else if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
      safe_mode_warning_ui.update(world.clock.delta_time());
    } else if world.state.current_overlay_kind() == Some(OverlayKind::ClearWarning) {
      clear_warning_ui.update(world.clock.delta_time());
    } else if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
      screenshot_capture_ui.update(world.clock.delta_time());
    }
    if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
      return;
    }
  }

  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.update(
        world.clock.delta_time(),
        &services.animation,
        &services.random,
      ) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      let _ = settings_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::DisplaySettings) => {
      let _ = display_settings_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::ToolbarCustom) => {}
    Some(UiNodeKind::ScreensaverList) => {
      let _ = screensaver_list_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::ScreenshotRecording) => {
      settings_ui
        .screenshot_recording_mut()
        .update(world.clock.delta_time());
    }
    Some(UiNodeKind::ScreenshotSettings) => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .update(world.clock.delta_time());
    }
    Some(UiNodeKind::SecuritySettings) => {
      security_uis.settings.update(world.clock.delta_time());
    }
    Some(UiNodeKind::SecurityDetails) => {}
    Some(UiNodeKind::StorageManagement) => {
      let _ = storage_management_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::StorageManagementClear) => {
      let _ = storage_management_clear_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::StorageManagementExport) => {
      let _ = storage_management_export_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::StorageManagementView) => {
      storage_management_view_ui.update(world.clock.delta_time(), &services.layout, &services.i18n);
    }
    Some(UiNodeKind::LanguageSelect) => {
      let _ = language_select_ui
        .as_mut()
        .and_then(|ui| ui.update(world.clock.delta_time()));
    }
    Some(UiNodeKind::Mods) => {
      let _ = mods_ui.update(world.clock.delta_time());
    }
    Some(UiNodeKind::GameList) => {
      let _ = game_list_ui.update(world.clock.delta_time());
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
    display_settings_ui,
    screensaver_list_ui,
    security_uis,
    storage_management_ui,
    storage_management_clear_ui,
    storage_management_export_ui,
    storage_management_view_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_list_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
    clear_warning_ui,
    export_settings_ui,
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
    if handle_host_key_action(event.action.as_str(), event.state, world) {
      if world.is_stopped() {
        break;
      }
      continue;
    }
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
  security_uis: &mut SecurityUis,
  game_package_ui: &mut GamePackageUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
) {
  while let Some(event) = services.input.next_action_event() {
    let _ = handle_host_key_action(event.action.as_str(), event.state, world);
    if world.is_stopped() {
      return;
    }
  }
  if let Some(command) =
    safe_mode_warning_ui.handle_raw_key_events(&mut services.input, world.safe_mode_warning_all)
  {
    safe_mode_warning_ui.start();
    apply_safe_mode_warning_command(command, security_uis, game_package_ui, services, world);
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
      if let Some(command) = safe_mode_warning_ui.handle_event(&event, world.safe_mode_warning_all)
      {
        safe_mode_warning_ui.start();
        apply_safe_mode_warning_command(command, security_uis, game_package_ui, services, world);
        return;
      }
    }
  }
}

fn route_clear_warning_overlay_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  clear_warning_ui: &mut ClearWarningUi,
) {
  while let Some(event) = services.input.next_action_event() {
    let _ = handle_host_key_action(event.action.as_str(), event.state, world);
    if world.is_stopped() {
      return;
    }
  }
  if let Some(command) = clear_warning_ui.handle_raw_key_events(&mut services.input) {
    apply_clear_warning_command(command, clear_warning_ui, services, world);
    return;
  }
  for sys_event in services.input.drain_system_events() {
    match sys_event {
      SystemEvent::Mouse(mouse) => {
        services.hit_area.route_mouse_event(
          clear_warning_ui.objects_mut(),
          &mut services.text_input,
          &services.canvas,
          mouse,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        services.hit_area.focus_lost(clear_warning_ui.objects_mut());
      }
      _ => {}
    }
    while let Some(event) = clear_warning_ui.objects_mut().pop_event() {
      if let Some(command) = clear_warning_ui.handle_event(&event) {
        apply_clear_warning_command(command, clear_warning_ui, services, world);
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  event: MouseEvent,
) -> bool {
  let Some(pool) = current_objects_mut(
    world,
    home_ui,
    settings_ui,
    display_settings_ui,
    screensaver_list_ui,
    security_uis,
    storage_management_ui,
    storage_management_clear_ui,
    storage_management_export_ui,
    storage_management_view_ui,
    language_select_ui,
    terminal_check_ui,
    mods_ui,
    game_list_ui,
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
  if services
    .markdown
    .route_mouse_event(pool, &services.text_input, event)
  {
    return true;
  }
  if services
    .hyperlink
    .route_mouse_event(pool, &services.text_input, event)
  {
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  event: MouseEvent,
) -> bool {
  let consumed = route_component_mouse(
    services,
    world,
    home_ui,
    settings_ui,
    display_settings_ui,
    screensaver_list_ui,
    security_uis,
    storage_management_ui,
    storage_management_clear_ui,
    storage_management_export_ui,
    storage_management_view_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_list_ui,
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
    display_settings_ui,
    screensaver_list_ui,
    security_uis,
    storage_management_ui,
    storage_management_clear_ui,
    storage_management_export_ui,
    storage_management_view_ui,
    language_select_ui,
    terminal_check_ui,
    mods_ui,
    game_list_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
    clear_warning_ui,
    export_settings_ui,
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  loop {
    let event = current_objects_mut(
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
      screensaver_list_ui,
      security_uis,
      storage_management_ui,
      storage_management_clear_ui,
      storage_management_export_ui,
      storage_management_view_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_list_ui,
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
      display_settings_ui,
      screensaver_list_ui,
      security_uis,
      storage_management_ui,
      storage_management_clear_ui,
      storage_management_export_ui,
      storage_management_view_ui,
      language_select_ui.as_deref_mut(),
      terminal_check_ui,
      mods_ui,
      game_list_ui,
      game_package_ui,
      screensaver_package_ui,
      input_demo_ui,
      clear_warning_ui,
      export_settings_ui,
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
  display_settings_ui: &mut DisplaySettingsUi,
  screensaver_list_ui: &mut ScreensaverListUi,
  security_uis: &mut SecurityUis,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_list_ui: &mut GameListUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if let UiEvent::Action(action) = event {
    if handle_host_key_action(action.action.as_str(), action.state, world) {
      return;
    }
  }

  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      if let Some(command) = home_ui.handle_event(event) {
        apply_home_command(command, world);
      }
    }
    Some(UiNodeKind::Settings) => {
      if let Some(command) = settings_ui.handle_event(event) {
        apply_settings_command(command, settings_ui, security_uis, services, world);
      }
    }
    Some(UiNodeKind::DisplaySettings) => {
      if let Some(command) = display_settings_ui.handle_event(event) {
        apply_display_settings_command(command, display_settings_ui, services, world);
      }
    }
    Some(UiNodeKind::ToolbarCustom) => {
      if let Some(command) = display_settings_ui.custom_mut().handle_event(event) {
        apply_toolbar_custom_command(command, display_settings_ui, services, world);
      }
    }
    Some(UiNodeKind::ScreensaverList) => {
      if let Some(command) = screensaver_list_ui.handle_event(event) {
        apply_screensaver_list_command(command, screensaver_list_ui, services, world);
      }
    }
    Some(UiNodeKind::ScreenshotRecording) => {
      if let Some(command) = settings_ui.screenshot_recording_mut().handle_event(event) {
        apply_screenshot_recording_command(command, settings_ui, services, world);
      }
    }
    Some(UiNodeKind::ScreenshotSettings) => {
      if let Some(command) = settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .handle_event(event)
      {
        apply_screenshot_settings_command(command, settings_ui, services, world);
      }
    }
    Some(UiNodeKind::SecuritySettings) => {
      if let Some(command) = security_uis.settings.handle_event(event) {
        apply_security_settings_command(command, security_uis, services, world);
      }
    }
    Some(UiNodeKind::SecurityDetails) => {
      if let Some(command) = security_uis.details.handle_event(event) {
        apply_security_details_command(command, security_uis, services, world);
      }
    }
    Some(UiNodeKind::StorageManagement) => {
      if let Some(command) = storage_management_ui.handle_event(event) {
        apply_storage_management_command(command, storage_management_ui, services, world);
      }
    }
    Some(UiNodeKind::StorageManagementClear) => {
      if let Some(command) = storage_management_clear_ui.handle_event(event) {
        apply_storage_management_clear_command(
          command,
          storage_management_clear_ui,
          clear_warning_ui,
          services,
          world,
        );
      }
    }
    Some(UiNodeKind::StorageManagementExport) => {
      if let Some(command) = storage_management_export_ui.handle_event(event) {
        apply_storage_management_export_command(
          command,
          storage_management_export_ui,
          export_settings_ui,
          services,
          world,
        );
      }
    }
    Some(UiNodeKind::StorageManagementView) => {
      if let Some(command) = storage_management_view_ui.handle_event(event, &services.i18n) {
        apply_storage_management_view_command(command, storage_management_view_ui, services, world);
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
    Some(UiNodeKind::GameList) => {
      if let Some(command) = game_list_ui.handle_event(event) {
        apply_game_list_command(command, game_list_ui, services, world);
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

pub(super) fn handle_host_key_action(
  action: &str,
  state: KeyState,
  world: &mut RuntimeWorld,
) -> bool {
  match action {
    HOST_KEY_SCREENSHOT
    | HOST_KEY_RECORDING
    | HOST_KEY_SCREENSAVER
    | HOST_KEY_TOP_TOOLBAR
    | HOST_KEY_RECORDING_PAUSE
    | HOST_KEY_TOP_TOOLBAR_SWITCH => true,
    HOST_KEY_FORCE_STOP => {
      if state == KeyState::Pressed {
        world.state.enter_shutdown();
        set_crash_phase(world.state.crash_phase());
        world.state.enter_stopped();
        set_crash_phase(world.state.crash_phase());
      }
      true
    }
    _ => false,
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

pub(super) fn route_export_settings_overlay_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  export_settings_ui: &mut ExportSettingsUi,
  export_loading_ui: &mut ExportLoadingUi,
  export_loading: &mut ExportLoadingRuntime,
) {
  let was_active = services.text_input.is_active();
  while let Some(event) = services.input.next_action_event() {
    if handle_host_key_action(event.action.as_str(), event.state, world) {
      if world.is_stopped() {
        break;
      }
      continue;
    }
    if let Some(command) = export_settings_ui.handle_event(&UiEvent::Action(event)) {
      apply_export_settings_command(
        command,
        export_settings_ui,
        export_loading_ui,
        export_loading,
        services,
        world,
      );
    }
    if world.is_stopped() {
      break;
    }
  }
  // 若 action 刚激活了 text_input，跳过 Enter 的 TerminalKey 避免瞬间 Submit
  let just_activated = !was_active && services.text_input.is_active();
  for sys_event in services.input.drain_system_events() {
    match sys_event {
      SystemEvent::Mouse(mouse) => {
        services.hit_area.route_mouse_event(
          export_settings_ui.objects_mut(),
          &mut services.text_input,
          &services.canvas,
          mouse,
        );
      }
      SystemEvent::TerminalKey(key) => {
        if just_activated && key.code == TerminalKeyCode::Enter {
          continue; // 跳过触发 FocusInput 的 Enter，避免立刻 Submit
        }
        services.text_input.route_terminal_key(
          export_settings_ui.objects_mut(),
          &mut services.clipboard,
          key,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        services
          .hit_area
          .focus_lost(export_settings_ui.objects_mut());
      }
      _ => {}
    }
    while let Some(event) = export_settings_ui.objects_mut().pop_event() {
      if let Some(command) = export_settings_ui.handle_event(&event) {
        apply_export_settings_command(
          command,
          export_settings_ui,
          export_loading_ui,
          export_loading,
          services,
          world,
        );
        return;
      }
    }
    if world.is_stopped() {
      break;
    }
  }
}

/// ExportSettings overlay 输入中路由——只走 system events，不 dispatch action，
/// 避免 Enter 被 action map 拦截而打断 IME 组字。
pub(super) fn route_export_settings_text_input_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  export_settings_ui: &mut ExportSettingsUi,
  export_loading_ui: &mut ExportLoadingUi,
  export_loading: &mut ExportLoadingRuntime,
) {
  for sys_event in services.input.drain_system_events() {
    match sys_event {
      SystemEvent::Mouse(mouse) => {
        services.hit_area.route_mouse_event(
          export_settings_ui.objects_mut(),
          &mut services.text_input,
          &services.canvas,
          mouse,
        );
      }
      SystemEvent::TerminalKey(key) => {
        services.text_input.route_terminal_key(
          export_settings_ui.objects_mut(),
          &mut services.clipboard,
          key,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        services
          .hit_area
          .focus_lost(export_settings_ui.objects_mut());
      }
      _ => {}
    }
    while let Some(event) = export_settings_ui.objects_mut().pop_event() {
      if let Some(command) = export_settings_ui.handle_event(&event) {
        apply_export_settings_command(
          command,
          export_settings_ui,
          export_loading_ui,
          export_loading,
          services,
          world,
        );
        return;
      }
    }
    if world.is_stopped() {
      break;
    }
  }
}
