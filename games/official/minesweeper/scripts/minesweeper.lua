local OFFICIAL = {
  [1] = { rows = 9, cols = 9, mines = 10 },
  [2] = { rows = 16, cols = 16, mines = 40 },
  [3] = { rows = 16, cols = 30, mines = 99 },
}

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function new_matrix(rows, cols, value)
  local out = {}
  for r = 1, rows do
    out[r] = {}
    for c = 1, cols do
      out[r][c] = value
    end
  end
  return out
end

local function in_bounds(state, r, c)
  return r >= 1 and r <= state.rows and c >= 1 and c <= state.cols
end

local function count_adjacent(state, r, c)
  local count = 0
  for dr = -1, 1 do
    for dc = -1, 1 do
      if not (dr == 0 and dc == 0) then
        local nr, nc = r + dr, c + dc
        if in_bounds(state, nr, nc) and state.mines_map[nr][nc] then
          count = count + 1
        end
      end
    end
  end
  return count
end

local function build_board(state, safe_r, safe_c)
  state.mines_map = new_matrix(state.rows, state.cols, false)
  state.adj = new_matrix(state.rows, state.cols, 0)
  local placed = 0
  while placed < state.mines do
    local r = math.random(state.rows)
    local c = math.random(state.cols)
    if not state.mines_map[r][c] and not (r == safe_r and c == safe_c) then
      state.mines_map[r][c] = true
      placed = placed + 1
    end
  end
  for r = 1, state.rows do
    for c = 1, state.cols do
      if not state.mines_map[r][c] then
        state.adj[r][c] = count_adjacent(state, r, c)
      end
    end
  end
  state.started = true
end

local function load_best_record(state)
  local best = load_data("best_record")
  state.best = { [1] = 0, [2] = 0, [3] = 0 }
  if type(best) == "table" then
    for i = 1, 3 do
      state.best[i] = math.max(0, math.floor(tonumber(best[tostring(i)]) or 0))
    end
  end
end

local function save_best_record(state)
  save_data("best_record", {
    ["1"] = state.best[1],
    ["2"] = state.best[2],
    ["3"] = state.best[3],
  })
  request_refresh_best_score()
end

local function save_progress(state)
  save_data("state", {
    difficulty = state.difficulty,
    rows = state.rows,
    cols = state.cols,
    mines = state.mines,
    started = state.started,
    cursor_r = state.cursor_r,
    cursor_c = state.cursor_c,
    revealed = state.revealed,
    marks = state.marks,
    mines_map = state.mines_map,
    adj = state.adj,
    won = state.won,
    lost = state.lost,
    elapsed_ms = state.elapsed_ms,
    guide = state.guide,
    message = state.message,
  })
end

