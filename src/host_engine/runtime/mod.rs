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
  ActionMapEntry, BorderStyle, DrawTextParams, EngineServices, EngineTask, HostAreaKind, ImPolicy,
  Key, LogSource, PackageEvent, ScreenshotService, ScreenshotTask, TaskId, TextColor,
  translate_action_map,
};
use crate::host_engine::ui::{
  ClearWarningCommand, ClearWarningTarget, ClearWarningUi, DisplaySettingsCommand,
  DisplaySettingsUi, ExportFormat, ExportLoadingUi, ExportSettingsCommand, ExportSettingsUi,
  ExportType, GameListCommand, GameListUi, GamePackageCommand, GamePackageUi, HomeUi,
  HomeUiCommand, InputDemoCommand, InputDemoUi, LanguageLoadingUi, LanguageSelectCommand,
  LanguageSelectUi, ModsCommand, ModsUi, SafeModeWarningCommand, SafeModeWarningUi,
  ScreensaverPackageCommand, ScreensaverPackageUi, ScreenshotCaptureCommand, ScreenshotCaptureUi,
  SecurityDetailsCommand, SecurityDetailsUi, SecuritySettingsCommand, SecuritySettingsUi,
  SettingsUi, SettingsUiCommand, StorageManagementClearCommand, StorageManagementClearUi,
  StorageManagementCommand, StorageManagementExportCommand, StorageManagementExportUi,
  StorageManagementUi, StorageManagementViewCommand, StorageManagementViewUi, TerminalCheckCommand,
  TerminalCheckLayout, TerminalCheckUi, WindowSizeWarningCommand, WindowSizeWarningUi,
};
use std::time::Duration;

const SCREENSHOT_DOUBLE_F1_WINDOW: Duration = Duration::from_millis(300);

#[derive(Default)]
pub(super) struct LanguageLoadingRuntime {
  active: bool,
  pending_language: Option<String>,
  enter_terminal_check_after_finish: bool,
}

