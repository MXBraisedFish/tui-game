local Render = load_function("storage_details/render.lua")

function handle_event(lua_state, event)
  lua_state = lua_state or { back = false }
  lua_state.back = false

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "action" and event.name == "return" then
    lua_state.back = true
  end

  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
