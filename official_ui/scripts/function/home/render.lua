local C = load_function("home/constants.lua")
local L = load_function("home/layout.lua")
local Logo = load_function("home/logo.lua")
local Menu = load_function("home/menu.lua")

local M = {}

local function version_text(state)
  if type(state) == "table" and state.version ~= nil and tostring(state.version) ~= "" then
    return "v" .. tostring(state.version)
  end
  return C.DEFAULT_TEXT.version
end

function M.render(state)
  canvas_clear()

  local content_y = L.content_top(C.CONTENT_HEIGHT)
  local logo_y = content_y
  local menu_y = content_y + C.LOGO_HEIGHT + 1
  local version_y = menu_y + C.MENU_HEIGHT + 1

  Logo.draw_logo(logo_y)
  Menu.draw_menu(state or {}, menu_y)

  local version = version_text(state)
  canvas_draw_text(L.center_x(L.text_width(version), 0), version_y, version, C.VERSION_COLOR, nil, nil, nil)
end

return M