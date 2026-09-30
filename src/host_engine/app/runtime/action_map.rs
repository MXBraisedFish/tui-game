use super::*;
use std::collections::BTreeMap;

pub(super) const HOST_KEY_SCREENSHOT: &str = "host_key.screenshot";
pub(super) const HOST_KEY_RECORDING: &str = "host_key.recording";
pub(super) const HOST_KEY_SCREENSAVER: &str = "host_key.screensaver";
pub(super) const HOST_KEY_FORCE_STOP: &str = "host_key.force_stop";
pub(super) const HOST_KEY_TOP_TOOLBAR: &str = "host_key.top_toolbar";
pub(super) const HOST_KEY_RECORDING_PAUSE: &str = "host_key.recording.pause";
pub(super) const HOST_KEY_TOP_TOOLBAR_SWITCH: &str = "host_key.top_toolbar.switch";

pub(super) const HOST_KEY_ORDER: &[&str] = &[
  HOST_KEY_SCREENSHOT,
  HOST_KEY_RECORDING,
  HOST_KEY_RECORDING_PAUSE,
  HOST_KEY_SCREENSAVER,
  HOST_KEY_FORCE_STOP,
  HOST_KEY_TOP_TOOLBAR,
  HOST_KEY_TOP_TOOLBAR_SWITCH,
];

fn host_key_defaults() -> ActionKeyMap {
  [
    (HOST_KEY_SCREENSHOT, vec![vec!["f1".to_string()]]),
    (HOST_KEY_RECORDING, vec![vec!["f2".to_string()]]),
    (
      HOST_KEY_RECORDING_PAUSE,
      vec![vec!["f2".to_string(), "q".to_string()]],
    ),
    (HOST_KEY_SCREENSAVER, vec![vec!["f3".to_string()]]),
    (HOST_KEY_FORCE_STOP, vec![vec!["f4".to_string()]]),
    (HOST_KEY_TOP_TOOLBAR, vec![vec!["f5".to_string()]]),
    (
      HOST_KEY_TOP_TOOLBAR_SWITCH,
      vec![vec!["f5".to_string(), "q".to_string()]],
    ),
  ]
  .into_iter()
  .map(|(action, keys)| (action.to_string(), keys))
  .collect()
}

pub(super) fn synchronize_key_bindings_profile(
  services: &mut EngineServices,
) -> KeyBindingsProfile {
  let packages = services.package.games();
  let games = packages
    .iter()
    .filter_map(|package| {
      let game = package.game.as_ref()?;
      let actions = game
        .actions
        .iter()
        .map(|(action, config)| (action.clone(), config.keys.clone()))
        .collect::<BTreeMap<_, _>>();
      Some((package.id.storage_key(), actions))
    })
    .collect();
  let mut profile = services
    .storage
    .read_key_bindings_profile(&mut services.log);
  let mut changed = profile.synchronize(host_key_defaults(), games);
  for package in &packages {
    let Some(game) = &package.game else {
      continue;
    };
    let user = profile
      .user
      .games
      .entry(package.id.storage_key())
      .or_default();
    for (action, config) in &game.actions {
      if config.lock && user.get(action) != Some(&config.keys) {
        user.insert(action.clone(), config.keys.clone());
        changed = true;
      }
    }
  }
  if changed {
    let _ = services
      .storage
      .write_key_bindings_profile(&profile, &mut services.log);
  }
  services
    .package
    .set_user_game_key_actions(profile.user.games.clone());
  profile
}

pub(super) fn host_key_action_entries_from_profile(
  services: &EngineServices,
  profile: &KeyBindingsProfile,
) -> Vec<ActionMapEntry> {
  HOST_KEY_ORDER
    .iter()
    .map(|action| ActionMapEntry {
      action: (*action).to_string(),
      description: services.i18n.get_runtime_text("host_key", action),
      keys: profile
        .user
        .global
        .get(*action)
        .cloned()
        .unwrap_or_default(),
    })
    .collect()
}

pub(super) fn load_host_key_action_map(services: &mut EngineServices) -> KeyBindingsProfile {
  let profile = synchronize_key_bindings_profile(services);
  let mut entries = host_key_action_entries_from_profile(services, &profile);
  // 组合键必须先于它们包含的单键注册；InputService 会按顺序消费已命中的键。
  entries.sort_by_key(|entry| {
    std::cmp::Reverse(entry.keys.iter().map(Vec::len).max().unwrap_or_default())
  });

  match translate_action_map(&entries) {
    Ok(bindings) => services.input.load_system_key_bindings(bindings),
    Err(error) => {
      services.log.error_operation_failed(
        LogSource::Input,
        "translate_action_map",
        "host",
        format!("{error:?}"),
      );
      services.input.load_system_key_bindings(Vec::new());
    }
  }
  profile
}

pub(super) fn host_key_rich_text_params(
  profile: &KeyBindingsProfile,
) -> crate::host_engine::services::RichTextParams {
  let user = profile.user.global.clone().into_iter().collect();
  let defaults = profile.default.global.clone().into_iter().collect();
  crate::host_engine::services::RichTextParams::from_key_action_maps(&user, &defaults)
}

