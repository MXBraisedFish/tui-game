local M = {}

M.WIDTH = 98
M.HEIGHT = 26
M.LOGO_WIDTH = 61
M.LOGO_HEIGHT = 6
M.MENU_WIDTH = 30
M.MENU_HEIGHT = 5
M.CONTENT_HEIGHT = 15
M.LOGO_COLOR = "#ffa500"
M.LOGO_EMPTY_COLOR = "white"
M.SELECTED_COLOR = "light_cyan"
M.NORMAL_COLOR = "white"
M.DISABLED_COLOR = "dark_gray"
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
end

M.LOGO_LINES = {
  "████████╗██╗   ██╗██╗     ██████╗  █████╗ ███╗   ███╗███████╗",
  "╚══██╔══╝██║   ██║██║    ██╔════╝ ██╔══██╗████╗ ████║██╔════╝",
  "   ██║   ██║   ██║██║    ██║  ███╗███████║██╔████╔██║█████╗  ",
  "   ██║   ██║   ██║██║    ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝  ",
  "   ██║   ╚██████╔╝██║    ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗",
  "   ╚═╝    ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝",
}

M.DEFAULT_TEXT = {
  play = "Start Game",
  continue_game = "Continue",
  settings = "Settings",
  about = "About",
  quit = "Quit",
  enter = key_label(get_key("confirm").key_display.key_user),
  option1 = key_label(get_key("option1").key_display.key_user),
  option2 = key_label(get_key("option2").key_display.key_user),
  option3 = key_label(get_key("option3").key_display.key_user),
  option4 = key_label(get_key("option4").key_display.key_user),
  option5 = key_label(get_key("option5").key_display.key_user),
  select_key = key_label({get_key("prev_option").key_display.key_user, get_key("next_option").key_display.key_user}),
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  version = "v0.0.0",
  select = "Select",
  confirm = "Confirm"
}

return M
