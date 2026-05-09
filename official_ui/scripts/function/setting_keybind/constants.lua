local M = {}

M.SELECTED_COLOR = "light_cyan"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.TITLE_COLOR = "white"
M.MENU_HEIGHT = 3

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
  title = "Key Bindings",
  global = "Global Keys",
  system = "System Keys",
  game = "Game Keys",
  select = "Select",
  confirm = "Confirm",
  back = "Back",
  option1 = key_label(get_key("option1").key_display.key_user),
  option2 = key_label(get_key("option2").key_display.key_user),
  option3 = key_label(get_key("option3").key_display.key_user),
  select_key = key_label({get_key("prev_option").key_display.key_user, get_key("next_option").key_display.key_user}),
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  back_key = key_label(get_key("return").key_display.key_user)
}

return M
