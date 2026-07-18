use super::*;

pub(super) fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    let _ = world
      .state
      .remove_overlay_kind(OverlayKind::WindowSizeWarning);
    return;
  }

  let term = services.layout.physical_size();
  let (min_w, min_h) = get_min_window_size(world);
  let too_small = (term.width as u32) < min_w || (term.height as u32) < min_h;

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
      let runtime = world.state.runtime().unwrap();
      if let Some(overlay) = runtime.overlays().top() {
        let req_w = overlay.render.required_width as u16;
        let req_h = overlay.render.required_height as u16;
        if term.width >= req_w && term.height >= req_h {
          world
            .state
            .remove_overlay_kind(OverlayKind::WindowSizeWarning);
        }
      }
    }
    _ if too_small => {
      world.state.push_window_size_overlay(min_w, min_h);
    }
    _ => {}
  }
}

fn get_min_window_size(world: &RuntimeWorld) -> (u32, u32) {
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
    (95, 24)
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
        world.state.enter_shutdown();
        set_crash_phase(world.state.crash_phase());
        world.state.enter_stopped();
        set_crash_phase(world.state.crash_phase());
      } else {
        world.state.pop_overlay();
        if let Some(runtime) = world.state.runtime_mut() {
          runtime.set_main_host(MainHostState::Host(HostState::new()));
        }
      }
    }
  }
}