local function fresh_state(difficulty)
  local conf = OFFICIAL[difficulty or 1]
  local state = {
    difficulty = difficulty or 1,
    rows = conf.rows,
    cols = conf.cols,
    mines = conf.mines,
    started = false,
    cursor_r = 1,
    cursor_c = 1,
    revealed = new_matrix(conf.rows, conf.cols, false),
    marks = new_matrix(conf.rows, conf.cols, 0),
    mines_map = new_matrix(conf.rows, conf.cols, false),
    adj = new_matrix(conf.rows, conf.cols, 0),
    won = false,
    lost = false,
    elapsed_ms = 0,
    guide = false,
    message = "game.minesweeper.runtime_ready",
    best = { [1] = 0, [2] = 0, [3] = 0 },
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
    return fresh_state(1)
  end
  local difficulty = math.max(1, math.min(3, math.floor(tonumber(state.difficulty) or 1)))
  local conf = OFFICIAL[difficulty]
  state.difficulty = difficulty
  state.rows = math.max(1, math.floor(tonumber(state.rows) or conf.rows))
  state.cols = math.max(1, math.floor(tonumber(state.cols) or conf.cols))
  state.mines = math.max(1, math.floor(tonumber(state.mines) or conf.mines))
  state.started = state.started == true
  state.cursor_r = math.max(1, math.min(state.rows, math.floor(tonumber(state.cursor_r) or 1)))
  state.cursor_c = math.max(1, math.min(state.cols, math.floor(tonumber(state.cursor_c) or 1)))
  state.revealed = type(state.revealed) == "table" and state.revealed or new_matrix(state.rows, state.cols, false)
  state.marks = type(state.marks) == "table" and state.marks or new_matrix(state.rows, state.cols, 0)
  state.mines_map = type(state.mines_map) == "table" and state.mines_map or new_matrix(state.rows, state.cols, false)
  state.adj = type(state.adj) == "table" and state.adj or new_matrix(state.rows, state.cols, 0)
  state.won = state.won == true
  state.lost = state.lost == true
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.guide = state.guide == true
  state.message = state.message or "game.minesweeper.runtime_ready"
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function flood_reveal(state, start_r, start_c)
  local queue = { { start_r, start_c } }
  local head = 1
  while queue[head] do
    local node = queue[head]
    head = head + 1
    local r, c = node[1], node[2]
    if in_bounds(state, r, c) and not state.revealed[r][c] and state.marks[r][c] ~= 1 then
      state.revealed[r][c] = true
      if state.adj[r][c] == 0 and not state.mines_map[r][c] then
        for dr = -1, 1 do
          for dc = -1, 1 do
            if not (dr == 0 and dc == 0) then
              queue[#queue + 1] = { r + dr, c + dc }
            end
          end
        end
      end
    end
  end
end

local function reveal_current(state)
  if state.won or state.lost then
    return state
  end
  local r, c = state.cursor_r, state.cursor_c
  if state.marks[r][c] == 1 then
    return state
  end
  if not state.started then
    build_board(state, r, c)
  end
  if state.mines_map[r][c] then
    state.revealed[r][c] = true
    state.lost = true
    state.message = "game.minesweeper.lose_banner"
    save_progress(state)
    return state
  end
  flood_reveal(state, r, c)
  local revealed = 0
  for rr = 1, state.rows do
    for cc = 1, state.cols do
      if state.revealed[rr][cc] then
        revealed = revealed + 1
      end
    end
  end
  if revealed >= state.rows * state.cols - state.mines then
    state.won = true
    state.message = "game.minesweeper.win_banner"
    local elapsed = math.floor(state.elapsed_ms / 1000)
    local best = state.best[state.difficulty]
    if best <= 0 or elapsed < best then
      state.best[state.difficulty] = elapsed
      save_best_record(state)
    end
  end
  save_progress(state)
  return state
end

local function format_duration(sec)
  local h = math.floor(sec / 3600)
  local m = math.floor((sec % 3600) / 60)
  local s = sec % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function handle_event(state, event)
  if event.type == "tick" then
    if state.started and not state.won and not state.lost then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.minesweeper.runtime_resized"
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
    return fresh_state(state.difficulty)
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.minesweeper.runtime_saved"
  elseif event.name == "change_difficulty" then
    local next_d = state.difficulty + 1
    if next_d > 3 then next_d = 1 end
    local next_state = fresh_state(next_d)
    next_state.message = "game.minesweeper.runtime_difficulty_changed"
    return next_state
  elseif event.name == "toggle_guide" then
    state.guide = not state.guide
  elseif event.name == "move_left" then
    state.cursor_c = math.max(1, state.cursor_c - 1)
  elseif event.name == "move_right" then
    state.cursor_c = math.min(state.cols, state.cursor_c + 1)
  elseif event.name == "move_up" then
    state.cursor_r = math.max(1, state.cursor_r - 1)
  elseif event.name == "move_down" then
    state.cursor_r = math.min(state.rows, state.cursor_r + 1)
  elseif event.name == "flag" and not state.revealed[state.cursor_r][state.cursor_c] then
    state.marks[state.cursor_r][state.cursor_c] = state.marks[state.cursor_r][state.cursor_c] == 1 and 0 or 1
    save_progress(state)
  elseif event.name == "question" and not state.revealed[state.cursor_r][state.cursor_c] then
    state.marks[state.cursor_r][state.cursor_c] = state.marks[state.cursor_r][state.cursor_c] == 2 and 0 or 2
    save_progress(state)
  elseif event.name == "reveal" then
    return reveal_current(state)
  end
  return state
end

local function cell_text(state, r, c)
  if state.lost and state.mines_map[r][c] then
    return "@", "red"
  end
  if not state.revealed[r][c] then
    local mark = state.marks[r][c]
    if mark == 1 then return "!", "yellow" end
    if mark == 2 then return "?", "cyan" end
    return "#", "white"
  end
  if state.mines_map[r][c] then
    return "@", "red"
  end
  if state.adj[r][c] == 0 then
    return ".", "dark_gray"
  end
  return tostring(state.adj[r][c]), "white"
end

function render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  local time_line = tr("game.minesweeper.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000))
  local mines_left = state.mines
  for r = 1, state.rows do
    for c = 1, state.cols do
      if state.marks[r][c] == 1 then
        mines_left = mines_left - 1
      end
    end
  end
  local left_line = tr("game.minesweeper.mines_left") .. ": " .. tostring(mines_left)
  canvas_draw_text(centered_x(tr("game.minesweeper.name")), 1, tr("game.minesweeper.name"), "cyan", nil)
  canvas_draw_text(centered_x(left_line), 2, left_line, "yellow", nil)
  canvas_draw_text(centered_x(time_line), 3, time_line, "white", nil)

  local board_x = resolve_x(ANCHOR_CENTER, state.cols * 2, 0)
  local board_y = 5
  for r = 1, state.rows do
    for c = 1, state.cols do
      local ch, fg = cell_text(state, r, c)
      local bg = nil
      if r == state.cursor_r and c == state.cursor_c then
        bg = "yellow"
        if fg == "yellow" or fg == "white" then
          fg = "black"
        end
      elseif state.guide and math.abs(r - state.cursor_r) <= 1 and math.abs(c - state.cursor_c) <= 1 then
        bg = "gray"
        if fg == "white" then
          fg = "black"
        end
      end
      canvas_draw_text(board_x + (c - 1) * 2, board_y + r - 1, ch .. " ", fg, bg)
    end
  end

  local controls = tr("game.minesweeper.controls")
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.max(board_y + state.rows + 1, height - 3), message, state.won and "green" or (state.lost and "red" or "white"), nil)
  canvas_draw_text(centered_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best[1] <= 0 and state.best[2] <= 0 and state.best[3] <= 0 then
    return nil
  end
  return {
    best_string = "game.minesweeper.best_block",
    d1 = state.best[1] > 0 and format_duration(state.best[1]) or "--",
    d2 = state.best[2] > 0 and format_duration(state.best[2]) or "--",
    d3 = state.best[3] > 0 and format_duration(state.best[3]) or "--",
  }
end
