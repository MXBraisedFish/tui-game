local C = load_function("/constants.lua")
local B = load_function("/board.lua")
local S = load_function("/storage.lua")

local M = {}

function M.reset_game(state)
  state.board = B.empty()
  state.score = 0
  state.game_over = false
  state.won = false
  state.confirm_mode = nil
  state.start_frame = state.frame
  state.last_auto_save_sec = 0
  state.toast_text = nil
  state.toast_until = 0
  state.win_message_until = 0
  state.end_frame = nil
  B.spawn_tile(state.board)
  B.spawn_tile(state.board)
  state.dirty = true
end

function M.update_win_and_loss(state)
  local was_won = state.won
  state.won = B.has_target_tile(state.board)
  if state.won then
    state.win_message_until = state.frame + 3 * C.FPS
    if not was_won then
      state.end_frame = state.frame
      S.commit_stats(state)
    end
  end
end

function M.new_runtime_state()
  local state = {
    board = B.empty(),
    score = 0,
    game_over = false,
    won = false,
    confirm_mode = nil,
    frame = 0,
    start_frame = 0,
    win_message_until = 0,
    last_auto_save_sec = 0,
    toast_text = nil,
    toast_until = 0,
    dirty = true,
    last_elapsed_sec = -1,
    last_win_visible = false,
    last_toast_visible = false,
    last_key = "",
    last_key_frame = -100,
    launch_mode = "new",
    last_area = nil,
    end_frame = nil,
    last_term_w = 0,
    last_term_h = 0,
    best_score = 0,
    best_time_sec = 0,
  }
  state.last_term_w, state.last_term_h = get_terminal_size()
  return state
end

function M.init(saved_state)
  local state = M.new_runtime_state()
  S.load_best_record(state)
  local mode = string.lower(tostring(get_launch_mode() or "new"))
  state.launch_mode = mode == "continue" and "continue" or "new"
  if state.launch_mode == "continue" and S.restore_snapshot(state, saved_state) then
    M.update_win_and_loss(state)
  else
    M.reset_game(state)
  end
  state.dirty = true
  return state
end

return M
