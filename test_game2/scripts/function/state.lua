local M = {}

local function clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

local function new_star(width, height)
  return {
    x = random(2, math.max(2, width - 3)),
    y = random(6, math.max(6, height - 4))
  }
end

function M.clamp(value, min_value, max_value)
  return clamp(value, min_value, max_value)
end

function M.spawn_star(state)
  local width, height = get_terminal_size()
  state.star = new_star(width, height)
end

function M.init(previous_state)
  local width, height = get_terminal_size()
  local state = previous_state or {}

  state.player = state.player or {
    x = math.floor(width / 2),
    y = math.floor(height / 2)
  }
  state.player.x = clamp(state.player.x or math.floor(width / 2), 0, math.max(0, width - 1))
  state.player.y = clamp(state.player.y or math.floor(height / 2), 0, math.max(0, height - 1))

  state.score = state.score or 0
  state.best_score = state.best_score or 0
  state.moves = state.moves or 0
  state.running_ms = state.running_ms or 0
  state.last_event = state.last_event or "init"
  state.message = state.message or translate("test_game2.message.ready")
  state.saved_count = state.saved_count or 0
  state.created_at = state.created_at or now()
  state.star = state.star or new_star(width, height)

  if state.blink_timer == nil or not is_timer_exists(state.blink_timer) then
    state.blink_timer = timer_create(500, "star blink")
    timer_start(state.blink_timer)
  end

  return state
end

function M.reset(state)
  local width, height = get_terminal_size()
  state.player = {
    x = math.floor(width / 2),
    y = math.floor(height / 2)
  }
  state.score = 0
  state.moves = 0
  state.running_ms = 0
  state.last_event = "reset"
  state.message = translate("test_game2.message.reset")
  M.spawn_star(state)
  return state
end

function M.resize(state, width, height)
  state.player.x = clamp(state.player.x or 0, 0, math.max(0, width - 1))
  state.player.y = clamp(state.player.y or 0, 0, math.max(0, height - 1))
  if state.star ~= nil then
    state.star.x = clamp(state.star.x or 0, 0, math.max(0, width - 1))
    state.star.y = clamp(state.star.y or 0, 0, math.max(0, height - 1))
  end
  return state
end

return M