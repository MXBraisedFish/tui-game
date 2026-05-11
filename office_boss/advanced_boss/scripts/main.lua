local dashboard = load_function("dashboard.lua")
local renderer = load_function("renderer.lua")

function update(state)
  return dashboard.update(state)
end

function render(state)
  renderer.render(state)
end
