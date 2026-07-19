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
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  export_settings_ui: &mut ExportSettingsUi,
  screenshot_capture_ui: &mut ScreenshotCaptureUi,
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
    services
      .canvas
      .prepare(window_size_ui.objects(), &services.layout);
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

  if world.state.current_overlay_kind() == Some(OverlayKind::Screensaver) {
    apply_host_viewport(services, false);
    screensaver_overlay_ui.objects_mut().begin_render();
    services
      .canvas
      .prepare(screensaver_overlay_ui.objects(), &services.layout);
    screensaver_overlay_ui.render(
      &mut services.render,
      &mut services.canvas,
      &services.layout,
      &services.i18n,
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
    apply_host_viewport(services, false);
    language_loading_ui.objects_mut().begin_render();
    services
      .canvas
      .prepare(language_loading_ui.objects(), &services.layout);
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
    services
      .canvas
      .prepare(export_loading_ui.objects(), &services.layout);
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
    services
      .canvas
      .prepare(safe_mode_warning_ui.objects(), &services.layout);
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
    services
      .canvas
      .prepare(clear_warning_ui.objects(), &services.layout);
    clear_warning_ui.render(
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
    services
      .canvas
      .prepare(export_settings_ui.objects(), &services.layout);
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
    services.canvas.prepare(objects, &services.layout);
  }

  let mut input_cursor = None;
  match world.state.current_ui_kind() {
    Some(UiNodeKind::Home) => {
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
        capabilities.mouse,
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
        &services.scroll_box,
      );
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
