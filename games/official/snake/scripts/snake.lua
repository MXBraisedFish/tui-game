local GRID_W = 24
local GRID_H = 10
local BASE_MOVE_MS = 180
local BOOST_MOVE_MS = 90
local BOOST_DURATION_MS = 5000

local function tr(key)
  return translate(key)
end

local function now_state()
  return {
    snake = {
      { x = 12, y = 5 },
      { x = 11, y = 5 },
      { x = 10, y = 5 },
    },
    dir = "right",
    next_dir = "right",
    score = 0,
    elapsed_ms = 0,
    move_accum_ms = 0,
    boost_left_ms = 0,
    normal_eaten = 0,
    next_special_at = 15,
    normal_food = nil,
    special_food = nil,
    finished = false,
    won = false,
    message = "game.snake.msg_start",
    best_score = 0,
    best_time_ms = 0,
  }
end

local function clone_segments(snake)
  local out = {}
  for i = 1, #snake do
    out[i] = { x = snake[i].x, y = snake[i].y }
  end
  return out
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_score = math.max(0, math.floor(tonumber(best.score) or 0))
    state.best_time_ms = math.max(0, math.floor(tonumber(best.time_ms) or 0))
  end
end

local function spawn_food(state, include_special)
  local cells = {}
  for y = 1, GRID_H do
    for x = 1, GRID_W do
      local occupied = false
      for i = 1, #state.snake do
        if state.snake[i].x == x and state.snake[i].y == y then
          occupied = true
          break
        end
      end
      if not occupied then
        if state.normal_food and state.normal_food.x == x and state.normal_food.y == y then
          occupied = true
        end
        if include_special and state.special_food and state.special_food.x == x and state.special_food.y == y then
          occupied = true
        end
      end
      if not occupied then
        cells[#cells + 1] = { x = x, y = y }
      end
    end
  end

  if #cells == 0 then
    return nil
  end
  return cells[math.random(1, #cells)]
end

local function ensure_food(state)
  if state.normal_food == nil then
    state.normal_food = spawn_food(state, true)
  end
  if state.normal_eaten >= state.next_special_at and state.special_food == nil then
    state.special_food = spawn_food(state, false)
    state.next_special_at = state.next_special_at + 15
  end
end

local function restore_or_new()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    state = now_state()
  end
  if type(state.snake) ~= "table" or #state.snake < 1 then
    state = now_state()
  end
  load_best_record(state)
  ensure_food(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_or_new()
end

local function format_time(ms)
  local total = math.floor(ms / 1000)
  local h = math.floor(total / 3600)
  local m = math.floor((total % 3600) / 60)
  local s = total % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

local function centered_x(text)
  local width = select(1, measure_text(text))
  return resolve_x(ANCHOR_CENTER, width, 0)
end

local function opposite(dir)
  if dir == "up" then return "down" end
  if dir == "down" then return "up" end
  if dir == "left" then return "right" end
  return "left"
end

local function move_interval(state)
  if state.boost_left_ms > 0 then
    return BOOST_MOVE_MS
  end
  return BASE_MOVE_MS
end

local function save_progress(state)
  save_data("state", {
    snake = clone_segments(state.snake),
    dir = state.dir,
    next_dir = state.next_dir,
    score = state.score,
    elapsed_ms = state.elapsed_ms,
    move_accum_ms = state.move_accum_ms,
    boost_left_ms = state.boost_left_ms,
    normal_eaten = state.normal_eaten,
    next_special_at = state.next_special_at,
    normal_food = state.normal_food,
    special_food = state.special_food,
    finished = state.finished,
    won = state.won,
    message = state.message,
  })
end

local function save_best_record(state)
  save_data("best_record", {
    score = state.best_score,
    time_ms = state.best_time_ms,
  })
end

local function finish_game(state, won)
  state.finished = true
  state.won = won
  if state.score > state.best_score then
    state.best_score = state.score
  end
  if state.elapsed_ms > state.best_time_ms then
    state.best_time_ms = state.elapsed_ms
  end
  save_best_record(state)
  save_progress(state)
  request_refresh_best_score()
  state.message = won and "game.snake.win_banner" or "game.snake.lose_banner"
end

local function advance_snake(state)
  if state.finished then
    return state
  end

  state.dir = state.next_dir
  local head = state.snake[1]
  local nx, ny = head.x, head.y
  if state.dir == "left" then nx = nx - 1 end
  if state.dir == "right" then nx = nx + 1 end
  if state.dir == "up" then ny = ny - 1 end
  if state.dir == "down" then ny = ny + 1 end
  if nx < 1 then nx = GRID_W end
  if nx > GRID_W then nx = 1 end
  if ny < 1 then ny = GRID_H end
  if ny > GRID_H then ny = 1 end

  local growing = state.normal_food and nx == state.normal_food.x and ny == state.normal_food.y
  if state.special_food and nx == state.special_food.x and ny == state.special_food.y then
    growing = true
  end

  local tail_limit = #state.snake
  if not growing then
    tail_limit = tail_limit - 1
  end
  for i = 1, tail_limit do
    local part = state.snake[i]
    if part.x == nx and part.y == ny then
      finish_game(state, false)
      return state
    end
  end

  table.insert(state.snake, 1, { x = nx, y = ny })
  if state.normal_food and nx == state.normal_food.x and ny == state.normal_food.y then
    state.score = state.score + 10
    state.normal_eaten = state.normal_eaten + 1
    state.normal_food = nil
    state.message = "game.snake.msg_eat_normal"
  elseif state.special_food and nx == state.special_food.x and ny == state.special_food.y then
    state.score = state.score + 25
    state.special_food = nil
    state.boost_left_ms = BOOST_DURATION_MS
    state.message = "game.snake.msg_eat_special"
  else
    table.remove(state.snake)
  end

  if #state.snake >= GRID_W * GRID_H then
    finish_game(state, true)
    return state
  end

  ensure_food(state)
  return state
end

function handle_event(state, event)
  if event.type == "resize" then
    state.message = "game.snake.msg_resized"
    return state
  end

  if event.type == "quit" then
    request_exit()
    return state
  end

  if event.type == "action" then
    if event.name == "restart" then
      local next = now_state()
      load_best_record(next)
      ensure_food(next)
      next.message = "game.snake.msg_restart"
      return next
    end
    if event.name == "save" then
      save_progress(state)
      state.message = "game.snake.save_success"
      return state
    end
    if event.name == "confirm" and state.finished then
      request_exit()
      return state
    end
    if not state.finished then
      local dir = nil
      if event.name == "move_left" then dir = "left" end
      if event.name == "move_right" then dir = "right" end
      if event.name == "move_up" then dir = "up" end
      if event.name == "move_down" then dir = "down" end
      if dir and dir ~= opposite(state.dir) then
        state.next_dir = dir
      end
    end
    return state
  end

  if event.type == "tick" and not state.finished then
    state.elapsed_ms = state.elapsed_ms + event.dt_ms
    state.move_accum_ms = state.move_accum_ms + event.dt_ms
    if state.boost_left_ms > 0 then
      state.boost_left_ms = math.max(0, state.boost_left_ms - event.dt_ms)
    end
    while state.move_accum_ms >= move_interval(state) do
      state.move_accum_ms = state.move_accum_ms - move_interval(state)
      state = advance_snake(state)
      if state.finished then
        break
      end
    end
  end

  return state
end

local function draw_border(x, y)
  canvas_draw_text(x, y, "╔" .. string.rep("═", GRID_W) .. "╗", "white", nil)
  for row = 1, GRID_H do
    canvas_draw_text(x, y + row, "║" .. string.rep(" ", GRID_W) .. "║", "white", nil)
  end
  canvas_draw_text(x, y + GRID_H + 1, "╚" .. string.rep("═", GRID_W) .. "╝", "white", nil)
end

local function draw_board(state, x, y)
  draw_border(x, y)
  if state.normal_food then
    canvas_draw_text(x + state.normal_food.x, y + state.normal_food.y, "$", "dark_yellow", nil)
  end
  if state.special_food then
    canvas_draw_text(x + state.special_food.x, y + state.special_food.y, "%", "cyan", nil)
  end
  for i = #state.snake, 1, -1 do
    local part = state.snake[i]
    local color = i == 1 and "yellow" or "green"
    canvas_draw_text(x + part.x, y + part.y, "█", color, nil)
  end
end

function render(state)
  canvas_clear()
  local term_w, term_h = get_terminal_size()
  local board_w = GRID_W + 2
  local board_h = GRID_H + 2
  local origin_x, origin_y = resolve_rect(ANCHOR_CENTER, ANCHOR_MIDDLE, board_w, board_h, 0, 0)
  origin_y = math.max(origin_y, 4)

  local title = tr("game.snake.name")
  local desc = tr("game.snake.description")
  canvas_draw_text(centered_x(title), 1, title, "cyan", nil)
  canvas_draw_text(centered_x(desc), 2, desc, "dark_gray", nil)

  local score_line = tr("game.snake.score") .. ": " .. tostring(state.score)
  local best_line = tr("game.snake.best_score") .. ": " .. tostring(state.best_score)
  local time_line = tr("game.snake.time") .. ": " .. format_time(state.elapsed_ms)
  canvas_draw_text(resolve_x(ANCHOR_RIGHT, select(1, measure_text(score_line)), -2), 1, score_line, "yellow", nil)
  canvas_draw_text(resolve_x(ANCHOR_RIGHT, select(1, measure_text(best_line)), -2), 2, best_line, "green", nil)
  canvas_draw_text(resolve_x(ANCHOR_RIGHT, select(1, measure_text(time_line)), -2), 3, time_line, "white", nil)

  draw_board(state, origin_x, origin_y)

  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.max(0, term_h - 3), message, state.finished and "green" or "white", nil)
  local controls = tr("game.snake.runtime_controls")
  canvas_draw_text(centered_x(controls), math.max(0, term_h - 1), controls, "dark_gray", nil)
end

function best_score(state)
  if type(state.best_score) ~= "number" or state.best_score <= 0 then
    return nil
  end
  return {
    best_string = "game.snake.best_block",
    score = state.best_score,
    time = format_time(state.best_time_ms),
  }
end
