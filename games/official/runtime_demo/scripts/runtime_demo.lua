local FIELD_WIDTH = 17
local FIELD_HEIGHT = 9

local function fresh_state()
  return {
    player_x = 2,
    player_y = 2,
    goal_x = FIELD_WIDTH - 1,
    goal_y = FIELD_HEIGHT - 1,
    steps = 0,
    best_steps = nil,
    message = "Reach X with as few steps as possible.",
    finished = false,
  }
end

local function load_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    state = fresh_state()
  end

  local best_steps = load_data("best_steps")
  if type(best_steps) == "number" and best_steps > 0 then
    state.best_steps = math.floor(best_steps)
  end
  return state
end

function init_game()
  return load_state()
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

local function centered_x(text)
  local width = select(1, get_terminal_size())
  local text_width = select(1, measure_text(text))
  return math.max(0, math.floor((width - text_width) / 2))
end

local function save_state(state)
  save_data("state", state)
end

function handle_event(state, event)
  if event.type == "resize" then
    state.message = "Terminal size changed. Layout recalculated."
    return state
  end

  if event.type == "quit" then
    request_exit()
    return state
  end

  if event.type ~= "action" or state.finished then
    return state
  end

  if event.name == "move_left" then
    state.player_x = clamp(state.player_x - 1, 1, FIELD_WIDTH)
    state.steps = state.steps + 1
    state.message = "Keep moving toward the goal."
  elseif event.name == "move_right" then
    state.player_x = clamp(state.player_x + 1, 1, FIELD_WIDTH)
    state.steps = state.steps + 1
    state.message = "Keep moving toward the goal."
  elseif event.name == "move_up" then
    state.player_y = clamp(state.player_y - 1, 1, FIELD_HEIGHT)
    state.steps = state.steps + 1
    state.message = "Keep moving toward the goal."
  elseif event.name == "move_down" then
    state.player_y = clamp(state.player_y + 1, 1, FIELD_HEIGHT)
    state.steps = state.steps + 1
    state.message = "Keep moving toward the goal."
  elseif event.name == "restart" then
    state = fresh_state()
    state.best_steps = load_data("best_steps")
    state.message = "Field reset."
    return state
  elseif event.name == "confirm" then
    save_state(state)
    request_exit()
    return state
  else
    return state
  end

  if state.player_x == state.goal_x and state.player_y == state.goal_y then
    state.finished = true
    if type(state.best_steps) ~= "number" or state.steps < state.best_steps then
      state.best_steps = state.steps
      save_data("best_steps", state.best_steps)
      request_refresh_best_score()
      state.message = "New best record. Press Enter to save and exit."
    else
      state.message = "Goal reached. Press Enter to save and exit."
    end
    save_state(state)
  end

  return state
end

local function draw_field(state, origin_x, origin_y)
  for y = 0, FIELD_HEIGHT + 1 do
    local row = {}
    for x = 0, FIELD_WIDTH + 1 do
      if y == 0 or y == FIELD_HEIGHT + 1 or x == 0 or x == FIELD_WIDTH + 1 then
        row[#row + 1] = "#"
      elseif x == state.player_x and y == state.player_y then
        row[#row + 1] = "@"
      elseif x == state.goal_x and y == state.goal_y then
        row[#row + 1] = "X"
      else
        row[#row + 1] = "."
      end
    end
    canvas_draw_text(origin_x, origin_y + y, table.concat(row), "white", nil)
  end
end

function render(state)
  canvas_clear()

  local term_w, term_h = get_terminal_size()
  local title = "Runtime Demo"
  local desc = "Move @ to X. Enter saves and exits."
  local best = state.best_steps and tostring(state.best_steps) or "No record yet"
  local field_origin_x = math.max(0, math.floor((term_w - (FIELD_WIDTH + 2)) / 2))
  local field_origin_y = math.max(4, math.floor((term_h - (FIELD_HEIGHT + 2)) / 2))

  canvas_draw_text(centered_x(title), 1, title, "cyan", nil)
  canvas_draw_text(centered_x(desc), 2, desc, "dark_gray", nil)
  canvas_draw_text(math.max(0, term_w - 12), 1, "Steps: " .. tostring(state.steps), "yellow", nil)
  canvas_draw_text(math.max(0, term_w - 20), 2, "Best: " .. best, "green", nil)

  draw_field(state, field_origin_x, field_origin_y)

  canvas_draw_text(centered_x(state.message), math.max(0, term_h - 3), state.message, state.finished and "green" or "white", nil)
  canvas_draw_text(centered_x("[Arrows] Move  [Enter] Save/Exit  [R] Restart  [Esc] Exit"), math.max(0, term_h - 1), "[Arrows] Move  [Enter] Save/Exit  [R] Restart  [Esc] Exit", "dark_gray", nil)
end

function best_score(state)
  if type(state.best_steps) ~= "number" or state.best_steps <= 0 then
    return nil
  end
  return {
    best_string = "Best Steps\n{steps}",
    steps = state.best_steps,
  }
end
