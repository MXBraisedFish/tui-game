local DEFAULT_MAP = {
  "###################",
  "#........#........#",
  "#.###.##.#.##.###.#",
  "#*#.............#*#",
  "#.###.#.###.#.###.#",
  "#.....#..@..#.....#",
  "#####.### ###.#####",
  "#.......G.........#",
  "#.###.#.###.#.###.#",
  "#*....#.....#....*#",
  "###################",
}

local MAP = nil

local DIRS = {
  up = { 0, -1 },
  down = { 0, 1 },
  left = { -1, 0 },
  right = { 1, 0 },
}

local function tr(key)
  return translate(key)
end

local function load_map()
  if type(MAP) == "table" and #MAP > 0 then
    return MAP
  end

  local ok, raw = pcall(read_text, "data/map.txt")
  if ok and type(raw) == "string" and raw ~= "" then
    local lines = {}
    for line in raw:gmatch("[^\r\n]+") do
      if line ~= "" then
        lines[#lines + 1] = line
      end
    end
    if #lines > 0 then
      MAP = lines
      return MAP
    end
  end

  MAP = DEFAULT_MAP
  return MAP
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function parse_map()
  local map = load_map()
  local pellets, walls = {}, {}
  local player_x, player_y = 2, 2
  local ghost_x, ghost_y = 2, 2
  local dots = 0
  for y = 1, #map do
    walls[y] = {}
    pellets[y] = {}
    for x = 1, #map[y] do
      local ch = map[y]:sub(x, x)
      walls[y][x] = ch == "#"
      if ch == "@" then
        player_x, player_y = x, y
      elseif ch == "G" then
        ghost_x, ghost_y = x, y
      elseif ch == "." or ch == "*" then
        pellets[y][x] = ch
        dots = dots + 1
      end
    end
  end
  return walls, pellets, player_x, player_y, ghost_x, ghost_y, dots
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_score = math.max(0, math.floor(tonumber(best.score) or 0))
    state.best_level = math.max(1, math.floor(tonumber(best.level) or 1))
  end
end

local function save_best_record(state)
  save_data("best_record", {
    score = state.best_score,
    level = state.best_level,
  })
  request_refresh_best_score()
end

local function fresh_state()
  local walls, pellets, px, py, gx, gy, dots = parse_map()
  local state = {
    walls = walls,
    pellets = pellets,
    player_x = px,
    player_y = py,
    ghost_x = gx,
    ghost_y = gy,
    ghost_dir = "left",
    dots_left = dots,
    score = 0,
    level = 1,
    lives = 3,
    power_ms = 0,
    elapsed_ms = 0,
    finished = false,
    won = false,
    message = "game.pacman.status_ready",
    best_score = 0,
    best_level = 1,
  }
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return fresh_state()
end

local function is_wall(state, x, y)
  return y < 1 or y > #state.walls or x < 1 or x > #state.walls[y] or state.walls[y][x]
end

local function consume_pellet(state)
  local row = state.pellets[state.player_y]
  local pellet = row and row[state.player_x] or nil
  if pellet == "." then
    row[state.player_x] = nil
    state.score = state.score + 10
    state.dots_left = state.dots_left - 1
  elseif pellet == "*" then
    row[state.player_x] = nil
    state.score = state.score + 50
    state.dots_left = state.dots_left - 1
    state.power_ms = 6000
    state.message = "game.pacman.status_power"
  end
end

local function update_best(state)
  local dirty = false
  if state.score > state.best_score then
    state.best_score = state.score
    dirty = true
  end
  if state.level > state.best_level then
    state.best_level = state.level
    dirty = true
  end
  if dirty then
    save_best_record(state)
  end
end

local function maybe_finish(state)
  if state.dots_left <= 0 then
    state.finished = true
    state.won = true
    state.message = "game.pacman.win_banner"
  end
  update_best(state)
end

local function move_ghost(state)
  local preferred = {}
  local dx = state.player_x - state.ghost_x
  local dy = state.player_y - state.ghost_y
  if math.abs(dx) >= math.abs(dy) then
    preferred[1] = dx <= 0 and "left" or "right"
    preferred[2] = dy <= 0 and "up" or "down"
  else
    preferred[1] = dy <= 0 and "up" or "down"
    preferred[2] = dx <= 0 and "left" or "right"
  end
  preferred[3] = preferred[1] == "left" and "right" or "left"
  preferred[4] = preferred[2] == "up" and "down" or "up"
  local invert = state.power_ms > 0
  for i = 1, #preferred do
    local dir = preferred[i]
    if invert then
      if dir == "left" then dir = "right"
      elseif dir == "right" then dir = "left"
      elseif dir == "up" then dir = "down"
      elseif dir == "down" then dir = "up" end
    end
    local delta = DIRS[dir]
    local nx = state.ghost_x + delta[1]
    local ny = state.ghost_y + delta[2]
    if not is_wall(state, nx, ny) then
      state.ghost_dir = dir
      state.ghost_x = nx
      state.ghost_y = ny
      return
    end
  end
end

local function reset_positions(state)
  local _, _, px, py, gx, gy = parse_map()
  state.player_x, state.player_y = px, py
  state.ghost_x, state.ghost_y = gx, gy
  state.ghost_dir = "left"
  state.power_ms = 0
end

local function handle_collision(state)
  if state.player_x ~= state.ghost_x or state.player_y ~= state.ghost_y then
    return
  end
  if state.power_ms > 0 then
    state.score = state.score + 200
    state.level = state.level + 1
    state.message = "game.pacman.status_ghost_eaten"
    local _, _, _, _, gx, gy = parse_map()
    state.ghost_x, state.ghost_y = gx, gy
  else
    state.lives = state.lives - 1
    if state.lives <= 0 then
      state.finished = true
      state.won = false
      state.message = "game.pacman.lose_banner"
    else
      state.message = "game.pacman.status_wait"
      reset_positions(state)
    end
  end
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.finished then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
      state.power_ms = math.max(0, state.power_ms - (event.dt_ms or 16))
      move_ghost(state)
      handle_collision(state)
      maybe_finish(state)
    end
    return state
  end

  if event.type == "resize" then
    state.message = "game.pacman.runtime_resized"
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
    return fresh_state()
  elseif event.name == "move_up" or event.name == "move_down" or event.name == "move_left" or event.name == "move_right" then
    if state.finished then
      return state
    end
    local dir = event.name:gsub("move_", "")
    local delta = DIRS[dir]
    local nx = state.player_x + delta[1]
    local ny = state.player_y + delta[2]
    if not is_wall(state, nx, ny) then
      state.player_x = nx
      state.player_y = ny
      consume_pellet(state)
      handle_collision(state)
      maybe_finish(state)
    end
  end
  return state
end

local function format_duration(ms)
  local total = math.floor(ms / 1000)
  local h = math.floor(total / 3600)
  local m = math.floor((total % 3600) / 60)
  local s = total % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function render(state)
  canvas_clear()
  local _, term_h = get_terminal_size()
  local title = tr("game.pacman.name")
  canvas_draw_text(centered_x(title), 2, title, "yellow", nil)
  canvas_draw_text(4, 4, tr("game.pacman.current_score") .. ": " .. tostring(state.score), "white", nil)
  canvas_draw_text(4, 5, tr("game.pacman.level") .. ": " .. tostring(state.level), "white", nil)
  canvas_draw_text(4, 6, tr("game.pacman.lives") .. ": " .. tostring(state.lives), "white", nil)
  canvas_draw_text(4, 7, tr("game.pacman.game_time") .. ": " .. format_duration(state.elapsed_ms), "white", nil)
  local start_x = math.max(2, resolve_x(ANCHOR_CENTER, #MAP[1], 0))
  local start_y = 9
  for y = 1, #MAP do
    for x = 1, #MAP[y] do
      local fg, ch = "dark_gray", " "
      if state.walls[y][x] then
        fg, ch = "blue", "#"
      elseif state.player_x == x and state.player_y == y then
        fg, ch = "yellow", "@"
      elseif state.ghost_x == x and state.ghost_y == y then
        fg, ch = state.power_ms > 0 and "cyan" or "red", "&"
      else
        local pellet = state.pellets[y][x]
        if pellet == "." then
          fg, ch = "yellow", "."
        elseif pellet == "*" then
          fg, ch = "orange", "*"
        end
      end
      canvas_draw_text(start_x + x - 1, start_y + y - 1, ch, fg, nil)
    end
  end
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.min(term_h - 3, start_y + #MAP + 1), message, state.finished and (state.won and "green" or "red") or "white", nil)
  canvas_draw_text(centered_x(tr("game.pacman.controls")), term_h - 1, tr("game.pacman.controls"), "dark_gray", nil)
end

function best_score(state)
  return {
    best_string = "game.pacman.best_block",
    score = state.best_score,
    level = state.best_level,
  }
end
