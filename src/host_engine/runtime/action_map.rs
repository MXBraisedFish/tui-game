use super::*;

pub(super) fn load_current_action_map(services: &mut EngineServices, world: &RuntimeWorld) {
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
}

pub(super) fn load_window_size_action_map(services: &mut EngineServices) {
  load_action_map(services, &WindowSizeWarningUi::action_map(), "window_size");
}

fn load_home_action_map(services: &mut EngineServices) {
  load_action_map(services, &HomeUi::action_map(), "HomeUi");
}

fn load_settings_action_map(services: &mut EngineServices) {
  load_action_map(services, &SettingsUi::action_map(), "SettingsUi");
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
