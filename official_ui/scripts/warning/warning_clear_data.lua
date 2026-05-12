local Render = load_function("clear_data/render.lua")

local started_at = now()

local function ready_to_confirm()
  return now() - started_at >= 5000
end

function handle_event(lua_state, event)
  lua_state = lua_state or { confirm = false, back = false }
  lua_state.confirm = false
  lua_state.back = false

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "tick" or event.type == "resize" then
    return lua_state
  end

  if event.type == "action" and event.name == "confirm" then
    if event.status == "press" and ready_to_confirm() then
      lua_state.confirm = true
    end
    return lua_state
  end

  if event.type == "action" and event.status == "press" then
    lua_state.back = true
  end
  return lua_state
end

function render(root_state)
  Render.render(root_state or {}, started_at)
end
