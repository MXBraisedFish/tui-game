local M = {}

M.SELECTED_COLOR = CYAN
M.NORMAL_COLOR = "white"
M.KEY_COLOR = DARK_GRAY
M.TITLE_COLOR = "white"
M.MENU_HEIGHT = 3

local function append_key_labels(value, formatted)
  if type(value) == "table" then
    for _, item in ipairs(value) do append_key_labels(item, formatted) end
    return
  end
  local key = tostring(value or "")
  if key ~= "" then formatted[#formatted + 1] = "[" .. key .. "]" end
end

local function key_label(keys)
  local formatted = {}
  append_key_labels(keys, formatted)
  if #formatted == 0 then return "[]" end
  return table.concat(formatted, "/")
end

local function safe_key_value(action_name)
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return info.key_display.key_user
  end
  return tostring(action_name or "?")
end

local function safe_key_label(action_name)
  return key_label(safe_key_value(action_name))
end

M.DEFAULT_TEXT = {
  title = "Mod Settings",
  game = "Game Pack",
  screensaver = "Screensaver Pack",
  boss = "Boss Pack",
  select = "Select",
  confirm = "Confirm",
  back = "Back",
  option1 = safe_key_label("option1"),
  option2 = safe_key_label("option2"),
  option3 = safe_key_label("option3"),
  select_key = key_label({safe_key_value("prev_option"), safe_key_value("next_option")}),
  confirm_key = safe_key_label("confirm"),
  back_key = safe_key_label("return")
}

return M