#[derive(Default)]
pub(super) struct ExportLoadingRuntime {
  active: bool,
  task_id: Option<TaskId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenshotModeToastKind {
  Enter,
  Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScreenshotModeToast {
  kind: ScreenshotModeToastKind,
  elapsed: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingScreenshotHotkey {
  elapsed: Duration,
}

impl PendingScreenshotHotkey {
  fn new() -> Self {
    Self {
      elapsed: Duration::ZERO,
    }
  }

  fn update(&mut self, dt: Duration) -> bool {
    self.elapsed = self.elapsed.saturating_add(dt);
    self.elapsed < SCREENSHOT_DOUBLE_F1_WINDOW
  }
}

impl ScreenshotModeToast {
  fn new(kind: ScreenshotModeToastKind) -> Self {
    Self {
      kind,
      elapsed: Duration::ZERO,
    }
  }

  fn key(self) -> &'static str {
    match self.kind {
      ScreenshotModeToastKind::Enter => "screenshot.mode.enter",
      ScreenshotModeToastKind::Exit => "screenshot.mode.exit",
    }
  }

  fn update(&mut self, dt: Duration) -> bool {
    self.elapsed = self.elapsed.saturating_add(dt);
    self.elapsed < Duration::from_secs(3)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InputModeScope {
  overlay: Option<OverlayKind>,
  ui_path: Vec<UiNodeKind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InputModePolicy {
  action_map_dispatch: bool,
  raw_key_capture: bool,
}

struct SecurityUis {
  settings: SecuritySettingsUi,
  details: SecurityDetailsUi,
}

impl InputModePolicy {
  fn normal() -> Self {
    Self {
      action_map_dispatch: true,
      raw_key_capture: false,
    }
  }

  fn safe_mode_warning() -> Self {
    Self {
      action_map_dispatch: false,
      raw_key_capture: true,
    }
  }

  fn raw_overlay() -> Self {
    Self {
      action_map_dispatch: false,
      raw_key_capture: true,
    }
  }
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
  services.package.start_watcher(&mut services.async_runtime);
  load_host_key_action_map(services);

  let mut scheduler = FrameScheduler::new(60);

  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());
  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());

  let registry = services.i18n.language_registry().to_vec();
  let mut home_ui = HomeUi::init(&services.hit_area);
  let mut settings_ui = SettingsUi::init(&services.hit_area);
  let mut display_settings_ui = DisplaySettingsUi::init(&services.hit_area);
  let mut security_uis = SecurityUis {
    settings: SecuritySettingsUi::init(&services.hit_area),
    details: SecurityDetailsUi::init(
      &services.hit_area,
      &services.scroll_box,
      &services.markdown,
      &services.storage,
      &services.i18n,
    ),
  };
  let mut storage_management_ui = StorageManagementUi::init(&services.hit_area);
  let mut storage_management_clear_ui = StorageManagementClearUi::init(&services.hit_area);
  let mut storage_management_export_ui = StorageManagementExportUi::init(&services.hit_area);
  let mut storage_management_view_ui =
    StorageManagementViewUi::init(&services.hit_area, &services.table);
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
  let mut game_list_ui = GameListUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
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
  let mut export_loading_ui = ExportLoadingUi::init(&services.progress_bar, &services.time);
  let mut safe_mode_warning_ui = SafeModeWarningUi::init(&services.hit_area);
  let mut clear_warning_ui = ClearWarningUi::init(&services.hit_area);
  let mut export_settings_ui = ExportSettingsUi::init(&services.hit_area, &services.text_input);
  let mut screenshot_capture_ui = ScreenshotCaptureUi::init();
  let mut screenshot_mode_toast: Option<ScreenshotModeToast> = None;
  let mut pending_screenshot_hotkey: Option<PendingScreenshotHotkey> = None;
  let mut language_loading = LanguageLoadingRuntime::default();
  let mut export_loading = ExportLoadingRuntime::default();
  let mut input_mode_scope = None;

  if services
    .storage
    .read_language_code(&mut services.log)
    .is_none()
    && language_select_ui.is_some()
  {
    world.state.enter_ui_node(UiNodeState::language_select());
  } else if !services
    .storage
    .is_terminal_profile_complete(&mut services.log)
  {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }

  while !world.is_stopped() {
    let _frame = scheduler.begin_frame();

    world.clock.tick();
    if let Some(toast) = &mut screenshot_mode_toast {
      if !toast.update(world.clock.delta_time()) {
        screenshot_mode_toast = None;
      }
    }
    services
      .time
      .update(&mut services.runtime_objects, world.clock.delta_time());

    services
      .engine_events
      .extend(services.async_runtime.poll_events());
    let engine_events = drain_engine_events(services);

    services.input.begin_frame();
    services.input.poll();
    apply_language_loading_package_events(
      &engine_events.package,
      &mut language_loading,
      &mut language_loading_ui,
      services,
      world,
    );
    apply_export_loading_events(
      &engine_events.export,
      &mut export_loading,
      &mut export_loading_ui,
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
    restore_input_modes_if_scope_changed(services, world, &mut input_mode_scope);
    deactivate_hidden_pools(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      &mut display_settings_ui,
      &mut security_uis,
      &mut storage_management_ui,
      &mut storage_management_clear_ui,
      &mut storage_management_export_ui,
      &mut storage_management_view_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_list_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut window_size_ui,
      &mut safe_mode_warning_ui,
      &mut clear_warning_ui,
      &mut export_settings_ui,
      &mut screenshot_capture_ui,
      &mut export_loading_ui,
    );

    route_frame_input(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      &mut display_settings_ui,
      &mut security_uis,
      &mut storage_management_ui,
      &mut storage_management_clear_ui,
      &mut storage_management_export_ui,
      &mut storage_management_view_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_list_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut window_size_ui,
      &mut safe_mode_warning_ui,
      &mut clear_warning_ui,
      &mut export_settings_ui,
      &mut screenshot_capture_ui,
      &mut export_loading_ui,
      &mut language_loading_ui,
      &mut language_loading,
      &mut export_loading,
      &mut screenshot_mode_toast,
      &mut pending_screenshot_hotkey,
    );
    update_pending_screenshot_hotkey(
      services,
      world,
      &mut screenshot_capture_ui,
      &mut screenshot_mode_toast,
      &mut pending_screenshot_hotkey,
    );
    let dismiss_screenshot_toast = screenshot_capture_ui.take_mode_toast_dismiss_requested();
    if dismiss_screenshot_toast
      && world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture)
    {
      screenshot_mode_toast = None;
    }
    sync_input_method_policy(services);
    restore_input_modes_if_scope_changed(services, world, &mut input_mode_scope);

    if world.is_stopped() {
      break;
    }

    route_update(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      &mut display_settings_ui,
      &mut security_uis,
      &mut storage_management_ui,
      &mut storage_management_clear_ui,
      &mut storage_management_export_ui,
      &mut storage_management_view_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_list_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut safe_mode_warning_ui,
      &mut clear_warning_ui,
      &mut export_settings_ui,
      &mut screenshot_capture_ui,
      &mut export_loading_ui,
      &mut language_loading_ui,
      &mut language_loading,
      &mut export_loading,
    );
    sync_input_method_policy(services);
    services.input_method.update(world.clock.delta_time());
    restore_input_modes_if_scope_changed(services, world, &mut input_mode_scope);

    if world.is_stopped() {
      break;
    }

    let input_cursor = route_render(
      services,
      world,
      &mut home_ui,
      &mut settings_ui,
      &mut display_settings_ui,
      &mut security_uis,
      &mut storage_management_ui,
      &mut storage_management_clear_ui,
      &mut storage_management_export_ui,
      &mut storage_management_view_ui,
      language_select_ui.as_mut(),
      &mut terminal_check_ui,
      &mut mods_ui,
      &mut game_list_ui,
      &mut game_package_ui,
      &mut screensaver_package_ui,
      &mut input_demo_ui,
      &mut window_size_ui,
      &mut safe_mode_warning_ui,
      &mut clear_warning_ui,
      &mut export_settings_ui,
      &mut screenshot_capture_ui,
      &mut export_loading_ui,
      &mut language_loading_ui,
    );
    draw_screenshot_mode_toast(services, screenshot_mode_toast);
    let text_force_redraw = services.canvas.take_render_requested();
    let composed = services.compositor.compose(&services.canvas);
    if let Err(error) = services.presenter.present(
      &composed,
      &mut services.terminal,
      text_force_redraw,
      input_cursor,
    ) {
      services.log.error(
        LogSource::Render,
        format!("Frame presentation failed: {error}"),
      );
    }
    if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
      services.screenshot.remember_presented_frame(composed);
    }

    scheduler.wait_for_next_frame();
  }

  ExitState::new()
}

fn sync_input_method_policy(services: &mut EngineServices) {
  let policy = if !services.input.is_focused() || services.text_input.is_active() {
    ImPolicy::Free
  } else {
    ImPolicy::ForceAscii
  };
  let _ = services.input_method.set_policy(policy);
}

fn draw_screenshot_mode_toast(services: &mut EngineServices, toast: Option<ScreenshotModeToast>) {
  let Some(toast) = toast else {
    return;
  };
  let size = services.layout.physical_size();
  if size.width < 8 || size.height < 3 {
    return;
  }
  let text = services.i18n.get_runtime_text("screenshot", toast.key());
  let width = services
    .layout
    .get_text_width(&text, None)
    .saturating_add(4)
    .min(size.width);
  let x = size.width.saturating_sub(width) / 2;
  let y = 1.min(size.height.saturating_sub(3));
  let color = match toast.kind {
    ScreenshotModeToastKind::Enter => TextColor::Rgb {
      r: 95,
      g: 215,
      b: 105,
    },
    ScreenshotModeToastKind::Exit => TextColor::Rgb {
      r: 255,
      g: 76,
      b: 76,
    },
  };
  services.render.draw_top_border_rect(
    &mut services.canvas,
    x,
    y,
    width,
    3,
    &BorderStyle::Circle,
    Some(color.clone()),
    Some(TextColor::Rgb { r: 0, g: 0, b: 0 }),
    Some(TextColor::Rgb { r: 0, g: 0, b: 0 }),
    None,
  );
  services.render.draw_top_text(
    &mut services.canvas,
    &DrawTextParams {
      x: x.saturating_add(2),
      y: y.saturating_add(1),
      text,
      fg: Some(color),
      bg: Some(TextColor::Rgb { r: 0, g: 0, b: 0 }),
      max_width: Some(width.saturating_sub(4)),
      ..Default::default()
    },
  );
}

fn restore_input_modes_if_scope_changed(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  previous: &mut Option<InputModeScope>,
) {
  let current = InputModeScope {
    overlay: world.state.current_overlay_kind(),
    ui_path: world.state.current_ui_path_kinds(),
  };
  if previous.as_ref() == Some(&current) {
    return;
  }
  apply_input_mode_policy(services, input_mode_policy(world));
  *previous = Some(current);
}

fn input_mode_policy(world: &RuntimeWorld) -> InputModePolicy {
  match world.state.current_overlay_kind() {
    Some(OverlayKind::SafeModeWarning | OverlayKind::ClearWarning) => {
      InputModePolicy::safe_mode_warning()
    }
    Some(OverlayKind::ScreenshotCapture) => InputModePolicy::raw_overlay(),
    _ => InputModePolicy::normal(),
  }
}

fn apply_input_mode_policy(services: &mut EngineServices, policy: InputModePolicy) {
  if policy.action_map_dispatch {
    let _ = services.input.enable_action_map_dispatch();
  } else {
    let _ = services.input.disable_action_map_dispatch();
  }

  if policy.raw_key_capture {
    let _ = services.input.enable_raw_key_capture();
  } else {
    let _ = services.input.disable_raw_key_capture();
  }
}

fn route_frame_input(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  display_settings_ui: &mut DisplaySettingsUi,
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
  screenshot_capture_ui: &mut ScreenshotCaptureUi,
  _export_loading_ui: &mut ExportLoadingUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  _export_loading: &mut ExportLoadingRuntime,
  screenshot_mode_toast: &mut Option<ScreenshotModeToast>,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
) {
  if handle_screenshot_hotkey(
    services,
    world,
    screenshot_capture_ui,
    screenshot_mode_toast,
    pending_screenshot_hotkey,
  ) {
    return;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::WindowSizeWarning) {
    load_window_size_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  } else if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
    load_safe_mode_warning_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  } else if world.state.current_overlay_kind() == Some(OverlayKind::ClearWarning) {
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  } else if world.state.current_overlay_kind() == Some(OverlayKind::ExportSettings) {
    if services.text_input.is_active() {
      // 输入中不 dispatch action——避免 Enter 被当作 action 而打断 IME 组字
      services
        .input
        .dispatch_system_action_events(&mut services.log);
      while let Some(event) = services.input.next_action_event() {
        let _ = handle_host_key_action(event.action.as_str(), event.state, world);
      }
      route_export_settings_text_input_events(
        services,
        world,
        export_settings_ui,
        _export_loading_ui,
        _export_loading,
      );
    } else {
      load_export_settings_action_map(services);
      services.input.dispatch_action_events(&mut services.log);
      route_export_settings_overlay_events(
        services,
        world,
        export_settings_ui,
        _export_loading_ui,
        _export_loading,
      );
    }
  } else if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    if let Some(command) = screenshot_capture_ui.handle_input(
      &mut services.input,
      &services.layout,
      &services.i18n,
      &services.storage,
      &mut services.log,
    ) {
      apply_screenshot_capture_command(command, services, world, screenshot_capture_ui);
      if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
        *screenshot_mode_toast = Some(ScreenshotModeToast::new(ScreenshotModeToastKind::Exit));
      }
    }
  } else if matches!(
    world.state.current_overlay_kind(),
    Some(OverlayKind::LanguageLoading | OverlayKind::ExportLoading)
  ) {
    services
      .input
      .dispatch_system_action_events(&mut services.log);
    while let Some(event) = services.input.next_action_event() {
      let _ = handle_host_key_action(event.action.as_str(), event.state, world);
    }
    services.input.clear();
    let _ = services.input.drain_system_events();
  } else if services.text_input.is_active() {
    services
      .input
      .dispatch_system_action_events(&mut services.log);
    while let Some(event) = services.input.next_action_event() {
      let _ = handle_host_key_action(event.action.as_str(), event.state, world);
    }
    route_text_input_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
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
  } else {
    load_current_action_map(services, world);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
      services,
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  }
}

