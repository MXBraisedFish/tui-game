use super::*;
use crate::host_engine::services::{UiObjectPool, UiObjectPoolOwner};

pub(super) fn apply_home_command(command: HomeUiCommand, world: &mut RuntimeWorld) {
  match command {
    HomeUiCommand::Exit => {
      world.state.enter_shutdown();
      set_crash_phase(world.state.crash_phase());
      world.state.enter_stopped();
      set_crash_phase(world.state.crash_phase());
    }
    HomeUiCommand::StartGame => world.state.enter_ui_node(UiNodeState::game_list()),
    HomeUiCommand::ContinueGame => {}
    HomeUiCommand::OpenSettings => world.state.enter_ui_node(UiNodeState::settings()),
    HomeUiCommand::OpenAbout => world.state.enter_ui_node(UiNodeState::input_demo()),
  }
}

pub(super) fn apply_game_list_command(
  command: GameListCommand,
  game_list_ui: &mut GameListUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    GameListCommand::Back => {
      world.state.pop_ui_node();
      reset_game_list_ui(game_list_ui, services);
    }
    GameListCommand::FocusSearch => game_list_ui.focus_search(&mut services.text_input),
    GameListCommand::BlurSearch => game_list_ui.blur_search(&mut services.text_input),
    GameListCommand::FocusJump => game_list_ui.focus_jump(&mut services.text_input),
    GameListCommand::BlurJump => game_list_ui.blur_jump(&mut services.text_input),
    GameListCommand::ScrollInfoUp => {
      game_list_ui.scroll_info(&services.scroll_box, &services.layout, -3)
    }
    GameListCommand::ScrollInfoDown => {
      game_list_ui.scroll_info(&services.scroll_box, &services.layout, 3)
    }
    GameListCommand::SubmitJump(value) => {
      game_list_ui.submit_jump(&mut services.text_input, value);
    }
    GameListCommand::Confirm => {}
  }
}

pub(super) fn apply_input_demo_command(
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

pub(super) fn apply_settings_command(
  command: SettingsUiCommand,
  settings_ui: &mut SettingsUi,
  security_uis: &mut SecurityUis,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    SettingsUiCommand::Back => {
      world.state.pop_ui_node();
      reset_settings_ui(settings_ui, services);
    }
    SettingsUiCommand::OpenLanguageSelect => {
      world.state.enter_ui_node(UiNodeState::language_select())
    }
    SettingsUiCommand::OpenMods => world.state.enter_ui_node(UiNodeState::mods()),
    SettingsUiCommand::OpenStorageManagement => {
      world.state.enter_ui_node(UiNodeState::storage_management())
    }
    SettingsUiCommand::OpenSecuritySettings => {
      let defaults = services
        .storage
        .read_package_state_or_default(&mut services.log)
        .defaults;
      security_uis
        .settings
        .set_defaults(defaults.enabled, defaults.debug, defaults.safe_mode);
      world.state.enter_ui_node(UiNodeState::security_settings())
    }
    SettingsUiCommand::OpenDisplaySettings => {
      world.state.enter_ui_node(UiNodeState::display_settings())
    }
    SettingsUiCommand::OpenScreensaverList => {
      world.state.enter_ui_node(UiNodeState::screensaver_list())
    }
  }
}

pub(super) fn apply_display_settings_command(
  command: DisplaySettingsCommand,
  display_settings_ui: &mut DisplaySettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    DisplaySettingsCommand::Changed(profile) => {
      let _ = services
        .storage
        .write_display_settings_profile(&profile, &mut services.log);
    }
    DisplaySettingsCommand::OpenToolbarCustom => {
      display_settings_ui
        .custom_mut()
        .enter(&mut services.text_input);
      world.state.enter_ui_node(UiNodeState::toolbar_custom());
    }
    DisplaySettingsCommand::Back => {
      world.state.pop_ui_node();
      clear_exiting_pool(display_settings_ui.objects_mut(), services);
      *display_settings_ui = DisplaySettingsUi::init(
        &services.hit_area,
        &services.text_input,
        services.storage.display_settings_profile().clone(),
      );
    }
  }
}

