local MIN_SIZE = 2
local MAX_SIZE = 10
local DEFAULT_SIZE = 5

local function tr(key)
  return translate(key)
end

local function new_board(size, value)
  local board = {}
  for row = 1, size do
    board[row] = {}
    for col = 1, size do
      board[row][col] = value
    end
  end
  return board
end

local function clone_board(board, size)
  local out = new_board(size, false)
  for row = 1, size do
    for col = 1, size do
      out[row][col] = board[row][col]
    end
  end
  return out
end

local function clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

local function randomize_board(size)
  local board = new_board(size, true)
  for _ = 1, size * size do
    local row = math.random(1, size)
    local col = math.random(1, size)
    local points = {
      { row, col },
      { row - 1, col },
      { row + 1, col },
      { row, col - 1 },
      { row, col + 1 },
    }
    for i = 1, #points do
      local r = points[i][1]
      local c = points[i][2]
      if r >= 1 and r <= size and c >= 1 and c <= size then
        board[r][c] = not board[r][c]
      end
    end
  end
  return board
end

local function all_lit(board, size)
  for row = 1, size do
    for col = 1, size do
      if not board[row][col] then
        return false
      end
    end
  end
  return true
end

local function load_best_record()
  local best = load_data("best_record")
  if type(best) ~= "table" then
    return nil
  end
  return {
    max_size = math.floor(tonumber(best.max_size) or 0),
    min_steps = math.floor(tonumber(best.min_steps) or 0),
    min_time_sec = math.floor(tonumber(best.min_time_sec) or 0),
  }
end

local function should_replace_best(old_record, new_record)
  if old_record == nil then
    return true
  end
  if new_record.max_size ~= old_record.max_size then
    return new_record.max_size > old_record.max_size
  end
  if new_record.min_steps ~= old_record.min_steps then
    return new_record.min_steps < old_record.min_steps
  end
  return new_record.min_time_sec < old_record.min_time_sec
end

local function new_state()
  return {
    size = DEFAULT_SIZE,
    board = randomize_board(DEFAULT_SIZE),
    cursor_row = 1,
    cursor_col = 1,
    steps = 0,
    elapsed_ms = 0,
    won = false,
    message = "game.lights_out.runtime_ready",
    best = load_best_record(),
  }
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    state = new_state()
  end
  state.size = clamp(math.floor(tonumber(state.size) or DEFAULT_SIZE), MIN_SIZE, MAX_SIZE)
  if type(state.board) ~= "table" or #state.board ~= state.size then
    state.board = randomize_board(state.size)
  end
  state.cursor_row = clamp(math.floor(tonumber(state.cursor_row) or 1), 1, state.size)
  state.cursor_col = clamp(math.floor(tonumber(state.cursor_col) or 1), 1, state.size)
  state.steps = math.max(0, math.floor(tonumber(state.steps) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.won = state.won == true
  state.message = state.message or "game.lights_out.runtime_ready"
  state.best = load_best_record()
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    size = state.size,
    board = clone_board(state.board, state.size),
    cursor_row = state.cursor_row,
    cursor_col = state.cursor_col,
    steps = state.steps,
    elapsed_ms = state.elapsed_ms,
    won = state.won,
    message = state.message,
  })
end

local function cycle_size(state)
  local next_size = state.size + 1
  if next_size > MAX_SIZE then
    next_size = MIN_SIZE
  end
  state.size = next_size
  state.board = randomize_board(next_size)
  state.cursor_row = 1
  state.cursor_col = 1
  state.steps = 0
  state.elapsed_ms = 0
  state.won = false
  state.message = "game.lights_out.runtime_resized"
  save_progress(state)
  return state
end

local function toggle_cross(state, row, col)
  local points = {
    { row, col },
    { row - 1, col },
    { row + 1, col },
    { row, col - 1 },
    { row, col + 1 },
  }
  for i = 1, #points do
    local r = points[i][1]
    local c = points[i][2]
    if r >= 1 and r <= state.size and c >= 1 and c <= state.size then
      state.board[r][c] = not state.board[r][c]
    end
  end
end

