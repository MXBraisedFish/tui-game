local M = {}

M.MENU_WIDTH = 34
M.MENU_HEIGHT = 5
M.CONTENT_HEIGHT = 9
M.SELECTED_COLOR = "light_cyan"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.VERSION_COLOR = "dark_gray"

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

M.DEFAULT_TEXT = {
  language = "Language",
  keybind = "Key Bindings",
  mods = "Mod List",
  memory = "Memory Management",
  security = "Security Settings",
  enter = key_label(get_key("confirm").key_display.key_user),
  option1 = key_label(get_key("option1").key_display.key_user),
  option2 = key_label(get_key("option2").key_display.key_user),
  option3 = key_label(get_key("option3").key_display.key_user),
  option4 = key_label(get_key("option4").key_display.key_user),
  option5 = key_label(get_key("option5").key_display.key_user),
  back_key = key_label(get_key("return").key_display.key_user),
  select_key = key_label({get_key("prev_option").key_display.key_user, get_key("next_option").key_display.key_user}),
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  select = "Select",
  confirm = "Confirm",
  back = "Back"
}

return M
