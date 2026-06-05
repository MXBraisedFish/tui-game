use super::{ExecutionContext, HostSurface, OverlayStack, UiTree};

pub struct RuntimeSession {
  execution_context: ExecutionContext,
  host_surface: HostSurface,
  ui_tree: UiTree,
  overlay_stack: OverlayStack,
}

impl RuntimeSession {
  pub fn new() -> Self {
    Self {
      execution_context: ExecutionContext::Host,
      host_surface: HostSurface::MainMenu,
      ui_tree: UiTree::new(),
      overlay_stack: OverlayStack::new(),
    }
  }

  pub fn execution_context(&self) -> ExecutionContext {
    self.execution_context
  }

  pub fn host_surface(&self) -> HostSurface {
    self.host_surface
  }

  pub fn ui_tree(&self) -> &UiTree {
    &self.ui_tree
  }

  pub fn overlay_stack(&self) -> &OverlayStack {
    &self.overlay_stack
  }

  pub fn set_execution_context(&mut self, execution_context: ExecutionContext) {
    self.execution_context = execution_context;
  }

  pub fn set_host_surface(&mut self, host_surface: HostSurface) {
    self.host_surface = host_surface;
  }

  pub fn ui_tree_mut(&mut self) -> &mut UiTree {
    &mut self.ui_tree
  }

  pub fn overlay_stack_mut(&mut self) -> &mut OverlayStack {
    &mut self.overlay_stack
  }
}
