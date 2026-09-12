use super::*;
use crate::host_engine::services::{UiObjectPool, UiObjectPoolOwner};

pub(super) fn apply_home_command(
  command: HomeUiCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    HomeUiCommand::Exit => {
      world.state.request_shutdown();
    }
    HomeUiCommand::StartGame => world.state.enter_ui_node(UiNodeState::game_list()),
    HomeUiCommand::ContinueGame => {
      let Some(slot) = services.storage.continue_game_save() else {
        return;
      };
      start_game(slot.package, Some(slot.data), false, services, world);
    }
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
    GameListCommand::Confirm { package_id } => {
      if services.storage.continue_game_save().is_some() {
        world.pending_new_game = Some(package_id);
        world.state.push_cover_continue_overlay();
        load_cover_continue_action_map(services);
      } else {
        start_game(package_id, None, false, services, world);
      }
    }
  }
}

fn start_game(
  package_id: crate::host_engine::services::PackageId,
  continue_data: Option<serde_json::Value>,
  clear_continue_before_start: bool,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) -> bool {
  let continuing = continue_data.is_some();
  let Some(package) = services.package.find_by_id(&package_id) else {
    log_package_start_error(
      services,
      &package_id,
      "game package was not found".to_string(),
    );
    if continuing {
      let _ = services.storage.clear_continue_game_save(&mut services.log);
    }
    return false;
  };
  let Some(game) = package.game.as_ref() else {
    log_package_start_error(
      services,
      &package.id,
      "game package has no game configuration".to_string(),
    );
    if continuing {
      let _ = services.storage.clear_continue_game_save(&mut services.log);
    }
    return false;
  };
  if continue_data.is_some() && !game.save {
    services.log.warn_package(
      &package.id,
      LogSource::Lua,
      "Continue slot was provided for a game without save support",
    );
    let _ = services.storage.clear_continue_game_save(&mut services.log);
    return false;
  }
  let entry_path = match services.package.validate_for_launch(&package) {
    Ok(path) => path,
    Err(error) => {
      log_package_start_error(
        services,
        &package.id,
        format!("game entry resolution failed: {error}"),
      );
      if continuing {
        let _ = services.storage.clear_continue_game_save(&mut services.log);
      }
      return false;
    }
  };
  let entry = services
    .package
    .game_list()
    .into_iter()
    .find(|entry| entry.id == package_id);
  let package_state = services
    .storage
    .read_package_state_or_default(&mut services.log);
  let state = package_state.game(&package.id);
  let debug_enabled = state.map_or(package_state.defaults.debug, |state| state.debug);
  let mut safe_mode_enabled = state.map_or(
    matches!(
      package_state.defaults.safe_mode,
      crate::host_engine::services::SafeModeDefault::On
    ),
    |state| state.safe_mode,
  );
  if world.temporary_safe_mode_disabled.contains(&package.id) {
    safe_mode_enabled = false;
  }
  let (key_actions, key_default_actions) = entry
    .map(|entry| (entry.key_actions, entry.key_default_actions))
    .unwrap_or_default();
  if let Err(error) = services.package.validate_game_action_map(&key_actions) {
    log_package_start_error(
      services,
      &package.id,
      format!("game user action map validation failed: {error}"),
    );
    if continuing {
      let _ = services.storage.clear_continue_game_save(&mut services.log);
    }
    return false;
  }
  let action_entries = key_actions
    .iter()
    .map(
      |(action, keys)| crate::host_engine::services::ActionMapEntry {
        action: action.clone(),
        description: action.clone(),
        keys: keys.clone(),
      },
    )
    .collect::<Vec<_>>();
  if let Err(error) = crate::host_engine::services::translate_action_map(&action_entries) {
    log_package_start_error(
      services,
      &package.id,
      format!("game action map validation failed: {error:?}"),
    );
    if continuing {
      let _ = services.storage.clear_continue_game_save(&mut services.log);
    }
    return false;
  }
  let api = crate::host_engine::services::LuaApiConfig {
    debug_enabled,
    safe_mode_enabled,
    key_actions,
    key_default_actions,
    language_code: services.i18n.current_language_code().to_string(),
    missing_i18n_template: services
      .i18n
      .get_runtime_text("language_warning", "language_warning.missing"),
  };
  let best_data = services
    .storage
    .best_game_save(&package.id)
    .map(|best| best.data);
  let save_best_enabled = game.score.as_ref().is_some_and(|score| score.enabled);
  let spec = crate::host_engine::services::LuaSessionSpec {
    package_id: package.mod_id.clone(),
    session_kind: crate::host_engine::services::LuaSessionKind::Game,
    entry_path: entry_path.clone(),
    fixed_delta: std::time::Duration::from_secs_f64(1.0 / 60.0),
    base_size: lua_session_base_size(services, LuaSessionKind::Game),
    continue_data,
    best_data,
    save_game_enabled: game.save,
    save_best_enabled,
  };
  let session_log = services
    .log
    .open_session(
      crate::host_engine::services::LogSessionKind::Game,
      &package.id,
    )
    .ok();
  let session = match services.lua.create_session_with_api(spec, api) {
    Ok(session) => session,
    Err(error) => {
      flush_lua_startup_diagnostics(
        services,
        session_log,
        crate::host_engine::services::LuaSessionKind::Game,
        &error.diagnostic_commands,
      );
      let message = format!("{error}; entry={}", entry_path.display());
      if let Some(id) = session_log {
        services.log.error_session(id, LogSource::Lua, message);
        services.log.close_session(id);
      } else {
        log_package_start_error(services, &package.id, message);
      }
      if continuing {
        let _ = services.storage.clear_continue_game_save(&mut services.log);
      }
      show_game_start_fault(services, world);
      return false;
    }
  };
  let return_host = world
    .state
    .runtime()
    .and_then(|runtime| runtime.main_host().host())
    .cloned();
  let Some(return_host) = return_host else {
    if let Some(id) = session_log {
      services.log.close_session(id);
    }
    if continuing {
      let _ = services.storage.clear_continue_game_save(&mut services.log);
    }
    return false;
  };
  if clear_continue_before_start
    && services
      .storage
      .clear_continue_game_save(&mut services.log)
      .is_err()
  {
    if let Some(id) = session_log {
      services.log.close_session(id);
    }
    log_package_start_error(
      services,
      &package.id,
      "failed to clear the previous continue slot before starting a new game".to_string(),
    );
    return false;
  }
  if let Some(previous) = services.game.start(
    session,
    package.id.clone(),
    game.target_fps,
    crate::host_engine::services::Size {
      width: package.runtime.min_width.min(u16::MAX as u32) as u16,
      height: package.runtime.min_height.min(u16::MAX as u32) as u16,
    },
    game.save,
    save_best_enabled,
    session_log,
  ) {
    services.log.close_session(previous);
  }
  if let Some(runtime) = world.state.runtime_mut() {
    runtime.set_main_host(MainHostState::Game(
      crate::host_engine::core::state_machine::GameState::new(
        package.id,
        package.runtime.min_width,
        package.runtime.min_height,
        game.target_fps,
        return_host,
      ),
    ));
  }
  services.canvas.request_render();
  services.presenter.request_render();
  true
}

