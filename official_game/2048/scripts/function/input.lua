local C = load_function("/constants.lua")
local B = load_function("/board.lua")
local S = load_function("/storage.lua")
local State = load_function("/state.lua")

local M = {}

function M.normalize_key_name(event)
  if type(event) ~= "table" then
    return ""
  end
  if event.type == "key" and type(event.name) == "string" then
    return string.lower(event.name)
  end
  if event.type ~= "action" then
    return ""
  end
  local map = {
    move_left = "left",
    move_right = "right",
    move_up = "up",
    move_down = "down",
    save = "save",
    restart = "restart",
    quit_action = "quit_action",
    confirm_yes = "confirm_yes",
    confirm_no = "confirm_no",
  }
  return map[event.name] or ""
end

local function handle_confirm_key(state, key)
  if key == "confirm_yes" then
    if state.confirm_mode == "game_over" or state.confirm_mode == "restart" then
      State.reset_game(state)
      return "changed"
    end
    if state.confirm_mode == "exit" then
      S.commit_stats(state)
      return "exit"
    end
  end

  if key == "confirm_no" or key == "quit_action" then
    if state.confirm_mode == "game_over" then
      S.commit_stats(state)
      return "exit"
    end
    state.confirm_mode = nil
    state.dirty = true
    return "changed"
  end
  return "none"
end

local function reconcile_game_over_state(state)
  if state.confirm_mode == "game_over" and B.can_move_any(state.board) then
    state.game_over = false
    state.confirm_mode = nil
    state.end_frame = nil
    state.dirty = true
  end
end

local function apply_direction_key(key)
  local map = {
    move_up = "up",
    move_down = "down",
    move_left = "left",
    move_right = "right",
    up = "up",
    down = "down",
    left = "left",
    right = "right",
  }
  return map[key]
end

local function is_move_key(key)
  return apply_direction_key(key) ~= nil
end

local function should_debounce(state, key)
  if not is_move_key(key) then
    return false
  end
  if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
    return true
  end
  state.last_key = key
  state.last_key_frame = state.frame
  return false
end

function M.handle_input(state, key)
  if key == nil or key == "" then
    return "none"
  end
  if should_debounce(state, key) then
    return "none"
  end

  reconcile_game_over_state(state)

  if state.confirm_mode ~= nil then
    return handle_confirm_key(state, key)
  end

  if state.won then
    if key == "restart" then
      State.reset_game(state)
      return "changed"
    end
    if key == "quit_action" then
      S.commit_stats(state)
      return "exit"
    end
    return "none"
  end

  if key == "restart" then
    state.confirm_mode = "restart"
    state.dirty = true
    return "changed"
  end
  if key == "quit_action" then
    state.confirm_mode = "exit"
    state.dirty = true
    return "changed"
  end
  if key == "save" then
    S.request_game_save(state, true)
    return "changed"
  end

  if state.game_over then
    return "none"
  end

  local dir = apply_direction_key(key)
  if dir == nil then
    return "none"
  end

  local moved = B.apply_move(state, dir)
  if moved then
    B.spawn_tile(state.board)
    State.update_win_and_loss(state)
    state.dirty = true
    return "changed"
  end

  if not B.can_move_any(state.board) and not state.game_over then
    state.game_over = true
    state.confirm_mode = "game_over"
    state.end_frame = state.frame
    state.dirty = true
    S.commit_stats(state)
    return "changed"
  end

  return "none"
end

local function auto_save_if_needed(state)
  local elapsed = S.elapsed_seconds(state)
  if elapsed - state.last_auto_save_sec >= 60 then
    S.request_game_save(state, false)
    state.last_auto_save_sec = elapsed
  end
end

local function refresh_dirty_flags(state)
  local elapsed = math.floor((state.frame - state.start_frame) / C.FPS)
  if elapsed ~= state.last_elapsed_sec then
    state.last_elapsed_sec = elapsed
    state.dirty = true
  end
  local win_visible = state.frame <= state.win_message_until
  if win_visible ~= state.last_win_visible then
    state.last_win_visible = win_visible
    state.dirty = true
  end
  local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
  if toast_visible ~= state.last_toast_visible then
    state.last_toast_visible = toast_visible
    state.dirty = true
  end
end

local function sync_terminal_resize(state)
  local w, h = get_terminal_size()
  if w ~= state.last_term_w or h ~= state.last_term_h then
    state.last_term_w = w
    state.last_term_h = h
    state.last_area = nil
    state.dirty = true
  end
end

function M.handle_event(state, event)
  if event.type == "resize" then
    state.last_term_w = event.width or state.last_term_w
    state.last_term_h = event.height or state.last_term_h
    state.last_area = nil
    state.dirty = true
    return state
  end

  if event.type == "tick" then
    state.frame = state.frame + 1
    auto_save_if_needed(state)
    refresh_dirty_flags(state)
    sync_terminal_resize(state)
    return state
  end

  local action = M.handle_input(state, M.normalize_key_name(event))
  if action == "exit" then
    request_exit()
  end
  return state
end

return M