pub(super) fn apply_toolbar_custom_command(
  command: ToolbarCustomCommand,
  display_settings_ui: &mut DisplaySettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ToolbarCustomCommand::Changed(text) => display_settings_ui.set_custom_text(text),
    ToolbarCustomCommand::Submit(text) => {
      display_settings_ui.set_custom_text(text.clone());
      let mut profile = services.storage.display_settings_profile().clone();
      profile.top_toolbar_custom_text = text;
      let _ = services
        .storage
        .write_display_settings_profile(&profile, &mut services.log);
      display_settings_ui
        .custom_mut()
        .leave(&mut services.text_input);
      world.state.pop_ui_node();
    }
  }
}

pub(super) fn apply_screensaver_list_command(
  command: ScreensaverListCommand,
  ui: &mut ScreensaverListUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ScreensaverListCommand::Back => {
      world.state.pop_ui_node();
      reset_screensaver_list_ui(ui, services);
    }
    ScreensaverListCommand::FocusSearch => ui.focus_search(&mut services.text_input),
    ScreensaverListCommand::BlurSearch => ui.blur_search(&mut services.text_input),
    ScreensaverListCommand::Scroll(dy) => {
      ui.scroll_active(&services.scroll_box, &services.layout, dy)
    }
    ScreensaverListCommand::SetEnabled { id, enabled } => {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      let next_order = profile
        .screensavers
        .values()
        .filter_map(|state| state.enabled.then_some(state.order).flatten())
        .max()
        .map_or(0, |order| order.saturating_add(1));
      let defaults = &profile.defaults;
      let state = profile.screensavers.entry(id).or_insert(
        crate::host_engine::services::ScreensaverPackageState {
          enabled: defaults.enabled,
          debug: defaults.debug,
          order: None,
        },
      );
      state.enabled = enabled;
      state.order = enabled.then_some(next_order);
      let _ = services
        .storage
        .write_package_state(&profile, &mut services.log);
    }
    ScreensaverListCommand::SaveOrder(ids) => {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      let defaults = profile.defaults.clone();
      for (order, id) in ids.into_iter().enumerate() {
        let state = profile.screensavers.entry(id).or_insert(
          crate::host_engine::services::ScreensaverPackageState {
            enabled: defaults.enabled,
            debug: defaults.debug,
            order: None,
          },
        );
        state.enabled = true;
        state.order = Some(order as u32);
      }
      let _ = services
        .storage
        .write_package_state(&profile, &mut services.log);
    }
  }
}

pub(super) fn apply_security_settings_command(
  command: SecuritySettingsCommand,
  security_uis: &mut SecurityUis,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    SecuritySettingsCommand::Back => {
      world.state.pop_ui_node();
      clear_exiting_pool(security_uis.settings.objects_mut(), services);
      security_uis.settings = SecuritySettingsUi::init(&services.hit_area);
    }
    SecuritySettingsCommand::OpenDetails => {
      world.state.enter_ui_node(UiNodeState::security_details());
    }
    SecuritySettingsCommand::ResetStatus
    | SecuritySettingsCommand::ResetDebug
    | SecuritySettingsCommand::ResetSafeMode => {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      for entry in services.package.mod_games() {
        let mod_id = entry.mod_id;
        let mut initial = crate::host_engine::services::GamePackageState::default();
        initial.enabled = profile.defaults.enabled;
        initial.debug = profile.defaults.debug;
        initial.safe_mode =
          profile.defaults.safe_mode == crate::host_engine::services::SafeModeDefault::On;
        let state = profile.games.entry(mod_id.clone()).or_insert(initial);
        match command {
          SecuritySettingsCommand::ResetStatus => state.enabled = false,
          SecuritySettingsCommand::ResetDebug => state.debug = false,
          SecuritySettingsCommand::ResetSafeMode => {
            state.safe_mode = true;
            world.temporary_safe_mode_disabled.remove(&mod_id);
          }
          _ => unreachable!(),
        }
      }
      for entry in services.package.mod_screensavers() {
        let mut initial = crate::host_engine::services::ScreensaverPackageState::default();
        initial.enabled = profile.defaults.enabled;
        initial.debug = profile.defaults.debug;
        let state = profile.screensavers.entry(entry.mod_id).or_insert(initial);
        match command {
          SecuritySettingsCommand::ResetStatus => state.enabled = false,
          SecuritySettingsCommand::ResetDebug => state.debug = false,
          SecuritySettingsCommand::ResetSafeMode => {}
          _ => unreachable!(),
        }
      }
      let success = services
        .storage
        .write_package_state(&profile, &mut services.log)
        .is_ok();
      security_uis.settings.set_reset_result(success);
    }
    SecuritySettingsCommand::SetDefaultStatus(enabled) => {
      update_package_defaults(security_uis, services, |defaults| {
        defaults.enabled = enabled;
      });
    }
    SecuritySettingsCommand::SetDefaultDebug(debug) => {
      update_package_defaults(security_uis, services, |defaults| {
        defaults.debug = debug;
      });
    }
    SecuritySettingsCommand::SetDefaultSafeMode(safe_mode) => {
      if safe_mode == crate::host_engine::services::SafeModeDefault::OffPermanent {
        world.safe_mode_warning_all = true;
        world.state.push_safe_mode_warning_overlay();
      } else {
        update_package_defaults(security_uis, services, |defaults| {
          defaults.safe_mode = safe_mode;
        });
      }
    }
  }
}

