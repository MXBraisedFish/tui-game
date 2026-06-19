// 屏幕覆盖程序（仅屏保）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlaySessionState {
  Inactive,
  Running,
}

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

  pub fn start(&mut self, package_uid: &str) {
    self.active_screensaver_uid = Some(package_uid.to_string());
    self.state = OverlaySessionState::Running;
  }

  pub fn stop(&mut self) {
    self.active_screensaver_uid = None;
    self.state = OverlaySessionState::Inactive;
  }

  pub fn is_active(&self) -> bool {
    self.state == OverlaySessionState::Running
  }

  pub fn update(&mut self) {}
}
