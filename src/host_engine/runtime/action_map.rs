use super::*;

pub(super) const HOST_KEY_SCREENSHOT: &str = "host_key.screenshot";
pub(super) const HOST_KEY_RECORDING: &str = "host_key.recording";
pub(super) const HOST_KEY_SCREENSAVER: &str = "host_key.screensaver";
pub(super) const HOST_KEY_FORCE_STOP: &str = "host_key.force_stop";
pub(super) const HOST_KEY_TOP_TOOLBAR: &str = "host_key.top_toolbar";
pub(super) const HOST_KEY_RECORDING_PAUSE: &str = "host_key.recording.pause";
pub(super) const HOST_KEY_TOP_TOOLBAR_SWITCH: &str = "host_key.top_toolbar.switch";

pub(super) fn load_host_key_action_map(services: &mut EngineServices) {
  let entries = vec![
    ActionMapEntry {
      action: HOST_KEY_SCREENSAVER.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.screensaver"),
      keys: vec![vec!["f3".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_SCREENSHOT.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.screenshot"),
      keys: vec![vec!["f1".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_RECORDING.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.recording"),
      keys: vec![vec!["f2".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_FORCE_STOP.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.force_stop"),
      keys: vec![vec!["f4".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_TOP_TOOLBAR.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.top_toolbar"),
      keys: vec![vec!["f5".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_RECORDING_PAUSE.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.recording.pause"),
      keys: vec![vec!["f3".to_string(), "q".to_string()]],
    },
    ActionMapEntry {
      action: HOST_KEY_TOP_TOOLBAR_SWITCH.to_string(),
      description: services
        .i18n
        .get_runtime_text("host_key", "host_key.top_toolbar.switch"),
      keys: vec![vec!["f5".to_string(), "q".to_string()]],
    },
  ];

  let bindings = translate_action_map(&entries).expect("failed to translate host key action map");
  services.input.load_system_key_bindings(bindings);
}

pub(super) fn load_current_action_map(services: &mut EngineServices, world: &RuntimeWorld) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => load_home_action_map(services),
    Some(UiNodeKind::Settings) => load_settings_action_map(services),
    Some(UiNodeKind::DisplaySettings) => load_display_settings_action_map(services),
    Some(UiNodeKind::ScreensaverList) => load_screensaver_list_action_map(services),
    Some(UiNodeKind::SecuritySettings) => load_security_settings_action_map(services),
    Some(UiNodeKind::SecurityDetails) => load_security_details_action_map(services),
    Some(UiNodeKind::StorageManagement) => load_storage_management_action_map(services),
    Some(UiNodeKind::StorageManagementClear) => load_storage_management_clear_action_map(services),
    Some(UiNodeKind::StorageManagementExport) => {
      load_storage_management_export_action_map(services)
    }
    Some(UiNodeKind::StorageManagementView) => load_storage_management_view_action_map(services),
    Some(UiNodeKind::LanguageSelect) => load_language_select_action_map(services),
    Some(UiNodeKind::Mods) => load_mods_action_map(services),
    Some(UiNodeKind::GameList) => load_game_list_action_map(services),
    Some(UiNodeKind::GamePackage) => load_game_package_action_map(services),
    Some(UiNodeKind::ScreensaverPackage) => load_screensaver_package_action_map(services),
    Some(UiNodeKind::TerminalCheck) => load_terminal_check_action_map(services),
    Some(UiNodeKind::InputDemo) => load_input_demo_action_map(services),
    _ => {}
  }
}

pub(super) fn load_window_size_action_map(services: &mut EngineServices) {
  load_action_map(services, &WindowSizeWarningUi::action_map(), "window_size");
}

pub(super) fn load_safe_mode_warning_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &SafeModeWarningUi::action_map(),
    "safe_mode_warning",
  );
}

fn load_home_action_map(services: &mut EngineServices) {
  load_action_map(services, &HomeUi::action_map(), "HomeUi");
}

fn load_settings_action_map(services: &mut EngineServices) {
  load_action_map(services, &SettingsUi::action_map(), "SettingsUi");
}

fn load_display_settings_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &DisplaySettingsUi::action_map(),
    "DisplaySettingsUi",
  );
}

fn load_screensaver_list_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &ScreensaverListUi::action_map(),
    "ScreensaverListUi",
  );
}

fn load_security_settings_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &SecuritySettingsUi::action_map(),
    "SecuritySettingsUi",
  );
}

fn load_security_details_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &SecurityDetailsUi::action_map(),
    "SecurityDetailsUi",
  );
}

fn load_storage_management_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &StorageManagementUi::action_map(),
    "StorageManagementUi",
  );
}

fn load_storage_management_clear_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &StorageManagementClearUi::action_map(),
    "StorageManagementClearUi",
  );
}

fn load_storage_management_export_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &StorageManagementExportUi::action_map(),
    "StorageManagementExportUi",
  );
}

fn load_storage_management_view_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &StorageManagementViewUi::action_map(),
    "StorageManagementViewUi",
  );
}

pub(super) fn load_export_settings_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &ExportSettingsUi::action_map(),
    "ExportSettingsUi",
  );
}

fn load_language_select_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &LanguageSelectUi::action_map(),
    "LanguageSelectUi",
  );
}

fn load_mods_action_map(services: &mut EngineServices) {
  load_action_map(services, &ModsUi::action_map(), "ModsUi");
}

fn load_game_list_action_map(services: &mut EngineServices) {
  load_action_map(services, &GameListUi::action_map(), "GameListUi");
}

fn load_game_package_action_map(services: &mut EngineServices) {
  load_action_map(services, &GamePackageUi::action_map(), "GamePackageUi");
}

fn load_screensaver_package_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &ScreensaverPackageUi::action_map(),
    "ScreensaverPackageUi",
  );
}

fn load_terminal_check_action_map(services: &mut EngineServices) {
  load_action_map(services, &TerminalCheckUi::action_map(), "TerminalCheckUi");
}

fn load_input_demo_action_map(services: &mut EngineServices) {
  load_action_map(services, &InputDemoUi::action_map(), "InputDemoUi");
}

fn load_action_map(
  services: &mut EngineServices,
  action_map: &[crate::host_engine::services::ActionMapEntry],
  name: &str,
) {
  let bindings = translate_action_map(action_map)
    .unwrap_or_else(|_| panic!("failed to translate {name} action map"));
  services.input.load_key_bindings(bindings);
}