local function center_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function commit_best_if_needed(state)
  local record = {
    max_size = state.size,
    min_steps = state.steps,
    min_time_sec = math.floor(state.elapsed_ms / 1000),
  }
  if should_replace_best(state.best, record) then
    state.best = record
    save_data("best_record", record)
    request_refresh_best_score()
  end
end

function handle_event(state, event)
  if event.type == "tick" and not state.won then
    state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    return state
  end

  if event.type == "resize" then
    state.message = "game.lights_out.runtime_terminal_changed"
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
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.lights_out.runtime_saved"
    return state
  elseif event.name == "restart" then
    state.board = randomize_board(state.size)
    state.cursor_row = 1
    state.cursor_col = 1
    state.steps = 0
    state.elapsed_ms = 0
    state.won = false
    state.message = "game.lights_out.runtime_restart"
    save_progress(state)
    return state
  elseif event.name == "resize_cycle" then
    return cycle_size(state)
  elseif event.name == "move_left" then
    state.cursor_col = clamp(state.cursor_col - 1, 1, state.size)
    return state
  elseif event.name == "move_right" then
    state.cursor_col = clamp(state.cursor_col + 1, 1, state.size)
    return state
  elseif event.name == "move_up" then
    state.cursor_row = clamp(state.cursor_row - 1, 1, state.size)
    return state
  elseif event.name == "move_down" then
    state.cursor_row = clamp(state.cursor_row + 1, 1, state.size)
    return state
  elseif event.name == "toggle" and not state.won then
    toggle_cross(state, state.cursor_row, state.cursor_col)
    state.steps = state.steps + 1
    if all_lit(state.board, state.size) then
      state.won = true
      state.message = "game.lights_out.win_banner"
      commit_best_if_needed(state)
    else
      state.message = "game.lights_out.runtime_ready"
    end
    save_progress(state)
    return state
  end

  return state
end

function render(state)
  canvas_clear()

  local width, height = get_terminal_size()
  local top1 = tr("game.lights_out.best_size") .. ": " .. tostring(state.best and state.best.max_size or 0)
  local top2 = tr("game.lights_out.best_steps") .. ": " .. tostring(state.best and state.best.min_steps or 0)
  local top3 = tr("game.lights_out.steps") .. ": " .. tostring(state.steps)
  local top4 = tr("game.lights_out.time") .. ": " .. string.format("%02d:%02d:%02d",
    math.floor(state.elapsed_ms / 3600000),
    math.floor((state.elapsed_ms % 3600000) / 60000),
    math.floor((state.elapsed_ms % 60000) / 1000)
  )
  local controls = tr("game.lights_out.controls")

  canvas_draw_text(center_x(top1), 1, top1, "white", nil)
  canvas_draw_text(center_x(top2), 2, top2, "white", nil)
  canvas_draw_text(center_x(top3), 3, top3, "yellow", nil)
  canvas_draw_text(center_x(top4), 4, top4, "cyan", nil)

  local board_width = state.size * 4 + (state.size - 1)
  local board_x = resolve_x(ANCHOR_CENTER, board_width, 0)
  local board_y = math.max(6, math.floor((height - state.size) / 2))

  for row = 1, state.size do
    local line = {}
    for col = 1, state.size do
      local lit = state.board[row][col]
      local cell = lit and "██" or "· "
      if row == state.cursor_row and col == state.cursor_col then
        cell = lit and "▓▓" or "[]"
      end
      line[#line + 1] = cell
      if col < state.size then
        line[#line + 1] = " "
      end
    end
    canvas_draw_text(board_x, board_y + row - 1, table.concat(line), "white", nil)
  end

  canvas_draw_text(center_x(tr(state.message)), math.max(board_y + state.size + 1, height - 3), tr(state.message), state.won and "green" or "white", nil)
  canvas_draw_text(center_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best == nil or state.best.max_size == nil or state.best.max_size <= 0 then
    return nil
  end
  return {
    best_string = "game.lights_out.best_block",
    size = state.best.max_size,
    steps = state.best.min_steps,
    time = state.best.min_time_sec,
  }
end
