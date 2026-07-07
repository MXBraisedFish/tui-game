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
    HomeUiCommand::StartGame => {}
    HomeUiCommand::ContinueGame => {}
    HomeUiCommand::OpenSettings => world.state.enter_ui_node(UiNodeState::settings()),
    HomeUiCommand::OpenAbout => world.state.enter_ui_node(UiNodeState::input_demo()),
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
      game_package_ui.toggle_selected_enabled(&services.storage);
    }
    GamePackageCommand::ToggleDebug => {
      game_package_ui.toggle_selected_debug(&services.storage);
    }
    GamePackageCommand::RequestToggleSafeMode => {
      if game_package_ui.selected_safe_mode().unwrap_or(true) {
        world.state.push_safe_mode_warning_overlay();
      } else {
        if let Some(mod_id) = game_package_ui.selected_mod_id() {
          world.temporary_safe_mode_disabled.remove(&mod_id);
        }
        game_package_ui.enable_selected_safe_mode(&services.storage);
      }
    }
  }
}

pub(super) fn apply_safe_mode_warning_command(
  command: SafeModeWarningCommand,
  game_package_ui: &mut GamePackageUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
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
      game_package_ui.disable_selected_safe_mode_permanent(&services.storage);
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
      screensaver_package_ui.toggle_selected_enabled(&services.storage);
    }
    ScreensaverPackageCommand::ToggleDebug => {
      screensaver_package_ui.toggle_selected_debug(&services.storage);
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

pub(super) fn apply_terminal_check_command(
  command: TerminalCheckCommand,
  terminal_check_ui: &mut TerminalCheckUi,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
) {
  match command {
    TerminalCheckCommand::Next => {
      terminal_check_ui.persist_current_step(&mut services.storage);
      sync_terminal_capabilities_from_profile(services);
      terminal_check_ui.advance_step();
    }
    TerminalCheckCommand::Done { mouse } => {
      let _ = services.storage.update_terminal_profile(|p| {
        p.mouse = Some(mouse);
      });
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
  let profile = services.storage.read_terminal_profile_or_default();
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
