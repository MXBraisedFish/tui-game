local Render = load_function("language/render.lua")
local State = load_function("language/state.lua")

local function digit_value(name)
  local number = tonumber(name)
  if number ~= nil and number >= 1 and number <= 9 then
    return number
  end
  return nil
end

function handle_event(lua_state, event)
  lua_state = State.normalize_state(lua_state)

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "action" or event.type == "key" then
    if event.status ~= "press" then
      return lua_state
    end
  end

  if lua_state.jump then
    if event.type == "key" then
      local digit = digit_value(event.name)
      if digit ~= nil then
        lua_state.user_page = (lua_state.user_page or 0) * 10 + digit
      end
    elseif event.type == "action" then
      if event.name == "confirm" then
        lua_state = State.jump_page(lua_state, lua_state.user_page)
      elseif event.name == "return" then
        lua_state.jump = false
        lua_state.user_page = 0
      end
    end
    lua_state.confirm = false
    lua_state.back = false
    return State.normalize_state(lua_state)
  end

  if event.type == "action" then
    if event.name == "up_option" then
      lua_state = State.move(lua_state, -State.grid().columns)
    elseif event.name == "down_option" then
      lua_state = State.move(lua_state, State.grid().columns)
    elseif event.name == "left_option" then
      lua_state = State.move(lua_state, -1)
    elseif event.name == "right_option" then
      lua_state = State.move(lua_state, 1)
    elseif event.name == "prev_page" and (lua_state.page or 1) > 1 then
      lua_state = State.flip_page(lua_state, -1)
    elseif event.name == "next_page" and (lua_state.page or 1) < (lua_state.pages or 1) then
      lua_state = State.flip_page(lua_state, 1)
    elseif event.name == "jump" and (lua_state.pages or 1) > 1 then
      lua_state.jump = true
      lua_state.user_page = 0
    elseif event.name == "confirm" then
      lua_state.confirm = true
    elseif event.name == "return" then
      lua_state.back = true
    end
  end

  lua_state = State.return_state(lua_state)
  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