/// Lua 入口尚未成功创建 Session 时，无法走运行中 Session 的统一故障处理，
/// 但对玩家而言仍然是一次游戏启动故障，必须给出可见反馈。
fn show_game_start_fault(services: &mut EngineServices, world: &mut RuntimeWorld) {
  world.state.push_game_warning_overlay();
  services.input.clear();
  services.canvas.request_render();
  services.presenter.request_render();
}

pub(super) fn apply_cover_continue_command(
  command: CoverContinueCommand,
  cover_continue_ui: &mut CoverContinueUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    CoverContinueCommand::Start => {
      let Some(package_id) = world.pending_new_game.take() else {
        let _ = world.state.remove_overlay_kind(OverlayKind::CoverContinue);
        cover_continue_ui.reset();
        return;
      };
      let started = start_game(package_id, None, true, services, world);
      if started {
        let _ = world.state.remove_overlay_kind(OverlayKind::CoverContinue);
        cover_continue_ui.reset();
      } else {
        let _ = world.state.remove_overlay_kind(OverlayKind::CoverContinue);
        cover_continue_ui.reset();
        load_game_list_action_map(services);
      }
    }
    CoverContinueCommand::Back => {
      world.pending_new_game = None;
      let _ = world.state.remove_overlay_kind(OverlayKind::CoverContinue);
      cover_continue_ui.reset();
      load_game_list_action_map(services);
    }
  }
}

