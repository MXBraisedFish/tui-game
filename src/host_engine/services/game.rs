/// 游戏会话状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState {
  Inactive,
  Running,
  Paused,
}

/// 游戏服务，管理游戏会话的启停
pub struct GameService {
  state: GameSessionState,
  active_package_uid: Option<String>,
}

impl GameService {
  pub fn new() -> Self {
    Self {
      state: GameSessionState::Inactive,
      active_package_uid: None,
    }
  }

  /// 启动指定包的游戏会话
  pub fn start(&mut self, package_uid: &str) {
    self.active_package_uid = Some(package_uid.to_string());
    self.state = GameSessionState::Running;
  }

  /// 停止当前游戏会话
  pub fn stop(&mut self) {
    self.active_package_uid = None;
    self.state = GameSessionState::Inactive;
  }

  pub fn state(&self) -> GameSessionState {
    self.state
  }

  pub fn update(&mut self) {}
}
