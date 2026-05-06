local C = load_function("setting/constants.lua")
local L = load_function("setting/layout.lua")
local Menu = load_function("setting/menu.lua")

local M = {}

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}

  local title = L.language(root_state, "SETTING_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)

  local menu_y = L.content_top(C.MENU_HEIGHT)
  Menu.draw_menu(root_state, menu_y)

  local select_text = L.language(root_state, "SETTING_SELECT", C.DEFAULT_TEXT.select)
  local confirm_text = L.language(root_state, "SETTING_CONFIRM", C.DEFAULT_TEXT.confirm)
  local back_text = L.language(root_state, "SETTING_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.select_key .. " " .. select_text
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. confirm_text
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. back_text

  local _, terminal_height = get_terminal_size()
  canvas_draw_text(L.center_x(L.text_width(action), 0), (terminal_height or 26) - 1, action, C.VERSION_COLOR, nil, nil, nil)
end

return M
