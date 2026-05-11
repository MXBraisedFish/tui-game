local particles = load_function("particles.lua")
local view = load_function("view.lua")

function update(state)
  state = particles.update(state)
  return state
end

function render(state)
  view.render(state)
end
