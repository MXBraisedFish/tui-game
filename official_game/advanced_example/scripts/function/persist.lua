local M = {}

function M.best_score(game_state)
  return {
    best_string = "Best score: {score}",
    score = game_state.score or 0
  }
end

function M.save_game(game_state)
  return {
    player = game_state.player,
    star = game_state.star,
    score = game_state.score,
    moves = game_state.moves,
    message = game_state.message,
    started_at = game_state.started_at
  }
end

return M
