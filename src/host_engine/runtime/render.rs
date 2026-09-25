use super::*;
use super::{host_viewport::apply_host_viewport, router::current_objects_mut};
use crate::host_engine::services::UiObjectPoolOwner;

pub(super) fn route_render(
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
  mut language_select_ui: Option<&mut LanguageSelectUi>,
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
  game_warning_seconds_left: u8,
  exit_warning_ui: &mut ExitWarningUi,
  screensaver_overlay_ui: &mut ScreensaverOverlayUi,
  export_loading_ui: &mut ExportLoadingUi,
  language_loading_ui: &mut LanguageLoadingUi,
  top_toolbar: &mut TopToolbarRuntime,
  image_queue: usize,
  image_progress: Option<f32>,
) -> Option<(u16, u16)> {
  if let Some(OverlayKind::WindowSizeWarning) = world.state.current_overlay_kind() {
    apply_host_viewport(services, false);
    let runtime = world.state.runtime().unwrap();
    let overlay = runtime.overlays().top().unwrap();
    let req_w = overlay.render.required_width;
    let req_h = overlay.render.required_height;
    let term = services.layout.physical_size();

    window_size_ui.objects_mut().begin_render();
    window_size_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    window_size_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
      req_w,
      req_h,
      term.width,
      term.height,
      world.state.is_host_mode(),
      runtime.overlays().get(OverlayKind::Screensaver).is_some(),
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    apply_host_viewport(services, false);
    screenshot_capture_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::GameWarning) {
    apply_host_viewport(services, false);
    game_warning_ui.objects_mut().begin_render();
    game_warning_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    game_warning_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      game_warning_seconds_left,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::Screensaver) {
    apply_host_viewport(services, false);
    if services.screensaver.has_objects() {
      let commands = services.screensaver.take_draw_commands();
      services.screensaver.with_objects_mut(|objects| {
        objects.ui_mut().begin_render();
        objects.ui().prepare_canvas(&mut services.canvas, &services.layout);
      });
      apply_lua_draw_commands(services, commands);
    } else {
      screensaver_overlay_ui.objects_mut().begin_render();
      screensaver_overlay_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
      screensaver_overlay_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
    }
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
    apply_host_viewport(services, false);
    language_loading_ui.objects_mut().begin_render();
    language_loading_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    language_loading_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.progress_bar,
      &services.time,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ExportLoading) {
    apply_host_viewport(services, false);
    export_loading_ui.objects_mut().begin_render();
    export_loading_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    export_loading_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.progress_bar,
      &services.time,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
    apply_host_viewport(services, false);
    safe_mode_warning_ui.objects_mut().begin_render();
    safe_mode_warning_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    safe_mode_warning_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
      world.safe_mode_warning_all,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ClearWarning) {
    apply_host_viewport(services, false);
    clear_warning_ui.objects_mut().begin_render();
    clear_warning_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    clear_warning_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::CoverContinue) {
    apply_host_viewport(services, false);
    let continue_game = services
      .storage
      .continue_game_save()
      .and_then(|slot| services.package.find_by_id(&slot.package))
      .and_then(|package| package.game.map(|game| game.name))
      .map(|name| services.rich_text.visible_text(&name, None))
      .or_else(|| {
        services
          .storage
          .continue_game_save()
          .map(|slot| slot.package.mod_id)
      })
      .unwrap_or_default();
    cover_continue_ui.start(continue_game);
    cover_continue_ui.objects_mut().begin_render();
    cover_continue_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    cover_continue_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ExportSettings) {
    apply_host_viewport(services, false);
    export_settings_ui.objects_mut().begin_render();
    export_settings_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
    export_settings_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
      &services.text_input,
    );
    return None;
  }

  let show_top_toolbar = services.storage.display_settings_profile().top_toolbar;
  apply_host_viewport(services, show_top_toolbar);

  let lua_game_prepared = world.state.current_ui_kind().is_none() && services.game.is_active();
  if lua_game_prepared && services.game.has_objects() {
    let commands = services.game.take_draw_commands();
    services.game.with_objects_mut(|objects| {
      objects.ui_mut().begin_render();
      objects.ui().prepare_canvas(&mut services.canvas, &services.layout);
    });
    apply_lua_draw_commands(services, commands);
  }

  if world.state.current_ui_kind() == Some(UiNodeKind::ScreensaverList) {
    screensaver_list_ui.prepare_surfaces(
      &services.layout,
      &services.i18n,
      &services.text_input,
      &services.scroll_box,
      &services.package,
      &services.storage,
      &mut services.log,
    );
  }
  if world.state.current_ui_kind() == Some(UiNodeKind::GameKeyBindings) {
    settings_ui.key_bindings_mut().game_mut().prepare_surfaces(
      &services.layout,
      &services.i18n,
      &services.text_input,
      &services.scroll_box,
    );
  }
  if world.state.current_ui_kind() == Some(UiNodeKind::ScreenshotSettings) {
    settings_ui
      .screenshot_recording_mut()
      .screenshot_settings_mut()
      .prepare_surfaces(&services.scroll_box, &services.layout, &services.i18n);
  }
  if world.state.current_ui_kind() == Some(UiNodeKind::RecordingSettings) {
    settings_ui
      .screenshot_recording_mut()
      .recording_settings_mut()
      .prepare_surfaces(&services.scroll_box, &services.layout, &services.i18n);
  }
  if world.state.current_ui_kind() == Some(UiNodeKind::ScreenshotList) {
    settings_ui
      .screenshot_recording_mut()
      .screenshot_list_mut()
      .prepare_surfaces(
        &services.layout,
        &services.i18n,
        &services.text_input,
        &services.scroll_box,
      );
  }
  if world.state.current_ui_kind() == Some(UiNodeKind::RecordingList) {
    settings_ui
      .screenshot_recording_mut()
      .recording_list_mut()
      .prepare_surfaces(
        &services.layout,
        &services.i18n,
        &services.text_input,
        &services.scroll_box,
      );
  }

  if world.state.current_ui_kind() == Some(UiNodeKind::ExitWarning) {
    exit_warning_ui.objects_mut().begin_render();
    exit_warning_ui.objects().prepare_canvas(&mut services.canvas, &services.layout);
  } else if !lua_game_prepared {
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
      objects.begin_render();
      objects.prepare_canvas(&mut services.canvas, &services.layout);
    }
  }

  let mut input_cursor = None;
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
      let continue_name = services.storage.continue_game_save().and_then(|slot| {
        services
          .package
          .find_by_id(&slot.package)
          .and_then(|package| {
            package
              .game
              .as_ref()
              .filter(|game| game.save)
              .map(|game| services.rich_text.visible_text(&game.name, None))
          })
      });
      if continue_name.is_none() && services.storage.continue_game_save().is_some() {
        let _ = services.storage.clear_continue_game_save(&mut services.log);
      }
      home_ui.set_continue_game_name(continue_name);
      home_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::Settings) => {
      settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::KeyBindings) => {
      settings_ui.key_bindings_mut().render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::GlobalKeyBindings) => {
      settings_ui.key_bindings_mut().global_mut().render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::GameKeyBindings) => {
      settings_ui.key_bindings_mut().game_mut().render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
      );
    }
    Some(UiNodeKind::DisplaySettings) => {
      display_settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::ToolbarCustom) => {
      input_cursor = display_settings_ui.custom_mut().render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.text_input,
      );
    }
    Some(UiNodeKind::ScreensaverList) => {
      screensaver_list_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
      );
    }
    Some(UiNodeKind::ScreenshotRecording) => {
      settings_ui.screenshot_recording_mut().render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::ScreenshotSettings) => {
      input_cursor = settings_ui
        .screenshot_recording_mut()
        .screenshot_settings_mut()
        .render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.hit_area,
          &services.text_input,
          &services.scroll_box,
        );
    }
    Some(UiNodeKind::RecordingSettings) => {
      input_cursor = settings_ui
        .screenshot_recording_mut()
        .recording_settings_mut()
        .render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.hit_area,
          &services.text_input,
          &services.scroll_box,
        );
    }
    Some(UiNodeKind::ScreenshotList) => {
      input_cursor = settings_ui
        .screenshot_recording_mut()
        .screenshot_list_mut()
        .render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.hit_area,
          &services.text_input,
          &services.scroll_box,
        );
    }
    Some(UiNodeKind::RecordingList) => {
      input_cursor = settings_ui
        .screenshot_recording_mut()
        .recording_list_mut()
        .render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.hit_area,
          &services.text_input,
          &services.scroll_box,
        );
    }
    Some(UiNodeKind::SecuritySettings) => {
      security_uis.settings.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::SecurityDetails) => {
      security_uis.details.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.scroll_box,
        &services.markdown,
        &services.code_highlight,
      );
    }
    Some(UiNodeKind::StorageManagement) => {
      storage_management_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::StorageManagementClear) => {
      storage_management_clear_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::StorageManagementExport) => {
      storage_management_export_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::StorageManagementView) => {
      storage_management_view_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.storage,
        &services.hit_area,
        &services.table,
      );
    }
    Some(UiNodeKind::LanguageSelect) => {
      if let Some(ui) = language_select_ui {
        ui.render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.hit_area,
        );
      }
    }
    Some(UiNodeKind::Mods) => {
      mods_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
      );
    }
    Some(UiNodeKind::GameList) => {
      game_list_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        &services.package,
        &services.storage,
        &mut services.log,
        &world.temporary_safe_mode_disabled,
        &mut services.image,
        services.terminal.capabilities().mouse,
        services.terminal.capabilities().truecolor,
      );
    }
    Some(UiNodeKind::GamePackage) => {
      let capabilities = services.terminal.capabilities();
      game_package_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        &services.package,
        &services.storage,
        &mut services.log,
        &world.temporary_safe_mode_disabled,
        &mut services.image,
        capabilities.mouse,
        capabilities.truecolor,
      );
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      let capabilities = services.terminal.capabilities();
      screensaver_package_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
        &services.text_input,
        &services.scroll_box,
        &services.package,
        &services.storage,
        &mut services.log,
        &mut services.image,
        capabilities.truecolor,
      );
    }
    Some(UiNodeKind::TerminalCheck) => {
      terminal_check_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
      );
    }
    Some(UiNodeKind::InputDemo) => {
      input_demo_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.hit_area,
        &services.progress_bar,
      );
    }
    Some(UiNodeKind::ExitWarning) => {
      if let Some(mode @ (ExitWarningMode::ExportWarning | ExitWarningMode::WaitingForExports)) =
        exit_warning_mode(world)
      {
        let image = (image_queue > 0).then_some((image_queue, image_progress.unwrap_or(0.0)));
        let video_count = services.video.active_export_count();
        let video = (video_count > 0).then(|| {
          (
            video_count,
            services
              .video
              .first_active_progress()
              .map(|progress| progress.ratio)
              .unwrap_or(0.0),
          )
        });
        exit_warning_ui.render(
          &mut services.render,
          &mut services.canvas,
          &services.layout,
          &services.i18n,
          &services.progress_bar,
          &services.hit_area,
          mode,
          image,
          video,
        );
      }
    }
    _ => {}
  }

  if show_top_toolbar {
    let custom_text = (world.state.current_ui_kind() == Some(UiNodeKind::ToolbarCustom))
      .then(|| display_settings_ui.custom_text().to_string());
    top_toolbar.render(
      services,
      image_queue,
      image_progress,
      custom_text.as_deref(),
    );
  }

  input_cursor
}

