local L = load_function("language/layout.lua")

local M = {}

local ROOT_STATE_CACHE_KEY = "__tg_language_root_state_cache"
local GRID_CACHE_KEY = "__tg_language_grid_cache"

function M.set_root_state(root_state)
  _G[ROOT_STATE_CACHE_KEY] = root_state or {}
  _G[GRID_CACHE_KEY] = L.grid(_G[ROOT_STATE_CACHE_KEY])
end

function M.root_state()
  return _G[ROOT_STATE_CACHE_KEY] or {}
end

function M.grid()
  if _G[GRID_CACHE_KEY] == nil then
    _G[GRID_CACHE_KEY] = L.grid(M.root_state())
  end
  return _G[GRID_CACHE_KEY]
end

local function selected_index(select_code)
  local grid = M.grid()
  if #grid.order == 0 then
    return 1
  end
  for index, code in ipairs(grid.order) do
    if code == select_code then
      return index
    end
  end
  return 1
end

local function code_at(index)
  local grid = M.grid()
  if #grid.order == 0 then
    return nil
  end
  if index < 1 or index > #grid.order then
    return nil
  end
  return grid.order[index]
end

local function normalize_page(page, pages)
  page = tonumber(page) or 1
  pages = math.max(1, tonumber(pages) or 1)
  return math.max(1, math.min(pages, math.floor(page)))
end

local function page_bounds(page)
  local grid = M.grid()
  local current_page = normalize_page(page, grid.pages)
  local start_index = ((current_page - 1) * grid.per_page) + 1
  local end_index = math.min(#grid.order, start_index + grid.per_page - 1)
  return start_index, end_index
end

local function normalize_state(lua_state)
  lua_state = lua_state or {}
  local grid = M.grid()
  if #grid.order > 0 then
    lua_state.select = code_at(selected_index(lua_state.select))
  else
    lua_state.select = tostring(lua_state.select or "en_us")
  end
  lua_state.pages = grid.pages
  lua_state.page = normalize_page(lua_state.page, grid.pages)
  lua_state.user_page = lua_state.jump and (tonumber(lua_state.user_page) or 0) or 0
  lua_state.jump = lua_state.jump == true
  lua_state.confirm = false
  lua_state.back = false

  local selected = selected_index(lua_state.select)
  local selected_page = math.max(1, math.ceil(selected / grid.per_page))
  lua_state.page = normalize_page(selected_page, grid.pages)
  return lua_state
end

function M.normalize_state(lua_state)
  return normalize_state(lua_state)
end

function M.return_state(lua_state)
  local confirm = lua_state and lua_state.confirm == true
  local back = lua_state and lua_state.back == true
  lua_state = normalize_state(lua_state)
  lua_state.confirm = confirm
  lua_state.back = back
  return lua_state
end

function M.move(lua_state, delta)
  lua_state = normalize_state(lua_state)
  local grid = M.grid()
  local current_index = selected_index(lua_state.select)
  local start_index, end_index = page_bounds(lua_state.page)
  local next_index = current_index + delta

  if delta == -1 and current_index <= start_index then
    if lua_state.page > 1 then
      return M.flip_page(lua_state, -1)
    end
    next_index = end_index
  elseif delta == 1 and current_index >= end_index then
    if lua_state.page < grid.pages then
      return M.flip_page(lua_state, 1)
    end
    next_index = start_index
  elseif delta < -1 and next_index < start_index then
    next_index = math.min(end_index, current_index + ((grid.rows - 1) * grid.columns))
  elseif delta > 1 and next_index > end_index then
    next_index = start_index + ((current_index - start_index) % grid.columns)
    if next_index > end_index then
      next_index = start_index
    end
  end

  local next_code = code_at(next_index)
  if next_code ~= nil then
    lua_state.select = next_code
  end
  return normalize_state(lua_state)
end

function M.flip_page(lua_state, delta)
  lua_state = normalize_state(lua_state)
  local grid = M.grid()
  local target_page = normalize_page(lua_state.page + delta, grid.pages)
  local target_index = ((target_page - 1) * grid.per_page) + 1
  local next_code = code_at(target_index)
  if next_code ~= nil then
    lua_state.select = next_code
  end
  return normalize_state(lua_state)
end

function M.jump_page(lua_state, page)
  lua_state = normalize_state(lua_state)
  local grid = M.grid()
  local target_page = normalize_page(page, grid.pages)
  local target_index = ((target_page - 1) * grid.per_page) + 1
  local next_code = code_at(target_index)
  if next_code ~= nil then
    lua_state.select = next_code
  end
  lua_state.jump = false
  lua_state.user_page = 0
  return normalize_state(lua_state)
end

return M
