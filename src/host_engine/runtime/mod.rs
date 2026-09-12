mod action_map;
mod commands;
mod engine_events;
mod host_viewport;
mod overlay;
mod render;
mod router;
mod toolbar;

use action_map::*;
use commands::*;
use engine_events::{drain_engine_events, reconcile_game_save_profile};
use overlay::*;
use render::route_render;
use router::*;
use toolbar::TopToolbarRuntime;

use crate::host_engine::core::state_machine::{
  HostState, MainHostState, OverlayKind, OverlayStackTransition, RuntimeClosingState, UiNodeKind,
  UiNodeState,
};
use crate::host_engine::core::{
  ExitState, FrameScheduler, HostFaultDomain, RuntimeWorld, set_crash_phase, with_fault_domain,
};
use crate::host_engine::services::{
  ActionKeyMap, ActionMapEntry, AutoRecordingMode, BorderStyle, DisplayLogoMode, DisplayOrderMode,
  DrawTextParams, EngineServices, EngineTask, HostAreaKind, HostLogMessage, ImPolicy,
  InputActionEvent, KeyBindingsProfile, KeyState, LogLevel, LogPrintOptions, LogSource,
  LuaActionState, LuaEnqueueError, LuaErrorStage, LuaEventBroker, LuaEventData, LuaEventRoute,
  LuaHostCommand, LuaSessionDiagnostics, LuaSessionError, LuaSessionKind, LuaSessionToken,
  LuaTaskOperation, PackageEvent, PackageListEntry, PopupDismissEvent, PopupRequest,
  RandomGeneratorId, RandomSeed, RecordingState, Rect, ScreenshotAsyncEvent,
  ScreenshotDoubleAction, ScreenshotService, ScreenshotTask, Size, SystemEvent, TaskId, TextColor,
  UiEvent, UiObjectPoolOwner, VideoAsyncEvent, VideoExportStage, translate_action_map,
};
use crate::host_engine::ui::{
  ClearWarningCommand, ClearWarningTarget, ClearWarningUi, CoverContinueCommand, CoverContinueUi,
  DisplaySettingsCommand, DisplaySettingsUi, ExitWarningCommand, ExitWarningMode, ExitWarningUi,
  ExportFormat, ExportLoadingUi, ExportSettingsCommand, ExportSettingsUi, ExportType,
  GameKeyBindingsCommand, GameKeyBindingsUi, GameListCommand, GameListUi, GamePackageCommand,
  GamePackageUi, GameWarningCommand, GameWarningUi, GlobalKeyBindingsCommand, GlobalKeyBindingsUi,
  HomeUi, HomeUiCommand, InputDemoCommand, InputDemoUi, KeyBindingsCommand, KeyBindingsUi,
  LanguageLoadingUi, LanguageSelectCommand, LanguageSelectUi, MediaListNotice, MediaRenameError,
  ModsCommand, ModsUi, RecordingListCommand, RecordingListUi, RecordingSettingsCommand,
  RecordingSettingsUi, SafeModeWarningCommand, SafeModeWarningUi, ScreensaverListCommand,
  ScreensaverListUi, ScreensaverOverlayUi, ScreensaverPackageCommand, ScreensaverPackageUi,
  ScreenshotCaptureCommand, ScreenshotCaptureUi, ScreenshotListCommand, ScreenshotListUi,
  ScreenshotRecordingCommand, ScreenshotRecordingUi, ScreenshotSettingsCommand,
  ScreenshotSettingsUi, SecurityDetailsCommand, SecurityDetailsUi, SecuritySettingsCommand,
  SecuritySettingsUi, SettingsUi, SettingsUiCommand, StorageManagementClearCommand,
  StorageManagementClearUi, StorageManagementCommand, StorageManagementExportCommand,
  StorageManagementExportUi, StorageManagementUi, StorageManagementViewCommand,
  StorageManagementViewUi, TerminalCheckCommand, TerminalCheckLayout, TerminalCheckUi,
  ToolbarCustomCommand, WindowSizeWarningCommand, WindowSizeWarningUi,
};
use std::{
  collections::HashMap,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

const SCREENSHOT_DOUBLE_ACTION_WINDOW: Duration = Duration::from_millis(300);
const HOST_KEY_CHORD_WINDOW: Duration = Duration::from_millis(100);

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
  MediaRename {
    namespace: &'static str,
    error: MediaRenameError,
  },
  Operation {
    copy_succeeded: Option<bool>,
    save: Option<ScreenshotSaveState>,
  },
  VideoExport(VideoExportToastState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenshotSaveState {
  Loading,
  Succeeded,
  Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VideoExportToastState {
  Loading,
  Succeeded,
  Failed,
  AudioFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingScreenshotHotkey {
  elapsed: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingHostHotkey {
  elapsed: Duration,
}

struct AutoRecordingRuntime {
  profile: crate::host_engine::services::RecordingProfile,
  profile_revision: u64,
  startup_mode: AutoRecordingMode,
  host_started: bool,
  manually_stopped: bool,
  restart_after_split: bool,
}

impl AutoRecordingRuntime {
  fn new(profile: crate::host_engine::services::RecordingProfile, profile_revision: u64) -> Self {
    let startup_mode = profile.auto_recording;
    Self {
      profile,
      profile_revision,
      startup_mode,
      host_started: false,
      manually_stopped: false,
      restart_after_split: false,
    }
  }

  fn should_start_host(&self, state: RecordingState) -> bool {
    self.startup_mode == AutoRecordingMode::Host
      && !self.host_started
      && !self.manually_stopped
      && state == RecordingState::Stopped
  }
}

#[derive(Clone, Copy)]
enum RecordingPopupKind {
  AutoSplit,
  Pause,
  Start,
  Stop,
  Resume,
}

impl PendingHostHotkey {
  fn new() -> Self {
    Self {
      elapsed: Duration::ZERO,
    }
  }

  fn update(&mut self, dt: Duration) -> bool {
    self.elapsed = self.elapsed.saturating_add(dt);
    self.elapsed < HOST_KEY_CHORD_WINDOW
  }
}

impl PendingScreenshotHotkey {
  fn new() -> Self {
    Self {
      elapsed: Duration::ZERO,
    }
  }

  fn update(&mut self, dt: Duration) -> bool {
    self.elapsed = self.elapsed.saturating_add(dt);
    self.elapsed < SCREENSHOT_DOUBLE_ACTION_WINDOW
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingScreenshotSave {
  progress: f32,
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

  fn screenshot_overlay() -> Self {
    Self {
      action_map_dispatch: true,
      raw_key_capture: true,
    }
  }
}

/// 运行引擎主循环：初始化 UI 并循环处理输入、更新与渲染，直到退出。
pub fn run(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  let host_key_profile = load_host_key_action_map(services);
  reconcile_game_save_profile(services);

  let mut scheduler = FrameScheduler::new(60);
  scheduler.set_target_fps(
    services
      .storage
      .display_settings_profile()
      .game_list_fps
      .target_fps(),
  );

  world.state.enter_init();
  set_crash_phase(world.state.crash_phase());
  world.state.enter_runtime();
  set_crash_phase(world.state.crash_phase());
  services.log.info_message(
    LogSource::Runtime,
    HostLogMessage::new("log_info.runtime.started", "Runtime event loop started."),
  );

  let registry = services.i18n.language_registry().to_vec();
  let mut display_profile = services.storage.display_settings_profile().clone();
  let logo_mode = if display_profile.logo_mode == DisplayLogoMode::Order {
    let mode = HomeUi::sequential_logo_mode(display_profile.logo_sequence_cursor);
    display_profile.logo_sequence_cursor = display_profile.logo_sequence_cursor.wrapping_add(1);
    let _ = services
      .storage
      .write_display_settings_profile(&display_profile, &mut services.log);
    mode
  } else {
    display_profile.logo_mode
  };
  let logo_seed = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|duration| duration.as_nanos() as u64)
    .unwrap_or(0);
  let mut home_ui = HomeUi::init(
    &services.hit_area,
    &services.animation,
    &services.random,
    logo_mode,
    logo_seed,
  );
  let mut settings_ui = SettingsUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
  let mut display_settings_ui = DisplaySettingsUi::init(
    &services.hit_area,
    &services.text_input,
    services.storage.display_settings_profile().clone(),
  );
  let mut screensaver_list_ui = ScreensaverListUi::init(
    &services.hit_area,
    &services.text_input,
    &services.scroll_box,
  );
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
  let mut input_demo_ui = InputDemoUi::init(&services.hit_area, &services.progress_bar);
  let mut window_size_ui = WindowSizeWarningUi::init(&services.hit_area);
  let mut game_warning_ui = GameWarningUi::init();
  let mut language_loading_ui = LanguageLoadingUi::init(&services.progress_bar, &services.time);
  let mut export_loading_ui = ExportLoadingUi::init(&services.progress_bar, &services.time);
  let mut safe_mode_warning_ui = SafeModeWarningUi::init(&services.hit_area);
  let mut clear_warning_ui = ClearWarningUi::init(&services.hit_area);
  let mut cover_continue_ui = CoverContinueUi::init(&services.hit_area);
  let mut export_settings_ui = ExportSettingsUi::init(&services.hit_area, &services.text_input);
  let mut screenshot_capture_ui = ScreenshotCaptureUi::init();
  let mut exit_warning_ui = ExitWarningUi::init(&services.progress_bar, &services.hit_area);
  screenshot_capture_ui.set_host_key_params(host_key_rich_text_params(&host_key_profile));
  let mut screensaver_overlay_ui = ScreensaverOverlayUi::init();
  let mut top_toolbar = TopToolbarRuntime::new(&services.progress_bar);
  let screensaver_random = services.random.create(
    &mut services.runtime_objects,
    RandomSeed::U64(
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64,
    ),
  );
  let mut pending_screenshot_saves = HashMap::new();
  let mut pending_screenshot_hotkey: Option<PendingScreenshotHotkey> = None;
  let mut pending_recording_hotkey: Option<PendingHostHotkey> = None;
  let recording_profile = services
    .storage
    .read_recording_profile_or_default(&mut services.log);
  let recording_profile_revision = services.storage.recording_profile_revision();
  let mut auto_recording = AutoRecordingRuntime::new(recording_profile, recording_profile_revision);
  let mut pending_screensaver_hotkey: Option<PendingHostHotkey> = None;
  let mut pending_toolbar_hotkey: Option<PendingHostHotkey> = None;
  let mut language_loading = LanguageLoadingRuntime::default();
  let mut export_loading = ExportLoadingRuntime::default();
  let mut input_mode_scope = None;
  let mut lua_event_router = LuaEventBroker::new();
  let mut exception_countdown_elapsed = Duration::ZERO;
  let mut game_warning_elapsed = Duration::ZERO;

  if language_select_ui
    .as_ref()
    .is_some_and(LanguageSelectUi::is_first_selection)
  {
    world.state.enter_ui_node(UiNodeState::language_select());
  } else if !services
    .storage
    .is_terminal_profile_complete(&mut services.log)
  {
    world.state.enter_ui_node(UiNodeState::terminal_check());
  }
  let mut logged_ui = world.state.current_ui_kind();
  let mut logged_overlay = world.state.current_overlay_kind();

  while !world.state.is_shutdown() && !world.is_stopped() {
    let frame = scheduler.begin_frame();

    world.clock.tick();
    let frame_delta = world.clock.delta_time();
    services.popup.update(frame_delta);
    services
      .time
      .update(&mut services.runtime_objects, frame_delta);
    services.animation.update(
      &mut services.runtime_objects,
      crate::host_engine::services::AnimationClock::Ui,
      frame_delta,
    );
    services.animation.update(
      &mut services.runtime_objects,
      crate::host_engine::services::AnimationClock::Game,
      frame_delta,
    );
    let time = &services.time;
    let animation = &services.animation;
    services
      .game
      .with_objects_mut(|objects| update_lua_object_pool(objects, time, animation, frame_delta));
    services
      .screensaver
      .with_objects_mut(|objects| update_lua_object_pool(objects, time, animation, frame_delta));
    top_toolbar.update(frame_delta);

    let engine_events = with_fault_domain(HostFaultDomain::Async, || {
      services
        .engine_events
        .extend(services.async_runtime.poll_events());
      synchronize_lua_event_sessions(services, &mut lua_event_router);
      drain_engine_events(services, &mut lua_event_router, frame)
    });

    with_fault_domain(HostFaultDomain::Input, || {
      services.input.begin_frame();
      services.input.poll();
    });
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
    apply_screenshot_events(
      &engine_events.screenshot,
      &mut pending_screenshot_saves,
      services,
    );
    apply_video_events(&engine_events.video, services);
    update_exit_preparation(
      services,
      world,
      &pending_screenshot_saves,
      frame_delta,
      &mut exception_countdown_elapsed,
    );
    update_game_warning(world, frame_delta, &mut game_warning_elapsed);
    if game_warning_elapsed >= Duration::from_secs(5) {
      return_from_game_warning(services, world, &mut lua_event_router);
      game_warning_elapsed = Duration::ZERO;
    }
    if world.state.is_shutdown() || world.is_stopped() {
      break;
    }

    services.input.poll_resize_events(|w, h| {
      services.layout.resize_physical(w, h);
      services.canvas.resize(w, h);
      services.canvas.request_render();
      services.presenter.request_render();
      let _ = lua_event_router.push_system(
        frame,
        LuaEventData::Resize {
          width: w,
          height: h,
        },
      );
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
      &mut screensaver_list_ui,
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
      &mut cover_continue_ui,
      &mut export_settings_ui,
      &mut screenshot_capture_ui,
      &mut export_loading_ui,
    );
    if world.state.current_ui_kind() != Some(UiNodeKind::ExitWarning)
      || world.state.current_overlay_kind().is_some()
    {
      services.hit_area.deactivate(exit_warning_ui.objects_mut());
    }

    if matches!(
      world.state.closing_state(),
      Some(RuntimeClosingState::Exception { .. })
    ) {
      route_exception_exit_input(
        services,
        world,
        &exit_warning_ui,
        &mut pending_screenshot_saves,
      );
    } else {
      with_fault_domain(HostFaultDomain::Input, || {
        route_frame_input(
          services,
          world,
          &mut home_ui,
          &mut settings_ui,
          &mut display_settings_ui,
          &mut screensaver_list_ui,
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
          &mut game_warning_ui,
          &mut safe_mode_warning_ui,
          &mut clear_warning_ui,
          &mut cover_continue_ui,
          &mut export_settings_ui,
          &mut screenshot_capture_ui,
          &mut exit_warning_ui,
          &mut export_loading_ui,
          &mut language_loading_ui,
          &mut language_loading,
          &mut export_loading,
          &mut pending_screenshot_saves,
          &mut pending_screenshot_hotkey,
          &mut pending_recording_hotkey,
          &mut pending_screensaver_hotkey,
          &mut pending_toolbar_hotkey,
          &mut lua_event_router,
          frame,
        );
      });
    }
    apply_screenshot_operation_feedback(services, &mut pending_screenshot_saves);
    apply_video_submission_feedback(services);
    apply_media_list_notices(services, &mut settings_ui);
    if let Some(fonts) = services.screenshot.take_font_preview_request() {
      submit_font_preview_png(services, fonts, &mut pending_screenshot_saves);
    }
    update_pending_host_hotkeys(
      services,
      world,
      &mut screensaver_overlay_ui,
      screensaver_random,
      &mut top_toolbar,
      &mut pending_recording_hotkey,
      &mut auto_recording,
      &mut pending_screensaver_hotkey,
      &mut pending_toolbar_hotkey,
      &mut lua_event_router,
      world.clock.delta_time(),
    );
    update_pending_screenshot_hotkey(
      services,
      world,
      &mut screenshot_capture_ui,
      &mut pending_screenshot_hotkey,
    );
    queue_lua_overlay_transitions(services, world, &mut lua_event_router, frame);
    dispatch_lua_events(services, world, &mut lua_event_router);
    let dismiss_screenshot_toast = screenshot_capture_ui.take_mode_toast_dismiss_requested();
    let dismiss_screenshot_operation_toast =
      screenshot_capture_ui.take_operation_toast_dismiss_requested();
    if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
      if dismiss_screenshot_operation_toast {
        services
          .popup
          .dismiss(PopupDismissEvent::ScreenshotOperationInput);
      }
      if dismiss_screenshot_toast {
        services
          .popup
          .dismiss(PopupDismissEvent::ScreenshotModeInput);
      }
    }
    sync_input_method_policy(services);
    restore_input_modes_if_scope_changed(services, world, &mut input_mode_scope);

    if world.state.is_shutdown() || world.is_stopped() {
      break;
    }

    if !matches!(
      world.state.closing_state(),
      Some(RuntimeClosingState::Exception { .. })
    ) {
      with_fault_domain(HostFaultDomain::Ui, || {
        route_update(
          services,
          world,
          &mut home_ui,
          &mut settings_ui,
          &mut display_settings_ui,
          &mut screensaver_list_ui,
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
      });
      with_fault_domain(HostFaultDomain::Lua, || {
        update_lua_sessions(services, world, &mut lua_event_router, frame_delta)
      });
    }
    sync_input_method_policy(services);
    services.input_method.update(world.clock.delta_time());
    restore_input_modes_if_scope_changed(services, world, &mut input_mode_scope);
    log_host_view_changes(services, world, &mut logged_ui, &mut logged_overlay);

    if world.state.is_shutdown() || world.is_stopped() {
      break;
    }

    let input_cursor =
      if let Some(ExitWarningMode::Exception { seconds_left }) = exit_warning_mode(world) {
        let video_progress = video_exit_progress(services);
        exit_warning_ui.render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.progress_bar,
          &services.hit_area,
          ExitWarningMode::Exception { seconds_left },
          screenshot_exit_progress(&pending_screenshot_saves),
          video_progress,
        );
        None
      } else {
        route_render(
          services,
          world,
          &mut home_ui,
          &mut settings_ui,
          &mut display_settings_ui,
          &mut screensaver_list_ui,
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
          &mut game_warning_ui,
          &mut safe_mode_warning_ui,
          &mut clear_warning_ui,
          &mut cover_continue_ui,
          &mut export_settings_ui,
          &mut screenshot_capture_ui,
          game_warning_seconds_left(game_warning_elapsed),
          &mut exit_warning_ui,
          &mut screensaver_overlay_ui,
          &mut export_loading_ui,
          &mut language_loading_ui,
          &mut top_toolbar,
          pending_screenshot_saves.len(),
          pending_screenshot_saves
            .iter()
            .min_by_key(|(task_id, _)| task_id.0)
            .map(|(_, save)| save.progress),
        )
      };
    draw_popup(services);
    let text_force_redraw = services.canvas.take_render_requested();
    let composed = services.compositor.compose(&services.canvas);
    let presented = with_fault_domain(HostFaultDomain::Render, || {
      services
        .presenter
        .present(
          &composed,
          &mut services.terminal,
          text_force_redraw,
          input_cursor,
        )
        .unwrap_or_else(|error| panic!("frame presentation failed: {error}"));
      true
    });
    if presented {
      if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
        services
          .screenshot
          .remember_presented_frame(composed.clone());
      }
      services.recording.capture_presented_frame(&composed);
      update_auto_recording(services, &mut auto_recording);
    }

    scheduler.set_target_fps(
      services
        .game
        .target_fps()
        .and_then(|fps| u16::try_from(fps).ok())
        .or_else(|| {
          services
            .storage
            .display_settings_profile()
            .game_list_fps
            .target_fps()
        }),
    );
    scheduler.wait_for_next_frame();
  }

  // Runtime 不再分发 Lua 完成事件。先撤销 Broker 中的任务所有权并请求取消，
  // 避免 Shutdown 等待已失去 Session 消费者的网络或其它后台任务自然完成。
  lua_event_router.synchronize_sessions(None, None);
  services
    .async_runtime
    .cancel_tasks(lua_event_router.take_orphaned_tasks());
  for audio_id in lua_event_router.take_orphaned_audio() {
    let _ = services.audio.remove_owned(audio_id);
  }

  if matches!(
    services.recording.state(),
    RecordingState::Recording | RecordingState::Paused
  ) {
    let _ = stop_recording(services);
  }

  services.log.info_message(
    LogSource::Runtime,
    HostLogMessage::new(
      "log_info.runtime.stopped",
      "Runtime event loop stopped with exit mode {mode}.",
    )
    .param("mode", "shutdown-requested"),
  );

  ExitState::new()
}

fn log_host_view_changes(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  logged_ui: &mut Option<UiNodeKind>,
  logged_overlay: &mut Option<OverlayKind>,
) {
  let current_ui = world.state.current_ui_kind();
  if current_ui != *logged_ui {
    services.log.info_message(
      LogSource::Ui,
      HostLogMessage::new(
        "log_info.ui.changed",
        "Host UI changed from {from} to {to}.",
      )
      .param("from", format_optional_state(*logged_ui))
      .param("to", format_optional_state(current_ui)),
    );
    *logged_ui = current_ui;
  }

  let current_overlay = world.state.current_overlay_kind();
  if current_overlay != *logged_overlay {
    services.log.info_message(
      LogSource::Overlay,
      HostLogMessage::new(
        "log_info.overlay.changed",
        "Overlay state changed from {from} to {to}.",
      )
      .param("from", format_optional_state(*logged_overlay))
      .param("to", format_optional_state(current_overlay)),
    );
    *logged_overlay = current_overlay;
  }
}

fn format_optional_state<T: std::fmt::Debug>(state: Option<T>) -> String {
  state.map_or_else(|| "none".to_string(), |value| format!("{value:?}"))
}

/// Runs the deliberately small exception screen after a supervised Runtime
/// fault. Ordinary UI, Lua and business updates stay stopped; terminal resize,
/// focus-aware input and the exception countdown remain alive.
pub fn run_exception(services: &mut EngineServices, world: &mut RuntimeWorld) -> ExitState {
  if !world.state.is_runtime() {
    world.state.enter_runtime();
  }
  world.state.request_exception_shutdown();
  services.input.clear();
  load_exit_warning_action_map(services, false);

  let mut ui = ExitWarningUi::init(&services.progress_bar, &services.hit_area);
  let mut scheduler = FrameScheduler::new(30);
  let mut remaining: u8 = 3;
  let mut elapsed = Duration::ZERO;

  while remaining > 0 && !world.state.is_shutdown() {
    let _frame = scheduler.begin_frame();
    world.clock.tick();
    let delta = world.clock.delta_time();
    elapsed = elapsed.saturating_add(delta);
    let elapsed_seconds = elapsed.as_secs().min(u8::MAX as u64) as u8;
    if elapsed_seconds > 0 {
      elapsed = elapsed.saturating_sub(Duration::from_secs(elapsed_seconds as u64));
      remaining = remaining.saturating_sub(elapsed_seconds);
      world
        .state
        .set_closing_state(RuntimeClosingState::Exception {
          seconds_left: remaining,
        });
    }

    services.input.begin_frame();
    services.input.poll();
    services.input.poll_resize_events(|width, height| {
      services.layout.resize_physical(width, height);
      services.canvas.resize(width, height);
      services.canvas.request_render();
      services.presenter.request_render();
    });
    services.input.dispatch_action_events(&mut services.log);
    let exit_now = std::iter::from_fn(|| services.input.next_action_event()).any(|event| {
      ui.handle_event(
        ExitWarningMode::Exception {
          seconds_left: remaining,
        },
        &UiEvent::Action(event),
      ) == Some(ExitWarningCommand::ExitNow)
    });
    if exit_now || remaining == 0 {
      world.state.enter_shutdown();
      break;
    }

    services.canvas.begin_frame(&services.layout);
    ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.progress_bar,
      &services.hit_area,
      ExitWarningMode::Exception {
        seconds_left: remaining,
      },
      None,
      None,
    );
    let force_redraw = services.canvas.take_render_requested();
    let composed = services.compositor.compose(&services.canvas);
    if let Err(error) =
      services
        .presenter
        .present(&composed, &mut services.terminal, force_redraw, None)
    {
      panic!("exception page presentation failed: {error}");
    }
    scheduler.wait_for_next_frame();
  }

  world.state.enter_shutdown();
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

fn update_exit_preparation(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  pending_images: &HashMap<TaskId, PendingScreenshotSave>,
  delta: Duration,
  exception_elapsed: &mut Duration,
) {
  if matches!(
    world.state.closing_state(),
    Some(RuntimeClosingState::Requested)
  ) {
    if matches!(
      services.recording.state(),
      RecordingState::Recording | RecordingState::Paused
    ) {
      let _ = stop_recording(services);
    }
    if services.recording.state() != RecordingState::Stopped {
      return;
    }
  }
  let exports_active = !pending_images.is_empty() || services.video.active_export_count() > 0;
  match world.state.closing_state() {
    Some(RuntimeClosingState::Requested) => {
      if !exports_active {
        finish_runtime_exit(world);
      } else {
        world
          .state
          .set_closing_state(RuntimeClosingState::ExportWarning);
        world.state.enter_ui_node(UiNodeState::exit_warning());
        load_exit_warning_action_map(services, false);
        services.input.clear();
      }
    }
    Some(RuntimeClosingState::WaitingForExports) if !exports_active => {
      finish_runtime_exit(world);
    }
    Some(RuntimeClosingState::Stopping { .. }) => {
      finish_runtime_exit(world);
    }
    Some(RuntimeClosingState::Exception { seconds_left }) => {
      *exception_elapsed = exception_elapsed.saturating_add(delta);
      let elapsed_seconds = exception_elapsed.as_secs().min(u8::MAX as u64) as u8;
      if elapsed_seconds == 0 {
        return;
      }
      *exception_elapsed =
        exception_elapsed.saturating_sub(Duration::from_secs(elapsed_seconds as u64));
      let seconds_left = seconds_left.saturating_sub(elapsed_seconds);
      if seconds_left == 0 {
        finish_runtime_exit(world);
      } else {
        world
          .state
          .set_closing_state(RuntimeClosingState::Exception { seconds_left });
      }
    }
    _ => {
      *exception_elapsed = Duration::ZERO;
    }
  }
}

fn route_exception_exit_input(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  ui: &ExitWarningUi,
  pending_images: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let Some(mode) = exit_warning_mode(world) else {
    return;
  };
  load_exit_warning_action_map(services, false);
  services.input.dispatch_action_events(&mut services.log);
  let command = std::iter::from_fn(|| services.input.next_action_event())
    .find_map(|event| ui.handle_event(mode, &UiEvent::Action(event)));
  let Some(command) = command else { return };
  apply_exit_warning_command(command, services, world, pending_images);
}

fn apply_exit_warning_command(
  command: ExitWarningCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  pending_images: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  match command {
    ExitWarningCommand::WaitForExports => {
      world
        .state
        .set_closing_state(RuntimeClosingState::WaitingForExports);
      load_exit_warning_action_map(services, true);
      services.input.clear();
    }
    ExitWarningCommand::Back => {
      world.state.cancel_shutdown_request();
      if world.state.current_ui_kind() == Some(UiNodeKind::ExitWarning) {
        let _ = world.state.pop_ui_node();
      }
      load_current_action_map(services, world);
      services.input.clear();
    }
    ExitWarningCommand::ExitNow => {
      let waiting_for_exports = matches!(
        world.state.closing_state(),
        Some(RuntimeClosingState::WaitingForExports)
      );
      let show_stopping_popup = matches!(
        world.state.closing_state(),
        Some(RuntimeClosingState::ExportWarning | RuntimeClosingState::WaitingForExports)
      );
      services
        .async_runtime
        .cancel_tasks(pending_images.keys().copied());
      services
        .async_runtime
        .cancel_tasks(services.video.active_task_ids());
      pending_images.clear();
      if show_stopping_popup {
        services.popup.clear();
        services.popup.show(PopupRequest {
          text: services
            .i18n
            .get_runtime_text("exit_warning", "exit_warning.stop.tip.export.stopping"),
          color: TextColor::Rgb {
            r: 255,
            g: 76,
            b: 76,
          },
          duration: Duration::ZERO,
          dismiss_on: Vec::new(),
          replaceable: false,
          persistent: true,
        });
        world
          .state
          .set_closing_state(RuntimeClosingState::Stopping {
            waiting_for_exports,
          });
        services.input.clear();
      } else {
        finish_runtime_exit(world);
      }
    }
  }
}

fn finish_runtime_exit(world: &mut RuntimeWorld) {
  world.state.enter_shutdown();
  set_crash_phase(world.state.crash_phase());
}

fn exit_warning_mode(world: &RuntimeWorld) -> Option<ExitWarningMode> {
  match world.state.closing_state()? {
    RuntimeClosingState::Requested => None,
    RuntimeClosingState::ExportWarning => Some(ExitWarningMode::ExportWarning),
    RuntimeClosingState::WaitingForExports => Some(ExitWarningMode::WaitingForExports),
    RuntimeClosingState::Stopping {
      waiting_for_exports,
    } => Some(if waiting_for_exports {
      ExitWarningMode::WaitingForExports
    } else {
      ExitWarningMode::ExportWarning
    }),
    RuntimeClosingState::Exception { seconds_left } => {
      Some(ExitWarningMode::Exception { seconds_left })
    }
  }
}

fn screenshot_exit_progress(
  pending: &HashMap<TaskId, PendingScreenshotSave>,
) -> Option<(usize, f32)> {
  let progress = pending
    .iter()
    .min_by_key(|(task_id, _)| task_id.0)
    .map(|(_, task)| task.progress)
    .unwrap_or(0.0);
  (!pending.is_empty()).then_some((pending.len(), progress))
}

fn video_exit_progress(services: &EngineServices) -> Option<(usize, f32)> {
  let count = services.video.active_export_count();
  (count > 0).then(|| {
    (
      count,
      services
        .video
        .first_active_progress()
        .map(|progress| progress.ratio)
        .unwrap_or(0.0),
    )
  })
}

fn apply_screenshot_events(
  events: &[ScreenshotAsyncEvent],
  pending: &mut HashMap<TaskId, PendingScreenshotSave>,
  services: &mut EngineServices,
) {
  for event in events {
    if let ScreenshotAsyncEvent::Progress {
      task_id,
      completed_rows,
      total_rows,
    } = event
    {
      if let Some(save) = pending.get_mut(task_id) {
        save.progress = if *total_rows == 0 {
          1.0
        } else {
          *completed_rows as f32 / *total_rows as f32
        };
      }
      continue;
    }
    let (task_id, save) = match event {
      ScreenshotAsyncEvent::Saved { task_id, .. } => (*task_id, ScreenshotSaveState::Succeeded),
      ScreenshotAsyncEvent::Failed { task_id, .. } => (*task_id, ScreenshotSaveState::Failed),
      ScreenshotAsyncEvent::Progress { .. } => unreachable!(),
    };
    if pending.remove(&task_id).is_none() {
      continue;
    }
    show_popup(
      services,
      ScreenshotModeToastKind::Operation {
        copy_succeeded: None,
        save: Some(save),
      },
    );
  }
}

fn apply_video_events(events: &[VideoAsyncEvent], services: &mut EngineServices) {
  for event in events {
    let state = match event {
      VideoAsyncEvent::Preparing { .. } => Some(VideoExportToastState::Loading),
      VideoAsyncEvent::Saved { .. } => Some(VideoExportToastState::Succeeded),
      VideoAsyncEvent::Failed { stage, .. } => Some(if *stage == VideoExportStage::Audio {
        VideoExportToastState::AudioFailed
      } else {
        VideoExportToastState::Failed
      }),
      VideoAsyncEvent::Progress { .. }
      | VideoAsyncEvent::Finalizing { .. }
      | VideoAsyncEvent::Encoder { .. } => None,
    };
    if let Some(state) = state {
      show_popup(services, ScreenshotModeToastKind::VideoExport(state));
    }
  }
}

fn apply_video_submission_feedback(services: &mut EngineServices) {
  let Some(submitted) = services.video.take_submission_feedback() else {
    return;
  };
  show_popup(
    services,
    ScreenshotModeToastKind::VideoExport(if submitted {
      VideoExportToastState::Loading
    } else {
      VideoExportToastState::Failed
    }),
  );
}

fn apply_screenshot_operation_feedback(
  services: &mut EngineServices,
  pending: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let Some(feedback) = services.screenshot.take_operation_feedback() else {
    return;
  };
  if let Some(task_id) = feedback.save_task {
    pending.insert(task_id, PendingScreenshotSave { progress: 0.0 });
  }
  show_popup(
    services,
    ScreenshotModeToastKind::Operation {
      copy_succeeded: feedback.copy_succeeded,
      save: feedback.save_task.map(|_| ScreenshotSaveState::Loading),
    },
  );
}

fn draw_popup(services: &mut EngineServices) {
  let Some(popup) = services.popup.view() else {
    return;
  };
  let size = services.layout.physical_size();
  if size.width < 8 || size.height < 3 {
    return;
  }
  let text = popup.text;
  let width = services
    .layout
    .get_text_width(&text, None)
    .saturating_add(4)
    .min(size.width);
  let x = size.width.saturating_sub(width) / 2;
  let y = 1.min(size.height.saturating_sub(3));
  let color = popup.color;
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

fn popup_color(kind: ScreenshotModeToastKind) -> TextColor {
  match kind {
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
    ScreenshotModeToastKind::MediaRename { .. } => TextColor::Rgb {
      r: 255,
      g: 76,
      b: 76,
    },
    ScreenshotModeToastKind::Operation {
      copy_succeeded,
      save,
    } => {
      if copy_succeeded == Some(false) || save == Some(ScreenshotSaveState::Failed) {
        TextColor::Rgb {
          r: 255,
          g: 76,
          b: 76,
        }
      } else if save == Some(ScreenshotSaveState::Loading) {
        TextColor::Rgb {
          r: 249,
          g: 232,
          b: 147,
        }
      } else {
        TextColor::Rgb {
          r: 95,
          g: 215,
          b: 105,
        }
      }
    }
    ScreenshotModeToastKind::VideoExport(state) => match state {
      VideoExportToastState::Loading => TextColor::Rgb {
        r: 249,
        g: 232,
        b: 147,
      },
      VideoExportToastState::Succeeded => TextColor::Rgb {
        r: 95,
        g: 215,
        b: 105,
      },
      VideoExportToastState::Failed | VideoExportToastState::AudioFailed => TextColor::Rgb {
        r: 255,
        g: 76,
        b: 76,
      },
    },
  }
}

fn show_popup(services: &mut EngineServices, kind: ScreenshotModeToastKind) {
  let duration = match kind {
    ScreenshotModeToastKind::Enter | ScreenshotModeToastKind::Exit => Duration::from_secs(3),
    ScreenshotModeToastKind::MediaRename { .. }
    | ScreenshotModeToastKind::Operation { .. }
    | ScreenshotModeToastKind::VideoExport(_) => Duration::from_secs(2),
  };
  let dismiss_on = match kind {
    ScreenshotModeToastKind::Enter | ScreenshotModeToastKind::Exit => {
      vec![PopupDismissEvent::ScreenshotModeInput]
    }
    ScreenshotModeToastKind::Operation { .. } => {
      vec![PopupDismissEvent::ScreenshotOperationInput]
    }
    ScreenshotModeToastKind::MediaRename { .. } => {
      vec![PopupDismissEvent::MediaRenameResolved]
    }
    ScreenshotModeToastKind::VideoExport(_) => Vec::new(),
  };
  let request = PopupRequest {
    text: screenshot_toast_text(services, kind),
    color: popup_color(kind),
    duration,
    dismiss_on,
    replaceable: true,
    persistent: false,
  };
  services.popup.show(request);
}

fn screenshot_toast_text(services: &EngineServices, kind: ScreenshotModeToastKind) -> String {
  let text = |key| services.i18n.get_runtime_text("screenshot", key);
  match kind {
    ScreenshotModeToastKind::Enter => text("screenshot.mode.enter"),
    ScreenshotModeToastKind::Exit => text("screenshot.mode.exit"),
    ScreenshotModeToastKind::MediaRename { namespace, error } => services.i18n.get_runtime_text(
      namespace,
      &format!(
        "{namespace}.modify.{}",
        match error {
          MediaRenameError::Invalid => "invalid",
          MediaRenameError::Duplicate => "duplicate",
        }
      ),
    ),
    ScreenshotModeToastKind::Operation {
      copy_succeeded,
      save,
    } => {
      let mut parts = Vec::new();
      if let Some(succeeded) = copy_succeeded {
        parts.push(text(if succeeded {
          "screenshot.mode.copy.success"
        } else {
          "screenshot.mode.copy.failed"
        }));
      }
      if let Some(state) = save {
        parts.push(text(match state {
          ScreenshotSaveState::Loading => "screenshot.mode.save_png.loading",
          ScreenshotSaveState::Succeeded => "screenshot.mode.save_png.success",
          ScreenshotSaveState::Failed => "screenshot.mode.save_png.failed",
        }));
      }
      parts.join(" / ")
    }
    ScreenshotModeToastKind::VideoExport(state) => services.i18n.get_runtime_text(
      "recording",
      match state {
        VideoExportToastState::Loading => "recording.mode.export.loading",
        VideoExportToastState::Succeeded => "recording.mode.export.success",
        VideoExportToastState::Failed => "recording.mode.export.failed",
        VideoExportToastState::AudioFailed => "recording.mode.export.audio_failed",
      },
    ),
  }
}

fn apply_media_list_notices(services: &mut EngineServices, settings_ui: &mut SettingsUi) {
  let screenshot = settings_ui
    .screenshot_recording_mut()
    .screenshot_list_mut()
    .take_notice();
  let recording = settings_ui
    .screenshot_recording_mut()
    .recording_list_mut()
    .take_notice();
  for notice in [screenshot, recording].into_iter().flatten() {
    match notice {
      MediaListNotice::RenameError { namespace, error } => {
        show_popup(
          services,
          ScreenshotModeToastKind::MediaRename { namespace, error },
        );
      }
      MediaListNotice::ClearRenameError => {
        services
          .popup
          .dismiss(PopupDismissEvent::MediaRenameResolved);
      }
    }
  }
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
    Some(OverlayKind::ScreenshotCapture) => InputModePolicy::screenshot_overlay(),
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
  game_warning_ui: &mut GameWarningUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  cover_continue_ui: &mut CoverContinueUi,
  export_settings_ui: &mut ExportSettingsUi,
  screenshot_capture_ui: &mut ScreenshotCaptureUi,
  exit_warning_ui: &mut ExitWarningUi,
  _export_loading_ui: &mut ExportLoadingUi,
  language_loading_ui: &mut LanguageLoadingUi,
  language_loading: &mut LanguageLoadingRuntime,
  _export_loading: &mut ExportLoadingRuntime,
  pending_screenshot_saves: &mut HashMap<TaskId, PendingScreenshotSave>,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
  pending_recording_hotkey: &mut Option<PendingHostHotkey>,
  pending_screensaver_hotkey: &mut Option<PendingHostHotkey>,
  pending_toolbar_hotkey: &mut Option<PendingHostHotkey>,
  lua_events: &mut LuaEventBroker,
  frame: u64,
) {
  for event in services.input.drain_system_event_observations(128) {
    let data = match event {
      SystemEvent::Focus(event) => Some(LuaEventData::Focus {
        gained: event.gained,
      }),
      SystemEvent::Resize(event) => Some(LuaEventData::Resize {
        width: event.width,
        height: event.height,
      }),
      SystemEvent::Mouse(_) | SystemEvent::TerminalKey(_) => None,
    };
    if let Some(data) = data
      && let Err(error) = lua_events.push_system(frame, data)
    {
      log_lua_enqueue_error(services, error);
    }
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    load_screenshot_capture_action_map(services);
  }
  if world.state.current_overlay_kind().is_none() {
    let capture_finished = match world.state.current_ui_kind() {
      Some(UiNodeKind::GlobalKeyBindings)
        if settings_ui.key_bindings_mut().global_mut().is_capturing() =>
      {
        Some(
          settings_ui
            .key_bindings_mut()
            .global_mut()
            .handle_raw_key_events(&mut services.input, world.clock.delta_time()),
        )
      }
      Some(UiNodeKind::GameKeyBindings)
        if settings_ui.key_bindings_mut().game_mut().is_capturing() =>
      {
        Some(
          settings_ui
            .key_bindings_mut()
            .game_mut()
            .handle_raw_key_events(&mut services.input, world.clock.delta_time()),
        )
      }
      _ => None,
    };
    if let Some(finished) = capture_finished {
      if finished {
        services.canvas.request_render();
        let _ = services.input.disable_raw_key_capture();
      }
      // 绑定期间原始按键只用于采集，不触发宿主快捷键或页面动作。
      services.input.clear();
      let _ = services.input.drain_system_events();
      return;
    }
    if matches!(
      world.state.current_ui_kind(),
      Some(UiNodeKind::GlobalKeyBindings | UiNodeKind::GameKeyBindings)
    ) {
      let _ = services.input.take_raw_key_events();
    }
  }
  let host_actions = services.input.collect_action_events();
  if handle_screenshot_hotkey(
    services,
    world,
    screenshot_capture_ui,
    pending_screenshot_saves,
    pending_screenshot_hotkey,
    &host_actions,
  ) {
    return;
  }

  if handle_host_chord_input(
    services,
    world,
    display_settings_ui,
    pending_recording_hotkey,
    pending_screensaver_hotkey,
    pending_toolbar_hotkey,
    &host_actions,
  ) {
    return;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::WindowSizeWarning) {
    let screensaver_was_active = services.screensaver.is_active();
    let game_was_active = services.game.is_active();
    load_window_size_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      cover_continue_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
    if screensaver_was_active
      && world
        .state
        .runtime()
        .is_none_or(|runtime| runtime.overlays().get(OverlayKind::Screensaver).is_none())
    {
      if let Some(id) = services.screensaver.stop() {
        services.log.close_session(id);
      }
      synchronize_lua_event_sessions(services, lua_events);
    }
    if game_was_active && world.state.is_host_mode() {
      if let Some(id) = services.game.stop() {
        services.log.close_session(id);
      }
      synchronize_lua_event_sessions(services, lua_events);
    }
  } else if world.state.current_overlay_kind() == Some(OverlayKind::GameWarning) {
    load_game_warning_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    while let Some(event) = services.input.next_action_event() {
      if let Some(GameWarningCommand::Back) = game_warning_ui.handle_event(&UiEvent::Action(event))
      {
        return_from_game_warning(services, world, lua_events);
        break;
      }
    }
    services.input.clear();
    let _ = services.input.drain_system_events();
  } else if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
    load_safe_mode_warning_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      cover_continue_ui,
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      cover_continue_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  } else if world.state.current_overlay_kind() == Some(OverlayKind::CoverContinue) {
    load_cover_continue_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      cover_continue_ui,
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
      &host_actions,
    ) {
      apply_screenshot_capture_command(
        command,
        services,
        world,
        screenshot_capture_ui,
        pending_screenshot_saves,
      );
      if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
        show_popup(services, ScreenshotModeToastKind::Exit);
      }
    }
  } else if world.state.current_overlay_kind() == Some(OverlayKind::Screensaver) {
    services
      .input
      .dispatch_system_action_events(&mut services.log);
    while let Some(event) = services.input.next_action_event() {
      let _ = handle_host_key_action(event.action.as_str(), event.state, world);
    }
    let _ = services.input.drain_system_events();
    services.input.clear();
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
  } else if world.state.current_ui_kind() == Some(UiNodeKind::ExitWarning) {
    route_exit_warning_runtime_events(services, world, exit_warning_ui, pending_screenshot_saves);
  } else if world.state.is_game_mode() {
    load_game_action_map(services);
    services.input.dispatch_action_events(&mut services.log);
    while let Some(event) = services.input.next_action_event() {
      if handle_host_key_action(event.action.as_str(), event.state, world) {
        continue;
      }
      if let Err(error) = lua_events.push_system(
        frame,
        LuaEventData::Action {
          action: event.action,
          state: LuaActionState::from(event.state),
        },
      ) {
        log_lua_enqueue_error(services, error);
      }
    }
    let allow_mouse = services.input.is_focused();
    let base_rect = services.layout.developer_viewport_rect();
    for event in services.input.drain_system_events() {
      queue_lua_system_event(lua_events, frame, event, allow_mouse, base_rect);
    }
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
  } else {
    load_current_action_map(services, world);
    services.input.dispatch_action_events(&mut services.log);
    route_input_events(
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
      window_size_ui,
      safe_mode_warning_ui,
      clear_warning_ui,
      cover_continue_ui,
      export_settings_ui,
      screenshot_capture_ui,
      _export_loading_ui,
      language_loading_ui,
      language_loading,
      _export_loading,
    );
  }
}

fn queue_lua_system_event(
  router: &mut LuaEventBroker,
  frame: u64,
  event: SystemEvent,
  allow_mouse: bool,
  base_rect: Rect,
) {
  let data = match event {
    SystemEvent::Mouse(mut event)
      if allow_mouse
        && event.x >= base_rect.x
        && event.y >= base_rect.y
        && event.x < base_rect.x.saturating_add(base_rect.width)
        && event.y < base_rect.y.saturating_add(base_rect.height) =>
    {
      event.x = event.x.saturating_sub(base_rect.x);
      event.y = event.y.saturating_sub(base_rect.y);
      Some(LuaEventData::mouse(event))
    }
    SystemEvent::Resize(_)
    | SystemEvent::Focus(_)
    | SystemEvent::Mouse(_)
    | SystemEvent::TerminalKey(_) => None,
  };
  if let Some(data) = data
    && let Err(error) = router.push_system(frame, data)
  {
    // 溢出由 dispatch 阶段按 Session 隔离为故障；其余拒绝在这里无需中断宿主。
    debug_assert!(matches!(error, LuaEnqueueError::QueueOverflow(_)));
  }
}

fn dispatch_lua_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  router: &mut LuaEventBroker,
) {
  for token in router.take_overflowed_sessions() {
    handle_lua_queue_overflow(services, world, token);
  }
  synchronize_lua_event_sessions(services, router);

  // 先截取两个 Session 的本帧批次，再调用 Lua。回调期间新产生的事件只能进入
  // Broker 队尾，最早在下一宿主帧被消费。
  let game_deliveries = router.drain_frame(LuaSessionKind::Game);
  let screensaver_deliveries = router.drain_frame(LuaSessionKind::Screensaver);
  for (kind, deliveries) in [
    (LuaSessionKind::Game, game_deliveries),
    (LuaSessionKind::Screensaver, screensaver_deliveries),
  ] {
    let mut deliveries = deliveries.into_iter();
    while let Some(mut delivery) = deliveries.next() {
      if let LuaEventData::Resize { width, height } = &delivery.event.data {
        let physical = crate::host_engine::services::Size {
          width: *width,
          height: *height,
        };
        let size = lua_session_base_size_for_physical(services, kind, physical);
        replace_lua_resize_size(&mut delivery.event.data, size);
        match kind {
          LuaSessionKind::Game => services.game.set_base_size(size),
          LuaSessionKind::Screensaver => services.screensaver.set_base_size(size),
        }
      }
      let result = match kind {
        LuaSessionKind::Game => services.game.dispatch_event(&delivery),
        LuaSessionKind::Screensaver => services.screensaver.dispatch_event(&delivery),
      };
      if let Err(error) = result {
        handle_lua_fault(services, world, error);
        break;
      }
      match apply_lua_host_commands(kind, services, world, router) {
        LuaEventFlow::Continue => {}
        LuaEventFlow::Skip => {
          router.requeue_front(kind, deliveries);
          break;
        }
        LuaEventFlow::Clear => {
          router.requeue_front(
            kind,
            deliveries
              .filter(|delivery| !matches!(delivery.event.data, LuaEventData::Action { .. })),
          );
          router.clear_pending_actions(kind);
          break;
        }
      }
    }
  }
  synchronize_lua_event_sessions(services, router);
}

fn synchronize_lua_event_sessions(services: &mut EngineServices, router: &mut LuaEventBroker) {
  router.synchronize_sessions(
    services.game.session_token(),
    services.screensaver.session_token(),
  );
  services
    .async_runtime
    .cancel_tasks(router.take_orphaned_tasks());
  for audio_id in router.take_orphaned_audio() {
    let _ = services.audio.remove_owned(audio_id);
  }
}

fn update_lua_object_pool(
  objects: &mut crate::host_engine::services::LuaObjectPool,
  time: &crate::host_engine::services::TimeService,
  animation: &crate::host_engine::services::AnimationService,
  frame_delta: Duration,
) {
  objects.begin_frame();
  time.update(objects.runtime_mut(), frame_delta);
  animation.update(
    objects.runtime_mut(),
    crate::host_engine::services::AnimationClock::Ui,
    frame_delta,
  );
  animation.update(
    objects.runtime_mut(),
    crate::host_engine::services::AnimationClock::Game,
    frame_delta,
  );
}

fn handle_lua_queue_overflow(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  token: LuaSessionToken,
) {
  let package_id = match token.kind {
    LuaSessionKind::Game => services.game.package_id(),
    LuaSessionKind::Screensaver => services.screensaver.package_id(),
  }
  .unwrap_or("<stale-session>")
  .to_string();
  handle_lua_fault(
    services,
    world,
    LuaSessionError {
      package_id,
      session_kind: token.kind,
      stage: LuaErrorStage::EventQueue,
      callback: None,
      message: "pending event queue exceeded 1024 events".to_string(),
      diagnostic_commands: Vec::new(),
    },
  );
}

fn log_lua_enqueue_error(services: &mut EngineServices, error: LuaEnqueueError) {
  if !matches!(error, LuaEnqueueError::QueueOverflow(_)) {
    services.log.warn_operation_failed(
      LogSource::Lua,
      "queue_lua_event",
      "session_event_queue",
      format!("{error:?}"),
    );
  }
}

fn queue_lua_overlay_transitions(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  router: &mut LuaEventBroker,
  frame: u64,
) {
  for transition in world.state.take_overlay_transitions() {
    let data = match transition {
      OverlayStackTransition::Started => {
        // 覆盖屏接管交互时，不允许此前尚未派发的输入越过边界进入脚本。
        router.clear_pending_interactive(LuaSessionKind::Game);
        LuaEventData::OverlayStarted
      }
      OverlayStackTransition::Stopped => LuaEventData::OverlayStopped,
    };
    if let Err(error) = router.push_system(frame, data) {
      log_lua_enqueue_error(services, error);
    }
  }
}

fn update_lua_sessions(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  router: &mut LuaEventBroker,
  frame_delta: Duration,
) {
  let game_size = lua_session_base_size(services, LuaSessionKind::Game);
  let screensaver_size = lua_session_base_size(services, LuaSessionKind::Screensaver);
  services.game.set_base_size(game_size);
  services.screensaver.set_base_size(screensaver_size);

  if services.game.is_active()
    && let Err(error) = services.game.advance(frame_delta)
  {
    handle_lua_fault(services, world, error);
  }
  let _ = apply_lua_host_commands(LuaSessionKind::Game, services, world, router);
  if services.screensaver.is_active()
    && let Err(error) = services.screensaver.advance(frame_delta)
  {
    handle_lua_fault(services, world, error);
  }
  let _ = apply_lua_host_commands(LuaSessionKind::Screensaver, services, world, router);

  let visible_session = match world.state.current_overlay_kind() {
    Some(OverlayKind::Screensaver) if services.screensaver.is_active() => {
      Some(LuaSessionKind::Screensaver)
    }
    None if services.game.is_active() => Some(LuaSessionKind::Game),
    _ => None,
  };
  match visible_session {
    Some(LuaSessionKind::Game) => {
      if let Err(error) = services.game.render() {
        handle_lua_fault(services, world, error);
      }
      let _ = apply_lua_host_commands(LuaSessionKind::Game, services, world, router);
    }
    Some(LuaSessionKind::Screensaver) => {
      if let Err(error) = services.screensaver.render() {
        handle_lua_fault(services, world, error);
      }
      let _ = apply_lua_host_commands(LuaSessionKind::Screensaver, services, world, router);
    }
    None => {}
  }

  // 非当前画面的脚本仍可 Update，但其绘制结果在本帧不可见。帧末必须主动
  // 回收这些命令和计数，避免宿主覆盖屏让脚本误触单帧绘制上限。
  if visible_session != Some(LuaSessionKind::Game) {
    let _ = services.game.take_draw_commands();
  }
  if visible_session != Some(LuaSessionKind::Screensaver) {
    let _ = services.screensaver.take_draw_commands();
  }
}

fn lua_session_base_size(services: &EngineServices, kind: LuaSessionKind) -> Size {
  lua_session_base_size_for_physical(services, kind, services.layout.physical_size())
}

fn lua_session_base_size_for_physical(
  services: &EngineServices,
  kind: LuaSessionKind,
  physical: Size,
) -> Size {
  match kind {
    LuaSessionKind::Game => host_viewport::developer_size(
      physical,
      services.storage.display_settings_profile().top_toolbar,
    ),
    LuaSessionKind::Screensaver => physical,
  }
}

fn replace_lua_resize_size(data: &mut LuaEventData, size: Size) {
  if let LuaEventData::Resize { width, height } = data {
    *width = size.width;
    *height = size.height;
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LuaEventFlow {
  Continue,
  Skip,
  Clear,
}

fn apply_lua_host_commands(
  kind: LuaSessionKind,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  router: &mut LuaEventBroker,
) -> LuaEventFlow {
  let commands = take_lua_host_commands(kind, services);
  let mut flow = LuaEventFlow::Continue;
  for command in commands {
    if apply_lua_diagnostic_host_command(services, kind, &command) {
      continue;
    }
    match command {
      LuaHostCommand::RequestRender => {
        services.canvas.request_render();
        services.presenter.request_render();
      }
      LuaHostCommand::FileRequest {
        request_id,
        task,
        operation,
        virtual_path,
        event_tip,
      } => {
        let token = match kind {
          LuaSessionKind::Game => services.game.session_token(),
          LuaSessionKind::Screensaver => services.screensaver.session_token(),
        };
        let Some(token) = token else {
          continue;
        };
        let task_id = services.async_runtime.submit(EngineTask::File(task));
        if let Err(error) = router.register_task(
          task_id,
          token,
          LuaTaskOperation::File {
            request_id,
            kind: operation,
            virtual_path,
            event_tip,
          },
          LuaEventRoute::HandleEvent,
        ) {
          services.async_runtime.cancel_task(task_id);
          log_lua_session_message(
            services,
            kind,
            "warn",
            format!("Lua file request rejected: {error:?}"),
          );
        }
      }
      LuaHostCommand::I18nRequest {
        task,
        kind: event_kind,
        language_code,
        callback_language_code,
      } => {
        let token = match kind {
          LuaSessionKind::Game => services.game.session_token(),
          LuaSessionKind::Screensaver => services.screensaver.session_token(),
        };
        let Some(token) = token else {
          continue;
        };
        let task_id = services.async_runtime.submit(EngineTask::File(task));
        if let Err(error) = router.register_task(
          task_id,
          token,
          LuaTaskOperation::I18n {
            kind: event_kind,
            language_code,
            callback_language_code,
          },
          LuaEventRoute::HandleEvent,
        ) {
          services.async_runtime.cancel_task(task_id);
          log_lua_session_message(
            services,
            kind,
            "warn",
            format!("Lua i18n request rejected: {error:?}"),
          );
        }
      }
      LuaHostCommand::ExitGame if kind == LuaSessionKind::Game => {
        if let Some(id) = services.game.stop() {
          services.log.close_session(id);
        }
        let return_host = world
          .state
          .runtime()
          .and_then(|runtime| runtime.main_host().game())
          .map(|game| (*game.return_host).clone());
        if let (Some(runtime), Some(host)) = (world.state.runtime_mut(), return_host) {
          runtime.set_main_host(MainHostState::Host(host));
        }
        services.canvas.request_render();
        services.presenter.request_render();
      }
      LuaHostCommand::SaveGame if kind == LuaSessionKind::Game => {
        let identity = services.game.package().cloned();
        match services.game.save_game() {
          Ok(Some(value)) => {
            if let Some(package) = identity {
              let _ =
                services
                  .storage
                  .write_game_results(&package, Some(value), None, &mut services.log);
            }
          }
          Ok(None) => {}
          Err(error) => {
            handle_lua_fault(services, world, error);
            return flow;
          }
        }
      }
      LuaHostCommand::SaveBest if kind == LuaSessionKind::Game => {
        let identity = services.game.package().cloned();
        match services.game.save_best() {
          Ok(Some(value)) => match crate::host_engine::services::BestGameSave::try_from(value) {
            Ok(best) => {
              if let Some(package) = identity {
                let _ = services.storage.write_game_results(
                  &package,
                  None,
                  Some(best),
                  &mut services.log,
                );
              }
            }
            Err(error) => log_lua_session_message(services, LuaSessionKind::Game, "error", error),
          },
          Ok(None) => {}
          Err(error) => {
            handle_lua_fault(services, world, error);
            return flow;
          }
        }
      }
      LuaHostCommand::SkipActions if kind == LuaSessionKind::Game => flow = LuaEventFlow::Skip,
      LuaHostCommand::ClearActions if kind == LuaSessionKind::Game => flow = LuaEventFlow::Clear,
      LuaHostCommand::Draw(_) => {}
      LuaHostCommand::Log { .. }
      | LuaHostCommand::Print { .. }
      | LuaHostCommand::Ignored { .. } => {}
      LuaHostCommand::ExitGame
      | LuaHostCommand::SaveGame
      | LuaHostCommand::SaveBest
      | LuaHostCommand::SkipActions
      | LuaHostCommand::ClearActions => {}
    }
  }
  flow
}

fn take_lua_host_commands(
  kind: LuaSessionKind,
  services: &mut EngineServices,
) -> Vec<LuaHostCommand> {
  match kind {
    LuaSessionKind::Game => services.game.take_host_commands(),
    LuaSessionKind::Screensaver => services.screensaver.take_host_commands(),
  }
}

/// 返回命令是否属于已经完成的诊断输出。
fn apply_lua_diagnostic_host_command(
  services: &mut EngineServices,
  kind: LuaSessionKind,
  command: &LuaHostCommand,
) -> bool {
  match command {
    LuaHostCommand::Log { level, message } => {
      log_lua_session_message(services, kind, level, message.clone());
    }
    LuaHostCommand::Print {
      message,
      title,
      time,
      level,
      type_head,
    } => print_lua_session_message(
      services,
      kind,
      message.clone(),
      LogPrintOptions {
        time: *time,
        level: level.as_deref().and_then(lua_log_level),
        type_head: *type_head,
        title: title.clone(),
      },
    ),
    LuaHostCommand::Ignored { method, reason } => log_lua_session_message(
      services,
      kind,
      "debug",
      format!("{method} ignored: {reason}"),
    ),
    _ => return false,
  }
  true
}

/// Init 在 Session 注册到 Game/ScreensaverService 之前执行。若 Init 失败，
/// 通过刚创建的日志会话提交其故障前诊断，避免随构造中的 Session 一同丢失。
fn flush_lua_startup_diagnostics(
  services: &mut EngineServices,
  log_session: Option<crate::host_engine::services::LogSessionId>,
  kind: LuaSessionKind,
  commands: &[LuaHostCommand],
) {
  let Some(id) = log_session else {
    return;
  };
  let print_source = match kind {
    LuaSessionKind::Game => LogSource::Game,
    LuaSessionKind::Screensaver => LogSource::Screensaver,
  };
  for command in commands {
    match command {
      LuaHostCommand::Log { level, message } => match level.as_str() {
        "error" => services.log.error_session(id, LogSource::Lua, message),
        "warn" => services.log.warn_session(id, LogSource::Lua, message),
        "debug" => services.log.debug_session(id, LogSource::Lua, message),
        _ => services.log.info_session(id, LogSource::Lua, message),
      },
      LuaHostCommand::Print {
        message,
        title,
        time,
        level,
        type_head,
      } => services.log.print_session(
        id,
        print_source,
        message,
        LogPrintOptions {
          time: *time,
          level: level.as_deref().and_then(lua_log_level),
          type_head: *type_head,
          title: title.clone(),
        },
      ),
      LuaHostCommand::Ignored { method, reason } => {
        services
          .log
          .debug_session(id, LogSource::Lua, format!("{method} ignored: {reason}"))
      }
      _ => {}
    }
  }
}

/// 故障回调可能已成功执行过若干日志调用。Session 销毁前只提交这些诊断输出；
/// 保存、退出、文件任务和其他宿主副作用属于未完整完成的回调，必须丢弃。
fn flush_lua_fault_diagnostics(services: &mut EngineServices, kind: LuaSessionKind) {
  for command in take_lua_host_commands(kind, services) {
    let _ = apply_lua_diagnostic_host_command(services, kind, &command);
  }
}

fn handle_lua_fault(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  error: LuaSessionError,
) {
  flush_lua_fault_diagnostics(services, error.session_kind);
  let package = match error.session_kind {
    LuaSessionKind::Game => services.game.package(),
    LuaSessionKind::Screensaver => services.screensaver.package(),
  }
  .cloned();
  let diagnostics = match error.session_kind {
    LuaSessionKind::Game => services.game.diagnostics(),
    LuaSessionKind::Screensaver => services.screensaver.diagnostics(),
  };
  let message = format_lua_fault_message(&error, diagnostics.as_ref());
  log_lua_session_message(services, error.session_kind, "error", message);
  if let Some(package) = &package {
    services.log.error_message(
      LogSource::Runtime,
      HostLogMessage::new(
        "log_info.session.faulted",
        "{kind} session for {package} was isolated after a fault; see its package log.",
      )
      .param("kind", error.session_kind.as_str())
      .param("package", package.to_string()),
    );
  }
  match error.session_kind {
    LuaSessionKind::Game => {
      if let Some(id) = services.game.stop() {
        services.log.close_session(id);
      }
      world.state.push_game_warning_overlay();
    }
    LuaSessionKind::Screensaver => {
      if let Some(id) = services.screensaver.stop() {
        services.log.close_session(id);
      }
      let _ = world
        .state
        .remove_overlay_kind(OverlayKind::WindowSizeWarning);
      let _ = world.state.remove_overlay_kind(OverlayKind::Screensaver);
    }
  }
  services.canvas.request_render();
  services.presenter.request_render();
}

fn format_lua_fault_message(
  error: &LuaSessionError,
  diagnostics: Option<&LuaSessionDiagnostics>,
) -> String {
  diagnostics.map_or_else(
    || error.to_string(),
    |diagnostics| {
      format!(
        "{error}; entry={}; memory_bytes={}",
        diagnostics.entry_path.display(),
        diagnostics.memory_bytes,
      )
    },
  )
}

fn update_game_warning(world: &RuntimeWorld, delta: Duration, elapsed: &mut Duration) {
  let visible = world
    .state
    .runtime()
    .is_some_and(|runtime| runtime.overlays().get(OverlayKind::GameWarning).is_some());
  if visible {
    *elapsed = elapsed.saturating_add(delta);
  } else {
    *elapsed = Duration::ZERO;
  }
}

fn game_warning_seconds_left(elapsed: Duration) -> u8 {
  5_u8.saturating_sub(elapsed.as_secs().min(5) as u8)
}

fn return_from_game_warning(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  lua_events: &mut LuaEventBroker,
) {
  let return_host = world
    .state
    .runtime()
    .and_then(|runtime| runtime.main_host().game())
    .map(|game| (*game.return_host).clone());
  if let Some(id) = services.screensaver.stop() {
    services.log.close_session(id);
  }
  let _ = world.state.remove_overlay_kind(OverlayKind::GameWarning);
  let _ = world
    .state
    .remove_overlay_kind(OverlayKind::WindowSizeWarning);
  let _ = world.state.remove_overlay_kind(OverlayKind::Screensaver);
  if let (Some(runtime), Some(host)) = (world.state.runtime_mut(), return_host) {
    runtime.set_main_host(MainHostState::Host(host));
  }
  synchronize_lua_event_sessions(services, lua_events);
  services.input.clear();
  services.canvas.request_render();
  services.presenter.request_render();
}

fn log_lua_session_message(
  services: &mut EngineServices,
  kind: LuaSessionKind,
  level: &str,
  message: impl Into<String>,
) {
  let message = message.into();
  let id = match kind {
    LuaSessionKind::Game => services.game.log_session(),
    LuaSessionKind::Screensaver => services.screensaver.log_session(),
  };
  if let Some(id) = id {
    match level {
      "error" => services.log.error_session(id, LogSource::Lua, message),
      "warn" => services.log.warn_session(id, LogSource::Lua, message),
      "debug" => services.log.debug_session(id, LogSource::Lua, message),
      _ => services.log.info_session(id, LogSource::Lua, message),
    }
  } else if let Some(package) = match kind {
    LuaSessionKind::Game => services.game.package(),
    LuaSessionKind::Screensaver => services.screensaver.package(),
  }
  .cloned()
  {
    match level {
      "error" => services
        .log
        .error_package(&package, LogSource::Lua, message),
      "warn" => services.log.warn_package(&package, LogSource::Lua, message),
      "debug" => services
        .log
        .debug_package(&package, LogSource::Lua, message),
      _ => services.log.info_package(&package, LogSource::Lua, message),
    }
  } else {
    services.log.error_operation_failed(
      LogSource::Lua,
      "route_package_log",
      format!("{kind:?}"),
      message,
    );
  }
}

fn print_lua_session_message(
  services: &mut EngineServices,
  kind: LuaSessionKind,
  message: String,
  options: LogPrintOptions,
) {
  let id = match kind {
    LuaSessionKind::Game => services.game.log_session(),
    LuaSessionKind::Screensaver => services.screensaver.log_session(),
  };
  let source = match kind {
    LuaSessionKind::Game => LogSource::Game,
    LuaSessionKind::Screensaver => LogSource::Screensaver,
  };
  if let Some(id) = id {
    services.log.print_session(id, source, message, options);
    return;
  }
  if let Some(package) = match kind {
    LuaSessionKind::Game => services.game.package(),
    LuaSessionKind::Screensaver => services.screensaver.package(),
  }
  .cloned()
  {
    services
      .log
      .print_package(&package, source, message, options);
  } else {
    services.log.warn_operation_failed(
      LogSource::Lua,
      "route_package_print",
      format!("{kind:?}"),
      message,
    );
  }
}

fn lua_log_level(level: &str) -> Option<LogLevel> {
  match level {
    "trace" => Some(LogLevel::Trace),
    "debug" => Some(LogLevel::Debug),
    "info" => Some(LogLevel::Info),
    "warn" => Some(LogLevel::Warn),
    "error" => Some(LogLevel::Error),
    "fatal" => Some(LogLevel::Fatal),
    _ => None,
  }
}

fn route_exit_warning_runtime_events(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  ui: &mut ExitWarningUi,
  pending_images: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let Some(mode @ (ExitWarningMode::ExportWarning | ExitWarningMode::WaitingForExports)) =
    exit_warning_mode(world)
  else {
    return;
  };
  load_exit_warning_action_map(services, mode == ExitWarningMode::WaitingForExports);
  services.input.dispatch_action_events(&mut services.log);
  while let Some(event) = services.input.next_action_event() {
    if event.action == HOST_KEY_FORCE_STOP
      || handle_host_key_action(event.action.as_str(), event.state, world)
    {
      continue;
    }
    if let Some(command) = ui.handle_event(mode, &UiEvent::Action(event)) {
      apply_exit_warning_command(command, services, world, pending_images);
      return;
    }
  }

  for event in services.input.drain_system_events() {
    match event {
      SystemEvent::Mouse(mouse) if services.input.is_focused() => {
        services.hit_area.route_mouse_event(
          ui.objects_mut(),
          &mut services.text_input,
          &services.canvas,
          mouse,
        );
      }
      SystemEvent::Focus(focus) if !focus.gained => {
        services.hit_area.focus_lost(ui.objects_mut());
      }
      _ => {}
    }
    while let Some(event) = ui.objects_mut().pop_event() {
      if let Some(command) = ui.handle_event(mode, &event) {
        apply_exit_warning_command(command, services, world, pending_images);
        return;
      }
    }
  }
}

fn handle_screenshot_hotkey(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  pending_screenshot_saves: &mut HashMap<TaskId, PendingScreenshotSave>,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
  host_actions: &[InputActionEvent],
) -> bool {
  if !has_pressed_action(host_actions, HOST_KEY_SCREENSHOT) {
    return false;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    if screenshot_ui.is_guide_visible() {
      if screenshot_ui.can_dismiss_guide_by_screenshot_action() {
        screenshot_ui.dismiss_guide(&services.storage, &mut services.log);
      }
      services.input.clear();
      return true;
    }
    let command = if screenshot_ui.can_run_double_action() {
      screenshot_ui.select_whole_frame();
      match services
        .storage
        .read_screenshot_profile_or_default(&mut services.log)
        .double_action
      {
        ScreenshotDoubleAction::Copy => ScreenshotCaptureCommand::Copy,
        ScreenshotDoubleAction::CopyRichText => ScreenshotCaptureCommand::CopyRichText,
        ScreenshotDoubleAction::SavePng => ScreenshotCaptureCommand::SavePng,
        ScreenshotDoubleAction::All => ScreenshotCaptureCommand::All,
      }
    } else {
      ScreenshotCaptureCommand::Exit
    };
    apply_screenshot_capture_command(
      command,
      services,
      world,
      screenshot_ui,
      pending_screenshot_saves,
    );
    if world.state.current_overlay_kind() != Some(OverlayKind::ScreenshotCapture) {
      show_popup(services, ScreenshotModeToastKind::Exit);
    }
    services.input.clear();
    return true;
  }

  if pending_screenshot_hotkey.take().is_some() {
    run_quick_screenshot_action(services, pending_screenshot_saves);
    services.input.clear();
    return true;
  }

  *pending_screenshot_hotkey = Some(PendingScreenshotHotkey::new());
  services.input.clear();
  true
}

fn toggle_screensaver(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screensaver_ui: &mut ScreensaverOverlayUi,
  random_id: RandomGeneratorId,
  lua_events: &mut LuaEventBroker,
) {
  let screensaver_active = world
    .state
    .runtime()
    .is_some_and(|runtime| runtime.overlays().get(OverlayKind::Screensaver).is_some());
  if screensaver_active {
    if let Some(id) = services.screensaver.stop() {
      services.log.close_session(id);
    }
    synchronize_lua_event_sessions(services, lua_events);
    let _ = world
      .state
      .remove_overlay_kind(OverlayKind::WindowSizeWarning);
    let _ = world.state.remove_overlay_kind(OverlayKind::Screensaver);
    services.input.clear();
    return;
  }

  let Some(entry) = select_screensaver(services, random_id) else {
    services.input.clear();
    return;
  };
  let Some(package) = services.package.find_by_id(&entry.id) else {
    commands::log_package_start_error(
      services,
      &entry.id,
      "screensaver package was not found".to_string(),
    );
    services.input.clear();
    return;
  };
  let entry_path = match services.package.validate_for_launch(&package) {
    Ok(path) => path,
    Err(error) => {
      commands::log_package_start_error(
        services,
        &package.id,
        format!("screensaver launch validation failed: {error}"),
      );
      services.input.clear();
      return;
    }
  };
  let log_entry_path = entry_path.clone();
  let spec = crate::host_engine::services::LuaSessionSpec {
    package_id: package.mod_id.clone(),
    session_kind: LuaSessionKind::Screensaver,
    entry_path,
    fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
    base_size: lua_session_base_size(services, LuaSessionKind::Screensaver),
    continue_data: None,
    best_data: None,
    save_game_enabled: false,
    save_best_enabled: false,
  };
  let api = crate::host_engine::services::LuaApiConfig {
    debug_enabled: entry.debug,
    safe_mode_enabled: true,
    key_actions: entry.key_actions.clone(),
    key_default_actions: entry.key_default_actions.clone(),
    language_code: services.i18n.current_language_code().to_string(),
    missing_i18n_template: services
      .i18n
      .get_runtime_text("language_warning", "language_warning.missing"),
  };
  let session_log = services
    .log
    .open_session(
      crate::host_engine::services::LogSessionKind::Screensaver,
      &package.id,
    )
    .ok();
  let session = match services.lua.create_session_with_api(spec, api) {
    Ok(session) => session,
    Err(error) => {
      flush_lua_startup_diagnostics(
        services,
        session_log,
        LuaSessionKind::Screensaver,
        &error.diagnostic_commands,
      );
      let message = format!("{error}; entry={}", log_entry_path.display());
      if let Some(id) = session_log {
        services.log.error_session(id, LogSource::Lua, message);
        services.log.close_session(id);
      } else {
        commands::log_package_start_error(services, &package.id, message);
      }
      services.input.clear();
      return;
    }
  };
  if let Some(previous) = services
    .screensaver
    .start(session, package.id.clone(), session_log)
  {
    services.log.close_session(previous);
  }
  synchronize_lua_event_sessions(services, lua_events);
  let _ = world
    .state
    .remove_overlay_kind(OverlayKind::WindowSizeWarning);
  screensaver_ui.start(&entry);
  world
    .state
    .push_screensaver_overlay(entry.min_width, entry.min_height);
  services.input.clear();
}

fn handle_host_chord_input(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  display_settings_ui: &mut DisplaySettingsUi,
  pending_recording: &mut Option<PendingHostHotkey>,
  pending_screensaver: &mut Option<PendingHostHotkey>,
  pending_toolbar: &mut Option<PendingHostHotkey>,
  host_actions: &[InputActionEvent],
) -> bool {
  if has_pressed_action(host_actions, HOST_KEY_RECORDING_PAUSE) {
    services.popup.dismiss(PopupDismissEvent::RecordingControl);
    *pending_recording = None;
    match services.recording.state() {
      RecordingState::Recording => {
        if services.recording.pause() {
          if let Some(capture_id) = services.recording.audio_capture()
            && let Err(error) = services.audio.pause_capture(capture_id)
          {
            services.log.warn_operation_failed(
              LogSource::Audio,
              "pause_recording_capture",
              capture_id.0.to_string(),
              error.to_string(),
            );
          }
          show_recording_popup(services, RecordingPopupKind::Pause);
        }
      }
      RecordingState::Paused => {
        if services.recording.resume() {
          if let Some(capture_id) = services.recording.audio_capture()
            && let Err(error) = services.audio.resume_capture(capture_id)
          {
            services.log.warn_operation_failed(
              LogSource::Audio,
              "resume_recording_capture",
              capture_id.0.to_string(),
              error.to_string(),
            );
          }
          show_recording_popup(services, RecordingPopupKind::Resume);
        }
      }
      RecordingState::Stopped | RecordingState::Finalizing => {}
    }
    services.input.clear();
    return true;
  }
  if has_pressed_action(host_actions, HOST_KEY_RECORDING) {
    services.popup.dismiss(PopupDismissEvent::RecordingControl);
    *pending_recording = Some(PendingHostHotkey::new());
    return true;
  }
  let screensaver_pressed = has_pressed_action(host_actions, HOST_KEY_SCREENSAVER);
  let toolbar_pressed = has_pressed_action(host_actions, HOST_KEY_TOP_TOOLBAR);
  let toolbar_switch_pressed = has_pressed_action(host_actions, HOST_KEY_TOP_TOOLBAR_SWITCH);

  if world.state.current_ui_kind() == Some(UiNodeKind::ToolbarCustom)
    && (toolbar_pressed || toolbar_switch_pressed)
  {
    *pending_toolbar = None;
    services.input.clear();
    return true;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture)
    && screensaver_pressed
  {
    services.input.clear();
    return true;
  }

  if toolbar_switch_pressed {
    *pending_toolbar = None;
    toggle_toolbar_enabled(services, display_settings_ui);
    services.input.clear();
    return true;
  }
  if screensaver_pressed {
    *pending_screensaver = Some(PendingHostHotkey::new());
    return true;
  }
  if toolbar_pressed {
    *pending_toolbar = Some(PendingHostHotkey::new());
    return true;
  }
  false
}

fn has_pressed_action(events: &[InputActionEvent], action: &str) -> bool {
  events
    .iter()
    .any(|event| event.state == KeyState::Pressed && event.action == action)
}

#[allow(clippy::too_many_arguments)]
fn update_pending_host_hotkeys(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screensaver_ui: &mut ScreensaverOverlayUi,
  random_id: RandomGeneratorId,
  toolbar: &mut TopToolbarRuntime,
  pending_recording: &mut Option<PendingHostHotkey>,
  auto_recording: &mut AutoRecordingRuntime,
  pending_screensaver: &mut Option<PendingHostHotkey>,
  pending_toolbar: &mut Option<PendingHostHotkey>,
  lua_events: &mut LuaEventBroker,
  dt: Duration,
) {
  if pending_recording
    .as_mut()
    .is_some_and(|pending| !pending.update(dt))
  {
    *pending_recording = None;
    toggle_recording(services, auto_recording);
  }
  if pending_screensaver
    .as_mut()
    .is_some_and(|pending| !pending.update(dt))
  {
    *pending_screensaver = None;
    toggle_screensaver(services, world, screensaver_ui, random_id, lua_events);
  }
  if pending_toolbar
    .as_mut()
    .is_some_and(|pending| !pending.update(dt))
  {
    *pending_toolbar = None;
    if world.state.current_ui_kind() != Some(UiNodeKind::ToolbarCustom) {
      toolbar.cycle();
    }
  }
}

fn toggle_recording(services: &mut EngineServices, auto: &mut AutoRecordingRuntime) {
  match services.recording.state() {
    RecordingState::Stopped => {
      if start_recording(services) {
        auto.manually_stopped = false;
        services.log.info_message(
          LogSource::Runtime,
          HostLogMessage::new(
            "log_info.external.operation",
            "Host {operation} operation entered {state}.",
          )
          .param("operation", "recording")
          .param("state", "started"),
        );
        show_recording_popup(services, RecordingPopupKind::Start);
      }
    }
    RecordingState::Recording | RecordingState::Paused => {
      if stop_recording(services) {
        auto.manually_stopped = true;
        auto.restart_after_split = false;
        services.log.info_message(
          LogSource::Runtime,
          HostLogMessage::new(
            "log_info.external.operation",
            "Host {operation} operation entered {state}.",
          )
          .param("operation", "recording")
          .param("state", "finalizing"),
        );
        show_recording_popup(services, RecordingPopupKind::Stop);
      }
    }
    RecordingState::Finalizing => {}
  }
}

fn start_recording(services: &mut EngineServices) -> bool {
  let Some(frame) = services.recording.capture_last_frame() else {
    services.log.warn_operation_failed(
      LogSource::Runtime,
      "start_recording",
      "last_presented_frame",
      "no frame is available",
    );
    return false;
  };
  let frame_rate = Some(
    services
      .storage
      .read_recording_profile_or_default(&mut services.log)
      .capture_frame_rate
      .value(),
  );
  if !services
    .recording
    .start(frame, frame_rate, &services.storage)
  {
    return false;
  }
  if let Some(path) = services.recording.pending_audio_path() {
    let path_display = path.display().to_string();
    match services.audio.start_capture(path) {
      Ok(capture_id) => {
        let _ = services.recording.attach_audio_capture(capture_id);
      }
      Err(error) => services.log.warn_operation_failed(
        LogSource::Audio,
        "start_recording_capture",
        path_display,
        error.to_string(),
      ),
    }
  }
  true
}

fn stop_recording(services: &mut EngineServices) -> bool {
  if let Some(capture_id) = services.recording.audio_capture()
    && let Err(error) = services.audio.stop_capture(capture_id)
  {
    let _ = services.recording.detach_audio_capture(capture_id);
    services.log.warn_operation_failed(
      LogSource::Audio,
      "finalize_recording_capture",
      capture_id.0.to_string(),
      error.to_string(),
    );
  }
  services.recording.stop(&services.async_runtime)
}

fn update_auto_recording(services: &mut EngineServices, auto: &mut AutoRecordingRuntime) {
  let profile_revision = services.storage.recording_profile_revision();
  if auto.profile_revision != profile_revision {
    auto.profile = services
      .storage
      .read_recording_profile_or_default(&mut services.log);
    auto.profile_revision = services.storage.recording_profile_revision();
  }
  let profile = &auto.profile;
  if services.recording.state() == RecordingState::Recording
    && profile
      .auto_split
      .duration()
      .is_some_and(|duration| services.recording.snapshot().active_duration >= duration)
  {
    if stop_recording(services) {
      auto.restart_after_split = true;
    }
    return;
  }

  if services.recording.state() == RecordingState::Stopped && auto.restart_after_split {
    if start_recording(services) {
      auto.restart_after_split = false;
      show_recording_popup(services, RecordingPopupKind::AutoSplit);
    }
    return;
  }

  if auto.should_start_host(services.recording.state()) && start_recording(services) {
    auto.host_started = true;
    services.log.info_message(
      LogSource::Runtime,
      HostLogMessage::new(
        "log_info.external.operation",
        "Host {operation} operation entered {state}.",
      )
      .param("operation", "automatic_recording")
      .param("state", "started"),
    );
    show_recording_popup(services, RecordingPopupKind::Start);
  }
}

fn show_recording_popup(services: &mut EngineServices, kind: RecordingPopupKind) {
  let mode = services
    .storage
    .read_recording_profile_or_default(&mut services.log)
    .popup;
  let visible = match kind {
    RecordingPopupKind::AutoSplit => mode.shows_split(),
    RecordingPopupKind::Pause | RecordingPopupKind::Resume => mode.shows_pause_resume(),
    RecordingPopupKind::Start | RecordingPopupKind::Stop => mode.shows_start_stop(),
  };
  if !visible {
    return;
  }
  let (key, color) = match kind {
    RecordingPopupKind::AutoSplit => (
      "recording_settings.popup.auto_split",
      TextColor::Rgb {
        r: 249,
        g: 232,
        b: 147,
      },
    ),
    RecordingPopupKind::Pause => (
      "recording_settings.popup.pause",
      TextColor::Rgb {
        r: 249,
        g: 232,
        b: 147,
      },
    ),
    RecordingPopupKind::Start => (
      "recording_settings.popup.start",
      TextColor::Rgb {
        r: 95,
        g: 215,
        b: 105,
      },
    ),
    RecordingPopupKind::Resume => (
      "recording_settings.popup.resume",
      TextColor::Rgb {
        r: 95,
        g: 215,
        b: 105,
      },
    ),
    RecordingPopupKind::Stop => (
      "recording_settings.popup.stop",
      TextColor::Rgb {
        r: 255,
        g: 76,
        b: 76,
      },
    ),
  };
  services.popup.show(PopupRequest {
    text: services.i18n.get_runtime_text("recording_settings", key),
    color,
    duration: Duration::from_secs(2),
    dismiss_on: vec![PopupDismissEvent::RecordingControl],
    replaceable: true,
    persistent: false,
  });
}

fn toggle_toolbar_enabled(
  services: &mut EngineServices,
  display_settings_ui: &mut DisplaySettingsUi,
) {
  let mut profile = services.storage.display_settings_profile().clone();
  profile.top_toolbar = !profile.top_toolbar;
  if services
    .storage
    .write_display_settings_profile(&profile, &mut services.log)
    .is_ok()
  {
    display_settings_ui.set_top_toolbar(profile.top_toolbar);
  }
}

fn select_screensaver(
  services: &mut EngineServices,
  random_id: RandomGeneratorId,
) -> Option<PackageListEntry> {
  let package_state = services
    .storage
    .read_package_state_or_default(&mut services.log);
  let defaults = &package_state.defaults;
  let mut entries = services
    .package
    .screensaver_list()
    .into_iter()
    .filter(|entry| {
      package_state
        .screensaver(&entry.id)
        .map_or(defaults.enabled, |state| state.enabled)
    })
    .collect::<Vec<_>>();
  if entries.is_empty() {
    return None;
  }
  entries.sort_by(|left, right| {
    let left_order = package_state
      .screensaver(&left.id)
      .and_then(|state| state.order)
      .unwrap_or(u32::MAX);
    let right_order = package_state
      .screensaver(&right.id)
      .and_then(|state| state.order)
      .unwrap_or(u32::MAX);
    left_order
      .cmp(&right_order)
      .then_with(|| left.mod_id.cmp(&right.mod_id))
  });

  let mut display = services.storage.display_settings_profile().clone();
  let index = match display.screensaver_order {
    DisplayOrderMode::Random => services.random.int_range(
      &mut services.runtime_objects,
      random_id,
      0,
      entries.len() as i64,
    )? as usize,
    DisplayOrderMode::Order => {
      let index = sequential_screensaver_index(display.screensaver_sequence_cursor, entries.len())?;
      display.screensaver_sequence_cursor = display.screensaver_sequence_cursor.wrapping_add(1);
      let _ = services
        .storage
        .write_display_settings_profile(&display, &mut services.log);
      index
    }
  };
  entries.get(index).cloned()
}

fn sequential_screensaver_index(cursor: u64, enabled_count: usize) -> Option<usize> {
  (enabled_count > 0).then(|| (cursor % enabled_count as u64) as usize)
}

fn update_pending_screenshot_hotkey(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  pending_screenshot_hotkey: &mut Option<PendingScreenshotHotkey>,
) {
  let Some(pending) = pending_screenshot_hotkey else {
    return;
  };
  if pending.update(world.clock.delta_time()) {
    return;
  }
  *pending_screenshot_hotkey = None;
  start_screenshot_capture(services, world, screenshot_ui);
}

fn start_screenshot_capture(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
) {
  let Some(frame) = services.screenshot.capture_last_frame() else {
    services.log.warn_operation_failed(
      LogSource::Render,
      "start_screenshot",
      "last_presented_frame",
      "no frame is available",
    );
    services.input.clear();
    return;
  };
  let screenshot_profile = services
    .storage
    .read_screenshot_profile_or_default(&mut services.log);
  let show_guide = !screenshot_profile.guide_seen;
  let profile = synchronize_key_bindings_profile(services);
  screenshot_ui.set_host_key_params(host_key_rich_text_params(&profile));
  screenshot_ui.start(frame, show_guide, screenshot_profile.auto_exit);
  world.state.push_screenshot_capture_overlay();
  show_popup(services, ScreenshotModeToastKind::Enter);
  services.input.clear();
}

fn run_quick_screenshot_action(
  services: &mut EngineServices,
  pending_screenshot_saves: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let Some(frame) = services.screenshot.capture_last_frame() else {
    services.log.warn_operation_failed(
      LogSource::Render,
      "quick_screenshot",
      "last_presented_frame",
      "no frame is available",
    );
    return;
  };
  let rect = crate::host_engine::services::ScreenshotRect {
    x: 0,
    y: 0,
    width: frame.width(),
    height: frame.height(),
  };
  let action = services
    .storage
    .read_screenshot_profile_or_default(&mut services.log)
    .double_action;
  let copy_succeeded = match action {
    ScreenshotDoubleAction::Copy => Some(copy_screenshot_text(services, &frame, rect)),
    ScreenshotDoubleAction::CopyRichText => Some(copy_screenshot_rich_text(services, &frame, rect)),
    ScreenshotDoubleAction::All => Some(copy_screenshot_text(services, &frame, rect)),
    ScreenshotDoubleAction::SavePng => None,
  };
  let saves_png = matches!(
    action,
    ScreenshotDoubleAction::SavePng | ScreenshotDoubleAction::All
  );
  if saves_png {
    let task_id = submit_screenshot_png(services, frame.clone(), rect);
    pending_screenshot_saves.insert(task_id, PendingScreenshotSave { progress: 0.0 });
  } else {
    let _ =
      services
        .screenshot
        .write_json(&services.storage, &frame, rect, None, &mut services.log);
  }
  show_popup(
    services,
    ScreenshotModeToastKind::Operation {
      copy_succeeded,
      save: saves_png.then_some(ScreenshotSaveState::Loading),
    },
  );
}

fn apply_screenshot_capture_command(
  command: ScreenshotCaptureCommand,
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
  pending_screenshot_saves: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let mut completed_operation = false;
  match command {
    ScreenshotCaptureCommand::Exit => {
      finish_screenshot_capture(services, world, screenshot_ui);
    }
    ScreenshotCaptureCommand::Copy => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        completed_operation = true;
        let copied = copy_screenshot_text(services, &frame, rect);
        let _ =
          services
            .screenshot
            .write_json(&services.storage, &frame, rect, None, &mut services.log);
        screenshot_ui.clear_selection();
        show_popup(
          services,
          ScreenshotModeToastKind::Operation {
            copy_succeeded: Some(copied),
            save: None,
          },
        );
      }
    }
    ScreenshotCaptureCommand::CopyRichText => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        completed_operation = true;
        let copied = copy_screenshot_rich_text(services, &frame, rect);
        let _ =
          services
            .screenshot
            .write_json(&services.storage, &frame, rect, None, &mut services.log);
        screenshot_ui.clear_selection();
        show_popup(
          services,
          ScreenshotModeToastKind::Operation {
            copy_succeeded: Some(copied),
            save: None,
          },
        );
      }
    }
    ScreenshotCaptureCommand::SavePng => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        completed_operation = true;
        let task_id = submit_screenshot_png(services, frame, rect);
        pending_screenshot_saves.insert(task_id, PendingScreenshotSave { progress: 0.0 });
        screenshot_ui.clear_selection();
        show_popup(
          services,
          ScreenshotModeToastKind::Operation {
            copy_succeeded: None,
            save: Some(ScreenshotSaveState::Loading),
          },
        );
      }
    }
    ScreenshotCaptureCommand::All => {
      if let Some((frame, rect)) = screenshot_ui.current_selection() {
        completed_operation = true;
        let copied = copy_screenshot_text(services, &frame, rect);
        let task_id = submit_screenshot_png(services, frame, rect);
        pending_screenshot_saves.insert(task_id, PendingScreenshotSave { progress: 0.0 });
        screenshot_ui.clear_selection();
        show_popup(
          services,
          ScreenshotModeToastKind::Operation {
            copy_succeeded: Some(copied),
            save: Some(ScreenshotSaveState::Loading),
          },
        );
      }
    }
  }
  if completed_operation && screenshot_ui.auto_exit() {
    finish_screenshot_capture(services, world, screenshot_ui);
  }
}

