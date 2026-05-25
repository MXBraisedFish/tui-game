local M = {}

M.SELECTED_COLOR = CYAN
M.NORMAL_COLOR = "white"
M.KEY_COLOR = DARK_GRAY
M.TITLE_COLOR = "white"
M.ON_COLOR = "green"
M.OFF_COLOR = "red"
M.RESET_COLOR = "white"
M.BRACKET_COLOR = "white"
M.MENU_HEIGHT = 8

local function append_key_labels(value, formatted)
  if type(value) == "table" then
    for _, item in ipairs(value) do
      append_key_labels(item, formatted)
    end
    return
  end

  local key = tostring(value or "")
  if key ~= "" then
    formatted[#formatted + 1] = "[" .. key .. "]"
  end
end

local function key_label(keys)
  local formatted = {}
  append_key_labels(keys, formatted)
  if #formatted == 0 then
    return "[]"
  end
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
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return key_label(info.key_display.key_user)
  end
  return "[" .. tostring(action_name or "?") .. "]"
end

M.DEFAULT_TEXT = {
  title = "Security Settings",
  default_safe_mode = "Default safe mode: ",
  default_mod_game = "Game pack default state: ",
  default_mod_screensaver = "Screensaver pack default state: ",
  default_mod_boss = "Boss pack default state: ",
  reset_safe_mode = "Reset all game pack safe modes to on",
  reset_mod_game = "Reset all game pack states to off",
  reset_mod_screensaver = "Reset all screensaver pack states to off",
  reset_mod_boss = "Reset all boss pack states to off",
  reset_success = "Reset successful",
  reset_failed = "Reset failed",
  safe_mode_on = "On",
  safe_mode_off = "Off (Permanent)",
  mod_on = "Enabled",
  mod_off = "Disabled",
  select = "Select",
  confirm = "Confirm",
  toggle_confirm = "Toggle / Confirm",
  back = "Back",
  option1 = safe_key_label("option1"),
  option2 = safe_key_label("option2"),
  option3 = safe_key_label("option3"),
  option4 = safe_key_label("option4"),
  option5 = safe_key_label("option5"),
  option6 = safe_key_label("option6"),
  option7 = safe_key_label("option7"),
  option8 = safe_key_label("option8"),
  select_key = key_label({safe_key_value("prev_option"), safe_key_value("next_option")}),
  confirm_key = safe_key_label("confirm"),
  back_key = safe_key_label("return")
}

return M
