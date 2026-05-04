local C = load_function("home/constants.lua")
local L = load_function("home/layout.lua")
local Logo = load_function("home/logo.lua")
local Menu = load_function("home/menu.lua")

local M = {}

local function version_text(root_state)
  if type(root_state) == "table" and root_state.version ~= nil and tostring(root_state.version) ~= "" then
    return "v" .. tostring(root_state.version)
  end
  return C.DEFAULT_TEXT.version
end

function M.render(root_state)
  canvas_clear()

  local content_y = L.content_top(C.CONTENT_HEIGHT)
  local logo_y = content_y
  local menu_y = content_y + C.LOGO_HEIGHT + 1
  local version_y = menu_y + C.MENU_HEIGHT + 1
  local action_y = version_y + 2

  Logo.draw_logo(logo_y)
  Menu.draw_menu(root_state or {}, menu_y)

  local version = version_text(root_state)

  local action = C.DEFAULT_TEXT.select_key .. " " .. L.language(root_state, "HOME_SELECT", C.DEFAULT_TEXT.continue_game) .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "HOME_CONFIRM", C.DEFAULT_TEXT.continue_game)

  canvas_draw_text(L.center_x(L.text_width(version), 0), version_y, version, C.VERSION_COLOR, nil, nil, nil)
  canvas_draw_text(L.center_x(L.text_width(action), 0), action_y, action, C.VERSION_COLOR, nil, nil, nil)
end

return M
