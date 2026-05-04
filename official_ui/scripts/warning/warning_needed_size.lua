local Render = load_function("warning_needed_size/render.lua")

function handle_event(lua_state, event)
  lua_state = lua_state or { exit = false, mode = "root" }
  lua_state.mode = lua_state.mode or "root"
  lua_state.exit = false

  if type(event) == "table" and event.type == "action" and event.name == "return" then
    lua_state.exit = true
  end

  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
