use super::*;

pub(super) fn manage_window_size_overlay(services: &EngineServices, world: &mut RuntimeWorld) {
  if world.state.current_overlay_kind() == Some(OverlayKind::ScreenshotCapture) {
    let _ = world
      .state
      .remove_overlay_kind(OverlayKind::WindowSizeWarning);
    return;
  }

  let physical = services.layout.physical_size();
  let required_base = get_min_base_size(services, world);
  let screensaver_overlay_active = world
    .state
    .runtime()
    .is_some_and(|runtime| runtime.overlays().get(OverlayKind::Screensaver).is_some());
  let ordinary_overlay_active = has_program_overlay(world);
  let top_toolbar = !screensaver_overlay_active
    && !ordinary_overlay_active
    && services.storage.display_settings_profile().top_toolbar;
  let current_base = host_viewport::developer_size(physical, top_toolbar);
  let too_small = base_size_is_too_small(current_base, required_base);
  let required_terminal = host_viewport::required_physical_size(required_base, top_toolbar);

  match world.state.current_overlay_kind() {
    Some(OverlayKind::WindowSizeWarning) => {
      if !too_small {
        world
          .state
          .remove_overlay_kind(OverlayKind::WindowSizeWarning);
      } else if let Some(overlay) = world.state.runtime_mut().and_then(|runtime| {
        runtime
          .overlays_mut()
          .get_mut(OverlayKind::WindowSizeWarning)
      }) {
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

fn base_size_is_too_small(current: Size, required: (u64, u64)) -> bool {
  u64::from(current.width) < required.0 || u64::from(current.height) < required.1
}

fn get_min_base_size(services: &EngineServices, world: &RuntimeWorld) -> (u64, u64) {
  let runtime = world.state.runtime();
  let screensaver_size = runtime
    .and_then(|runtime| runtime.overlays().get(OverlayKind::Screensaver))
    .map(|overlay| {
      (
        overlay.render.required_width,
        overlay.render.required_height,
      )
    });
  let ordinary_overlay_active = has_program_overlay(world);
  let game_size = runtime
    .and_then(|runtime| runtime.main_host().game())
    .map(|game| (u64::from(game.min_width), u64::from(game.min_height)));

  select_required_base_size(
    screensaver_size,
    services.screensaver.is_active(),
    ordinary_overlay_active,
    world.state.is_host_mode(),
    game_size,
  )
}

fn has_program_overlay(world: &RuntimeWorld) -> bool {
  world.state.runtime().is_some_and(|runtime| {
    runtime
      .overlays()
      .stack
      .iter()
      .any(|overlay| overlay.kind.is_program_overlay())
  })
}

fn select_required_base_size(
  screensaver_size: Option<(u64, u64)>,
  screensaver_script_active: bool,
  ordinary_overlay_active: bool,
  host_mode: bool,
  game_size: Option<(u64, u64)>,
) -> (u64, u64) {
  if let Some(size) = screensaver_size {
    return if screensaver_script_active {
      size
    } else {
      program_default_min_size()
    };
  }
  if ordinary_overlay_active || host_mode {
    return program_default_min_size();
  }
  game_size.unwrap_or_else(program_default_min_size)
}

fn program_default_min_size() -> (u64, u64) {
  let (width, height) = host_viewport::PROGRAM_DEFAULT_MIN_SIZE;
  (u64::from(width), u64::from(height))
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

  #[test]
  fn required_size_uses_program_defaults_for_host_and_program_overlays() {
    let default = program_default_min_size();
    assert_eq!(
      select_required_base_size(None, false, false, true, None),
      default
    );
    assert_eq!(
      select_required_base_size(None, false, true, false, Some((140, 50))),
      default
    );
  }

  #[test]
  fn required_size_uses_game_or_script_screensaver_package_independently() {
    assert_eq!(
      select_required_base_size(None, false, false, false, Some((140, 50))),
      (140, 50)
    );
    assert_eq!(
      select_required_base_size(Some((30, 40)), true, false, false, Some((140, 50))),
      (30, 40)
    );
  }

  #[test]
  fn built_in_screensaver_uses_program_default_size() {
    assert_eq!(
      select_required_base_size(Some((0, 0)), false, false, true, None),
      program_default_min_size()
    );
  }

  #[test]
  fn zero_minimum_dimensions_are_legal_and_large_requirements_never_fit() {
    assert!(!base_size_is_too_small(
      Size {
        width: 0,
        height: 0
      },
      (0, 0)
    ));
    assert!(base_size_is_too_small(
      Size {
        width: u16::MAX,
        height: u16::MAX
      },
      (u64::from(u32::MAX), 0)
    ));
    assert_eq!(
      host_viewport::required_physical_size((u64::from(u32::MAX), u64::from(u32::MAX)), true),
      (u64::from(u32::MAX), u64::from(u32::MAX) + 2)
    );
  }

  #[test]
  fn toolbar_requirement_recovers_at_exact_physical_size_boundary() {
    let required_base = (30, 40);
    let required_terminal = host_viewport::required_physical_size(required_base, true);
    assert_eq!(required_terminal, (30, 42));
    assert!(base_size_is_too_small(
      host_viewport::developer_size(
        Size {
          width: 30,
          height: 41
        },
        true
      ),
      required_base
    ));
    assert!(!base_size_is_too_small(
      host_viewport::developer_size(
        Size {
          width: 30,
          height: 42
        },
        true
      ),
      required_base
    ));
    assert!(base_size_is_too_small(
      host_viewport::developer_size(
        Size {
          width: 29,
          height: 42
        },
        true
      ),
      required_base
    ));
    assert!(!base_size_is_too_small(
      host_viewport::developer_size(
        Size {
          width: 31,
          height: 43
        },
        true
      ),
      required_base
    ));
    assert!(!base_size_is_too_small(
      host_viewport::developer_size(
        Size {
          width: 30,
          height: 40
        },
        false
      ),
      required_base
    ));
  }
}
