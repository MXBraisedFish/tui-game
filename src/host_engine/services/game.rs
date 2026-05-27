// 游戏状态枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameSessionState {
  Inactive, // 未激活（没有游戏运行）
  Running, // 运行
  Paused // 暂停
}

// 游戏服务结构体
pub struct GameService {
  state: GameSessionState, // 游戏状态
  active_package_uid: Option<String> // 当前包运行id
}

impl GameService {
  pub fn new() -> Self {
    Self {
      state: GameSessionState::Inactive,
      active_package_uid: None
    }
  }

  // 启动
  pub fn start(&mut self, package_uid: &str) {
    self.active_package_uid = Some(package_uid.to_string());
    self.state = GameSessionState::Running;
  }

  // 停止
  pub fn stop(&mut self) {
    self.active_package_uid = None;
    self.state = GameSessionState::Inactive;
  }

  // 查询状态
  pub fn state(&self) -> GameSessionState {
    self.state
  }

  // 更新
  pub fn update(&mut self) {}
}