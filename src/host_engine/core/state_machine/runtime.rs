use super::{HostState, MainHostState, OverlayStackState};

/// 运行时状态，包含主宿主和覆盖层栈
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeState {
  pub main_host: MainHostState,
  pub overlays: OverlayStackState,
}

impl RuntimeState {

  /// 创建以 Host 模式启动的运行时状态
  pub fn new_host_runtime() -> Self {
    Self {
      main_host: MainHostState::Host(HostState::new()),
      overlays: OverlayStackState::new(),
    }
  }

  pub fn main_host(&self) -> &MainHostState {
    &self.main_host
  }

  pub fn main_host_mut(&mut self) -> &mut MainHostState {
    &mut self.main_host
  }

  pub fn overlays(&self) -> &OverlayStackState {
    &self.overlays
  }

  pub fn overlays_mut(&mut self) -> &mut OverlayStackState {
    &mut self.overlays
  }

  pub fn has_overlay(&self) -> bool {
    !self.overlays.stack.is_empty()
  }

  pub fn set_main_host(&mut self, main_host: MainHostState) {
    self.main_host = main_host;
  }
}
