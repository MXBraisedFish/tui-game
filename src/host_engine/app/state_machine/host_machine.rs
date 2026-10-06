//! Lifecycle transitions and application page/overlay access through the host state machine.

use super::{
  MainHostState, OverlayKind, OverlayState, RuntimeClosingState, RuntimeState, UiNodeKind,
  UiNodeState,
};
use crate::host_engine::core::CrashPhase;

/// The retained application lifecycle state and active runtime branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostMachineState {
  /// The operation is boot.
  Boot,
  /// The operation is init.
  Init,
  /// The operation is runtime.
  Runtime(RuntimeState),
  /// The operation is shutdown.
  Shutdown,
  /// The operation is stopped.
  Stopped,
}

impl HostMachineState {
  /// Create a host machine state with its initial state.
  pub fn new() -> Self {
    HostMachineState::Boot
  }

  /// Report whether this host machine state is runtime.
  pub fn is_runtime(&self) -> bool {
    matches!(self, HostMachineState::Runtime(_))
  }

  /// Report whether this host machine state is shutdown.
  pub fn is_shutdown(&self) -> bool {
    matches!(self, HostMachineState::Shutdown)
  }

  /// Report whether this host machine state is stopped.
  pub fn is_stopped(&self) -> bool {
    matches!(self, HostMachineState::Stopped)
  }

  /// Return the current runtime.
  pub fn runtime(&self) -> Option<&RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  /// Return mutable access to the owned runtime.
  pub fn runtime_mut(&mut self) -> Option<&mut RuntimeState> {
    match self {
      HostMachineState::Runtime(runtime) => Some(runtime),
      _ => None,
    }
  }

  /// Return the current crash phase.
  pub fn crash_phase(&self) -> CrashPhase {
    match self {
      HostMachineState::Boot => CrashPhase::Boot,
      HostMachineState::Init => CrashPhase::Init,
      HostMachineState::Runtime(_) => CrashPhase::Runtime,
      HostMachineState::Shutdown => CrashPhase::Shutdown,
      HostMachineState::Stopped => CrashPhase::Stopped,
    }
  }

  /// Initialize runtime state for the application lifecycle.
  pub fn enter_init(&mut self) {
    *self = HostMachineState::Init;
  }

  /// Enter the runtime lifecycle state, creating runtime state when needed.
  pub fn enter_runtime(&mut self) {
    *self = HostMachineState::Runtime(RuntimeState::new_host_runtime());
  }

  /// Enter the shutdown lifecycle state while retaining closing context.
  pub fn enter_shutdown(&mut self) {
    *self = HostMachineState::Shutdown;
  }