fn update_package_defaults(
  security_uis: &mut SecurityUis,
  services: &mut EngineServices,
  update: impl FnOnce(&mut crate::host_engine::services::PackageDefaultState),
) {
  let mut profile = services
    .storage
    .read_package_state_or_default(&mut services.log);
  update(&mut profile.defaults);
  if services
    .storage
    .write_package_state(&profile, &mut services.log)
    .is_ok()
  {
    security_uis.settings.set_defaults(
      profile.defaults.enabled,
      profile.defaults.debug,
      profile.defaults.safe_mode,
    );
  } else {
    security_uis.settings.set_reset_result(false);
  }
}

pub(super) fn apply_security_details_command(
  command: SecurityDetailsCommand,
  security_uis: &mut SecurityUis,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    SecurityDetailsCommand::Back => {
      world.state.pop_ui_node();
      clear_exiting_pool(security_uis.details.objects_mut(), services);
      security_uis.details = SecurityDetailsUi::init(
        &services.hit_area,
        &services.scroll_box,
        &services.markdown,
        &services.storage,
        &services.i18n,
      );
    }
    SecurityDetailsCommand::Scroll(amount) => {
      security_uis
        .details
        .scroll(amount, &services.scroll_box, &services.layout);
      services.canvas.request_render();
    }
  }
}

pub(super) fn apply_storage_management_command(
  command: StorageManagementCommand,
  storage_management_ui: &mut StorageManagementUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    StorageManagementCommand::Back => {
      world.state.pop_ui_node();
      reset_storage_management_ui(storage_management_ui, services);
    }
    StorageManagementCommand::OpenView => {
      world
        .state
        .enter_ui_node(UiNodeState::storage_management_view());
    }
    StorageManagementCommand::OpenClear => {
      world
        .state
        .enter_ui_node(UiNodeState::storage_management_clear());
    }
    StorageManagementCommand::OpenExport => {
      world
        .state
        .enter_ui_node(UiNodeState::storage_management_export());
    }
  }
}

pub(super) fn apply_storage_management_clear_command(
  command: StorageManagementClearCommand,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  clear_warning_ui: &mut ClearWarningUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    StorageManagementClearCommand::Back => {
      world.state.pop_ui_node();
      reset_storage_management_clear_ui(storage_management_clear_ui, services);
    }
    StorageManagementClearCommand::ClearCache
    | StorageManagementClearCommand::ClearLog
    | StorageManagementClearCommand::ClearMod
    | StorageManagementClearCommand::ClearProfile
    | StorageManagementClearCommand::ClearScreenshot
    | StorageManagementClearCommand::ClearRecording
    | StorageManagementClearCommand::ClearData => {
      let (target, path) = match command {
        StorageManagementClearCommand::ClearCache => {
          (ClearWarningTarget::Cache, services.storage.cache_dir_path())
        }
        StorageManagementClearCommand::ClearLog => {
          (ClearWarningTarget::Log, services.storage.log_dir_path())
        }
        StorageManagementClearCommand::ClearMod => {
          (ClearWarningTarget::Mod, services.storage.mod_dir_path())
        }
        StorageManagementClearCommand::ClearProfile => (
          ClearWarningTarget::Profile,
          services.storage.profiles_dir_path(),
        ),
        StorageManagementClearCommand::ClearScreenshot => (
          ClearWarningTarget::Screenshot,
          services.storage.screenshot_dir_path(),
        ),
        StorageManagementClearCommand::ClearRecording => (
          ClearWarningTarget::Recording,
          services.storage.recording_dir_path(),
        ),
        StorageManagementClearCommand::ClearData => {
          (ClearWarningTarget::Data, services.storage.data_dir_path())
        }
        StorageManagementClearCommand::Back => unreachable!(),
      };
      clear_warning_ui.start(target, path);
      world.state.push_clear_warning_overlay();
    }
  }
}