pub(super) fn load_current_action_map(services: &mut EngineServices, world: &RuntimeWorld) {
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => load_home_action_map(services),
    Some(UiNodeKind::Settings) => load_settings_action_map(services),
    Some(UiNodeKind::KeyBindings) => {
      load_action_map(services, &KeyBindingsUi::action_map(), "KeyBindingsUi")
    }
    Some(UiNodeKind::GlobalKeyBindings) => load_action_map(
      services,
      &GlobalKeyBindingsUi::action_map(),
      "GlobalKeyBindingsUi",
    ),
    Some(UiNodeKind::GameKeyBindings) => load_action_map(
      services,
      &GameKeyBindingsUi::action_map(),
      "GameKeyBindingsUi",
    ),
    Some(UiNodeKind::DisplaySettings) => load_display_settings_action_map(services),
    Some(UiNodeKind::ToolbarCustom) => {}
    Some(UiNodeKind::ScreensaverList) => load_screensaver_list_action_map(services),
    Some(UiNodeKind::ScreenshotRecording) => load_action_map(
      services,
      &ScreenshotRecordingUi::action_map(),
      "ScreenshotRecordingUi",
    ),
    Some(UiNodeKind::ScreenshotSettings) => load_action_map(
      services,
      &ScreenshotSettingsUi::action_map(),
      "ScreenshotSettingsUi",
    ),
    Some(UiNodeKind::RecordingSettings) => load_action_map(
      services,
      &RecordingSettingsUi::action_map(),
      "RecordingSettingsUi",
    ),
    Some(UiNodeKind::ScreenshotList) => load_action_map(
      services,
      &ScreenshotListUi::action_map(),
      "ScreenshotListUi",
    ),
    Some(UiNodeKind::RecordingList) => {
      load_action_map(services, &RecordingListUi::action_map(), "RecordingListUi")
    }
    Some(UiNodeKind::SecuritySettings) => load_security_settings_action_map(services),
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
    Some(UiNodeKind::ExitWarning) => load_exit_warning_action_map(
      services,
      world.state.closing_state() == Some(RuntimeClosingState::WaitingForExports),
    ),
    _ => {}
  }
}

pub(super) fn load_game_action_map(services: &mut EngineServices) {
  let Some(package_id) = services.game.package().cloned() else {
    services.input.load_key_bindings(Vec::new());
    return;
  };
  let Some(entry) = services
    .package
    .game_list()
    .into_iter()
    .find(|entry| entry.id == package_id)
  else {
    services.input.load_key_bindings(Vec::new());
    return;
  };
  let entries = entry
    .key_actions
    .into_iter()
    .map(|(action, keys)| ActionMapEntry {
      description: action.clone(),
      action,
      keys,
    })
    .collect::<Vec<_>>();
  match translate_action_map(&entries) {
    Ok(bindings) => services.input.load_key_bindings(bindings),
    Err(error) => {
      services.log.error_package(
        &package_id,
        LogSource::Input,
        format!("Failed to translate active game action map: {error:?}"),
      );
      services.input.load_key_bindings(Vec::new());
    }
  }
}

pub(super) fn load_window_size_action_map(services: &mut EngineServices) {
  load_action_map(services, &WindowSizeWarningUi::action_map(), "window_size");
}

pub(super) fn load_game_warning_action_map(services: &mut EngineServices) {
  load_action_map(services, &GameWarningUi::action_map(), "game_warning");
}

pub(super) fn load_cover_continue_action_map(services: &mut EngineServices) {
  load_action_map(services, &CoverContinueUi::action_map(), "CoverContinueUi");
}

pub(super) fn load_screenshot_capture_action_map(services: &mut EngineServices) {
  load_action_map(
    services,
    &ScreenshotCaptureUi::action_map(),
    "ScreenshotCaptureUi",
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

pub(super) fn load_exit_warning_action_map(
  services: &mut EngineServices,
  waiting_for_exports: bool,
) {
  let entries = if waiting_for_exports {
    ExitWarningUi::waiting_action_map()
  } else {
    ExitWarningUi::action_map()
  };
  load_action_map(services, &entries, "ExitWarningUi");
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

pub(super) fn load_game_list_action_map(services: &mut EngineServices) {
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
  match translate_action_map(action_map) {
    Ok(bindings) => services.input.load_key_bindings(bindings),
    Err(error) => {
      services.log.error_operation_failed(
        LogSource::Input,
        "translate_action_map",
        name,
        format!("{error:?}"),
      );
      services.input.load_key_bindings(Vec::new());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::RichTextService;

  #[test]
  fn host_rich_text_uses_user_and_default_global_maps_independently() {
    let mut profile = KeyBindingsProfile::default();
    profile.default.global.insert(
      HOST_KEY_SCREENSHOT.to_string(),
      vec![vec!["f1".to_string()]],
    );
    profile.user.global.insert(
      HOST_KEY_SCREENSHOT.to_string(),
      vec![vec!["1".to_string()], vec!["2".to_string()]],
    );

    let params = host_key_rich_text_params(&profile);
    let visible = RichTextService::new().visible_text(
      "f%{key:host_key.screenshot}|{key_default:host_key.screenshot}",
      Some(&params),
    );

    assert_eq!(visible, "[1]/[2]|[F1]");
  }
}
