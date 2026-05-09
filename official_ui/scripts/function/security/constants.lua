local M = {}

M.SELECTED_COLOR = "light_cyan"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.TITLE_COLOR = "white"
M.ON_COLOR = "green"
M.OFF_COLOR = "red"
M.RESET_COLOR = "white"
M.BRACKET_COLOR = "white"
M.MENU_HEIGHT = 4

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
  title = "Security Settings",
  default_safe_mode = "Default safe mode: ",
  default_mod = "Default mod state: ",
  reset_safe_mode = "Reset all mod safe modes to on",
  reset_mod = "Reset all mod states to off",
  safe_mode_on = "On",
  safe_mode_off = "Off (Permanent)",
  mod_on = "Enabled",
  mod_off = "Disabled",
  select = "Select",
  confirm = "Confirm",
  toggle_confirm = "Toggle / Confirm",
  back = "Back",
  option1 = key_label(get_key("option1").key_display.key_user),
  option2 = key_label(get_key("option2").key_display.key_user),
  option3 = key_label(get_key("option3").key_display.key_user),
  option4 = key_label(get_key("option4").key_display.key_user),
  select_key = key_label({get_key("prev_option").key_display.key_user, get_key("next_option").key_display.key_user}),
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  back_key = key_label(get_key("return").key_display.key_user)
}

return M
