local C = load_function("/constants.lua")
local U = load_function("/utils.lua")
local B = load_function("/board.lua")

local M = {}

function M.elapsed_seconds(state)
  local end_frame = state.end_frame
  if end_frame == nil then
    end_frame = state.frame
  end
  return math.floor((end_frame - state.start_frame) / C.FPS)
end

function M.make_snapshot(state)
  return {
    board = B.deep_copy(state.board),
    score = state.score,
    elapsed_sec = math.floor((state.frame - state.start_frame) / C.FPS),
  }
end

function M.restore_snapshot(state, snapshot)
  if type(snapshot) ~= "table" then
    return false
  end
  local board = B.normalize_board(snapshot.board)
  if board == nil then
    return false
  end

  local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
  state.board = board
  state.score = math.max(0, math.floor(tonumber(snapshot.score) or 0))
  state.start_frame = state.frame - math.floor(elapsed * C.FPS)
  state.last_auto_save_sec = elapsed
  state.game_over = false
  state.won = false
  state.confirm_mode = nil
  state.win_message_until = 0
  state.toast_text = nil
  state.toast_until = 0
  state.end_frame = nil
  state.dirty = true
  return true
end

function M.load_best_record(state)
  local data = get_best_score()
  if type(data) ~= "table" then
    state.best_score = 0
    state.best_time_sec = 0
    return
  end
  state.best_score = math.max(0, math.floor(tonumber(data.score) or 0))
  state.best_time_sec = math.max(0, math.floor(tonumber(data.time_sec) or 0))
end

function M.best_score_payload(state)
  if state.best_score <= 0 then
    return {
      best_string = "game.2048.best_none_block",
      score = 0,
      time = "--:--:--",
      time_sec = 0,
    }
  end
  return {
    best_string = "game.2048.best_block",
    score = state.best_score,
    time = U.format_duration(state.best_time_sec),
    time_sec = state.best_time_sec,
  }
end

function M.save_game_payload(state)
  return M.make_snapshot(state)
end

function M.request_game_save(state, show_toast)
  request_save_game()
  if show_toast then
    state.toast_text = U.tr("game.2048.save_success")
    state.toast_until = state.frame + 2 * C.FPS
    state.dirty = true
  end
end

function M.commit_stats(state)
  local score = tonumber(state.score) or 0
  local duration = M.elapsed_seconds(state)
  if score > state.best_score or (score == state.best_score and score > 0 and (state.best_time_sec == 0 or duration < state.best_time_sec)) then
    state.best_score = score
    state.best_time_sec = duration
    request_save_best_score()
  end
end

return M
