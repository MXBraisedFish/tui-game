local SIZE = 9
local BOX = 3
local HOLES = { [1] = 30, [2] = 40, [3] = 50, [4] = 60, [5] = 70 }

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function base_solution()
  local board = {}
  for r = 1, SIZE do
    board[r] = {}
    for c = 1, SIZE do
      board[r][c] = ((r - 1) * BOX + math.floor((r - 1) / BOX) + c - 1) % SIZE + 1
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

local function shuffled_positions()
  local positions = {}
  for r = 1, SIZE do
    for c = 1, SIZE do
      positions[#positions + 1] = { r = r, c = c }
    end
  end
  for i = #positions, 2, -1 do
    local j = math.random(i)
    positions[i], positions[j] = positions[j], positions[i]
  end
  return positions
end

local function build_puzzle(difficulty)
  local solution = base_solution()
  local board = clone_board(solution)
  local given = {}
  for r = 1, SIZE do
    given[r] = {}
    for c = 1, SIZE do
      given[r][c] = true
    end
  end
  local holes = HOLES[difficulty] or HOLES[3]
  local positions = shuffled_positions()
  for i = 1, holes do
    local pos = positions[i]
    board[pos.r][pos.c] = 0
    given[pos.r][pos.c] = false
  end
  return board, solution, given
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_difficulty = math.max(1, math.min(5, math.floor(tonumber(best.difficulty) or 1)))
    state.best_time_sec = math.max(0, math.floor(tonumber(best.time_sec) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", {
    difficulty = state.best_difficulty,
    time_sec = state.best_time_sec,
  })
  request_refresh_best_score()
end

local function push_undo(state, r, c, old_value)
  state.undo[#state.undo + 1] = { r = r, c = c, old = old_value }
  if #state.undo > 100 then
    table.remove(state.undo, 1)
  end
end

local function save_progress(state)
  save_data("state", {
    difficulty = state.difficulty,
    board = clone_board(state.board),
    solution = clone_board(state.solution),
    given = state.given,
    row = state.row,
    col = state.col,
    elapsed_ms = state.elapsed_ms,
    locator = state.locator,
    won = state.won,
    message = state.message,
    undo = state.undo,
  })
end

local function fresh_state(difficulty)
  local board, solution, given = build_puzzle(difficulty or 3)
  local state = {
    difficulty = difficulty or 3,
    board = board,
    solution = solution,
    given = given,
    row = 1,
    col = 1,
    elapsed_ms = 0,
    locator = false,
    won = false,
    message = "game.sudoku.runtime_ready",
    undo = {},
    best_difficulty = 1,
    best_time_sec = 0,
  }
  load_best_record(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    return fresh_state(3)
  end
  if type(state.board) ~= "table" or type(state.solution) ~= "table" or type(state.given) ~= "table" then
    return fresh_state(3)
  end
  state.difficulty = math.max(1, math.min(5, math.floor(tonumber(state.difficulty) or 3)))
  state.board = state.board
  state.solution = state.solution
  state.given = state.given
  state.row = math.max(1, math.min(9, math.floor(tonumber(state.row) or 1)))
  state.col = math.max(1, math.min(9, math.floor(tonumber(state.col) or 1)))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.locator = state.locator == true
  state.won = state.won == true
  state.message = state.message or "game.sudoku.runtime_ready"
  state.undo = type(state.undo) == "table" and state.undo or {}
  state.best_difficulty = 1
  state.best_time_sec = 0
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function format_duration(sec)
  local h = math.floor(sec / 3600)
  local m = math.floor((sec % 3600) / 60)
  local s = sec % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

local function check_won(state)
  for r = 1, SIZE do
    for c = 1, SIZE do
      if state.board[r][c] ~= state.solution[r][c] then
        return false
      end
    end
  end
  state.won = true
  state.message = "game.sudoku.win_banner"
  local elapsed = math.floor(state.elapsed_ms / 1000)
  if state.best_time_sec <= 0 or elapsed < state.best_time_sec then
    state.best_time_sec = elapsed
    state.best_difficulty = state.difficulty
    save_best_record(state)
  end
  save_progress(state)
  return true
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.won then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.sudoku.runtime_resized"
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
  elseif event.name == "restart" then
    local next = fresh_state(state.difficulty)
    next.message = "game.sudoku.runtime_restart"
    return next
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.sudoku.save_success"
  elseif event.name == "change_difficulty" then
    local next_d = state.difficulty + 1
    if next_d > 5 then next_d = 1 end
    local next = fresh_state(next_d)
    next.message = "game.sudoku.runtime_difficulty_changed"
    return next
  elseif event.name == "toggle_locator" then
    state.locator = not state.locator
    state.message = state.locator and "game.sudoku.locator_on" or "game.sudoku.locator_off"
  elseif event.name == "undo" then
    local step = table.remove(state.undo)
    if step then
      state.board[step.r][step.c] = step.old
      state.message = "game.sudoku.undo_done"
      save_progress(state)
    else
      state.message = "game.sudoku.undo_empty"
    end
  elseif event.name == "move_left" then
    state.col = math.max(1, state.col - 1)
  elseif event.name == "move_right" then
    state.col = math.min(SIZE, state.col + 1)
  elseif event.name == "move_up" then
    state.row = math.max(1, state.row - 1)
  elseif event.name == "move_down" then
    state.row = math.min(SIZE, state.row + 1)
  elseif event.name == "clear_cell" and not state.given[state.row][state.col] and not state.won then
    push_undo(state, state.row, state.col, state.board[state.row][state.col])
    state.board[state.row][state.col] = 0
    save_progress(state)
  elseif string.sub(event.name, 1, 6) == "digit_" and not state.given[state.row][state.col] and not state.won then
    local value = tonumber(string.sub(event.name, 7)) or 0
    if value >= 1 and value <= 9 then
      push_undo(state, state.row, state.col, state.board[state.row][state.col])
      state.board[state.row][state.col] = value
      save_progress(state)
      check_won(state)
    end
  end
  return state
end

function render(state)
  canvas_clear()
  local _, height = get_terminal_size()
  local best_line = state.best_time_sec > 0
    and (tr("game.sudoku.best_difficulty") .. " " .. tr("game.sudoku.difficulty." .. tostring(state.best_difficulty)) .. "  " .. tr("game.sudoku.best_time") .. " " .. format_duration(state.best_time_sec))
    or tr("game.sudoku.best_none")
  local time_line = tr("game.sudoku.time") .. " " .. format_duration(math.floor(state.elapsed_ms / 1000))
  local diff_line = tr("game.sudoku.difficulty") .. ": " .. tr("game.sudoku.difficulty." .. tostring(state.difficulty))
  canvas_draw_text(centered_x(best_line), 1, best_line, "dark_gray", nil)
  canvas_draw_text(centered_x(time_line), 2, time_line, "light_cyan", nil)
  canvas_draw_text(centered_x(diff_line), 3, diff_line, "white", nil)

  local board_x = resolve_x(ANCHOR_CENTER, 25, 0)
  local board_y = 5
  for r = 1, SIZE do
    local parts = {}
    for c = 1, SIZE do
      local value = state.board[r][c] == 0 and "." or tostring(state.board[r][c])
      local color = state.given[r][c] and "white" or "yellow"
      local bg = nil
      if r == state.row and c == state.col then
        bg = "yellow"
        color = "black"
      elseif state.locator then
        local same_box = math.floor((r - 1) / BOX) == math.floor((state.row - 1) / BOX)
          and math.floor((c - 1) / BOX) == math.floor((state.col - 1) / BOX)
        if r == state.row or c == state.col or same_box then
          bg = "gray"
        end
      end
      canvas_draw_text(board_x + (c - 1) * 2, board_y + r - 1, value .. " ", color, bg)
    end
  end

  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.max(board_y + 11, height - 3), message, state.won and "green" or "white", nil)
  canvas_draw_text(centered_x(tr("game.sudoku.controls")), height - 1, tr("game.sudoku.controls"), "dark_gray", nil)
end

function best_score(state)
  if state.best_time_sec <= 0 then
    return nil
  end
  return {
    best_string = "game.sudoku.best_block",
    difficulty = tr("game.sudoku.difficulty." .. tostring(state.best_difficulty)),
    time = format_duration(state.best_time_sec),
  }
end
