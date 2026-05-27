// 屏幕覆盖程序
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
  Screensaver, // 屏保
  Boss // 老板界面
}

// 覆盖层状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlaySessionState {
  Inactive, // 未激活
  Running // 激活
}

pub struct OverlayService {
  screensaver_state: OverlaySessionState, // 屏保状态
  boss_state: OverlaySessionState, // 屏保状态
  active_screensaver_uid: Option<String>, // 屏保id
  active_boss_uid: Option<String> // 老板界面id
}

impl OverlayService {
  pub fn new() -> Self {
    Self {
      screensaver_state: OverlaySessionState::Inactive,
      boss_state: OverlaySessionState::Inactive,
      active_screensaver_uid: None,
      active_boss_uid: None
    }
  }

  // 启动
  pub fn start(&mut self, kind: OverlayKind, package_uid: &str) {
    match kind {
      OverlayKind::Screensaver => {
        self.active_screensaver_uid = Some(package_uid.to_string());
        self.screensaver_state = OverlaySessionState::Running;
      }
      OverlayKind::Boss => {
        self.active_boss_uid = Some(package_uid.to_string());
        self.boss_state = OverlaySessionState::Running;
      }
    }
  }

  // 关闭
  pub fn stop(&mut self, kind: OverlayKind) {
    match kind {
      OverlayKind::Screensaver => {
        self.active_screensaver_uid = None;
        self.screensaver_state = OverlaySessionState::Inactive;
      }
      OverlayKind::Boss => {
        self.active_boss_uid = None;
        self.boss_state = OverlaySessionState::Inactive;
      }
    }
  }

  // 查看哪个覆盖界面正在运行
  pub fn is_active(&self, kind: OverlayKind) -> bool {
    match kind {
      OverlayKind::Screensaver => self.screensaver_state == OverlaySessionState::Running,
      OverlayKind::Boss => self.boss_state == OverlaySessionState::Running
    }
  }

  // 查看是否有覆盖界面正在运行
  pub fn any_active(&self) -> bool {
    self.is_active(OverlayKind::Screensaver) || self.is_active(OverlayKind::Boss)
  }

  // 更新
  pub fn update(&mut self) {}
}