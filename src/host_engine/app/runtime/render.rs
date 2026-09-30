use super::*;
use super::{host_viewport::apply_host_viewport, router::current_objects_mut};
use crate::host_engine::services::UiObjectPoolOwner;

pub(super) fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  context: &mut RuntimeUiContext<'_>,
  game_warning_seconds_left: u8,
) -> Option<(u16, u16)> {
  let RuntimeUiContext {
    home_ui,
    settings_ui,
    display_settings_ui,
    screensaver_list_ui,
    security_settings_ui,
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
    game_warning_ui,
    clear_warning_ui,
    cover_continue_ui,
    export_settings_ui,
    screenshot_capture_ui,
    exit_warning_ui,
    screensaver_overlay_ui,
    export_loading_ui,
    language_loading_ui,
    top_toolbar,
    pending_screenshot_saves,
    ..
  } = context;
  let image_queue = pending_screenshot_saves.len();
  let image_progress = pending_screenshot_saves
    .iter()
    .min_by_key(|(task_id, _)| task_id.0)
    .map(|(_, save)| save.progress);
  if let Some(OverlayKind::WindowSizeWarning) = world.state.current_overlay_kind() {
    apply_host_viewport(services, false);
    let runtime = world.state.runtime().unwrap();
    let overlay = runtime.overlays().top().unwrap();
    let req_w = overlay.render.required_width;
    let req_h = overlay.render.required_height;
    let term = services.layout.physical_size();

    window_size_ui.objects_mut().begin_render();
    window_size_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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
    game_warning_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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
        objects
          .ui()
          .prepare_canvas(&mut services.canvas, &services.layout);
      });
      apply_lua_draw_commands(&mut services.render, &mut services.canvas, commands);
    } else {
      screensaver_overlay_ui.objects_mut().begin_render();
      screensaver_overlay_ui
        .objects()
        .prepare_canvas(&mut services.canvas, &services.layout);
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
    language_loading_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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
    export_loading_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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

  if world.state.current_overlay_kind() == Some(OverlayKind::ClearWarning) {
    apply_host_viewport(services, false);
    clear_warning_ui.objects_mut().begin_render();
    clear_warning_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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
    cover_continue_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
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
    export_settings_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
    let input_cursor = export_settings_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
      &services.hit_area,
      &services.text_input,
    );
    return input_cursor;
  }

  let show_top_toolbar = services.storage.display_settings_profile().top_toolbar;
  apply_host_viewport(services, show_top_toolbar);

  let lua_game_prepared = world.state.current_ui_kind().is_none() && services.game.is_active();
  if lua_game_prepared && services.game.has_objects() {
    let commands = services.game.take_draw_commands();
    services.game.with_objects_mut(|objects| {
      objects.ui_mut().begin_render();
      objects
        .ui()
        .prepare_canvas(&mut services.canvas, &services.layout);
    });
    apply_lua_draw_commands(&mut services.render, &mut services.canvas, commands);
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
    exit_warning_ui
      .objects()
      .prepare_canvas(&mut services.canvas, &services.layout);
  } else if !lua_game_prepared
    && let Some(objects) = current_objects_mut(
      world,
      home_ui,
      settings_ui,
      display_settings_ui,
      screensaver_list_ui,
      security_settings_ui,
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
    )
  {
    objects.begin_render();
    objects.prepare_canvas(&mut services.canvas, &services.layout);
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
      input_cursor = settings_ui.key_bindings_mut().game_mut().render(
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
      input_cursor = screensaver_list_ui.render(
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
      security_settings_ui.render(
        &mut services.render,
        &mut services.canvas,
        &services.layout,
        &services.i18n,
        &services.hit_area,
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
      let capabilities = services.terminal.capabilities();
      input_cursor = game_list_ui.render(&mut GameListRenderContext {
        render: &mut services.render,
        canvas: &mut services.canvas,
        layout: &services.layout,
        i18n: &services.i18n,
        hit_area: &services.hit_area,
        text_input: &services.text_input,
        scroll_box: &services.scroll_box,
        package: &services.package,
        storage: &services.storage,
        log: &mut services.log,
        mouse_supported: capabilities.mouse,
        truecolor_supported: capabilities.truecolor,
      });
    }
    Some(UiNodeKind::GamePackage) => {
      let capabilities = services.terminal.capabilities();
      input_cursor = game_package_ui.render(&mut PackageListRenderContext {
        render: &mut services.render,
        canvas: &mut services.canvas,
        layout: &services.layout,
        i18n: &services.i18n,
        hit_area: &services.hit_area,
        text_input: &services.text_input,
        scroll_box: &services.scroll_box,
        package: &services.package,
        storage: &services.storage,
        log: &mut services.log,
        image: &mut services.image,
        mouse_supported: capabilities.mouse,
        truecolor_supported: capabilities.truecolor,
      });
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      let capabilities = services.terminal.capabilities();
      input_cursor = screensaver_package_ui.render(&mut PackageListRenderContext {
        render: &mut services.render,
        canvas: &mut services.canvas,
        layout: &services.layout,
        i18n: &services.i18n,
        hit_area: &services.hit_area,
        text_input: &services.text_input,
        scroll_box: &services.scroll_box,
        package: &services.package,
        storage: &services.storage,
        log: &mut services.log,
        image: &mut services.image,
        mouse_supported: capabilities.mouse,
        truecolor_supported: capabilities.truecolor,
      });
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
  render: &mut crate::host_engine::services::RenderService,
  canvas: &mut crate::host_engine::services::CanvasService,
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
        LuaDrawTarget::Base => render.draw_text_at(canvas, x, y, &params),
        LuaDrawTarget::Slice(id) => {
          render.draw_text_at_on(canvas, id, x, y, &params);
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
        LuaDrawTarget::Base => {
          render.draw_filled_rect(canvas, x, y, width, height, fill_char, fg, bg)
        }
        LuaDrawTarget::Slice(id) => {
          render.draw_filled_rect_on(canvas, id, x, y, width, height, fill_char, fg, bg);
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
        LuaDrawTarget::Base => {
          render.draw_border_rect(canvas, x, y, width, height, &border, fg, bg, None, None)
        }
        LuaDrawTarget::Slice(id) => {
          render.draw_border_rect_on(canvas, id, x, y, width, height, &border, fg, bg, None, None);
        }
      },
      LuaDrawCommand::EraseRect {
        target,
        x,
        y,
        width,
        height,
      } => match target {
        LuaDrawTarget::Base => canvas.erase_rect(x, y, width, height),
        LuaDrawTarget::Slice(id) => {
          canvas.erase_rect_on(id, x, y, width, height);
        }
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{
    CanvasService, ComposedCell, FrameCompositor, LayoutService, RenderService, Size,
  };
  use std::time::{Duration, SystemTime, UNIX_EPOCH};
  use tg_service_lua::{LuaPolicy, LuaService, LuaSessionKind, LuaSessionSpec};

  fn row_text(frame: &crate::host_engine::services::ComposedFrame, y: u16) -> String {
    (0..6)
      .map(|x| match frame.get(x, y).unwrap() {
        ComposedCell::Text(cell) => cell.text.as_str(),
        ComposedCell::Empty => " ",
      })
      .collect()
  }

  #[test]
  fn lua_slice_draws_are_clipped_in_game_and_screensaver_frames() {
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let directory = std::env::temp_dir().join(format!(
      "tg_slice_composition_{}_{}",
      std::process::id(),
      suffix
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let entry_path = directory.join("main.lua");
    std::fs::write(
      &entry_path,
      r#"
        function Init(ctx)
          layer = slice.create(10, 5)
        end
        function HandleEvent(event) end
        function Update(dt) end
        function UpdateFrame(dt, alpha) end
        function Render()
          slice.draw(layer, -4, 1)
          draw.text(0, 0, "abcdefghij", { slice_layer = layer })
          draw.fill_rect(0, 1, 10, 1, { char = "F", slice_layer = layer })
          draw.stroke_rect(0, 2, 10, 3, {
            border_char = {
              left_top = "A", top = "T", right_top = "C",
              left = "L", right = "R",
              left_bottom = "D", bottom = "B", right_bottom = "E",
            },
            slice_layer = layer,
          })
          draw.erase_rect(4, 0, 1, 1, { slice_layer = layer })
          draw.text(8, 1, "BASE")
        end
      "#,
    )
    .unwrap();

    for kind in [LuaSessionKind::Game, LuaSessionKind::Screensaver] {
      let mut session = LuaService::with_policy(LuaPolicy::default())
        .create_session(LuaSessionSpec {
          package_id: format!("slice_test_{}", kind.as_str()),
          session_kind: kind,
          entry_path: entry_path.clone(),
          fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
          base_size: Size {
            width: 12,
            height: 6,
          },
          continue_data: None,
          best_data: None,
          save_game_enabled: false,
          save_best_enabled: false,
        })
        .unwrap();
      session
        .with_objects_mut(|objects| objects.begin_frame())
        .unwrap();
      session.render().unwrap();
      let commands = session.take_draw_commands();

      let mut layout = LayoutService::new();
      layout.resize_physical(12, 6);
      let mut canvas = CanvasService::new();
      canvas.begin_frame(&layout);
      session
        .with_objects(|objects| objects.ui().prepare_canvas(&mut canvas, &layout))
        .unwrap();
      apply_lua_draw_commands(&mut RenderService::new(), &mut canvas, commands);
      let frame = FrameCompositor::new().compose(&canvas);

      assert_eq!((frame.width(), frame.height()), (12, 6));
      assert_eq!(row_text(&frame, 1), " fghij");
      assert_eq!(row_text(&frame, 2), "FFFFFF");
      assert_eq!(row_text(&frame, 3), "TTTTTC");
      assert_eq!(row_text(&frame, 4), "     R");
      assert_eq!(row_text(&frame, 5), "BBBBBE");
      let base_tail = (8..12)
        .map(|x| match frame.get(x, 1).unwrap() {
          ComposedCell::Text(cell) => cell.text.as_str(),
          ComposedCell::Empty => " ",
        })
        .collect::<String>();
      assert_eq!(base_tail, "BASE");
    }

    std::fs::remove_dir_all(directory).unwrap();
  }
}
