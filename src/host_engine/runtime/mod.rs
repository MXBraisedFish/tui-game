mod action_map;
mod commands;
mod engine_events;
mod host_viewport;
mod overlay;
mod render;
mod router;

use action_map::*;
use commands::*;
use engine_events::drain_engine_events;
use overlay::*;
use render::route_render;
use router::*;

use crate::host_engine::core::state_machine::{
  HostState, MainHostState, OverlayKind, UiNodeKind, UiNodeState,
};
use crate::host_engine::core::{ExitState, FrameScheduler, RuntimeWorld, set_crash_phase};
use crate::host_engine::services::{
  EngineServices, HostAreaKind, PackageEvent, translate_action_map,
};
use crate::host_engine::ui::{
  GamePackageCommand, GamePackageUi, HomeUi, HomeUiCommand, InputDemoCommand, InputDemoUi,
  LanguageLoadingUi, LanguageSelectCommand, LanguageSelectUi, ModsCommand, ModsUi,
  ScreensaverPackageCommand, ScreensaverPackageUi, SettingsUi, SettingsUiCommand,
  TerminalCheckCommand, TerminalCheckLayout, TerminalCheckUi, WindowSizeWarningCommand,
  WindowSizeWarningUi,
};

#[derive(Default)]
pub(super) struct LanguageLoadingRuntime {
  active: bool,
  pending_language: Option<String>,
  enter_terminal_check_after_finish: bool,
}

/// 运行引擎主循环：初始化 UI 并循环处理输入、更新与渲染，直到退出。
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  services.terminal.enter(&mut services.log);

  services
    .input
    .start_key_listener(&mut services.async_runtime);
  services
    .input
    .start_system_listener(&mut services.async_runtime);

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
  let mut language_loading_ui = LanguageLoadingUi::init(&services.progress_bar, &services.time);
  let mut language_loading = LanguageLoadingRuntime::default();

  if services.storage.read_language_code().is_none() && language_select_ui.is_some() {
    world.state.enter_ui_node(UiNodeState::language_select());
  } else if !services.storage.is_terminal_profile_complete() {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }

  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();
    services
      .time
      .update(&mut services.runtime_objects, world.clock.delta_time());

    services
      .engine_events
      .extend(services.async_runtime.poll_events());
    let package_events = drain_engine_events(services);

    services.input.begin_frame();
    services.input.poll();
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

    route_frame_input(
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

fn route_frame_input(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
) {
  if world.state.current_overlay_kind() == Some(OverlayKind::WindowSizeWarning) {
    load_window_size_action_map(services);
    services.input.dispatch_action_events();
    route_input_events(
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
      window_size_ui,
      language_loading_ui,
      language_loading,
    );
  } else if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
    while services.input.next_action_event().is_some() {}
    services.input.clear();
    let _ = services.input.drain_system_events();
  } else if services.text_input.is_active() {
    route_text_input_events(
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
  } else {
    load_current_action_map(services, world);
    services.input.dispatch_action_events();
    route_input_events(
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
      window_size_ui,
      language_loading_ui,
      language_loading,
    );
  }
}
