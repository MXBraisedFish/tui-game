local M = {}

local function spawn_star(width, height)
  return {
    x = random(2, math.max(2, width - 3)),
    y = random(5, math.max(5, height - 4))
  }
end

function M.init(saved_state)
  local width, height = get_terminal_size()
  local next_state = saved_state or {}
  next_state.player = next_state.player or { x = math.floor(width / 2), y = math.floor(height / 2) }
  next_state.star = next_state.star or spawn_star(width, height)
  next_state.score = next_state.score or 0
  next_state.moves = next_state.moves or 0
  next_state.message = next_state.message or "Ready."
  next_state.started_at = next_state.started_at or now()
  next_state.tick_timer = timer_create(1000, "example tick")
  if get_timer_status(next_state.tick_timer) == "init" then
    timer_start(next_state.tick_timer)
  end
  return next_state
end

function M.spawn_star(width, height)
  return spawn_star(width, height)
end

return M
