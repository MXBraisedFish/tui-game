local State = load_function("display/state.lua")
local Render = load_function("display/render.lua")

local cached_root_state = {}

function handle_event(lua_state, event)
  lua_state = lua_state or State.initial_state(cached_root_state)
  return State.handle_event(lua_state, cached_root_state, event)
end

function render(root_state)
  cached_root_state = root_state or {}
  Render.render(cached_root_state)
end
