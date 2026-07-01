/// 游戏状态，持有游戏循环
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
  pub game_loop: GameLoopState,
}

/// 游戏循环状态
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLoopState;

impl GameState {
  pub fn game_loop(&self) -> &GameLoopState {
    &self.game_loop
  }

  pub fn game_loop_mut(&mut self) -> &mut GameLoopState {
    &mut self.game_loop
  }
}
