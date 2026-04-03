local SIZE = 4

local function tr(key)
  return translate(key)
end

local function solved_board()
  local board = {}
  local v = 1
  for r = 1, SIZE do
    board[r] = {}
    for c = 1, SIZE do
      if r == SIZE and c == SIZE then
        board[r][c] = 0
      else
        board[r][c] = v
        v = v + 1
      end
    end
  end
  return board
end

local function clone_board(board)
  local out = {}
  for r = 1, SIZE do
    out[r] = {}
    for c = 1, SIZE do
      out[r][c] = board[r][c]
    end
  end
  return out
end

local function find_blank(board)
  for r = 1, SIZE do
    for c = 1, SIZE do
      if board[r][c] == 0 then
        return r, c
      end
    end
  end
  return SIZE, SIZE
end

local function can_move_blank(board, dir)
  local br, bc = find_blank(board)
  if dir == "up" then return br > 1 end
  if dir == "down" then return br < SIZE end
  if dir == "left" then return bc > 1 end
  if dir == "right" then return bc < SIZE end
  return false
end

local function move_blank(board, dir)
  local br, bc = find_blank(board)
  local tr, tc = br, bc
  if dir == "up" then
    tr = br - 1
  elseif dir == "down" then
    tr = br + 1
  elseif dir == "left" then
    tc = bc - 1
  elseif dir == "right" then
    tc = bc + 1
  else
    return false
  end
  if tr < 1 or tr > SIZE or tc < 1 or tc > SIZE then
    return false
  end
  board[br][bc], board[tr][tc] = board[tr][tc], board[br][bc]
  return true
end

local function opposite_dir(dir)
  if dir == "up" then return "down" end
  if dir == "down" then return "up" end
  if dir == "left" then return "right" end
  if dir == "right" then return "left" end
  return dir
end

