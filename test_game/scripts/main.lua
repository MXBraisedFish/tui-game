local ui = load_function("ui.lua")

local function clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

function init_game(state)
  local width, height = get_terminal_size()
  local initial_state = state or {}

  initial_state.player_x = initial_state.player_x or math.floor(width / 2)
  initial_state.player_y = initial_state.player_y or math.floor(height / 2)
  initial_state.moves = initial_state.moves or 0
  initial_state.last_event = initial_state.last_event or "init"
  initial_state.running_ms = initial_state.running_ms or 0
  initial_state.message = initial_state.message or translate("test_game.message.ready")

  return initial_state
end

function handle_event(state, event)
  if event.type == "action" then
    local width, height = get_terminal_size()
    local max_x = math.max(0, width - 1)
    local max_y = math.max(0, height - 1)

    if event.name == "move_up" then
      state.player_y = clamp(state.player_y - 1, 0, max_y)
      state.moves = state.moves + 1
    elseif event.name == "move_down" then
      state.player_y = clamp(state.player_y + 1, 0, max_y)
      state.moves = state.moves + 1
    elseif event.name == "move_left" then
      state.player_x = clamp(state.player_x - 1, 0, max_x)
      state.moves = state.moves + 1
    elseif event.name == "move_right" then
      state.player_x = clamp(state.player_x + 1, 0, max_x)
      state.moves = state.moves + 1
    elseif event.name == "confirm" then
      state.message = translate("test_game.message.confirm")
    elseif event.name == "quit" then
      request_exit()
    end

    state.last_event = event.name
  elseif event.type == "key" then
    state.last_event = "key:" .. tostring(event.name)
  elseif event.type == "resize" then
    state.player_x = clamp(state.player_x, 0, math.max(0, event.width - 1))
    state.player_y = clamp(state.player_y, 0, math.max(0, event.height - 1))
    state.last_event = "resize"
  elseif event.type == "tick" then
    state.running_ms = state.running_ms + (event.dt_ms or 0)
  end

  return state
end

function render(state)
  ui.render(state)
end

function exit_game(state)
  state.message = translate("test_game.message.exit")
  return state
end
