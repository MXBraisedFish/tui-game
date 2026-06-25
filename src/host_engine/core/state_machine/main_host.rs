use super::{GameState, HostState};

/// 主宿主状态，区分当前运行的是 Host 界面还是 Game 游戏
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostState {
  Host(HostState),
  Game(GameState),
}

impl MainHostState {
  pub fn is_host(&self) -> bool {
    matches!(self, MainHostState::Host(_))
  }

  pub fn is_game(&self) -> bool {
    matches!(self, MainHostState::Game(_))
  }

  pub fn host(&self) -> Option<&HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  pub fn host_mut(&mut self) -> Option<&mut HostState> {
    match self {
      MainHostState::Host(host) => Some(host),
      _ => None,
    }
  }

  pub fn game(&self) -> Option<&GameState> {
    match self {
      MainHostState::Game(game) => Some(game),
      _ => None,
    }
  }

  pub fn game_mut(&mut self) -> Option<&mut GameState> {
    match self {
      MainHostState::Game(game) => Some(game),
      _ => None,
    }
  }

  pub fn switch_to_host(&mut self, host: HostState) {
    *self = MainHostState::Host(host);
  }

  pub fn switch_to_game(&mut self, game: GameState) {
    *self = MainHostState::Game(game);
  }
}