local function shuffle_board(board)
  local dirs = { "up", "down", "left", "right" }
  local prev = ""
  local steps = 100
  for _ = 1, steps do
    local available = {}
    for i = 1, #dirs do
      local dir = dirs[i]
      if can_move_blank(board, dir) and dir ~= opposite_dir(prev) then
        available[#available + 1] = dir
      end
    end
    if #available == 0 then
      for i = 1, #dirs do
        if can_move_blank(board, dirs[i]) then
          available[#available + 1] = dirs[i]
        end
      end
    end
    local picked = available[math.random(1, #available)]
    move_blank(board, picked)
    prev = picked
  end
end

local function is_solved(board)
  local value = 1
  for r = 1, SIZE do
    for c = 1, SIZE do
      if r == SIZE and c == SIZE then
        return board[r][c] == 0
      end
      if board[r][c] ~= value then
        return false
      end
      value = value + 1
    end
  end
  return true
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_steps = math.max(0, math.floor(tonumber(best.steps) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(best.time_sec) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", {
    steps = state.best_steps,
    time_sec = state.best_time_sec,
  })
  request_refresh_best_score()
end

local function fresh_state()
  local state = {
    board = solved_board(),
    steps = 0,
    elapsed_ms = 0,
    won = false,
    move_mode = "blank",
    message = "game.sliding_puzzle.runtime_ready",
    best_steps = 0,
    best_time_sec = 0,
  }
  shuffle_board(state.board)
  if is_solved(state.board) then
    move_blank(state.board, "left")
  end
  load_best_record(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" or type(state.board) ~= "table" or #state.board ~= SIZE then
    return fresh_state()
  end
  state.steps = math.max(0, math.floor(tonumber(state.steps) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.won = state.won == true
  state.move_mode = state.move_mode == "number" and "number" or "blank"
  state.message = state.message or "game.sliding_puzzle.runtime_ready"
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    board = clone_board(state.board),
    steps = state.steps,
    elapsed_ms = state.elapsed_ms,
    won = state.won,
    move_mode = state.move_mode,
    message = state.message,
  })
end

local function maybe_update_best(state)
  local elapsed = math.floor(state.elapsed_ms / 1000)
  local improved = false
  if state.best_steps <= 0 or state.steps < state.best_steps then
    improved = true
  elseif state.steps == state.best_steps and (state.best_time_sec <= 0 or elapsed < state.best_time_sec) then
    improved = true
  end
  if improved then
    state.best_steps = state.steps
    state.best_time_sec = elapsed
    save_best_record(state)
  end
end

local function perform_move(state, dir)
  if state.won then
    return state
  end
  local actual = dir
  if state.move_mode == "number" then
    actual = opposite_dir(dir)
  end
  if move_blank(state.board, actual) then
    state.steps = state.steps + 1
    if is_solved(state.board) then
      state.won = true
      state.message = "game.sliding_puzzle.win_banner"
      maybe_update_best(state)
    end
    save_progress(state)
  end
  return state
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.won then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end

  if event.type == "resize" then
    state.message = "game.sliding_puzzle.runtime_resized"
    return state
  end

  if event.type == "quit" then
    request_exit()
    return state
  end

  if event.type ~= "action" then
    return state
  end

  if event.name == "quit_action" then
    request_exit()
    return state
  elseif event.name == "restart" then
    return fresh_state()
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.sliding_puzzle.runtime_saved"
    return state
  elseif event.name == "toggle_mode" then
    state.move_mode = state.move_mode == "blank" and "number" or "blank"
    state.message = "game.sliding_puzzle.runtime_mode_changed"
    return state
  elseif event.name == "move_left" then
    return perform_move(state, "left")
  elseif event.name == "move_right" then
    return perform_move(state, "right")
  elseif event.name == "move_up" then
    return perform_move(state, "up")
  elseif event.name == "move_down" then
    return perform_move(state, "down")
  end

  return state
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function format_duration(total_seconds)
  local h = math.floor(total_seconds / 3600)
  local m = math.floor((total_seconds % 3600) / 60)
  local s = total_seconds % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

local function draw_board(state, x, y)
  local border = "+------+------+------+------+"
  canvas_draw_text(x, y, border, "white", nil)
  for row = 1, SIZE do
    local cells = {}
    for col = 1, SIZE do
      local value = state.board[row][col]
      local text = value == 0 and "    " or string.format("%4d", value)
      cells[#cells + 1] = "|" .. text .. " "
    end
    cells[#cells + 1] = "|"
    canvas_draw_text(x, y + (row - 1) * 2 + 1, table.concat(cells), "white", nil)
    canvas_draw_text(x, y + (row - 1) * 2 + 2, border, "white", nil)
  end
end

function render(state)
  canvas_clear()
  local _, height = get_terminal_size()
  local best_line = state.best_steps > 0
    and (tr("game.sliding_puzzle.best_title") .. "  " .. tr("game.sliding_puzzle.best_steps") .. " " .. tostring(state.best_steps) .. "  " .. tr("game.sliding_puzzle.best_time") .. " " .. format_duration(state.best_time_sec))
    or tr("game.sliding_puzzle.best_none")
  local info_line = tr("game.sliding_puzzle.time") .. " " .. format_duration(math.floor(state.elapsed_ms / 1000))
    .. "  " .. tr("game.sliding_puzzle.steps") .. " " .. tostring(state.steps)
  local mode_line = tr("game.sliding_puzzle.mode_label") .. ": " ..
    (state.move_mode == "number" and tr("game.sliding_puzzle.mode_number") or tr("game.sliding_puzzle.mode_blank"))

  canvas_draw_text(centered_x(best_line), 1, best_line, "dark_gray", nil)
  canvas_draw_text(centered_x(info_line), 2, info_line, "light_cyan", nil)
  canvas_draw_text(centered_x(mode_line), 3, mode_line, "white", nil)

  local board_width = 29
  local board_height = 9
  local x, y = resolve_rect(ANCHOR_CENTER, ANCHOR_MIDDLE, board_width, board_height, 0, 1)
  y = math.max(y, 5)
  draw_board(state, x, y)

  local message = tr(state.message)
  local controls = tr("game.sliding_puzzle.runtime_controls")
  canvas_draw_text(centered_x(message), math.max(y + board_height + 1, height - 3), message, state.won and "green" or "white", nil)
  canvas_draw_text(centered_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best_steps <= 0 then
    return nil
  end
  return {
    best_string = "game.sliding_puzzle.best_block",
    steps = state.best_steps,
    time = format_duration(state.best_time_sec),
  }
end