pub(super) fn log_package_start_error(
  services: &mut EngineServices,
  package_id: &crate::host_engine::services::PackageId,
  message: String,
) {
  services
    .log
    .error_package(package_id, LogSource::Lua, message);
  let path = services
    .log
    .package_log_path(package_id)
    .map(|path| path.display().to_string())
    .unwrap_or_else(|_| "<unavailable>".to_string());
  services.log.error_message(
    LogSource::Runtime,
    HostLogMessage::new(
      "log_info.session.faulted",
      "{kind} session for {package} was isolated after a fault; see its package log.",
    )
    .param("kind", package_id.package_type.as_str())
    .param("package", package_id.to_string())
    .param("path", path),
  );
}

pub(super) fn apply_input_demo_command(
  command: InputDemoCommand,
  input_demo_ui: &mut InputDemoUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    InputDemoCommand::Back => {
      input_demo_ui.leave(&mut services.audio);
      world.state.pop_ui_node();
      reset_input_demo_ui(input_demo_ui, services);
    }
    InputDemoCommand::TogglePlayback => input_demo_ui.toggle_playback(&mut services.audio),
    InputDemoCommand::Stop => input_demo_ui.stop(&mut services.audio),
    InputDemoCommand::Restart => input_demo_ui.restart(&mut services.audio),
    InputDemoCommand::VolumeDown => {
      input_demo_ui.adjust_volume(&mut services.audio, -0.1);
    }
    InputDemoCommand::VolumeUp => {
      input_demo_ui.adjust_volume(&mut services.audio, 0.1);
    }
    InputDemoCommand::ToggleLoop => input_demo_ui.toggle_loop(&mut services.audio),
  }
  services.canvas.request_render();
  services.presenter.request_render();
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
    SettingsUiCommand::OpenKeyBindings => world.state.enter_ui_node(UiNodeState::key_bindings()),
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
    SettingsUiCommand::OpenScreenshotRecording => world
      .state
      .enter_ui_node(UiNodeState::screenshot_recording()),
  }
}

pub(super) fn apply_key_bindings_command(
  command: KeyBindingsCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    KeyBindingsCommand::Back => {
      world.state.pop_ui_node();
      let ui = settings_ui.key_bindings_mut();
      clear_exiting_pool(ui.objects_mut(), services);
      *ui = KeyBindingsUi::init(
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
      );
    }
    KeyBindingsCommand::OpenGlobal => {
      let profile = synchronize_key_bindings_profile(services);
      let entries = host_key_action_entries_from_profile(services, &profile);
      settings_ui
        .key_bindings_mut()
        .global_mut()
        .load(entries, profile);
      world
        .state
        .enter_ui_node(UiNodeState::global_key_bindings());
    }
    KeyBindingsCommand::OpenGame => {
      let profile = synchronize_key_bindings_profile(services);
      let games = services.package.games();
      settings_ui
        .key_bindings_mut()
        .game_mut()
        .load(games, profile);
      world.state.enter_ui_node(UiNodeState::game_key_bindings());
    }
  }
}

pub(super) fn apply_global_key_bindings_command(
  command: GlobalKeyBindingsCommand,
  _settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    GlobalKeyBindingsCommand::Back(profile) => {
      if services
        .storage
        .write_key_bindings_profile(&profile, &mut services.log)
        .is_ok()
      {
        let _ = load_host_key_action_map(services);
        world.state.pop_ui_node();
      }
    }
    GlobalKeyBindingsCommand::Conflict(mut request) => {
      request.text = services
        .i18n
        .get_runtime_text("key_bindings_global", "key_bindings_global.conflict");
      services.popup.show(request);
    }
    GlobalKeyBindingsCommand::CaptureStarted => {
      let _ = services.input.enable_raw_key_capture();
      let _ = services.input.take_raw_key_events();
    }
  }
}