fn handle_screenshot_hotkey(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  screenshot_mode_toast: &mut Option<ScreenshotModeToast>,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
) -> bool {
  if !services.input.was_pressed(Key::Fn(1)) {
    return false;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    if screenshot_ui.is_guide_visible() {
      return false;
    }
    let command = if screenshot_ui.can_full_save_by_double_f1() {
      ScreenshotCaptureCommand::FullFrameSave
    } else {
      ScreenshotCaptureCommand::Exit
    };
    apply_screenshot_capture_command(command, services, world, screenshot_ui);
    if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
      *screenshot_mode_toast = Some(ScreenshotModeToast::new(ScreenshotModeToastKind::Exit));
    }
    services.input.clear();
    return true;
  }

  if pending_screenshot_hotkey.take().is_some() {
    save_last_frame_screenshot(services);
    services.input.clear();
    return true;
  }

  *pending_screenshot_hotkey = Some(PendingScreenshotHotkey::new());
  services.input.clear();
  true
}

fn update_pending_screenshot_hotkey(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  screenshot_mode_toast: &mut Option<ScreenshotModeToast>,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
) {
  let Some(pending) = pending_screenshot_hotkey else {
    return;
  };
  if pending.update(world.clock.delta_time()) {
    return;
  }
  *pending_screenshot_hotkey = None;
  start_screenshot_capture(services, world, screenshot_ui, screenshot_mode_toast);
}

