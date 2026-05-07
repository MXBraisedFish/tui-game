local M = {}

function M.best_score(state)
  local score = state.best_score or state.score or 0
  return {
    best_string = translate("test_game2.label.best") .. ": {score}",
    score = score,
    saved_at = now()
  }
end

function M.save_state(state)
  return {
    player = state.player,
    star = state.star,
    score = state.score or 0,
    best_score = state.best_score or 0,
    moves = state.moves or 0,
    saved_count = state.saved_count or 0,
    created_at = state.created_at,
    saved_at = now()
  }
end

return M