pub(super) fn apply_clear_warning_command(
  command: ClearWarningCommand,
  clear_warning_ui: &mut ClearWarningUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  if command == ClearWarningCommand::Confirm {
    if let Some(target) = clear_warning_ui.target() {
      let result = match target {
        ClearWarningTarget::Cache => services.storage.clear_cache(&mut services.log),
        ClearWarningTarget::Log => services.storage.clear_log(&mut services.log),
        ClearWarningTarget::Mod => services.storage.clear_mod(&mut services.log),
        ClearWarningTarget::Profile => services.storage.clear_profiles(&mut services.log),
        ClearWarningTarget::Screenshot => services.storage.clear_screenshot(&mut services.log),
        ClearWarningTarget::Recording => services.storage.clear_recording(&mut services.log),
        ClearWarningTarget::Data => services.storage.clear_data(&mut services.log),
      };
      if let Err(error) = result {
        services.log.error(
          LogSource::Storage,
          format!("Failed to clear storage target {:?}: {}", target, error),
        );
      } else if matches!(target, ClearWarningTarget::Mod | ClearWarningTarget::Data) {
        let package_language = services.i18n.current_language().to_string();
        let missing_template = services
          .i18n
          .get_runtime_text("language_warning", "language_warning.missing");
        let _ = services.package.request_rescan_for_language(
          &services.async_runtime,
          &package_language,
          &missing_template,
        );
      }
    }
  }
  let _ = world.state.remove_overlay_kind(OverlayKind::ClearWarning);
}

pub(super) fn apply_storage_management_export_command(
  command: StorageManagementExportCommand,
  storage_management_export_ui: &mut StorageManagementExportUi,
  export_settings_ui: &mut ExportSettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    StorageManagementExportCommand::Back => {
      world.state.pop_ui_node();
      reset_storage_management_export_ui(storage_management_export_ui, services);
    }
    StorageManagementExportCommand::ExportCache
    | StorageManagementExportCommand::ExportLog
    | StorageManagementExportCommand::ExportMod
    | StorageManagementExportCommand::ExportProfile
    | StorageManagementExportCommand::ExportScreenshot
    | StorageManagementExportCommand::ExportRecording
    | StorageManagementExportCommand::ExportData => {
      let export_type = match command {
        StorageManagementExportCommand::ExportCache => ExportType::Cache,
        StorageManagementExportCommand::ExportLog => ExportType::Log,
        StorageManagementExportCommand::ExportMod => ExportType::Mod,
        StorageManagementExportCommand::ExportProfile => ExportType::Profile,
        StorageManagementExportCommand::ExportScreenshot => ExportType::Screenshot,
        StorageManagementExportCommand::ExportRecording => ExportType::Recording,
        StorageManagementExportCommand::ExportData => ExportType::Data,
        StorageManagementExportCommand::Back => unreachable!(),
      };
      export_settings_ui.start(export_type, services.storage.root_dir().to_path_buf());
      world.state.push_export_settings_overlay();
    }
  }
}

pub(super) fn apply_storage_management_view_command(
  command: StorageManagementViewCommand,
  storage_management_view_ui: &mut StorageManagementViewUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    StorageManagementViewCommand::Back => {
      world.state.pop_ui_node();
      reset_storage_management_view_ui(storage_management_view_ui, services);
    }
    StorageManagementViewCommand::CopyAll(text) | StorageManagementViewCommand::CopyPath(text) => {
      if !services.clipboard.write_text(&text) {
        services
          .log
          .warn(LogSource::Ui, "Clipboard write failed".to_string());
      }
    }
  }
}

