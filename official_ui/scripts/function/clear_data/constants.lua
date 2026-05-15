local M = {}

M.CONFIRM_DELAY_MS = 5000
M.TITLE_COLOR = "white"
M.WARN_COLOR = "yellow"
M.CANCEL_COLOR = "green"
M.CONFIRM_COLOR = "red"
M.DISABLED_COLOR = DARK_GRAY
M.KEY_COLOR = "white"
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
  title = "Clear Data Warning",
  warn = "Clearing data will permanently delete all records. This action cannot be undone. Are you sure you want to proceed?",
  path = "Directory: ",
  second = "s",
  confirm = "Confirm",
  cancel = "Cancel",
  confirm_key = safe_key_label("confirm"),
  cancel_key = safe_key_label("cancel")
}

return M
