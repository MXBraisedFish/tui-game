local L = load_function("keybind_system/layout.lua")

local M = {}

M.listening_slot = 0

local root_state_cache = {}

local function non_empty(value, fallback)
  local text = tostring(value or "")
  if text == "" then
    return tostring(fallback or "")
  end
  return text
end

local function page_list()
  if type(root_state_cache.page_list) == "table" then
    return root_state_cache.page_list
  end
  return {}
end

local function action_list()
  if type(root_state_cache.action_list) == "table" then
    return root_state_cache.action_list
  end
  return {}
end

local function list_capacity()
  return math.max(1, L.layout().list_capacity or 1)
end

local function action_capacity()
  return math.max(1, L.layout().table_body_height or 1)
end

local function clamp(value, min_value, max_value)
  value = math.floor(tonumber(value) or min_value)
  if value < min_value then return min_value end
  if value > max_value then return max_value end
  return value
end

local function index_of_page(page_id)
  page_id = tostring(page_id or "")
  for index, item in ipairs(page_list()) do
    if tostring(item.id or "") == page_id then
      return index
    end
  end
  return 1
end

local function index_of_action(action_id)
  action_id = tostring(action_id or "")
  for index, item in ipairs(action_list()) do
    if tostring(item.id or "") == action_id then
      return index
    end
  end
  return 1
end

local function page_id_at(index)
  local item = page_list()[index]
  return item and tostring(item.id or "") or ""
end

local function action_id_at(index)
  local item = action_list()[index]
  return item and tostring(item.id or "") or ""
end

local function page_count()
  return #page_list()
end

local function action_count()
  return #action_list()
end

function M.set_root_state(root_state)
  root_state_cache = root_state or {}
end

function M.root_state()
  return root_state_cache
end

function M.pages()
  local count = page_count()
  if count == 0 then return 1 end
  return math.max(1, math.ceil(count / list_capacity()))
end

function M.visible_page_range(page)
  local capacity = list_capacity()
  page = clamp(page or 1, 1, M.pages())
  local first = (page - 1) * capacity + 1
  local last = math.min(page_count(), first + capacity - 1)
  return first, last
end

function M.select_first_visible_page(lua_state)
  local first, last = M.visible_page_range(lua_state.page)
  local selected_index = index_of_page(lua_state.select)
  if selected_index < first or selected_index > last then
    lua_state.select = page_id_at(first)
  end
  return lua_state
end

function M.has_empty_actions()
  for _, action in ipairs(action_list()) do
    if action.empty then return true end
  end
  return false
end

function M.move_page_select(lua_state, delta)
  local count = page_count()
  if count == 0 then
    lua_state.select = ""
    return lua_state
  end
  local index = index_of_page(lua_state.select) + delta
  if index < 1 then index = count end
  if index > count then index = 1 end
  lua_state.select = page_id_at(index)
  lua_state.page = math.max(1, math.ceil(index / list_capacity()))
  lua_state.action_select = ""
  lua_state.action_scroll = 0
  return lua_state
end

function M.move_action_select(lua_state, delta)
  local count = action_count()
  if count == 0 then
    lua_state.action_select = ""
    return lua_state
  end
  local index = index_of_action(lua_state.action_select) + delta
  if index < 1 then index = count end
  if index > count then index = 1 end
  lua_state.action_select = action_id_at(index)

  local capacity = action_capacity()
  if index <= lua_state.action_scroll then
    lua_state.action_scroll = math.max(0, index - 1)
  elseif index > lua_state.action_scroll + capacity then
    lua_state.action_scroll = math.max(0, index - capacity)
  end
  return lua_state
end

function M.scroll_actions(lua_state, delta)
  local max_scroll = math.max(0, action_count() - action_capacity())
  lua_state.action_scroll = clamp((lua_state.action_scroll or 0) + delta, 0, max_scroll)
  return lua_state
end

function M.normalize(lua_state)
  lua_state = lua_state or {}
  lua_state.select = non_empty(lua_state.select, root_state_cache.select)
  lua_state.action_select = non_empty(lua_state.action_select, root_state_cache.action_select)
  lua_state.focus = non_empty(lua_state.focus, root_state_cache.focus or "list")
  if lua_state.focus ~= "keys" then lua_state.focus = "list" end
  lua_state.mode = non_empty(lua_state.mode, root_state_cache.mode or "add")
  if lua_state.mode ~= "delete" then lua_state.mode = "add" end
  lua_state.order = non_empty(lua_state.order, root_state_cache.order or "asc")
  if lua_state.order ~= "desc" then lua_state.order = "asc" end
  lua_state.sort = non_empty(lua_state.sort, root_state_cache.sort or "name")
  if lua_state.sort ~= "conflict" then lua_state.sort = "name" end
  lua_state.pages = M.pages()
  lua_state.page = clamp(lua_state.page or root_state_cache.page or 1, 1, lua_state.pages)
  lua_state.user_page = math.max(0, math.floor(tonumber(lua_state.user_page) or 0))
  lua_state.jump = lua_state.jump == true
  lua_state.action_scroll = math.max(0, math.floor(tonumber(lua_state.action_scroll) or 0))
  lua_state.key_slot = clamp(lua_state.key_slot or root_state_cache.key_slot or 1, 1, 4)
  lua_state.waiting_slot = clamp(lua_state.waiting_slot or 0, 0, 4)

  if page_count() == 0 then
    lua_state.select = ""
  elseif page_id_at(index_of_page(lua_state.select)) ~= lua_state.select then
    lua_state.select = page_id_at(1)
  end

  if action_count() == 0 then
    lua_state.action_select = ""
  elseif action_id_at(index_of_action(lua_state.action_select)) ~= lua_state.action_select then
    lua_state.action_select = action_id_at(1)
  end

  local max_scroll = math.max(0, action_count() - action_capacity())
  if lua_state.action_scroll > max_scroll then
    lua_state.action_scroll = max_scroll
  end

  return lua_state
end

return M
