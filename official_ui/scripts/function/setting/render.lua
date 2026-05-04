local C = load_function("setting/constants.lua")
local L = load_function("setting/layout.lua")
local Menu = load_function("setting/menu.lua")

local M = {}

function M.render(root_state)
  canvas_clear()

  local content_y = L.content_top(C.CONTENT_HEIGHT)
  local menu_y = content_y
  local action_y = menu_y + C.MENU_HEIGHT + 2

  Menu.draw_menu(root_state or {}, menu_y)

  local select_text = L.language(root_state, "SETTING_SELECT", C.DEFAULT_TEXT.select)
  local confirm_text = L.language(root_state, "SETTING_CONFIRM", C.DEFAULT_TEXT.confirm)
  local back_text = L.language(root_state, "SETTING_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.select_key .. " " .. select_text
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. confirm_text
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. back_text

  canvas_draw_text(L.center_x(L.text_width(action), 0), action_y, action, C.VERSION_COLOR, nil, nil, nil)
end

return M