  /// Record a normal shutdown request for lifecycle processing.
  pub fn request_shutdown(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.request_close();
    }
  }

  /// Record an exceptional shutdown request for fault handling.
  pub fn request_exception_shutdown(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.request_exception_close();
    }
  }

  /// Return the current closing state.
  pub fn closing_state(&self) -> Option<RuntimeClosingState> {
    self.runtime()?.closing()
  }

  /// Update the closing state used by this host machine state.
  pub fn set_closing_state(&mut self, state: RuntimeClosingState) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.set_closing(state);
    }
  }

  /// Clear a pending shutdown request so runtime interaction can continue.
  pub fn cancel_shutdown_request(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.cancel_close();
    }
  }

  /// Mark the application lifecycle as stopped.
  pub fn enter_stopped(&mut self) {
    *self = HostMachineState::Stopped;
  }

  /// Return the current UI kind.
  pub fn current_ui_kind(&self) -> Option<UiNodeKind> {
    let runtime = self.runtime()?;
    let MainHostState::Host(host) = runtime.main_host() else {
      return None;
    };
    let node = host.ui_tree().current()?;
    Some(node.kind)
  }

  /// Return the current UI path kinds.
  pub fn current_ui_path_kinds(&self) -> Vec<UiNodeKind> {
    let Some(runtime) = self.runtime() else {
      return Vec::new();
    };
    let MainHostState::Host(host) = runtime.main_host() else {
      return Vec::new();
    };
    host.ui_tree().path().iter().map(|node| node.kind).collect()
  }

  /// Enter a program page through the runtime navigation tree.
  pub fn enter_ui_node(&mut self, node: UiNodeState) {
    if let Some(runtime) = self.runtime_mut()
      && let Some(host) = runtime.main_host_mut().host_mut()
    {
      host.ui_tree_mut().enter(node);
    }
  }

  /// Return to the previous program page when the navigation path can be shortened.
  pub fn pop_ui_node(&mut self) -> Option<UiNodeState> {
    self
      .runtime_mut()?
      .main_host_mut()
      .host_mut()?
      .ui_tree_mut()
      .back()
  }

  /// Return the current overlay kind.
  pub fn current_overlay_kind(&self) -> Option<OverlayKind> {
    self.runtime()?.overlays().current_kind()
  }

  /// Show the window size overlay through the prioritized runtime stack.
  pub fn push_window_size_overlay(&mut self, min_w: u64, min_h: u64) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::WindowSizeWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: min_w,
          required_height: min_h,
        },
      });
    }
  }

  /// Show the game warning overlay through the prioritized runtime stack.
  pub fn push_game_warning_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::GameWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the language loading overlay through the prioritized runtime stack.
  pub fn push_language_loading_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::LanguageLoading,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the clear warning overlay through the prioritized runtime stack.
  pub fn push_clear_warning_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ClearWarning,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the cover continue overlay through the prioritized runtime stack.
  pub fn push_cover_continue_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::CoverContinue,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the export settings overlay through the prioritized runtime stack.
  pub fn push_export_settings_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ExportSettings,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the export loading overlay through the prioritized runtime stack.
  pub fn push_export_loading_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ExportLoading,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the screenshot capture overlay through the prioritized runtime stack.
  pub fn push_screenshot_capture_overlay(&mut self) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::ScreenshotCapture,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: 0,
          required_height: 0,
        },
      });
    }
  }

  /// Show the screensaver overlay through the prioritized runtime stack.
  pub fn push_screensaver_overlay(&mut self, min_width: u32, min_height: u32) {
    if let Some(runtime) = self.runtime_mut() {
      runtime.overlays_mut().push(OverlayState {
        kind: OverlayKind::Screensaver,
        logic: super::OverlayLogicState,
        render: super::OverlayRenderState {
          required_width: u64::from(min_width),
          required_height: u64::from(min_height),
        },
      });
    }
  }

  /// Remove the currently highest-priority overlay and retain its transition.
  pub fn pop_overlay(&mut self) -> Option<OverlayState> {
    self.runtime_mut()?.overlays_mut().pop()
  }

  /// Remove an overlay of the requested kind and report whether it was present.
  pub fn remove_overlay_kind(&mut self, kind: OverlayKind) -> Option<OverlayState> {
    self.runtime_mut()?.overlays_mut().remove_kind(kind)
  }

  /// Drain and return the queued overlay transitions.
  pub fn take_overlay_transitions(&mut self) -> Vec<super::OverlayStackTransition> {
    self
      .runtime_mut()
      .map(|runtime| runtime.overlays_mut().drain_transitions())
      .unwrap_or_default()
  }

  /// Report whether this host machine state is host mode.
  pub fn is_host_mode(&self) -> bool {
    self
      .runtime()
      .map(|r| r.main_host().is_host())
      .unwrap_or(true)
  }

  /// Report whether this host machine state is game mode.
  pub fn is_game_mode(&self) -> bool {
    self
      .runtime()
      .is_some_and(|runtime| runtime.main_host().is_game())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normal_shutdown_moves_runtime_through_shutdown_to_stopped() {
    let mut state = HostMachineState::new();
    state.enter_init();
    state.enter_runtime();
    assert!(state.is_runtime());

    state.request_shutdown();
    assert_eq!(state.closing_state(), Some(RuntimeClosingState::Requested));

    state.enter_shutdown();
    assert!(state.is_shutdown());
    state.enter_stopped();
    assert!(state.is_stopped());
  }

  #[test]
  fn overlay_navigation_preserves_the_underlying_ui_path() {
    let mut state = HostMachineState::new();
    state.enter_runtime();
    state.enter_ui_node(UiNodeState::settings());
    let path_before = state.current_ui_path_kinds();

    state.push_language_loading_overlay();
    state.push_clear_warning_overlay();
    assert_eq!(
      state.current_overlay_kind(),
      Some(OverlayKind::ClearWarning)
    );
    assert_eq!(state.current_ui_path_kinds(), path_before);

    assert_eq!(
      state.pop_overlay().map(|overlay| overlay.kind),
      Some(OverlayKind::ClearWarning)
    );
    assert_eq!(
      state.current_overlay_kind(),
      Some(OverlayKind::LanguageLoading)
    );
    assert_eq!(state.current_ui_path_kinds(), path_before);

    state.remove_overlay_kind(OverlayKind::LanguageLoading);
    assert_eq!(state.current_overlay_kind(), None);
    assert_eq!(state.current_ui_path_kinds(), path_before);
  }
}
