local M = {}

local PARTICLE_COUNT = 24

local function new_particle(width, height)
  return {
    x = random(0, math.max(0, width - 1)),
    y = random(4, math.max(4, height - 4)),
    dx = random(0, 1) == 0 and -1 or 1,
    dy = random(0, 1) == 0 and -1 or 1,
    char = random(0, 1) == 0 and "*" or "+"
  }
end

function M.update(state)
  local width, height = get_terminal_size()
  state.started_ms = state.started_ms or running_time()
  state.particles = state.particles or {}
  state.timer = state.timer or timer_create(120, "saver particle step")
  if get_timer_status(state.timer) == "init" then
    timer_start(state.timer)
  end

  while #state.particles < PARTICLE_COUNT do
    table.insert(state.particles, new_particle(width, height))
  end

  if is_timer_completed(state.timer) then
    timer_restart(state.timer)
    for _, particle in ipairs(state.particles) do
      particle.x = particle.x + particle.dx
      particle.y = particle.y + particle.dy
      if particle.x <= 0 or particle.x >= width - 1 then
        particle.dx = -particle.dx
      end
      if particle.y <= 4 or particle.y >= height - 4 then
        particle.dy = -particle.dy
      end
      particle.x = math.max(0, math.min(width - 1, particle.x))
      particle.y = math.max(4, math.min(math.max(4, height - 4), particle.y))
    end
  end

  return state
end

return M
