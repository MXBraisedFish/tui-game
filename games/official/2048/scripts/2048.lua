local SIZE = 4
local TARGET_TILE = 2048

local function tr(key)
  return translate(key)
end

local function new_board()
  local board = {}
  for row = 1, SIZE do
    board[row] = {}
    for col = 1, SIZE do
      board[row][col] = 0
    end
  end
  return board
end

local function clone_board(board)
  local out = new_board()
  for row = 1, SIZE do
    for col = 1, SIZE do
      out[row][col] = board[row][col]
    end
  end
  return out
end

local function empty_cells(board)
  local cells = {}
  for row = 1, SIZE do
    for col = 1, SIZE do
      if board[row][col] == 0 then
        cells[#cells + 1] = { row = row, col = col }
      end
    end
  end
  return cells
end

local function spawn_tile(board)
  local cells = empty_cells(board)
  if #cells == 0 then
    return false
  end
  local pick = cells[math.random(1, #cells)]
  board[pick.row][pick.col] = math.random() < 0.9 and 2 or 4
  return true
end

local function reverse_line(line)
  return { line[4], line[3], line[2], line[1] }
end

local function merge_line(values)
  local compact = {}
  for i = 1, #values do
    if values[i] ~= 0 then
      compact[#compact + 1] = values[i]
    end
  end

  local out = {}
  local gained = 0
  local i = 1
  while i <= #compact do
    if i < #compact and compact[i] == compact[i + 1] then
      local merged = compact[i] * 2
      out[#out + 1] = merged
      gained = gained + merged
      i = i + 2
    else
      out[#out + 1] = compact[i]
      i = i + 1
    end
  end

  while #out < SIZE do
    out[#out + 1] = 0
  end

  return out, gained
end

local function lines_equal(a, b)
  for i = 1, SIZE do
    if a[i] ~= b[i] then
      return false
    end
  end
  return true
end

local function can_move(board)
  if #empty_cells(board) > 0 then
    return true
  end
  for row = 1, SIZE do
    for col = 1, SIZE do
      local value = board[row][col]
      if row < SIZE and board[row + 1][col] == value then
        return true
      end
      if col < SIZE and board[row][col + 1] == value then
        return true
      end
    end
  end
  return false
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_score = math.max(0, math.floor(tonumber(best.score) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(best.time_sec) or 0))
  end
end

local function fresh_state()
  local state = {
    board = new_board(),
    score = 0,
    elapsed_ms = 0,
    won = false,
    game_over = false,
    message = "game.2048.runtime_ready",
    best_score = 0,
    best_time_sec = 0,
  }
  load_best_record(state)
  spawn_tile(state.board)
  spawn_tile(state.board)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    return fresh_state()
  end
  if type(state.board) ~= "table" or #state.board ~= SIZE then
    return fresh_state()
  end
  state.score = math.max(0, math.floor(tonumber(state.score) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.won = state.won == true
  state.game_over = state.game_over == true
  state.message = state.message or "game.2048.runtime_ready"
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
    score = state.score,
    elapsed_ms = state.elapsed_ms,
    won = state.won,
    game_over = state.game_over,
    message = state.message,
  })
end

local function persist_best_record(state)
  save_data("best_record", {
    score = state.best_score,
    time_sec = state.best_time_sec,
  })
  request_refresh_best_score()
end

local function update_best_if_needed(state)
  local elapsed_sec = math.floor(state.elapsed_ms / 1000)
  if state.score > state.best_score then
    state.best_score = state.score
    state.best_time_sec = elapsed_sec
    persist_best_record(state)
  elseif state.score == state.best_score and state.score > 0 and (state.best_time_sec == 0 or elapsed_sec < state.best_time_sec) then
    state.best_time_sec = elapsed_sec
    persist_best_record(state)
  end
end

local function apply_move(state, dir)
  local moved = false
  local gained = 0

  local function get_row(row)
    return { state.board[row][1], state.board[row][2], state.board[row][3], state.board[row][4] }
  end

  local function set_row(row, line)
    for col = 1, SIZE do
      state.board[row][col] = line[col]
    end
  end

  local function get_col(col)
    return { state.board[1][col], state.board[2][col], state.board[3][col], state.board[4][col] }
  end

  local function set_col(col, line)
    for row = 1, SIZE do
      state.board[row][col] = line[row]
    end
  end

  if dir == "left" or dir == "right" then
    for row = 1, SIZE do
      local old = get_row(row)
      local line = old
      if dir == "right" then
        line = reverse_line(line)
      end
      local merged, line_gained = merge_line(line)
      if dir == "right" then
        merged = reverse_line(merged)
      end
      set_row(row, merged)
      if not lines_equal(old, merged) then
        moved = true
      end
      gained = gained + line_gained
    end
  else
    for col = 1, SIZE do
      local old = get_col(col)
      local line = old
      if dir == "down" then
        line = reverse_line(line)
      end
      local merged, line_gained = merge_line(line)
      if dir == "down" then
        merged = reverse_line(merged)
      end
      set_col(col, merged)
      if not lines_equal(old, merged) then
        moved = true
      end
      gained = gained + line_gained
    end
  end

  if moved then
    state.score = state.score + gained
    spawn_tile(state.board)
    if not state.won then
      for row = 1, SIZE do
        for col = 1, SIZE do
          if state.board[row][col] >= TARGET_TILE then
            state.won = true
            state.message = "game.2048.win_banner"
          end
        end
      end
    end
    if not can_move(state.board) then
      state.game_over = true
      state.message = "game.2048.game_over"
    else
      if not state.won then
        state.message = "game.2048.runtime_ready"
      end
    end
    update_best_if_needed(state)
    save_progress(state)
  end
  return moved
end

local function center_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function format_duration(total_seconds)
  local h = math.floor(total_seconds / 3600)
  local m = math.floor((total_seconds % 3600) / 60)
  local s = total_seconds % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.game_over then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end

  if event.type == "resize" then
    state.message = "game.2048.runtime_resized"
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
    state.message = "game.2048.save_success"
    return state
  elseif event.name == "move_left" and not state.game_over then
    apply_move(state, "left")
  elseif event.name == "move_right" and not state.game_over then
    apply_move(state, "right")
  elseif event.name == "move_up" and not state.game_over then
    apply_move(state, "up")
  elseif event.name == "move_down" and not state.game_over then
    apply_move(state, "down")
  end

  return state
end

function render(state)
  canvas_clear()

  local width, height = get_terminal_size()
  local elapsed = math.floor(state.elapsed_ms / 1000)
  local top1 = tr("game.2048.time") .. " " .. format_duration(elapsed)
  local top2 = tr("game.2048.score") .. " " .. tostring(state.score)
  local top3 = tr("game.2048.best_score") .. " " .. tostring(state.best_score)
  local top4 = tr("game.2048.best_time") .. " " .. format_duration(state.best_time_sec)
  local controls = tr("game.2048.controls")

  canvas_draw_text(center_x(top1), 1, top1, "cyan", nil)
  canvas_draw_text(center_x(top2), 2, top2, "yellow", nil)
  canvas_draw_text(center_x(top3), 3, top3, "white", nil)
  canvas_draw_text(center_x(top4), 4, top4, "white", nil)

  local board_width = 4 * 8 + 3
  local board_x = resolve_x(ANCHOR_CENTER, board_width, 0)
  local board_y = math.max(6, math.floor((height - 10) / 2))

  for row = 1, SIZE do
    local line = {}
    for col = 1, SIZE do
      local value = state.board[row][col]
      local text = value == 0 and "." or tostring(value)
      if #text > 6 then
        text = string.sub(text, 1, 6)
      end
      line[#line + 1] = string.format("%6s", text)
      if col < SIZE then
        line[#line + 1] = " |"
      end
    end
    canvas_draw_text(board_x, board_y + row - 1, table.concat(line), "white", nil)
  end

  local msg_color = state.game_over and "red" or (state.won and "green" or "white")
  canvas_draw_text(center_x(tr(state.message)), math.max(board_y + 6, height - 3), tr(state.message), msg_color, nil)
  canvas_draw_text(center_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best_score <= 0 then
    return nil
  end
  return {
    best_string = "game.2048.best_block",
    score = state.best_score,
    time = format_duration(state.best_time_sec),
  }
end
