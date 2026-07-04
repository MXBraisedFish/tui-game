use super::*;

pub(super) fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  let term = services.layout.physical_size();

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
      let runtime = world.state.runtime().unwrap();
      if let Some(overlay) = runtime.overlays().top() {
        let req_w = overlay.render.required_width as u16;
        let req_h = overlay.render.required_height as u16;
        if term.width >= req_w && term.height >= req_h {
          world.state.pop_overlay();
        }
      }
    }
    None => {
      let (min_w, min_h) = get_min_window_size(world);
      if (term.width as u32) < min_w || (term.height as u32) < min_h {
        world.state.push_window_size_overlay(min_w, min_h);
      }
    }
    _ => {}
  }
}

fn get_min_window_size(world: &RuntimeWorld) -> (u32, u32) {
  if world.state.is_host_mode() {
    (95, 24)
  } else {
    (95, 24)
  }
}

pub(super) fn apply_window_size_command(cmd: WindowSizeWarningCommand, world: &mut RuntimeWorld) {
  match cmd {
    WindowSizeWarningCommand::Exit => {
      if world.state.is_host_mode() {
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
