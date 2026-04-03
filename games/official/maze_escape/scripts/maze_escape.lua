local DEFAULT_COLS = 19
local DEFAULT_ROWS = 13
local MIN_MODE = 1
local MAX_MODE = 4

local TILE_WALL = "#"
local TILE_PATH = "."
local TILE_DOOR = "%"
local TILE_KEY = "*"
local TILE_EXIT = "&"
local TILE_PLAYER = "@"

local function tr(key)
  return translate(key)
end

local function copy_grid(grid)
  local out = {}
  for r = 1, #grid do
    out[r] = {}
    for c = 1, #grid[r] do
      out[r][c] = grid[r][c]
    end
  end
  return out
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best.max_area = math.max(0, math.floor(tonumber(best.max_area) or 0))
    state.best.max_cols = math.max(0, math.floor(tonumber(best.max_cols) or 0))
    state.best.max_rows = math.max(0, math.floor(tonumber(best.max_rows) or 0))
    state.best.max_mode = math.max(0, math.floor(tonumber(best.max_mode) or 0))
    local min_time = tonumber(best.min_time_sec)
    state.best.min_time_sec = min_time and math.max(0, math.floor(min_time)) or nil
  end
end

local function save_best_record(state)
  save_data("best_record", {
    max_area = state.best.max_area,
    max_cols = state.best.max_cols,
    max_rows = state.best.max_rows,
    max_mode = state.best.max_mode,
    min_time_sec = state.best.min_time_sec,
  })
end

local function format_duration(sec)
  local total = math.max(0, math.floor(sec or 0))
  local h = math.floor(total / 3600)
  local m = math.floor((total % 3600) / 60)
  local s = total % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

local function mode_has_keys(mode)
  return mode == 2 or mode == 4
end

local function mode_is_timed(mode)
  return mode == 3 or mode == 4
end

local function mode_label(mode)
  return tr("game.maze_escape.mode" .. tostring(mode))
end

local function shuffled_dirs()
  local dirs = {
    { dr = -2, dc = 0 },
    { dr = 2, dc = 0 },
    { dr = 0, dc = -2 },
    { dr = 0, dc = 2 },
  }
  for i = #dirs, 2, -1 do
    local j = math.random(1, i)
    dirs[i], dirs[j] = dirs[j], dirs[i]
  end
  return dirs
end

local function in_inner(rows, cols, r, c)
  return r >= 2 and r <= rows - 1 and c >= 2 and c <= cols - 1
end

