local L = load_function("game_list/layout.lua")

local M = {}

local current_root_state = {}
local current_layout = L.layout()

local function games()
  if type(current_root_state.game_list) == "table" then
    return current_root_state.game_list
  end
  return {}
end

local function game_count()
  return #games()
end

local function list_capacity()
  current_layout = L.layout()
  return math.max(1, current_layout.list_capacity or 1)
end

function M.set_root_state(root_state)
  current_root_state = root_state or {}
  current_layout = L.layout()
end

function M.layout()
  current_layout = L.layout()
  return current_layout
end

function M.pages()
  return math.max(1, math.ceil(game_count() / list_capacity()))
end

function M.page()
  local page = tonumber(current_root_state.page or 1) or 1
  return math.max(1, math.min(M.pages(), math.floor(page)))
end

function M.first_uid()
  local list = games()
  if #list == 0 then
    return ""
  end
  return tostring(list[1].uid or "")
end

function M.index_of(uid)
  uid = tostring(uid or "")
  for index, game in ipairs(games()) do
    if tostring(game.uid or "") == uid then
      return index
    end
  end
  return 1
end

function M.page_of(uid)
  return math.max(1, math.ceil(M.index_of(uid) / list_capacity()))
end

function M.uid_by_offset(uid, offset)
  local count = game_count()
  if count == 0 then
    return ""
  end
  local index = M.index_of(uid) + offset
  if index < 1 then
    index = count
  elseif index > count then
    index = 1
  end
  local game = games()[index]
  return game and tostring(game.uid or "") or ""
end

function M.visible_range(page)
  local capacity = list_capacity()
  local start_index = (math.max(1, page or M.page()) - 1) * capacity + 1
  local end_index = math.min(game_count(), start_index + capacity - 1)
  return start_index, end_index
end

return M
