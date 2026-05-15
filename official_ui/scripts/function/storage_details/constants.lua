local M = {}

M.NORMAL_COLOR = "white"
M.KEY_COLOR = DARK_GRAY
M.TITLE_COLOR = "white"
M.HEADER_COLOR = "yellow"
M.TIP_COLOR = DARK_GRAY
M.PATH_COLOR = DARK_GRAY

local function key_label(keys)
  if type(keys) == "string" then
    return "[" .. keys .. "]"
  elseif type(keys) == "table" then
    local formatted = {}
    for index, key in ipairs(keys) do
      formatted[index] = "[" .. key .. "]"
    end
    return table.concat(formatted, "/")
  end
  return "[]"
end

local function safe_key_label(action_name)
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return key_label(info.key_display.key_user)
  end
  return "[" .. tostring(action_name or "?") .. "]"
end

M.DEFAULT_TEXT = {
  title = "View Storage Details",
  dir = "Directory",
  size = "Size",
  path = "Path",
  root = "Root",
  data_dir = "Data",
  cache_dir = "Cache",
  profiles_dir = "Profiles",
  log_dir = "Log",
  mod_dir = "Mod",
  tip = "Sizes use 1024-based units",
  back = "Back",
  back_key = safe_key_label("return")
}

return M
