local M = {}

M.TITLE_COLOR = "white"
M.WARN_COLOR = "yellow"
M.CANCEL_COLOR = "green"
M.CONFIRM_COLOR = "blue"
M.DISABLED_COLOR = "dark_gray"
M.CONFIRM_DELAY_MS = 10000

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
  title = "Default Safe Mode Warning",
  warn = "Safe mode is designed to protect your computer and information.\nIf disabled, the host cannot block high-risk mod operations.\nMake sure you fully trust all mod package authors.\nConfirm disable?",
  close_permanent = "Disable permanently",
  cancel = "Cancel",
  second = "s",
  close_permanent_key = key_label(get_key("close_permanent").key_display.key_user),
  cancel_key = key_label(get_key("cancel").key_display.key_user)
}

return M
