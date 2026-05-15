local L = load_function("mod_boss_list/layout.lua")

local M = {}

local current_root_state = {}
local current_layout = L.layout("full")

local function mods()
  if type(current_root_state.mod_list) == "table" then
    return current_root_state.mod_list
  end
  return {}
end

local function mod_count()
  return #mods()
end

local function list_mode()
  local mode = tostring(current_root_state.list_mode or "full")
  if mode == "brief" then
    return "brief"
  end
  return "full"
end

local function list_capacity()
  current_layout = L.layout(list_mode())
  return math.max(1, current_layout.list_capacity or 1)
end

function M.set_root_state(root_state)
  current_root_state = root_state or {}
  current_layout = L.layout(list_mode())
end

function M.layout()
  current_layout = L.layout(list_mode())
  return current_layout
end

function M.pages()
  return math.max(1, math.ceil(mod_count() / list_capacity()))
end

function M.page()
  local page = tonumber(current_root_state.page or 1) or 1
  return math.max(1, math.min(M.pages(), math.floor(page)))
end

function M.first_uid()
  local list = mods()
  if #list == 0 then
    return ""
  end
  return tostring(list[1].uid or "")
end

function M.index_of(uid)
  uid = tostring(uid or "")
  for index, item in ipairs(mods()) do
    if tostring(item.uid or "") == uid then
      return index
    end
  end
  return 1
end

function M.page_of(uid)
  return math.max(1, math.ceil(M.index_of(uid) / list_capacity()))
end

function M.uid_by_offset(uid, offset)
  local count = mod_count()
  if count == 0 then
    return ""
  end
  local index = M.index_of(uid) + offset
  if index < 1 then
    index = count
  elseif index > count then
    index = 1
  end
  local item = mods()[index]
  return item and tostring(item.uid or "") or ""
end

function M.visible_range(page)
  local capacity = list_capacity()
  local start_index = (math.max(1, page or M.page()) - 1) * capacity + 1
  local end_index = math.min(mod_count(), start_index + capacity - 1)
  return start_index, end_index
end

return M