fn finish_screenshot_capture(
  services: &mut EngineServices,
  world: &mut RuntimeWorld,
  screenshot_ui: &mut ScreenshotCaptureUi,
) {
  let _ = world
    .state
    .remove_overlay_kind(OverlayKind::ScreenshotCapture);
  screenshot_ui.finish();
  // 截屏覆盖屏拥有期间产生的按键状态不得在恢复游戏 action map 后继续传播。
  services.input.clear();
  let _ = services.input.take_raw_key_events();
  while services.input.next_action_event().is_some() {}
}

pub(super) fn copy_screenshot_text(
  services: &mut EngineServices,
  frame: &crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) -> bool {
  let text = ScreenshotService::plain_text(frame, rect);
  let copied = services.clipboard.write_text(&text);
  if !copied {
    services.log.warn_operation_failed(
      LogSource::Storage,
      "copy_screenshot_text",
      "clipboard",
      "clipboard rejected the text",
    );
  }
  copied
}

pub(super) fn copy_screenshot_rich_text(
  services: &mut EngineServices,
  frame: &crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) -> bool {
  let text = ScreenshotService::rich_text(frame, rect);
  let copied = services.clipboard.write_text(&text);
  if !copied {
    services.log.warn_operation_failed(
      LogSource::Storage,
      "copy_screenshot_rich_text",
      "clipboard",
      "clipboard rejected the text",
    );
  }
  copied
}

