local Render = load_function("mod_security/render.lua")

local started_at = now()

local function ready_to_close_temporary()
  return now() - started_at >= 5000
end

local function ready_to_close_permanent()
  return now() - started_at >= 10000
end

function handle_event(lua_state, event)
  lua_state = lua_state or {
    close_temporary = false,
    close_permanent = false,
    back = false
  }
  lua_state.close_temporary = false
  lua_state.close_permanent = false
  lua_state.back = false

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "tick" or event.type == "resize" then
    return lua_state
  end

  if event.type == "action" then
    if event.name == "close_permanent" then
      if ready_to_close_permanent() then
        lua_state.close_permanent = true
      end
      return lua_state
    elseif event.name == "close_temporary" then
      if ready_to_close_temporary() then
        lua_state.close_temporary = true
      end
      return lua_state
    end
  end

  lua_state.back = true
  return lua_state
end

function render(root_state)
  Render.render(root_state or {}, started_at)
end