pub(super) fn apply_game_key_bindings_command(
  command: GameKeyBindingsCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    GameKeyBindingsCommand::Back(profile) => {
      if services
        .storage
        .write_key_bindings_profile(&profile, &mut services.log)
        .is_ok()
      {
        services
          .package
          .set_user_game_key_actions(profile.user.games.clone());
        world.state.pop_ui_node();
      }
    }
    GameKeyBindingsCommand::Conflict(mut request) => {
      request.text = services
        .i18n
        .get_runtime_text("key_bindings_game", "key_bindings_game.conflict");
      services.popup.show(request);
    }
    GameKeyBindingsCommand::FocusSearch => {
      let ui = settings_ui.key_bindings_mut().game_mut();
      let id = ui.search_input();
      let _ = services.text_input.focus(ui.objects_mut(), id);
    }
    GameKeyBindingsCommand::BlurSearch => {
      let ui = settings_ui.key_bindings_mut().game_mut();
      let _ = services.text_input.blur(ui.objects_mut());
    }
    GameKeyBindingsCommand::CaptureStarted => {
      let _ = services.input.enable_raw_key_capture();
      let _ = services.input.take_raw_key_events();
    }
    GameKeyBindingsCommand::Scroll(dy) => settings_ui.key_bindings_mut().game_mut().scroll_active(
      &services.scroll_box,
      &services.layout,
      dy,
    ),
  }
}

pub(super) fn apply_screenshot_recording_command(
  command: ScreenshotRecordingCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ScreenshotRecordingCommand::Back => {
      world.state.pop_ui_node();
      let ui = settings_ui.screenshot_recording_mut();
      clear_exiting_pool(ui.objects_mut(), services);
      *ui = ScreenshotRecordingUi::init(
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
      );
    }
    ScreenshotRecordingCommand::OpenScreenshotSettings => {
      let profile = services
        .storage
        .read_screenshot_profile_or_default(&mut services.log);
      let _ = services
        .storage
        .write_screenshot_profile(&profile, &mut services.log);
      *settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut() = ScreenshotSettingsUi::init(
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        profile,
      );
      world
        .state
        .enter_ui_node(UiNodeState::screenshot_settings());
    }
    ScreenshotRecordingCommand::OpenRecordingSettings => {
      let profile = services
        .storage
        .read_recording_profile_or_default(&mut services.log);
      let fonts = services
        .storage
        .read_screenshot_profile_or_default(&mut services.log)
        .fonts;
      let _ = services
        .storage
        .write_recording_profile(&profile, &mut services.log);
      *settings_ui
        .screenshot_recording_mut()
        .recording_settings_mut() = RecordingSettingsUi::init(
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        profile,
        fonts,
      );
      world.state.enter_ui_node(UiNodeState::recording_settings());
    }
    ScreenshotRecordingCommand::OpenScreenshotList => {
      let path = services.storage.screenshot_cache_dir_path();
      let ui = settings_ui.screenshot_recording_mut().screenshot_list_mut();
      ui.reset_for_entry(
        &mut services.text_input,
        &services.scroll_box,
        &services.layout,
      );
      if let Err(error) = ui.reload(&path) {
        services.log.error_operation_failed(
          LogSource::Ui,
          "scan_screenshot_cache",
          path.display().to_string(),
          error.to_string(),
        );
      }
      world.state.enter_ui_node(UiNodeState::screenshot_list());
    }
    ScreenshotRecordingCommand::OpenRecordingList => {
      let path = services.storage.recording_cache_dir_path();
      let ui = settings_ui.screenshot_recording_mut().recording_list_mut();
      ui.reset_for_entry(
        &mut services.text_input,
        &services.scroll_box,
        &services.layout,
      );
      if let Err(error) = ui.reload(&path) {
        services.log.error_operation_failed(
          LogSource::Ui,
          "scan_recording_cache",
          path.display().to_string(),
          error.to_string(),
        );
      }
      world.state.enter_ui_node(UiNodeState::recording_list());
    }
  }
}

