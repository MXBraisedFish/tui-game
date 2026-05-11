local M = {}

M.MIN_WIDTH = 98
M.MIN_HEIGHT = 26
M.TEXT_COLOR = "white"
M.HINT_COLOR = "dark_gray"
M.VALUE_COLOR = "light_cyan"
M.WARNING_COLOR = "yellow"
M.KEY_COLOR = "dark_gray"
M.BORDER_COLOR = "white"

function key_label(keys)
  if type(keys) == "string" then
    return "[" .. keys .. "]"
  elseif type(keys) == "table" then
    local formatted = {}
    for i, key in ipairs(keys) do
      formatted[i] = "[" .. key .. "]"
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
  actual = "Current terminal size: ",
  needed = "Required terminal size: ",
  hint = "Please resize the terminal until the interface restores.",
  exit_action = "Exit the program",
  return_action = "Return to game list",
  return_key_name = "Return/Exit",
  return_key = safe_key_label("return")
}

return M