pub(super) fn apply_mods_command(
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
    ModsCommand::OpenGame => world.state.enter_ui_node(UiNodeState::game_package()),
    ModsCommand::OpenScreensaver => world
      .state
      .enter_ui_node(UiNodeState::screensaver_package()),
  }
}

pub(super) fn apply_game_package_command(
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
    GamePackageCommand::FocusSearch => game_package_ui.focus_search(&mut services.text_input),
    GamePackageCommand::BlurSearch => game_package_ui.blur_search(&mut services.text_input),
    GamePackageCommand::FocusJump => game_package_ui.focus_jump(&mut services.text_input),
    GamePackageCommand::BlurJump => game_package_ui.blur_jump(&mut services.text_input),
    GamePackageCommand::ScrollInfoUp => {
      game_package_ui.scroll_info(&services.scroll_box, &services.layout, -3);
    }
    GamePackageCommand::ScrollInfoDown => {
      game_package_ui.scroll_info(&services.scroll_box, &services.layout, 3);
    }
    GamePackageCommand::SubmitJump(value) => {
      game_package_ui.submit_jump(&mut services.text_input, value);
    }
    GamePackageCommand::ToggleEnabled => {
      game_package_ui.toggle_selected_enabled(&services.storage, &mut services.log);
    }
    GamePackageCommand::ToggleDebug => {
      game_package_ui.toggle_selected_debug(&services.storage, &mut services.log);
    }
    GamePackageCommand::RequestToggleSafeMode => {
      if game_package_ui.selected_safe_mode().unwrap_or(true) {
        world.safe_mode_warning_all = false;
        world.state.push_safe_mode_warning_overlay();
      } else {
        if let Some(mod_id) = game_package_ui.selected_mod_id() {
          world.temporary_safe_mode_disabled.remove(&mod_id);
        }
        game_package_ui.enable_selected_safe_mode(&services.storage, &mut services.log);
      }
    }
  }
}

pub(super) fn apply_safe_mode_warning_command(
  command: SafeModeWarningCommand,
  security_uis: &mut SecurityUis,
  game_package_ui: &mut GamePackageUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  if world.safe_mode_warning_all {
    if command == SafeModeWarningCommand::DisablePermanent {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      profile.defaults.safe_mode = crate::host_engine::services::SafeModeDefault::OffPermanent;
      for entry in services.package.mod_games() {
        let initial = crate::host_engine::services::GamePackageState {
          enabled: profile.defaults.enabled,
          debug: profile.defaults.debug,
          safe_mode: false,
        };
        profile
          .games
          .entry(entry.mod_id)
          .or_insert(initial)
          .safe_mode = false;
      }
      if services
        .storage
        .write_package_state(&profile, &mut services.log)
        .is_ok()
      {
        world.temporary_safe_mode_disabled.clear();
        security_uis.settings.set_defaults(
          profile.defaults.enabled,
          profile.defaults.debug,
          profile.defaults.safe_mode,
        );
      } else {
        security_uis.settings.set_reset_result(false);
      }
    }
    world.safe_mode_warning_all = false;
    let _ = world
      .state
      .remove_overlay_kind(OverlayKind::SafeModeWarning);
    return;
  }
  match command {
    SafeModeWarningCommand::Cancel => {}
    SafeModeWarningCommand::DisableTemporary => {
      if let Some(mod_id) = game_package_ui.selected_mod_id() {
        world.temporary_safe_mode_disabled.insert(mod_id);
      }
      game_package_ui.disable_selected_safe_mode_temporary();
    }
    SafeModeWarningCommand::DisablePermanent => {
      if let Some(mod_id) = game_package_ui.selected_mod_id() {
        world.temporary_safe_mode_disabled.remove(&mod_id);
      }
      // TODO: log warn when storage write fails inside disable_selected_safe_mode_permanent
      game_package_ui.disable_selected_safe_mode_permanent(&services.storage, &mut services.log);
    }
  }
  let _ = world
    .state
    .remove_overlay_kind(OverlayKind::SafeModeWarning);
}