fn apply_lua_draw_commands(
  services: &mut EngineServices,
  commands: Vec<crate::host_engine::services::LuaDrawCommand>,
) {
  use crate::host_engine::services::{LuaDrawCommand, LuaDrawTarget};
  for command in commands {
    match command {
      LuaDrawCommand::Text {
        target,
        x,
        y,
        params,
      } => match target {
        LuaDrawTarget::Base => services
          .render
          .draw_text_at(&mut services.canvas, x, y, &params),
        LuaDrawTarget::Slice(id) => {
          services
            .render
            .draw_text_at_on(&mut services.canvas, id, x, y, &params);
        }
      },
      LuaDrawCommand::FillRect {
        target,
        x,
        y,
        width,
        height,
        fill_char,
        fg,
        bg,
      } => match target {
        LuaDrawTarget::Base => services.render.draw_filled_rect(
          &mut services.canvas,
          x,
          y,
          width,
          height,
          fill_char,
          fg,
          bg,
        ),
        LuaDrawTarget::Slice(id) => {
          services.render.draw_filled_rect_on(
            &mut services.canvas,
            id,
            x,
            y,
            width,
            height,
            fill_char,
            fg,
            bg,
          );
        }
      },
      LuaDrawCommand::StrokeRect {
        target,
        x,
        y,
        width,
        height,
        border,
        fg,
        bg,
      } => match target {
        LuaDrawTarget::Base => services.render.draw_border_rect(
          &mut services.canvas,
          x,
          y,
          width,
          height,
          &border,
          fg,
          bg,
          None,
          None,
        ),
        LuaDrawTarget::Slice(id) => {
          services.render.draw_border_rect_on(
            &mut services.canvas,
            id,
            x,
            y,
            width,
            height,
            &border,
            fg,
            bg,
            None,
            None,
          );
        }
      },
      LuaDrawCommand::EraseRect {
        target,
        x,
        y,
        width,
        height,
      } => match target {
        LuaDrawTarget::Base => services.canvas.erase_rect(x, y, width, height),
        LuaDrawTarget::Slice(id) => {
          services.canvas.erase_rect_on(id, x, y, width, height);
        }
      },
    }
  }
}