pub(super) fn apply_screenshot_list_command(
  command: ScreenshotListCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  let ui = settings_ui.screenshot_recording_mut().screenshot_list_mut();
  match command {
    ScreenshotListCommand::Back => {
      ui.blur_search(&mut services.text_input);
      clear_exiting_pool(ui.objects_mut(), services);
      world.state.pop_ui_node();
    }
    ScreenshotListCommand::FocusSearch => ui.focus_search(&mut services.text_input),
    ScreenshotListCommand::BlurSearch => ui.blur_search(&mut services.text_input),
    ScreenshotListCommand::SelectList(dy) => {
      ui.select_list(&services.scroll_box, &services.layout, dy)
    }
    ScreenshotListCommand::ScrollList(dy) => {
      ui.scroll_list(&services.scroll_box, &services.layout, dy)
    }
    ScreenshotListCommand::ScrollInfo { dx, dy } => {
      ui.scroll_info(&services.scroll_box, &services.layout, dx, dy)
    }
    ScreenshotListCommand::BeginRename => ui.begin_rename(&mut services.text_input),
    ScreenshotListCommand::CancelRename => ui.cancel_rename(&mut services.text_input),
    ScreenshotListCommand::CommitRename { old_name, new_name } => {
      let directory = services.storage.screenshot_cache_dir_path();
      if let Err(error) =
        ui.commit_rename(&directory, &old_name, &new_name, &mut services.text_input)
      {
        ui.rename_io_failed();
        services.log.error_operation_failed(
          LogSource::Ui,
          "rename_screenshot",
          format!("{old_name}->{new_name}"),
          error.to_string(),
        );
      }
    }
    ScreenshotListCommand::CopyScreenshot { frame, rect, rich } => {
      let copied = if rich {
        super::copy_screenshot_rich_text(services, &frame, rect)
      } else {
        super::copy_screenshot_text(services, &frame, rect)
      };
      services.screenshot.report_operation(Some(copied), None);
    }
    ScreenshotListCommand::SaveScreenshot {
      source_path,
      frame,
      rect,
      copy,
    } => {
      let copied = copy.then(|| super::copy_screenshot_text(services, &frame, rect));
      let task_id = super::submit_screenshot_png(services, frame, rect);
      services
        .screenshot
        .register_source_export(task_id, source_path);
      services.screenshot.report_operation(copied, Some(task_id));
    }
    ScreenshotListCommand::ExportRecording { .. } => {}
    ScreenshotListCommand::RequestDelete { path } => {
      if services.screenshot.is_source_exporting(&path) {
        show_delete_blocked_popup(services, "screenshot_list", "screenshot_list.popup.no_del");
      } else {
        ui.begin_delete(path);
      }
    }
    ScreenshotListCommand::ConfirmDelete { path } => {
      if services.screenshot.is_source_exporting(&path) {
        ui.cancel_delete();
        show_delete_blocked_popup(services, "screenshot_list", "screenshot_list.popup.no_del");
      } else if let Err(error) = ui.finish_delete(&path) {
        ui.cancel_delete();
        services.log.error_operation_failed(
          LogSource::Storage,
          "delete_screenshot",
          path.display().to_string(),
          error.to_string(),
        );
      }
    }
  }
}

