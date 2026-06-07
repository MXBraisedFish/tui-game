use super::{GameState, HostState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostState {
  Host(HostState),
  Game(GameState),
}

// MainHost状态
impl MainHostState {
  // 宿主状态查询方法
  pub fn is_host(&self) -> bool {
    matches!(self, MainHostState::Host(_))
  }

  // 游戏状态查询方法
  pub fn is_game(&self) -> bool {
    matches!(self, MainHostState::Game(_))
  }

  // 宿主状态访问方法
  pub fn host(&self) -> Option<&HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  // 宿主状态访问方法（可变）
  pub fn host_mut(&mut self) -> Option<&mut HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  // 游戏状态访问方法
  pub fn game(&self) -> Option<&GameState> {
    match self {
      MainHostState::Game(game) => Some(game),
      _ => None,
    }
  }

  // 游戏状态访问方法（可变）
  pub fn game_mut(&mut self) -> Option<&mut GameState> {
    match self {
      MainHostState::Game(game) => Some(game),
      _ => None,
    }
  }
}
