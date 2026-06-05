use crate::host_engine::core::{ExecutionContext, HostSurface, RuntimeSession, UiNode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeKeyboardContext {
  overlay_active: bool,
  execution_context: ExecutionContext,
  host_surface: HostSurface,
  current_ui_node: Option<UiNode>,
}

impl RuntimeKeyboardContext {
  pub fn from_session(session: &RuntimeSession) -> Self {
    Self {
      overlay_active: session.is_overlay_active(),
      execution_context: session.execution_context(),
      host_surface: session.host_surface(),
      current_ui_node: session.current_ui_node(),
    }
  }

  pub fn is_overlay_active(&self) -> bool {
    self.overlay_active
  }

  pub fn execution_context(&self) -> ExecutionContext {
    self.execution_context
  }

  pub fn host_surface(&self) -> HostSurface {
    self.host_surface
  }

  pub fn current_ui_node(&self) -> Option<UiNode> {
    self.current_ui_node
  }
}