pub(super) fn apply_recording_list_command(
  command: RecordingListCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  let ui = settings_ui.screenshot_recording_mut().recording_list_mut();
  match command {
    RecordingListCommand::Back => {
      ui.blur_search(&mut services.text_input);
      clear_exiting_pool(ui.objects_mut(), services);
      world.state.pop_ui_node();
    }
    RecordingListCommand::FocusSearch => ui.focus_search(&mut services.text_input),
    RecordingListCommand::BlurSearch => ui.blur_search(&mut services.text_input),
    RecordingListCommand::SelectList(dy) => {
      ui.select_list(&services.scroll_box, &services.layout, dy)
    }
    RecordingListCommand::ScrollList(dy) => {
      ui.scroll_list(&services.scroll_box, &services.layout, dy);
    }
    RecordingListCommand::ScrollInfo { dx, dy } => {
      ui.scroll_info(&services.scroll_box, &services.layout, dx, dy)
    }
    RecordingListCommand::BeginRename => ui.begin_rename(&mut services.text_input),
    RecordingListCommand::CancelRename => ui.cancel_rename(&mut services.text_input),
    RecordingListCommand::CommitRename { old_name, new_name } => {
      let directory = services.storage.recording_cache_dir_path();
      if let Err(error) =
        ui.commit_rename(&directory, &old_name, &new_name, &mut services.text_input)
      {
        ui.rename_io_failed();
        services.log.error_operation_failed(
          LogSource::Ui,
          "rename_recording",
          format!("{old_name}->{new_name}"),
          error.to_string(),
        );
      }
    }
    RecordingListCommand::ExportRecording { path } => {
      services.ffmpeg.refresh_if_missing();
      let profile = services
        .storage
        .read_recording_profile_or_default(&mut services.log);
      let fonts = services
        .storage
        .read_screenshot_profile_or_default(&mut services.log)
        .fonts;
      if let Err(error) = services.video.submit_recording_export(
        &services.async_runtime,
        &services.storage,
        &services.ffmpeg,
        path.clone(),
        fonts,
        profile,
      ) {
        services.log.error_operation_failed(
          LogSource::Storage,
          format!("submit_recording_export:{}", error.stage),
          path.display().to_string(),
          error.message,
        );
      }
    }
    RecordingListCommand::RequestDelete { path } => {
      if services.video.is_source_exporting(&path) {
        show_delete_blocked_popup(
          services,
          "recording_list",
          "recording_list_list.popup.no_del",
        );
      } else {
        ui.begin_delete(path);
      }
    }
    RecordingListCommand::ConfirmDelete { path } => {
      if services.video.is_source_exporting(&path) {
        ui.cancel_delete();
        show_delete_blocked_popup(
          services,
          "recording_list",
          "recording_list_list.popup.no_del",
        );
      } else if let Err(error) = ui.finish_delete(&path) {
        ui.cancel_delete();
        services.log.error_operation_failed(
          LogSource::Storage,
          "delete_recording",
          path.display().to_string(),
          error.to_string(),
        );
      }
    }
    RecordingListCommand::CopyScreenshot { .. } | RecordingListCommand::SaveScreenshot { .. } => {}
  }
}

fn show_delete_blocked_popup(services: &mut EngineServices, namespace: &str, key: &str) {
  services.popup.show(PopupRequest {
    text: services.i18n.get_runtime_text(namespace, key),
    color: TextColor::Rgb {
      r: 255,
      g: 76,
      b: 76,
    },
    duration: Duration::from_secs(2),
    dismiss_on: Vec::new(),
    replaceable: true,
    persistent: false,
  });
}

pub(super) fn apply_recording_settings_command(
  command: RecordingSettingsCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  let ui = settings_ui
    .screenshot_recording_mut()
    .recording_settings_mut();
  match command {
    RecordingSettingsCommand::Changed(profile) => {
      let _ = services
        .storage
        .write_recording_profile(&profile, &mut services.log);
    }
    RecordingSettingsCommand::ChangedFonts(fonts) => {
      let mut profile = services
        .storage
        .read_screenshot_profile_or_default(&mut services.log);
      profile.fonts = fonts;
      let _ = services
        .storage
        .write_screenshot_profile(&profile, &mut services.log);
    }
    RecordingSettingsCommand::ExportFontPreview(fonts) => {
      services.screenshot.request_font_preview(fonts);
    }
    RecordingSettingsCommand::OpenFonts => ui.open_fonts(),
    RecordingSettingsCommand::StartAddFont => ui.start_add_font(&mut services.text_input),
    RecordingSettingsCommand::StartModifyFont => ui.start_modify_font(&mut services.text_input),
    RecordingSettingsCommand::FinishFontEdit(value) => {
      ui.finish_font_edit(&mut services.text_input, value)
    }
    RecordingSettingsCommand::CancelFontEdit => ui.cancel_font_edit(&mut services.text_input),
    RecordingSettingsCommand::ScrollFonts(dy) => {
      ui.scroll_fonts(&services.scroll_box, &services.layout, dy)
    }
    RecordingSettingsCommand::Back => {
      clear_exiting_pool(ui.objects_mut(), services);
      world.state.pop_ui_node();
    }
  }
}

