use super::*;
use super::{host_viewport::apply_host_viewport, router::current_objects_mut};
use crate::host_engine::services::UiObjectPoolOwner;

pub(super) fn route_render(
  services: &mut EngineServices,
  world: &RuntimeWorld,
  home_ui: &mut HomeUi,
  settings_ui: &mut SettingsUi,
  storage_management_ui: &mut StorageManagementUi,
  storage_management_clear_ui: &mut StorageManagementClearUi,
  storage_management_export_ui: &mut StorageManagementExportUi,
  storage_management_view_ui: &mut StorageManagementViewUi,
  mut language_select_ui: Option<&mut LanguageSelectUi>,
  terminal_check_ui: &mut TerminalCheckUi,
  mods_ui: &mut ModsUi,
  game_package_ui: &mut GamePackageUi,
  screensaver_package_ui: &mut ScreensaverPackageUi,
  input_demo_ui: &mut InputDemoUi,
  window_size_ui: &mut WindowSizeWarningUi,
  safe_mode_warning_ui: &mut SafeModeWarningUi,
  clear_warning_ui: &mut ClearWarningUi,
  language_loading_ui: &mut LanguageLoadingUi,
) -> Option<(u16, u16)> {
  if let Some(OverlayKind::WindowSizeWarning) = world.state.current_overlay_kind() {
    apply_host_viewport(services);
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
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::LanguageLoading) {
    apply_host_viewport(services);
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

  if world.state.current_overlay_kind() == Some(OverlayKind::SafeModeWarning) {
    apply_host_viewport(services);
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
    );
    return None;
  }

  if world.state.current_overlay_kind() == Some(OverlayKind::ClearWarning) {
    apply_host_viewport(services);
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

  apply_host_viewport(services);

  if let Some(objects) = current_objects_mut(
    world,
    home_ui,
    settings_ui,
    storage_management_ui,
    storage_management_clear_ui,
    storage_management_export_ui,
    storage_management_view_ui,
    language_select_ui.as_deref_mut(),
    terminal_check_ui,
    mods_ui,
    game_package_ui,
    screensaver_package_ui,
    input_demo_ui,
  ) {
    objects.begin_render();
    services.canvas.prepare(objects, &services.layout);
  }

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
    Some(UiNodeKind::GamePackage) => {
      let mouse_supported = services.terminal.capabilities().mouse;
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
        &world.temporary_safe_mode_disabled,
        &mut services.image,
        mouse_supported,
      );
    }
    Some(UiNodeKind::ScreensaverPackage) => {
      let mouse_supported = services.terminal.capabilities().mouse;
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
        &mut services.image,
        mouse_supported,
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

  None
}
