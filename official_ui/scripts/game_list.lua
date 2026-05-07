local Render = load_function("game_list/render.lua")
local State = load_function("game_list/state.lua")

local function normalize_order(order)
  if order == "desc" then
    return "desc"
  end
  return "asc"
end

local function normalize_sort(sort)
  if sort == "name" or sort == "author" then
    return sort
  end
  return "official_mod"
end

local function next_sort(sort)
  sort = normalize_sort(sort)
  if sort == "official_mod" then
    return "name"
  elseif sort == "name" then
    return "author"
  end
  return "official_mod"
end

local function toggle_order(order)
  if normalize_order(order) == "asc" then
    return "desc"
  end
  return "asc"
end

local function normalize_lua_state(lua_state)
  lua_state = lua_state or {}
  if type(lua_state.game_list) == "table" then
    State.set_root_state(lua_state)
  end
  lua_state.select = tostring(lua_state.select or State.first_uid())
  lua_state.confirm = false
  lua_state.back = false
  lua_state.order = normalize_order(lua_state.order)
  lua_state.sort = normalize_sort(lua_state.sort)
  lua_state.pages = State.pages()
  lua_state.page = math.max(1, math.min(lua_state.pages, math.floor(lua_state.page or State.page())))
  lua_state.user_page = lua_state.jump and math.max(0, math.floor(lua_state.user_page or 0)) or 0
  lua_state.jump = lua_state.jump == true
  lua_state.info_scroll = math.max(0, math.floor(lua_state.info_scroll or 0))
  lua_state.info_scroll = math.min(lua_state.info_scroll, Render.max_info_scroll(lua_state))
  return lua_state
end

local function update_select(lua_state, offset)
  local uid = State.uid_by_offset(lua_state.select, offset)
  if uid ~= nil then
    lua_state.select = uid
    lua_state.page = State.page_of(uid)
  end
end

local function update_page(lua_state, offset)
  lua_state.page = math.max(1, math.min(lua_state.pages, lua_state.page + offset))
end

local function append_user_page(lua_state, digit)
  local value = tostring(lua_state.user_page or 0)
  if value == "0" then
    value = ""
  end
  value = value .. tostring(digit)
  lua_state.user_page = tonumber(value) or 0
end

function handle_event(lua_state, event)
  lua_state = normalize_lua_state(lua_state)

  if type(event) ~= "table" then
    return lua_state
  end

  if event.type == "resize" then
    lua_state.pages = State.pages()
    lua_state.page = math.max(1, math.min(lua_state.pages, lua_state.page))
    lua_state.info_scroll = math.min(lua_state.info_scroll, Render.max_info_scroll(lua_state))
    return lua_state
  end

  if event.type ~= "action" and event.type ~= "key" then
    return lua_state
  end

  if lua_state.jump then
    if event.type == "action" then
      if event.name == "confirm" then
        if lua_state.user_page >= 1 and lua_state.user_page <= lua_state.pages then
          lua_state.page = lua_state.user_page
        end
        lua_state.user_page = 0
        lua_state.jump = false
      elseif event.name == "return" then
        lua_state.user_page = 0
        lua_state.jump = false
      end
    elseif event.type == "key" then
      local digit = tonumber(event.name)
      if digit ~= nil and digit >= 0 and digit <= 9 then
        append_user_page(lua_state, digit)
      end
    end
    return lua_state
  end

  if event.type == "action" then
    if event.name == "prev_option" then
      update_select(lua_state, -1)
    elseif event.name == "next_option" then
      update_select(lua_state, 1)
    elseif event.name == "prev_page" then
      update_page(lua_state, -1)
    elseif event.name == "next_page" then
      update_page(lua_state, 1)
    elseif event.name == "scroll_up" then
      lua_state.info_scroll = math.max(0, lua_state.info_scroll - 1)
    elseif event.name == "scroll_down" then
      lua_state.info_scroll = math.min(Render.max_info_scroll(lua_state), lua_state.info_scroll + 1)
    elseif event.name == "jump" then
      if lua_state.pages > 1 then
        lua_state.user_page = 0
        lua_state.jump = true
      end
    elseif event.name == "order" then
      lua_state.order = toggle_order(lua_state.order)
    elseif event.name == "sort" then
      lua_state.sort = next_sort(lua_state.sort)
    elseif event.name == "confirm" then
      lua_state.confirm = true
    elseif event.name == "return" then
      lua_state.back = true
    end
  end

  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