pub(super) fn apply_screensaver_package_command(
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
    ScreensaverPackageCommand::ScrollInfoUp => {
      screensaver_package_ui.scroll_info(&services.scroll_box, &services.layout, -3);
    }
    ScreensaverPackageCommand::ScrollInfoDown => {
      screensaver_package_ui.scroll_info(&services.scroll_box, &services.layout, 3);
    }
    ScreensaverPackageCommand::SubmitJump(value) => {
      screensaver_package_ui.submit_jump(&mut services.text_input, value);
    }
    ScreensaverPackageCommand::ToggleEnabled => {
      screensaver_package_ui.toggle_selected_enabled(&services.storage, &mut services.log);
    }
    ScreensaverPackageCommand::ToggleDebug => {
      screensaver_package_ui.toggle_selected_debug(&services.storage, &mut services.log);
    }
  }
}

pub(super) fn apply_language_select_command(
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
      let enter_terminal_check_after_finish = !services
        .storage
        .is_terminal_profile_complete(&mut services.log);
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

pub(super) fn apply_terminal_check_command(
  command: TerminalCheckCommand,
  terminal_check_ui: &mut TerminalCheckUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    TerminalCheckCommand::Next => {
      terminal_check_ui.persist_current_step(&mut services.storage, &mut services.log);
      sync_terminal_capabilities_from_profile(services);
      terminal_check_ui.advance_step();
    }
    TerminalCheckCommand::Done { mouse } => {
      if let Err(e) = services
        .storage
        .update_terminal_profile(&mut services.log, |p| {
          p.mouse = Some(mouse);
        })
      {
        services.log.warn(
          LogSource::Storage,
          format!("Failed to update terminal profile: {e}"),
        );
      }
      sync_terminal_capabilities_from_profile(services);
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

fn sync_terminal_capabilities_from_profile(services: &mut EngineServices) {
  let profile = services
    .storage
    .read_terminal_profile_or_default(&mut services.log);
  services.terminal.apply_capability_profile(
    profile.unicode,
    profile.color.as_deref(),
    profile.mouse,
  );
}

pub(super) fn apply_language_loading_package_events(
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
  language_loading_ui.restart_animation(&services.time);
  world.state.push_language_loading_overlay();
  let _ = services.storage.write_language_code(code);
  services
    .i18n
    .load_runtime_language(&services.storage, &mut services.log, code);
  let _ = services.log.refresh_labels_from_i18n(&services.i18n);
  language_loading_ui.set_progress(&services.progress_bar, 0.5, 0.5);
  let package_language = services.i18n.current_language().to_string();
  let missing_template = services
    .i18n
    .get_runtime_text("language_warning", "language_warning.missing");
  let requested = services.package.request_rescan_for_language(
    &services.async_runtime,
    &package_language,
    &missing_template,
  );
  if !requested {
    finish_language_loading(language_loading, language_loading_ui, services, world);
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

fn reset_storage_management_ui(ui: &mut StorageManagementUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = StorageManagementUi::init(&services.hit_area);
}

fn reset_storage_management_clear_ui(
  ui: &mut StorageManagementClearUi,
  services: &mut EngineServices,
) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = StorageManagementClearUi::init(&services.hit_area);
}

fn reset_storage_management_export_ui(
  ui: &mut StorageManagementExportUi,
  services: &mut EngineServices,
) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = StorageManagementExportUi::init(&services.hit_area);
}

fn reset_storage_management_view_ui(
  ui: &mut StorageManagementViewUi,
  services: &mut EngineServices,
) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = StorageManagementViewUi::init(&services.hit_area, &services.table);
}

fn reset_mods_ui(ui: &mut ModsUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = ModsUi::init(&services.hit_area);
}

fn reset_game_list_ui(ui: &mut GameListUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = GameListUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
}

fn reset_game_package_ui(ui: &mut GamePackageUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = GamePackageUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
}

fn reset_screensaver_list_ui(ui: &mut ScreensaverListUi, services: &mut EngineServices) {
  clear_exiting_pool(ui.objects_mut(), services);
  *ui = ScreensaverListUi::init(
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

pub(super) fn apply_export_settings_command(
  command: ExportSettingsCommand,
  export_settings_ui: &mut ExportSettingsUi,
  export_loading_ui: &mut ExportLoadingUi,
  export_loading: &mut ExportLoadingRuntime,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ExportSettingsCommand::Cancel => {
      export_settings_ui.blur_input(&mut services.text_input);
      let _ = world.state.remove_overlay_kind(OverlayKind::ExportSettings);
    }
    ExportSettingsCommand::FocusInput => {
      export_settings_ui.focus_input(&mut services.text_input);
    }
    ExportSettingsCommand::BlurInput => {
      export_settings_ui.blur_input(&mut services.text_input);
    }
    ExportSettingsCommand::CancelInput => {
      export_settings_ui.cancel_input(&mut services.text_input);
    }
    ExportSettingsCommand::ConfirmExport => {
      export_settings_ui.blur_input(&mut services.text_input);

      let name = export_settings_ui.resolved_name();
      let out_dir = std::path::PathBuf::from(export_settings_ui.resolved_path());
      let format = match export_settings_ui.format() {
        ExportFormat::Zip => crate::host_engine::services::export::ExportFormat::Zip,
        ExportFormat::Tar => crate::host_engine::services::export::ExportFormat::Tar,
        ExportFormat::TarGz => crate::host_engine::services::export::ExportFormat::TarGz,
      };
      let scope = match export_settings_ui.export_scope() {
        Some(ExportType::Cache) => crate::host_engine::services::export::ExportScope::Cache,
        Some(ExportType::Log) => crate::host_engine::services::export::ExportScope::Log,
        Some(ExportType::Mod) => crate::host_engine::services::export::ExportScope::Mod,
        Some(ExportType::Profile) => crate::host_engine::services::export::ExportScope::Profile,
        Some(ExportType::Screenshot) => {
          crate::host_engine::services::export::ExportScope::Screenshot
        }
        Some(ExportType::Recording) => crate::host_engine::services::export::ExportScope::Recording,
        Some(ExportType::Data) => crate::host_engine::services::export::ExportScope::Data,
        None => {
          let _ = world.state.remove_overlay_kind(OverlayKind::ExportSettings);
          return;
        }
      };

      let task_id = services.export.submit_export(
        &services.async_runtime,
        crate::host_engine::services::ExportTask {
          scope,
          output_dir: out_dir,
          file_stem: name,
          format,
          root_dir: services.storage.root_dir().to_path_buf(),
        },
      );
      export_loading.active = true;
      export_loading.task_id = Some(task_id);
      export_loading_ui.set_progress(&services.progress_bar, 0.0, 0.0);
      export_loading_ui.restart_animation(&services.time);
      let _ = world.state.remove_overlay_kind(OverlayKind::ExportSettings);
      world.state.push_export_loading_overlay();
    }
  }
}

pub(super) fn apply_export_loading_events(
  events: &[crate::host_engine::services::ExportAsyncEvent],
  export_loading: &mut ExportLoadingRuntime,
  export_loading_ui: &mut ExportLoadingUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  let Some(active_task) = export_loading.task_id else {
    return;
  };

  for event in events {
    match event {
      crate::host_engine::services::ExportAsyncEvent::Started { task_id, total }
        if *task_id == active_task =>
      {
        let preview = if *total == 0 { 1.0 } else { 0.0 };
        export_loading_ui.set_progress(&services.progress_bar, 0.0, preview);
      }
      crate::host_engine::services::ExportAsyncEvent::Progress {
        task_id,
        packed,
        total,
      } if *task_id == active_task => {
        let progress = if *total == 0 {
          1.0
        } else {
          (*packed as f32 / *total as f32).clamp(0.0, 1.0)
        };
        export_loading_ui.set_progress(&services.progress_bar, progress, progress);
      }
      crate::host_engine::services::ExportAsyncEvent::Finished { task_id, path }
        if *task_id == active_task =>
      {
        export_loading_ui.set_progress(&services.progress_bar, 1.0, 1.0);
        services
          .log
          .info(LogSource::Storage, format!("导出成功: {}", path.display()));
        finish_export_loading(export_loading, world);
      }
      crate::host_engine::services::ExportAsyncEvent::Failed { task_id, error }
        if *task_id == active_task =>
      {
        services
          .log
          .error(LogSource::Storage, format!("导出失败: {error}"));
        finish_export_loading(export_loading, world);
      }
      _ => {}
    }
  }
}

fn finish_export_loading(export_loading: &mut ExportLoadingRuntime, world: &mut RuntimeWorld) {
  export_loading.active = false;
  export_loading.task_id = None;
  let _ = world.state.remove_overlay_kind(OverlayKind::ExportLoading);
}
