local M = {}

M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.TITLE_COLOR = "white"
M.HEADER_COLOR = "yellow"
M.TIP_COLOR = "dark_gray"
M.PATH_COLOR = "dark_gray"

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
  back_key = key_label(get_key("return").key_display.key_user)
}

return M
