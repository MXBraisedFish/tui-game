local Render = load_function("mod_screensaver_list/render.lua")
local State = load_function("mod_screensaver_list/state.lua")

function handle_event(lua_state, event)
  lua_state = lua_state or {}
  lua_state.confirm = false
  lua_state.back = false
  lua_state.toggle_debug = false

  State.set_root_state(lua_state)
  local max_info_scroll = Render.max_info_scroll(lua_state)
  lua_state.info_scroll = math.max(0, math.min(max_info_scroll, math.floor(lua_state.info_scroll or 0)))

  if type(event) ~= "table" then return lua_state end
  if (event.type == "action" or event.type == "key") and event.status ~= "press" then return lua_state end

  if event.type == "action" then
    if event.name == "prev_option" then
      lua_state.select = State.uid_by_offset(lua_state.select, -1)
      lua_state.page = State.page_of(lua_state.select)
      lua_state.info_scroll = 0
    elseif event.name == "next_option" then
      lua_state.select = State.uid_by_offset(lua_state.select, 1)
      lua_state.page = State.page_of(lua_state.select)
      lua_state.info_scroll = 0
    elseif event.name == "confirm" then
      lua_state.confirm = true
    elseif event.name == "debug" then
      lua_state.toggle_debug = true
    elseif event.name == "list" then
      lua_state.list_mode = tostring(lua_state.list_mode or "full") == "brief" and "full" or "brief"
    elseif event.name == "scroll_up" then
      lua_state.info_scroll = math.max(0, (lua_state.info_scroll or 0) - 1)
    elseif event.name == "scroll_down" then
      lua_state.info_scroll = math.min(max_info_scroll, (lua_state.info_scroll or 0) + 1)
    elseif event.name == "order" then
      lua_state.order = tostring(lua_state.order or "asc") == "asc" and "desc" or "asc"
    elseif event.name == "sort" then
      local sort = tostring(lua_state.sort or "name")
      if sort == "name" then lua_state.sort = "author" elseif sort == "author" then lua_state.sort = "toggle" elseif sort == "toggle" then lua_state.sort = "debug" else lua_state.sort = "name" end
    elseif event.name == "prev_page" then
      if (lua_state.pages or 1) > 1 then lua_state.page = math.max(1, (lua_state.page or 1) - 1); local s,e=State.visible_range(lua_state.page); local list=lua_state.mod_list or {}; lua_state.select=tostring((list[s] or {}).uid or lua_state.select or "") end
    elseif event.name == "next_page" then
      if (lua_state.pages or 1) > 1 then lua_state.page = math.min(lua_state.pages or 1, (lua_state.page or 1) + 1); local s,e=State.visible_range(lua_state.page); local list=lua_state.mod_list or {}; lua_state.select=tostring((list[s] or {}).uid or lua_state.select or "") end
    elseif event.name == "jump" and (lua_state.pages or 1) > 1 then
      lua_state.jump = true; lua_state.user_page = 0
    elseif event.name == "return" then
      if lua_state.jump then lua_state.jump = false; lua_state.user_page = 0 else lua_state.back = true end
    end
  elseif event.type == "key" and lua_state.jump then
    local key = tostring(event.key or "")
    if key:match("^[0-9]$") then
      lua_state.user_page = math.min(9999, (lua_state.user_page or 0) * 10 + tonumber(key))
    elseif key == "backspace" then
      lua_state.user_page = math.floor((lua_state.user_page or 0) / 10)
    elseif key == "enter" then
      if (lua_state.user_page or 0) >= 1 and (lua_state.user_page or 0) <= (lua_state.pages or 1) then
        lua_state.page = lua_state.user_page
        local s,e=State.visible_range(lua_state.page); local list=lua_state.mod_list or {}; lua_state.select=tostring((list[s] or {}).uid or lua_state.select or "")
      end
      lua_state.jump = false; lua_state.user_page = 0
    end
  end

  lua_state.pages = State.pages()
  lua_state.page = math.max(1, math.min(lua_state.pages, lua_state.page or 1))
  return lua_state
end

function render(root_state)
  Render.render(root_state or {})
end
