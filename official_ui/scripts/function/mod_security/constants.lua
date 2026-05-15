local M = {}

M.TEMPORARY_DELAY_MS = 5000
M.PERMANENT_DELAY_MS = 10000
M.TITLE_COLOR = "white"
M.WARN_COLOR = "yellow"
M.CANCEL_COLOR = "green"
M.CONFIRM_COLOR = "red"
M.DISABLED_COLOR = DARK_GRAY
M.MOD_COLOR = "white"

local function key_label(keys)
  if type(keys) == "string" then
    return "[" .. keys .. "]"
  elseif type(keys) == "table" then
    local formatted = {}
    for index, key in ipairs(keys) do
      formatted[index] = "[" .. tostring(key) .. "]"
    end
    return table.concat(formatted, "/")
  end
  return "[]"
end

local function action_key(action)
  local value = get_key(action)
  if type(value) == "table" and type(value.key_display) == "table" then
    return value.key_display.key_user
  end
  return nil
end

M.DEFAULT_TEXT = {
  title = "Disable Safe Mode Warning",
  warn = "Safe mode is designed to protect your computer and personal information.\nDisabling this allows mod to perform high-risk operations.\nPlease ensure that you fully trust the author of this mod.\nAre you sure you want to disable it?",
  mod = "Mod: ",
  second = "s",
  cancel = "Cancel",
  close_temporary = "Disable (Only this session)",
  close_permanent = "Disable (Permanent)",
  cancel_key = key_label(action_key("cancel")),
  temporary_key = key_label(action_key("close_temporary")),
  permanent_key = key_label(action_key("close_permanent"))
}

return M