pub(super) fn apply_screenshot_settings_command(
  command: ScreenshotSettingsCommand,
  settings_ui: &mut SettingsUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    ScreenshotSettingsCommand::Changed(profile) => {
      let _ = services
        .storage
        .write_screenshot_profile(&profile, &mut services.log);
    }
    ScreenshotSettingsCommand::ExportFontPreview(fonts) => {
      services.screenshot.request_font_preview(fonts);
    }
    ScreenshotSettingsCommand::OpenFonts => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .open_fonts();
    }
    ScreenshotSettingsCommand::StartAddFont => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .start_add_font(&mut services.text_input);
    }
    ScreenshotSettingsCommand::StartModifyFont => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .start_modify_font(&mut services.text_input);
    }
    ScreenshotSettingsCommand::FinishFontEdit(value) => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .finish_font_edit(&mut services.text_input, value);
    }
    ScreenshotSettingsCommand::CancelFontEdit => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .cancel_font_edit(&mut services.text_input);
    }
    ScreenshotSettingsCommand::ScrollFonts(dy) => {
      settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .scroll_fonts(&services.scroll_box, &services.layout, dy);
    }
    ScreenshotSettingsCommand::Back => {
      let ui = settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut();
      clear_exiting_pool(ui.objects_mut(), services);
      world.state.pop_ui_node();
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
    ScreensaverListCommand::SetEnabled {
      package_id,
      enabled,
    } => {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      let next_order = profile
        .screensavers
        .values()
        .filter_map(|state| state.playlist_enabled.then_some(state.order).flatten())
        .max()
        .map_or(0, |order| order.saturating_add(1));
      let defaults = &profile.defaults;
      let state = profile
        .screensavers
        .entry(package_id.storage_key())
        .or_insert(crate::host_engine::services::ScreensaverPackageState {
          enabled: defaults.enabled,
          debug: defaults.debug,
          playlist_enabled: false,
          order: None,
        });
      state.playlist_enabled = enabled;
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
        let state = profile.screensavers.entry(id.storage_key()).or_insert(
          crate::host_engine::services::ScreensaverPackageState {
            enabled: defaults.enabled,
            debug: defaults.debug,
            playlist_enabled: false,
            order: None,
          },
        );
        state.playlist_enabled = true;
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
    SecuritySettingsCommand::ResetTerminal => {
      let success = services
        .storage
        .reset_terminal_profile(&mut services.log)
        .is_ok();
      show_security_reset_popup(services, success, true);
    }
    SecuritySettingsCommand::ResetStatus
    | SecuritySettingsCommand::ResetDebug
    | SecuritySettingsCommand::ResetSafeMode => {
      let mut profile = services
        .storage
        .read_package_state_or_default(&mut services.log);
      for entry in services.package.mod_games() {
        let package_key = entry.id.storage_key();
        let mut initial = crate::host_engine::services::GamePackageState::default();
        initial.enabled = profile.defaults.enabled;
        initial.debug = profile.defaults.debug;
        initial.safe_mode =
          profile.defaults.safe_mode == crate::host_engine::services::SafeModeDefault::On;
        let state = profile.games.entry(package_key.clone()).or_insert(initial);
        match command {
          SecuritySettingsCommand::ResetStatus => state.enabled = false,
          SecuritySettingsCommand::ResetDebug => state.debug = false,
          SecuritySettingsCommand::ResetSafeMode => {
            state.safe_mode = true;
            world.temporary_safe_mode_disabled.remove(&entry.id);
          }
          _ => unreachable!(),
        }
      }
      for entry in services.package.mod_screensavers() {
        let mut initial = crate::host_engine::services::ScreensaverPackageState::default();
        initial.enabled = profile.defaults.enabled;
        initial.debug = profile.defaults.debug;
        let state = profile
          .screensavers
          .entry(entry.id.storage_key())
          .or_insert(initial);
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
      show_security_reset_popup(services, success, false);
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
    show_security_reset_popup(services, false, false);
  }
}

fn show_security_reset_popup(services: &mut EngineServices, success: bool, terminal: bool) {
  let mut text = services.i18n.get_runtime_text(
    "security_settings",
    if success {
      "security_settings.reset.success"
    } else {
      "security_settings.reset.fail"
    },
  );
  if success && terminal {
    text.push_str(&services.i18n.get_runtime_text(
      "security_settings",
      "security_settings.reset.terminal.success",
    ));
  }
  services.popup.show(PopupRequest {
    text,
    color: if success {
      TextColor::Rgb {
        r: 95,
        g: 215,
        b: 105,
      }
    } else {
      TextColor::Rgb {
        r: 255,
        g: 76,
        b: 76,
      }
    },
    duration: Duration::from_secs(3),
    dismiss_on: vec![PopupDismissEvent::UiInput],
    replaceable: true,
    persistent: false,
  });
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
        services.log.error_operation_failed(
          LogSource::Storage,
          "clear_storage",
          format!("{target:?}"),
          error.to_string(),
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
        services.log.warn_operation_failed(
          LogSource::Ui,
          "write_clipboard",
          "storage_management",
          "clipboard backend rejected text",
        );
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
        if let Some(package_id) = game_package_ui.selected_package_id() {
          world.temporary_safe_mode_disabled.remove(&package_id);
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
          .entry(entry.id.storage_key())
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
        show_security_reset_popup(services, false, false);
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
      if let Some(package_id) = game_package_ui.selected_package_id() {
        world.temporary_safe_mode_disabled.insert(package_id);
      }
      game_package_ui.disable_selected_safe_mode_temporary();
    }
    SafeModeWarningCommand::DisablePermanent => {
      if let Some(package_id) = game_package_ui.selected_package_id() {
        world.temporary_safe_mode_disabled.remove(&package_id);
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
    LanguageSelectCommand::ConfirmAndApply(code) => {
      language_loading.pending_language = None;
      let enter_terminal_check_after_finish = !services
        .storage
        .is_terminal_profile_complete(&mut services.log);
      world.state.pop_ui_node();
      start_language_loading(
        &code,
        enter_terminal_check_after_finish,
        language_loading,
        language_loading_ui,
        services,
        world,
      );
      reset_language_select_ui(language_select_ui, services);
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
    LanguageSelectCommand::Exit => {
      language_loading.pending_language = None;
      world.state.request_shutdown();
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
        services.log.warn_operation_failed(
          LogSource::Storage,
          "update_profile",
          "terminal",
          e.to_string(),
        );
      }
      sync_terminal_capabilities_from_profile(services);
      world.state.pop_ui_node();
    }
    TerminalCheckCommand::Exit => {
      world.state.request_shutdown();
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
  *ui = SettingsUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
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
  *ui = InputDemoUi::init(&services.hit_area, &services.progress_bar);
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
        services.log.info_message(
          LogSource::Storage,
          HostLogMessage::new(
            "log_info.export.data_finished",
            "Data export {id} finished: {path}",
          )
          .param("id", task_id.0.to_string())
          .param("path", path.display().to_string()),
        );
        finish_export_loading(export_loading, world);
      }
      crate::host_engine::services::ExportAsyncEvent::Failed { task_id, error }
        if *task_id == active_task =>
      {
        services.log.error_message(
          LogSource::Storage,
          HostLogMessage::new(
            "log_info.export.data_failed",
            "Data export {id} failed: {error}",
          )
          .param("id", task_id.0.to_string())
          .param("error", error),
        );
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