local function bfs(grid, sr, sc)
  local rows = #grid
  local cols = #grid[1]
  local dist = {}
  local prev = {}
  for r = 1, rows do
    dist[r] = {}
    prev[r] = {}
    for c = 1, cols do
      dist[r][c] = -1
      prev[r][c] = nil
    end
  end

  local qr, qc = { sr }, { sc }
  local head = 1
  dist[sr][sc] = 0
  local far_r, far_c = sr, sc

  while head <= #qr do
    local r = qr[head]
    local c = qc[head]
    head = head + 1
    if dist[r][c] > dist[far_r][far_c] then
      far_r, far_c = r, c
    end
    local dirs = { { -1, 0 }, { 1, 0 }, { 0, -1 }, { 0, 1 } }
    for i = 1, #dirs do
      local nr = r + dirs[i][1]
      local nc = c + dirs[i][2]
      if nr >= 1 and nr <= rows and nc >= 1 and nc <= cols then
        if dist[nr][nc] < 0 and grid[nr][nc] ~= TILE_WALL then
          dist[nr][nc] = dist[r][c] + 1
          prev[nr][nc] = { r = r, c = c }
          qr[#qr + 1] = nr
          qc[#qc + 1] = nc
        end
      end
    end
  end

  return far_r, far_c, dist, prev
end

local function reconstruct_path(prev, tr, tc)
  local path = {}
  local r, c = tr, tc
  while r and c do
    table.insert(path, 1, { r = r, c = c })
    local p = prev[r][c]
    if not p then
      break
    end
    r, c = p.r, p.c
  end
  return path
end

local function generate_maze(cols, rows, mode)
  local grid = {}
  for r = 1, rows do
    grid[r] = {}
    for c = 1, cols do
      grid[r][c] = TILE_WALL
    end
  end

  local function carve(r, c)
    grid[r][c] = TILE_PATH
    local dirs = shuffled_dirs()
    for i = 1, #dirs do
      local nr = r + dirs[i].dr
      local nc = c + dirs[i].dc
      if in_inner(rows, cols, nr, nc) and grid[nr][nc] == TILE_WALL then
        grid[r + math.floor(dirs[i].dr / 2)][c + math.floor(dirs[i].dc / 2)] = TILE_PATH
        carve(nr, nc)
      end
    end
  end

  carve(2, 2)
  local exit_r, exit_c, _, prev = bfs(grid, 2, 2)
  local path = reconstruct_path(prev, exit_r, exit_c)
  grid[exit_r][exit_c] = TILE_EXIT

  local key_r, key_c = nil, nil
  local door_r, door_c = nil, nil
  if mode_has_keys(mode) and #path >= 6 then
    local key_idx = math.max(2, math.floor(#path / 3))
    local door_idx = math.min(#path - 1, math.floor(#path * 2 / 3))
    key_r, key_c = path[key_idx].r, path[key_idx].c
    door_r, door_c = path[door_idx].r, path[door_idx].c
    if grid[key_r][key_c] == TILE_PATH then
      grid[key_r][key_c] = TILE_KEY
    end
    if grid[door_r][door_c] == TILE_PATH then
      grid[door_r][door_c] = TILE_DOOR
    end
  end

  local time_limit = nil
  if mode_is_timed(mode) then
    time_limit = math.max(20, math.floor(#path * 1.2))
  end

  return grid, exit_r, exit_c, time_limit
end

local function default_state()
  return {
    cols = DEFAULT_COLS,
    rows = DEFAULT_ROWS,
    mode = 1,
    grid = {},
    player_r = 2,
    player_c = 2,
    exit_r = 2,
    exit_c = 2,
    keys_held = 0,
    steps = 0,
    elapsed_ms = 0,
    time_limit_sec = nil,
    won = false,
    lost = false,
    message = "game.maze_escape.runtime_ready",
    best = {
      max_area = 0,
      max_cols = 0,
      max_rows = 0,
      max_mode = 0,
      min_time_sec = nil,
    },
  }
end

local function reset_maze(state, cols, rows, mode)
  state.cols = cols or DEFAULT_COLS
  state.rows = rows or DEFAULT_ROWS
  state.mode = mode or 1
  if state.cols % 2 == 0 then state.cols = state.cols - 1 end
  if state.rows % 2 == 0 then state.rows = state.rows - 1 end
  local grid, exit_r, exit_c, time_limit = generate_maze(state.cols, state.rows, state.mode)
  state.grid = grid
  state.player_r = 2
  state.player_c = 2
  state.exit_r = exit_r
  state.exit_c = exit_c
  state.keys_held = 0
  state.steps = 0
  state.elapsed_ms = 0
  state.time_limit_sec = time_limit
  state.won = false
  state.lost = false
  state.message = "game.maze_escape.runtime_ready"
  return state
end

local function save_progress(state)
  save_data("state", {
    cols = state.cols,
    rows = state.rows,
    mode = state.mode,
    grid = copy_grid(state.grid),
    player_r = state.player_r,
    player_c = state.player_c,
    exit_r = state.exit_r,
    exit_c = state.exit_c,
    keys_held = state.keys_held,
    steps = state.steps,
    elapsed_ms = state.elapsed_ms,
    time_limit_sec = state.time_limit_sec,
    won = state.won,
    lost = state.lost,
    message = state.message,
    best = state.best,
  })
end

local function restore_or_new()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" or type(state.grid) ~= "table" then
    state = default_state()
    reset_maze(state, DEFAULT_COLS, DEFAULT_ROWS, 1)
  end
  if type(state.best) ~= "table" then
    state.best = default_state().best
  end
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_or_new()
end

local function persist_best(state)
  local area = state.cols * state.rows
  if area > state.best.max_area then
    state.best.max_area = area
    state.best.max_cols = state.cols
    state.best.max_rows = state.rows
  end
  if state.mode > state.best.max_mode then
    state.best.max_mode = state.mode
  end
  local sec = math.floor(state.elapsed_ms / 1000)
  if state.best.min_time_sec == nil or sec < state.best.min_time_sec then
    state.best.min_time_sec = sec
  end
  save_best_record(state)
  request_refresh_best_score()
end

local function try_move(state, dr, dc)
  if state.won or state.lost then
    return state
  end
  local nr = state.player_r + dr
  local nc = state.player_c + dc
  if nr < 1 or nr > state.rows or nc < 1 or nc > state.cols then
    return state
  end
  local tile = state.grid[nr][nc]
  if tile == TILE_WALL then
    return state
  end
  if tile == TILE_DOOR then
    if state.keys_held <= 0 then
      return state
    end
    state.keys_held = state.keys_held - 1
    state.grid[nr][nc] = TILE_PATH
  elseif tile == TILE_KEY then
    state.keys_held = state.keys_held + 1
    state.grid[nr][nc] = TILE_PATH
  end
  state.player_r = nr
  state.player_c = nc
  state.steps = state.steps + 1
  if nr == state.exit_r and nc == state.exit_c then
    state.won = true
    state.message = "game.maze_escape.win_banner"
    persist_best(state)
    save_progress(state)
  end
  return state
end

function handle_event(state, event)
  if event.type == "resize" then
    state.message = "game.maze_escape.runtime_resized"
    return state
  end

  if event.type == "quit" then
    request_exit()
    return state
  end

  if event.type == "action" then
    if event.name == "quit_action" then
      request_exit()
      return state
    elseif event.name == "restart" then
      return reset_maze(state, state.cols, state.rows, state.mode)
    elseif event.name == "save" then
      save_progress(state)
      state.message = "game.maze_escape.runtime_saved"
      return state
    elseif event.name == "cycle_mode" then
      local next_mode = state.mode + 1
      if next_mode > MAX_MODE then next_mode = MIN_MODE end
      local next = reset_maze(state, state.cols, state.rows, next_mode)
      next.message = "game.maze_escape.runtime_mode_changed"
      return next
    elseif event.name == "confirm" and (state.won or state.lost) then
      request_exit()
      return state
    elseif event.name == "move_left" then
      return try_move(state, 0, -1)
    elseif event.name == "move_right" then
      return try_move(state, 0, 1)
    elseif event.name == "move_up" then
      return try_move(state, -1, 0)
    elseif event.name == "move_down" then
      return try_move(state, 1, 0)
    end
    return state
  end

  if event.type == "tick" and not state.won and not state.lost then
    state.elapsed_ms = state.elapsed_ms + event.dt_ms
    if state.time_limit_sec and math.floor(state.elapsed_ms / 1000) >= state.time_limit_sec then
      state.lost = true
      state.message = "game.maze_escape.lose_banner"
      save_progress(state)
    end
  end

  return state
end

local function centered_x(text)
  local width = select(1, measure_text(text))
  return resolve_x(ANCHOR_CENTER, width, 0)
end

local function draw_board(state, x, y)
  for r = 1, state.rows do
    local pieces = {}
    for c = 1, state.cols do
      local ch = state.grid[r][c]
      if r == state.player_r and c == state.player_c then
        ch = TILE_PLAYER
      end
      pieces[#pieces + 1] = ch
    end
    local line = table.concat(pieces)
    canvas_draw_text(x, y + r - 1, line, "white", nil)
  end
end

function render(state)
  canvas_clear()
  local term_w, term_h = get_terminal_size()
  local title = tr("game.maze_escape.name")
  local desc = tr("game.maze_escape.description")
  canvas_draw_text(centered_x(title), 1, title, "cyan", nil)
  canvas_draw_text(centered_x(desc), 2, desc, "dark_gray", nil)

  local board_w = state.cols
  local board_h = state.rows
  local origin_x, origin_y = resolve_rect(ANCHOR_CENTER, ANCHOR_MIDDLE, board_w, board_h, 0, 0)
  origin_y = math.max(origin_y, 5)

  local info_line = string.format(
    "%s: %s  %s: %d  %s: %d",
    tr("game.maze_escape.mode"),
    mode_label(state.mode),
    tr("game.maze_escape.steps"),
    state.steps,
    tr("game.maze_escape.keys"),
    state.keys_held
  )
  canvas_draw_text(centered_x(info_line), 4, info_line, "white", nil)

  if state.time_limit_sec then
    local remain = math.max(0, state.time_limit_sec - math.floor(state.elapsed_ms / 1000))
    local timer_line = tr("game.maze_escape.remaining") .. ": " .. format_duration(remain)
    canvas_draw_text(centered_x(timer_line), 5, timer_line, "yellow", nil)
    origin_y = math.max(origin_y, 7)
  else
    local timer_line = tr("game.maze_escape.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000))
    canvas_draw_text(centered_x(timer_line), 5, timer_line, "yellow", nil)
    origin_y = math.max(origin_y, 7)
  end

  draw_board(state, origin_x, origin_y)

  local message = tr(state.message)
  local controls = tr("game.maze_escape.runtime_controls")
  canvas_draw_text(centered_x(message), math.max(0, term_h - 3), message, state.won and "green" or (state.lost and "red" or "white"), nil)
  canvas_draw_text(centered_x(controls), math.max(0, term_h - 1), controls, "dark_gray", nil)
end

function best_score(state)
  if state.best.max_area <= 0 then
    return nil
  end
  local size = string.format("%dx%d", state.best.max_cols, state.best.max_rows)
  local fastest = state.best.min_time_sec and format_duration(state.best.min_time_sec) or "--:--:--"
  return {
    best_string = "game.maze_escape.best_block",
    size = size,
    mode = state.best.max_mode,
    fastest = fastest,
  }
end
