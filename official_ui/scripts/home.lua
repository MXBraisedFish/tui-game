local Render = load_function("home/render.lua")

function handle_event(state, event)
  return state or {}
end

function render(state)
  Render.render(state or {})
end