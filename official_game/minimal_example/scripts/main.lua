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
  local next_state = state or {}
  next_state.x = next_state.x or math.floor(width / 2)
  next_state.y = next_state.y or math.floor(height / 2)
  next_state.moves = next_state.moves or 0
  return next_state
end

function handle_event(state, event)
  if event.type == "action" then
    local width, height = get_terminal_size()
    if event.name == "up" then
      state.y = clamp(state.y - 1, 0, height - 1)
      state.moves = state.moves + 1
    elseif event.name == "down" then
      state.y = clamp(state.y + 1, 0, height - 1)
      state.moves = state.moves + 1
    elseif event.name == "left" then
      state.x = clamp(state.x - 1, 0, width - 1)
      state.moves = state.moves + 1
    elseif event.name == "right" then
      state.x = clamp(state.x + 1, 0, width - 1)
      state.moves = state.moves + 1
    elseif event.name == "quit" then
      request_exit()
    end
  elseif event.type == "resize" then
    state.x = clamp(state.x, 0, math.max(0, event.width - 1))
    state.y = clamp(state.y, 0, math.max(0, event.height - 1))
  end
  return state
end

function render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  canvas_draw_text(resolve_x(ANCHOR_CENTER, 15), 1, translate("minimal_example.title"), "yellow", nil, BOLD)
  canvas_draw_rich_text(2, math.max(0, height - 2), translate("minimal_example.help"), "grey", nil, ALIGN_LEFT, math.max(1, width - 4))
  canvas_draw_text(state.x, state.y, "@", "light_cyan", nil, BOLD)
  canvas_draw_text(2, 3, "Moves: " .. tostring(state.moves), "white")
end

function exit_game(state)
  return state
end
