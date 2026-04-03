local DIFFICULTY_TO_SIZE = {
  [1] = 2,
  [2] = 4,
  [3] = 6,
}

local SYMBOLS = {
  "!", "@", "#", "$", "%", "^", "&", "*", "A",
  "B", "C", "D", "E", "F", "G", "H", "I", "J"
}

local HIDE_DELAY_MS = 700

local function tr(key)
  return translate(key)
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

local function size_for_difficulty(difficulty)
  return DIFFICULTY_TO_SIZE[clamp(difficulty, 1, 3)]
end

local function new_matrix(size, value)
  local matrix = {}
  for row = 1, size do
    matrix[row] = {}
    for col = 1, size do
      matrix[row][col] = value
    end
  end
  return matrix
end

local function clone_matrix(matrix, size)
  local out = new_matrix(size, false)
  for row = 1, size do
    for col = 1, size do
      out[row][col] = matrix[row][col]
    end
  end
  return out
end

local function symbol_for_pair(pair_id)
  return SYMBOLS[((pair_id - 1) % #SYMBOLS) + 1]
end

local function shuffled_deck(size)
  local pair_count = (size * size) / 2
  local deck = {}
  for pair_id = 1, pair_count do
    deck[#deck + 1] = pair_id
    deck[#deck + 1] = pair_id
  end
  for i = #deck, 2, -1 do
    local j = math.random(1, i)
    deck[i], deck[j] = deck[j], deck[i]
  end
  return deck
end

local function generate_board(size)
  local board = new_matrix(size, 0)
  local deck = shuffled_deck(size)
  local index = 1
  for row = 1, size do
    for col = 1, size do
      board[row][col] = deck[index]
      index = index + 1
    end
  end
  return board
end

local function load_best_record()
  local best = load_data("best_record")
  if type(best) ~= "table" then
    return nil
  end
  return {
    difficulty = math.floor(tonumber(best.difficulty) or 0),
    min_steps = math.floor(tonumber(best.min_steps) or 0),
    min_time_sec = math.floor(tonumber(best.min_time_sec) or 0),
  }
end

local function should_replace_best(old_record, new_record)
  if old_record == nil then
    return true
  end
  if new_record.difficulty ~= old_record.difficulty then
    return new_record.difficulty > old_record.difficulty
  end
  if new_record.min_steps ~= old_record.min_steps then
    return new_record.min_steps < old_record.min_steps
  end
  return new_record.min_time_sec < old_record.min_time_sec
end

local function fresh_state(difficulty)
  local size = size_for_difficulty(difficulty or 2)
  return {
    difficulty = clamp(difficulty or 2, 1, 3),
    size = size,
    board = generate_board(size),
    revealed = new_matrix(size, false),
    matched = new_matrix(size, false),
    cursor_row = 1,
    cursor_col = 1,
    steps = 0,
    elapsed_ms = 0,
    won = false,
    first_pick = nil,
    pending_hide = nil,
    message = "game.memory_flip.runtime_ready",
    best = load_best_record(),
  }
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    return fresh_state(2)
  end

  state.difficulty = clamp(math.floor(tonumber(state.difficulty) or 2), 1, 3)
  state.size = size_for_difficulty(state.difficulty)
  if type(state.board) ~= "table" or #state.board ~= state.size then
    return fresh_state(state.difficulty)
  end
  if type(state.revealed) ~= "table" or #state.revealed ~= state.size then
    state.revealed = new_matrix(state.size, false)
  end
  if type(state.matched) ~= "table" or #state.matched ~= state.size then
    state.matched = new_matrix(state.size, false)
  end
  state.cursor_row = clamp(math.floor(tonumber(state.cursor_row) or 1), 1, state.size)
  state.cursor_col = clamp(math.floor(tonumber(state.cursor_col) or 1), 1, state.size)
  state.steps = math.max(0, math.floor(tonumber(state.steps) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.won = state.won == true
  state.message = state.message or "game.memory_flip.runtime_ready"
  state.best = load_best_record()
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    difficulty = state.difficulty,
    board = clone_matrix(state.board, state.size),
    revealed = clone_matrix(state.revealed, state.size),
    matched = clone_matrix(state.matched, state.size),
    cursor_row = state.cursor_row,
    cursor_col = state.cursor_col,
    steps = state.steps,
    elapsed_ms = state.elapsed_ms,
    won = state.won,
    first_pick = state.first_pick,
    pending_hide = state.pending_hide,
    message = state.message,
  })
end

local function all_matched(state)
  for row = 1, state.size do
    for col = 1, state.size do
      if not state.matched[row][col] then
        return false
      end
    end
  end
  return true
end

local function center_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function commit_best(state)
  local record = {
    difficulty = state.difficulty,
    min_steps = state.steps,
    min_time_sec = math.floor(state.elapsed_ms / 1000),
  }
  if should_replace_best(state.best, record) then
    state.best = record
    save_data("best_record", record)
    request_refresh_best_score()
  end
end

local function restart_with_difficulty(difficulty)
  local state = fresh_state(difficulty)
  save_progress(state)
  return state
end

local function apply_flip(state)
  if state.won or state.pending_hide ~= nil then
    return state
  end

  local row = state.cursor_row
  local col = state.cursor_col
  if state.matched[row][col] or state.revealed[row][col] then
    return state
  end

  state.revealed[row][col] = true
  state.steps = state.steps + 1

  if state.first_pick == nil then
    state.first_pick = { row = row, col = col }
    state.message = "game.memory_flip.runtime_pick_second"
    save_progress(state)
    return state
  end

  local first = state.first_pick
  state.first_pick = nil
  if state.board[first.row][first.col] == state.board[row][col] then
    state.matched[first.row][first.col] = true
    state.matched[row][col] = true
    state.message = "game.memory_flip.runtime_match"
    if all_matched(state) then
      state.won = true
      state.message = "game.memory_flip.win_banner"
      commit_best(state)
    end
  else
    state.pending_hide = {
      row1 = first.row,
      col1 = first.col,
      row2 = row,
      col2 = col,
      remain_ms = HIDE_DELAY_MS,
    }
    state.message = "game.memory_flip.runtime_miss"
  end
  save_progress(state)
  return state
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.won then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    if state.pending_hide ~= nil then
      state.pending_hide.remain_ms = state.pending_hide.remain_ms - (event.dt_ms or 16)
      if state.pending_hide.remain_ms <= 0 then
        state.revealed[state.pending_hide.row1][state.pending_hide.col1] = false
        state.revealed[state.pending_hide.row2][state.pending_hide.col2] = false
        state.pending_hide = nil
        state.message = "game.memory_flip.runtime_ready"
        save_progress(state)
      end
    end
    return state
  end

  if event.type == "resize" then
    state.message = "game.memory_flip.runtime_resized"
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
    state.message = "game.memory_flip.runtime_saved"
    return state
  elseif event.name == "restart" then
    return restart_with_difficulty(state.difficulty)
  elseif event.name == "difficulty_cycle" then
    local next_difficulty = state.difficulty + 1
    if next_difficulty > 3 then
      next_difficulty = 1
    end
    local next = restart_with_difficulty(next_difficulty)
    next.message = "game.memory_flip.runtime_difficulty_changed"
    return next
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
  elseif event.name == "flip" then
    return apply_flip(state)
  end

  return state
end

function render(state)
  canvas_clear()

  local width, height = get_terminal_size()
  local top1 = tr("game.memory_flip.best_difficulty") .. ": " .. tostring(state.best and state.best.difficulty or 0)
  local top2 = tr("game.memory_flip.best_steps") .. ": " .. tostring(state.best and state.best.min_steps or 0)
  local top3 = tr("game.memory_flip.steps") .. ": " .. tostring(state.steps)
  local top4 = tr("game.memory_flip.time") .. ": " .. string.format("%02d:%02d:%02d",
    math.floor(state.elapsed_ms / 3600000),
    math.floor((state.elapsed_ms % 3600000) / 60000),
    math.floor((state.elapsed_ms % 60000) / 1000)
  )
  local controls = tr("game.memory_flip.controls")

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
      local visible = state.matched[row][col] or state.revealed[row][col]
      local cell = visible and (" " .. symbol_for_pair(state.board[row][col]) .. " ") or " # "
      if row == state.cursor_row and col == state.cursor_col then
        if visible then
          cell = "[" .. symbol_for_pair(state.board[row][col]) .. "]"
        else
          cell = "[#]"
        end
      end
      line[#line + 1] = cell
      if col < state.size then
        line[#line + 1] = " "
      end
    end
    canvas_draw_text(board_x, board_y + row - 1, table.concat(line), "white", nil)
  end

  local msg_color = state.won and "green" or "white"
  canvas_draw_text(center_x(tr(state.message)), math.max(board_y + state.size + 1, height - 3), tr(state.message), msg_color, nil)
  canvas_draw_text(center_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best == nil or state.best.difficulty == nil or state.best.difficulty <= 0 then
    return nil
  end
  return {
    best_string = "game.memory_flip.best_block",
    difficulty = state.best.difficulty,
    steps = state.best.min_steps,
    time = state.best.min_time_sec,
  }
end
