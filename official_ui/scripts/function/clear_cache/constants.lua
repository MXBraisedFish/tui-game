local M = {}

M.CONFIRM_DELAY_MS = 5000
M.TITLE_COLOR = "white"
M.WARN_COLOR = "yellow"
M.CANCEL_COLOR = "green"
M.CONFIRM_COLOR = "red"
M.DISABLED_COLOR = "dark_gray"
M.KEY_COLOR = "white"
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
  title = "Clear Cache Warning",
  warn = "Clearing cache will permanently delete all logs and cached data. This action cannot be undone. Are you sure you want to proceed?",
  cache_path = "Cache Directory: ",
  log_path = "Log Directory: ",
  second = "s",
  confirm = "Confirm",
  cancel = "Cancel",
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  cancel_key = key_label(get_key("cancel").key_display.key_user)
}

return M
