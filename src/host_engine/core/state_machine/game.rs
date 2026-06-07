#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub game_loop: GameLoopState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameLoopState;

// 游戏状态
impl GameState {
    pub fn game_loop(&self) -> &GameLoopState {
        &self.game_loop
    }

    pub fn game_loop_mut(&mut self) -> &mut GameLoopState {
        &mut self.game_loop
    }
}
