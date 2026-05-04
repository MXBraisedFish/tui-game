local Render = load_function("setting/render.lua")

local function normalize_select(select)
  if type(select) ~= "number" then
    return 1
  end
  select = math.floor(select)
  if select < 1 then
    return 5
  end
  if select > 5 then
    return 1
  end
  return select
end

function handle_event(lua_state, event)
  lua_state = lua_state or { select = 1, confirm = false, back = false }
  lua_state.select = normalize_select(lua_state.select)
  lua_state.confirm = false
  lua_state.back = false

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "action" then
    if event.name == "prev_option" then
      lua_state.select = normalize_select(lua_state.select - 1)
    elseif event.name == "next_option" then
      lua_state.select = normalize_select(lua_state.select + 1)
    elseif event.name == "confirm" then
      lua_state.confirm = true
    elseif event.name == "return" then
      lua_state.back = true
    elseif event.name == "option1" then
      lua_state.select = 1
    elseif event.name == "option2" then
      lua_state.select = 2
    elseif event.name == "option3" then
      lua_state.select = 3
    elseif event.name == "option4" then
      lua_state.select = 4
    elseif event.name == "option5" then
      lua_state.select = 5
    end
  end

  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
