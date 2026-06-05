use super::{
  ExecutionContext,
  FocusState,
  HostSurface,
  OverlayKind,
  OverlayStack,
  RuntimeAction,
  RuntimeState,
  UiNode,
  UiTree,
};

pub struct RuntimeSession {
  runtime_state: RuntimeState,
  focus_state: FocusState,
  execution_context: ExecutionContext,
  host_surface: HostSurface,
  ui_tree: UiTree,
  overlay_stack: OverlayStack,
}

impl RuntimeSession {
  pub fn new() -> Self {
    Self {
      runtime_state: RuntimeState::Running,
      focus_state: FocusState::Focused,
      execution_context: ExecutionContext::Host,
      host_surface: HostSurface::MainMenu,
      ui_tree: UiTree::new(),
      overlay_stack: OverlayStack::new(),
    }
  }

  pub fn runtime_state(&self) -> RuntimeState {
    self.runtime_state
  }

  pub fn is_running(&self) -> bool {
    matches!(self.runtime_state, RuntimeState::Running)
  }

  pub fn request_stop(&mut self) {
    self.runtime_state = RuntimeState::Stopping;
  }

  pub fn focus_state(&self) -> FocusState {
    self.focus_state
  }

  pub fn is_terminal_focused(&self) -> bool {
    matches!(self.focus_state, FocusState::Focused)
  }

  pub fn set_terminal_focused(&mut self, focused: bool) {
    self.focus_state = if focused { FocusState::Focused } else { FocusState::Unfocused };
  }

  pub fn should_accept_keyboard_input(&self) -> bool {
    self.is_terminal_focused()
  }

  pub fn push_overlay(&mut self, overlay: OverlayKind) {
    self.overlay_stack.push(overlay);
  }

  pub fn pop_overlay(&mut self) -> Option<OverlayKind> {
    self.overlay_stack.pop()
  }

  pub fn top_overlay(&self) -> Option<OverlayKind> {
    self.overlay_stack.top()
  }

  pub fn clear_overlays(&mut self) {
    self.overlay_stack.clear();
  }

  pub fn has_overlay(&self) -> bool {
    !self.overlay_stack.is_empty()
  }

  pub fn is_overlay_active(&self) -> bool {
    self.has_overlay()
  }

  pub fn overlay_depth(&self) -> usize {
    self.overlay_stack().len()
  }

  pub fn handle_runtime_action(&mut self, action: RuntimeAction) {
    match action {
      RuntimeAction::Cancel => {
        if self.is_overlay_active() {
          self.pop_overlay();
        } else {
          self.request_stop();
        }
      }
      RuntimeAction::RequestStop => {
        self.request_stop();
      }
      RuntimeAction::CloseOverlay => {
        self.pop_overlay();
      }
      RuntimeAction::PushDebugOverlay => {
        self.push_overlay(OverlayKind::ConfirmExit);
      }
      RuntimeAction::PopDebugOverlay => {
        self.pop_overlay();
      }
    }
  }

  pub fn ui_path(&self) -> &[UiNode] {
    self.ui_tree.active_path()
  }

  pub fn current_ui_node(&self) -> Option<UiNode> {
    self.ui_tree.current()
  }

  pub fn enter_ui_node(&mut self, node: UiNode) {
    self.ui_tree.enter(node);
  }

  pub fn back_ui_node(&mut self) -> Option<UiNode> {
    self.ui_tree.back()
  }

  pub fn reset_ui_tree(&mut self) {
    self.ui_tree.reset();
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
