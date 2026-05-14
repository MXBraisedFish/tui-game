local C = load_function("mod_hub/constants.lua")
local L = load_function("mod_hub/layout.lua")
local Menu = load_function("mod_hub/menu.lua")

local M = {}

local function draw_title(root_state)
  local title = L.language(root_state, "MOD_LIST_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function draw_action_line(root_state)
  local select_text = L.language(root_state, "MOD_LIST_SELECT", C.DEFAULT_TEXT.select)
  local confirm_text = L.language(root_state, "MOD_LIST_CONFIRM", C.DEFAULT_TEXT.confirm)
  local back_text = L.language(root_state, "MOD_LIST_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.select_key .. " " .. select_text
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. confirm_text
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. back_text
  local _, terminal_height = get_terminal_size()
  canvas_draw_text(L.center_x(L.text_width(action), 0), (terminal_height or 26) - 1, action, C.KEY_COLOR, nil, nil, nil)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  draw_title(root_state)
  Menu.draw_menu(root_state, L.content_top(C.MENU_HEIGHT))
  draw_action_line(root_state)
end

return M
