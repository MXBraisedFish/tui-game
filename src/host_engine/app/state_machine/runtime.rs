use super::{HostState, MainHostState, OverlayStackState};

pub const EXCEPTION_EXIT_COUNTDOWN_SECONDS: u8 = 3;

/// Runtime 仍保持存活时的退出准备状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeClosingState {
  Requested,
  ExportWarning,
  WaitingForExports,
  /// 强制退出已确认，保留一帧用于呈现服务停止提示。
  Stopping {
    waiting_for_exports: bool,
  },
  Exception {
    seconds_left: u8,
  },
}

/// 运行时状态，包含主宿主和覆盖层栈
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeState {
  pub main_host: MainHostState,
  pub overlays: OverlayStackState,
  pub closing: Option<RuntimeClosingState>,
}

impl RuntimeState {
  /// 创建以 Host 模式启动的运行时状态
  pub fn new_host_runtime() -> Self {
    Self {
      main_host: MainHostState::Host(HostState::new()),
      overlays: OverlayStackState::new(),
      closing: None,
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

  pub fn set_main_host(&mut self, main_host: MainHostState) {
    self.main_host = main_host;
  }

  pub fn closing(&self) -> Option<RuntimeClosingState> {
    self.closing
  }

  pub fn request_close(&mut self) {
    self.closing = Some(RuntimeClosingState::Requested);
  }

  pub fn set_closing(&mut self, state: RuntimeClosingState) {
    self.closing = Some(state);
  }

  pub fn request_exception_close(&mut self) {
    self.closing = Some(RuntimeClosingState::Exception {
      seconds_left: EXCEPTION_EXIT_COUNTDOWN_SECONDS,
    });
  }

  pub fn cancel_close(&mut self) {
    self.closing = None;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shutdown_request_keeps_runtime_alive_until_preparation_finishes() {
    let mut state = RuntimeState::new_host_runtime();
    state.request_close();
    assert_eq!(state.closing(), Some(RuntimeClosingState::Requested));

    state.set_closing(RuntimeClosingState::ExportWarning);
    assert_eq!(state.closing(), Some(RuntimeClosingState::ExportWarning));

    state.cancel_close();
    assert_eq!(state.closing(), None);
  }

  #[test]
  fn exception_shutdown_always_starts_with_three_seconds() {
    let mut state = RuntimeState::new_host_runtime();
    state.request_exception_close();
    assert_eq!(
      state.closing(),
      Some(RuntimeClosingState::Exception { seconds_left: 3 })
    );
  }
}