fn start_screenshot_capture(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  screenshot_mode_toast: &mut Option<ScreenshotModeToast>,
) {
  let Some(frame) = services.screenshot.capture_last_frame() else {
    services.log.warn(
      LogSource::Render,
      "Screenshot requested before first frame was presented",
    );
    services.input.clear();
    return;
  };
  let show_guide = !services
    .storage
    .read_screenshot_profile_or_default(&mut services.log)
    .guide_seen;
  screenshot_ui.start(frame, show_guide);
  world.state.push_screenshot_capture_overlay();
  *screenshot_mode_toast = Some(ScreenshotModeToast::new(ScreenshotModeToastKind::Enter));
  services.input.clear();
}

fn save_last_frame_screenshot(services: &mut EngineServices) {
  let Some(frame) = services.screenshot.capture_last_frame() else {
    services.log.warn(
      LogSource::Render,
      "Screenshot requested before first frame was presented",
    );
    return;
  };
  let rect = crate::host_engine::services::ScreenshotRect {
    x: 0,
    y: 0,
    width: frame.width(),
    height: frame.height(),
  };
  submit_screenshot_png(services, frame, rect);
}

fn apply_screenshot_capture_command(
  command: ScreenshotCaptureCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
) {
  match command {
    ScreenshotCaptureCommand::Exit => {
      let _ = world
        .state
        .remove_overlay_kind(OverlayKind::ScreenshotCapture);
    }
    ScreenshotCaptureCommand::Copy => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        copy_screenshot_text(services, &frame, rect);
        let _ =
          services
            .screenshot
            .write_json(&services.storage, &frame, rect, None, &mut services.log);
        let _ = world
          .state
          .remove_overlay_kind(OverlayKind::ScreenshotCapture);
      }
    }
    ScreenshotCaptureCommand::CopyRichText => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        copy_screenshot_rich_text(services, &frame, rect);
        let _ =
          services
            .screenshot
            .write_json(&services.storage, &frame, rect, None, &mut services.log);
        let _ = world
          .state
          .remove_overlay_kind(OverlayKind::ScreenshotCapture);
      }
    }
    ScreenshotCaptureCommand::SavePng => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        submit_screenshot_png(services, frame, rect);
        let _ = world
          .state
          .remove_overlay_kind(OverlayKind::ScreenshotCapture);
      }
    }
    ScreenshotCaptureCommand::All => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        copy_screenshot_text(services, &frame, rect);
        submit_screenshot_png(services, frame, rect);
        let _ = world
          .state
          .remove_overlay_kind(OverlayKind::ScreenshotCapture);
      }
    }
    ScreenshotCaptureCommand::FullFrameSave => {
      if let Some((frame, rect)) = screenshot_ui.whole_frame() {
        submit_screenshot_png(services, frame, rect);
      }
      let _ = world
        .state
        .remove_overlay_kind(OverlayKind::ScreenshotCapture);
    }
  }
}

fn copy_screenshot_text(
  services: &mut EngineServices,
  frame: &crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) {
  let text = ScreenshotService::plain_text(frame, rect);
  if !services.clipboard.write_text(&text) {
    services.log.warn(
      LogSource::Storage,
      "Failed to copy screenshot text to clipboard",
    );
  }
}

fn copy_screenshot_rich_text(
  services: &mut EngineServices,
  frame: &crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) {
  let text = ScreenshotService::rich_text(frame, rect);
  if !services.clipboard.write_text(&text) {
    services.log.warn(
      LogSource::Storage,
      "Failed to copy screenshot rich text to clipboard",
    );
  }
}

fn submit_screenshot_png(
  services: &mut EngineServices,
  frame: crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) {
  let png_path = ScreenshotService::next_png_path(&services.storage);
  let _ = services.screenshot.write_json(
    &services.storage,
    &frame,
    rect,
    Some(&png_path),
    &mut services.log,
  );
  let _ = services
    .async_runtime
    .submit(EngineTask::Screenshot(ScreenshotTask {
      frame,
      selection: rect,
      png_path,
    }));
}
