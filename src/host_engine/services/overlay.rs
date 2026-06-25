/// 覆盖层会话状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlaySessionState {
  Inactive,
  Running,
}

/// 覆盖层服务，管理屏保等覆盖层的启停
pub struct OverlayService {
  state: OverlaySessionState,
  active_screensaver_uid: Option<String>,
}

impl OverlayService {
  pub fn new() -> Self {
    Self {
      state: OverlaySessionState::Inactive,
      active_screensaver_uid: None,
    }
  }

  /// 启动指定包的屏保覆盖层
  pub fn start(&mut self, package_uid: &str) {
    self.active_screensaver_uid = Some(package_uid.to_string());
    self.state = OverlaySessionState::Running;
  }

  /// 停止当前覆盖层
  pub fn stop(&mut self) {
    self.active_screensaver_uid = None;
    self.state = OverlaySessionState::Inactive;
  }

  pub fn is_active(&self) -> bool {
    self.state == OverlaySessionState::Running
  }

  pub fn update(&mut self) {}
}