pub(super) fn submit_screenshot_png(
  services: &mut EngineServices,
  frame: crate::host_engine::services::ComposedFrame,
  rect: crate::host_engine::services::ScreenshotRect,
) -> TaskId {
  let png_path = ScreenshotService::next_png_path(&services.storage);
  let source_path = services.screenshot.write_json(
    &services.storage,
    &frame,
    rect,
    Some(&png_path),
    &mut services.log,
  );
  let task_id = services
    .async_runtime
    .submit(EngineTask::Screenshot(ScreenshotTask {
      frame,
      selection: rect,
      png_path,
      fonts: services
        .storage
        .read_screenshot_profile_or_default(&mut services.log)
        .fonts,
    }));
  if let Some(source_path) = source_path {
    services
      .screenshot
      .register_source_export(task_id, source_path);
  }
  task_id
}

fn submit_font_preview_png(
  services: &mut EngineServices,
  fonts: Vec<String>,
  pending_screenshot_saves: &mut HashMap<TaskId, PendingScreenshotSave>,
) {
  let frame = ScreenshotService::font_preview_frame();
  let Some(rect) = ScreenshotService::whole_frame_rect(&frame) else {
    return;
  };
  let png_path = ScreenshotService::next_png_path(&services.storage);
  let source_path = services.screenshot.write_json(
    &services.storage,
    &frame,
    rect,
    Some(&png_path),
    &mut services.log,
  );
  let task_id = services
    .async_runtime
    .submit(EngineTask::Screenshot(ScreenshotTask {
      frame,
      selection: rect,
      png_path,
      fonts,
    }));
  if let Some(source_path) = source_path {
    services
      .screenshot
      .register_source_export(task_id, source_path);
  }
  pending_screenshot_saves.insert(task_id, PendingScreenshotSave { progress: 0.0 });
  show_popup(
    services,
    ScreenshotModeToastKind::Operation {
      copy_succeeded: None,
      save: Some(ScreenshotSaveState::Loading),
    },
  );
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::time::Duration;

  use super::{
    AutoRecordingRuntime, format_lua_fault_message, has_pressed_action, queue_lua_system_event,
    replace_lua_resize_size, sequential_screensaver_index,
  };
  use crate::host_engine::services::{
    AutoRecordingMode, InputActionEvent, InputEventType, KeyState, LuaErrorStage, LuaEventBroker,
    LuaEventData, LuaExecutionStats, LuaSessionDiagnostics, LuaSessionError, LuaSessionKind,
    LuaSessionToken, MouseEvent, MouseEventKind, RecordingProfile, RecordingState, Rect,
    SystemEvent,
  };

  #[test]
  fn lua_fault_log_does_not_append_stale_successful_callback_stats() {
    let error = LuaSessionError {
      package_id: "test.package".to_string(),
      session_kind: LuaSessionKind::Game,
      stage: LuaErrorStage::ExecutionLimit,
      callback: Some("Render"),
      message: "instructions execution limit exceeded: elapsed_ms=15.000; time_limit_ms=75.000; instructions=201000; instruction_limit=200000".to_string(),
      diagnostic_commands: Vec::new(),
    };
    let diagnostics = LuaSessionDiagnostics {
      entry_path: PathBuf::from("scripts/main.lua"),
      stats: LuaExecutionStats {
        instructions: 0,
        elapsed: Duration::ZERO,
        memory_bytes: 1,
      },
      memory_bytes: 97_617,
    };

    let message = format_lua_fault_message(&error, Some(&diagnostics));
    assert!(message.contains("instructions=201000; instruction_limit=200000"));
    assert!(message.contains("memory_bytes=97617"));
    assert!(!message.contains("instructions=0"));
    assert!(!message.contains("elapsed_ms=0"));
  }

  #[test]
  fn screensaver_sequence_handles_empty_and_changed_enabled_lists() {
    assert_eq!(sequential_screensaver_index(8, 0), None);
    assert_eq!(sequential_screensaver_index(8, 5), Some(3));
    assert_eq!(sequential_screensaver_index(5, 5), Some(0));
  }

  #[test]
  fn host_hotkeys_are_matched_by_semantic_action() {
    let events = vec![InputActionEvent {
      event_type: InputEventType::Keyboard,
      action: "host_key.screenshot".to_string(),
      state: KeyState::Pressed,
    }];
    assert!(has_pressed_action(&events, "host_key.screenshot"));
    assert!(!has_pressed_action(&events, "host_key.recording"));
  }

  #[test]
  fn host_auto_recording_uses_the_startup_snapshot() {
    let mut disabled = AutoRecordingRuntime::new(RecordingProfile::default(), 0);
    disabled.profile.auto_recording = AutoRecordingMode::Host;
    assert!(!disabled.should_start_host(RecordingState::Stopped));

    let mut profile = RecordingProfile::default();
    profile.auto_recording = AutoRecordingMode::Host;
    let mut enabled = AutoRecordingRuntime::new(profile, 0);
    assert!(enabled.should_start_host(RecordingState::Stopped));
    assert!(!enabled.should_start_host(RecordingState::Recording));
    enabled.manually_stopped = true;
    assert!(!enabled.should_start_host(RecordingState::Stopped));
  }

  #[test]
  fn lua_mouse_coordinates_are_clipped_and_mapped_to_the_base_viewport() {
    let mut broker = LuaEventBroker::new();
    broker.synchronize_sessions(
      Some(LuaSessionToken {
        kind: LuaSessionKind::Game,
        generation: 1,
      }),
      None,
    );
    let base = Rect {
      x: 10,
      y: 3,
      width: 20,
      height: 10,
    };
    queue_lua_system_event(
      &mut broker,
      1,
      SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 12,
        y: 7,
      }),
      true,
      base,
    );
    queue_lua_system_event(
      &mut broker,
      1,
      SystemEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Move,
        button: None,
        scroll: None,
        x: 9,
        y: 7,
      }),
      true,
      base,
    );

    let events = broker.drain_frame(LuaSessionKind::Game);
    assert_eq!(events.len(), 1);
    assert!(matches!(
      events[0].event.data,
      LuaEventData::Mouse { x: 2, y: 4, .. }
    ));
  }

  #[test]
  fn lua_resize_event_exposes_the_session_base_size() {
    let mut data = LuaEventData::Resize {
      width: 160,
      height: 50,
    };

    replace_lua_resize_size(
      &mut data,
      crate::host_engine::services::Size {
        width: 120,
        height: 40,
      },
    );

    assert_eq!(
      data,
      LuaEventData::Resize {
        width: 120,
        height: 40,
      }
    );
  }
}
