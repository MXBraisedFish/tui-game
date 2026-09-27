use super::*;

pub(super) fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    let _ = world
      .state
      .remove_overlay_kind(OverlayKind::WindowSizeWarning);
    return;
  }

  let physical = services.layout.physical_size();
  let required_base = get_min_base_size(world);
  let screensaver_active = world
    .state
    .runtime()
    .is_some_and(|runtime| runtime.overlays().get(OverlayKind::Screensaver).is_some());
  let top_toolbar = !screensaver_active && services.storage.display_settings_profile().top_toolbar;
  let current_base = if screensaver_active {
    physical
  } else {
    host_viewport::developer_size(physical, top_toolbar)
  };
  let too_small = base_size_is_too_small(current_base, required_base);
  let required_terminal = host_viewport::required_physical_size(required_base, top_toolbar);

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
      if !too_small {
        world
          .state
          .remove_overlay_kind(OverlayKind::WindowSizeWarning);
      } else if let Some(overlay) = world
        .state
        .runtime_mut()
        .and_then(|runtime| runtime.overlays_mut().top_mut())
      {
        overlay.render.required_width = required_terminal.0;
        overlay.render.required_height = required_terminal.1;
      }
    }
    _ if too_small => {
      world
        .state
        .push_window_size_overlay(required_terminal.0, required_terminal.1);
    }
    _ => {}
  }
}

fn base_size_is_too_small(current: Size, required: (u32, u32)) -> bool {
  u32::from(current.width) < required.0 || u32::from(current.height) < required.1
}

fn get_min_base_size(world: &RuntimeWorld) -> (u32, u32) {
  if let Some(overlay) = world
    .state
    .runtime()
    .and_then(|runtime| runtime.overlays().get(OverlayKind::Screensaver))
  {
    return (
      overlay.render.required_width,
      overlay.render.required_height,
    );
  }
  if world.state.is_host_mode() {
    (95, 24)
  } else {
    world
      .state
      .runtime()
      .and_then(|runtime| runtime.main_host().game())
      .map_or((95, 24), |game| (game.min_width, game.min_height))
  }
}

pub(super) fn apply_window_size_command(cmd: WindowSizeWarningCommand, world: &mut RuntimeWorld) {
  match cmd {
    WindowSizeWarningCommand::Exit => {
      let screensaver_active = world
        .state
        .runtime()
        .is_some_and(|runtime| runtime.overlays().get(OverlayKind::Screensaver).is_some());
      if screensaver_active {
        let _ = world
          .state
          .remove_overlay_kind(OverlayKind::WindowSizeWarning);
        let _ = world.state.remove_overlay_kind(OverlayKind::Screensaver);
      } else if world.state.is_host_mode() {
        world.state.pop_overlay();
        world.state.request_shutdown();
      } else {
        world.state.pop_overlay();
        let return_host = world
          .state
          .runtime()
          .and_then(|runtime| runtime.main_host().game())
          .map(|game| (*game.return_host).clone())
          .unwrap_or_else(HostState::new);
        if let Some(runtime) = world.state.runtime_mut() {
          runtime.set_main_host(MainHostState::Host(return_host));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn window_warning_compares_the_base_canvas_instead_of_the_physical_terminal() {
    let physical = Size {
      width: 30,
      height: 40,
    };
    let base = host_viewport::developer_size(physical, true);

    assert_eq!(
      base,
      Size {
        width: 30,
        height: 38,
      }
    );
    assert!(base_size_is_too_small(base, (30, 40)));
    assert!(!base_size_is_too_small(physical, (30, 40)));
  }
}
