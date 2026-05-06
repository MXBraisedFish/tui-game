local C = load_function("setting/constants.lua")
local L = load_function("setting/layout.lua")

local M = {}

local function menu_texts(root_state)
  return {
    {
      key = C.DEFAULT_TEXT.option1,
      label = L.language(root_state, "SETTING_LANGUAGE", C.DEFAULT_TEXT.language),
    },
    {
      key = C.DEFAULT_TEXT.option2,
      label = L.language(root_state, "SETTING_KEYBIND", C.DEFAULT_TEXT.keybind),
    },
    {
      key = C.DEFAULT_TEXT.option3,
      label = L.language(root_state, "SETTING_MODS", C.DEFAULT_TEXT.mods),
    },
    {
      key = C.DEFAULT_TEXT.option4,
      label = L.language(root_state, "SETTING_MEMORY", C.DEFAULT_TEXT.memory),
    },
    {
      key = C.DEFAULT_TEXT.option5,
      label = L.language(root_state, "SETTING_SECURITY", C.DEFAULT_TEXT.security),
    },
  }
end

local function selected_index(root_state)
  if type(root_state) == "table" and type(root_state.select) == "number" then
    return math.max(1, math.min(5, math.floor(root_state.select)))
  end
  return 1
end

local function menu_width(items)
  local max_width = 0
  for _, item in ipairs(items) do
    local key_width = math.max(L.text_width(item.key), L.text_width(C.DEFAULT_TEXT.enter))
    local width = L.text_width("▶ ") + key_width + L.text_width(" " .. item.label)
    if width > max_width then
      max_width = width
    end
  end
  return max_width
end

function M.draw_menu(root_state, origin_y)
  local items = menu_texts(root_state)
  local selected = selected_index(root_state)
  local x = L.center_x(menu_width(items), 0)
  for index, item in ipairs(items) do
    local is_selected = index == selected
    local prefix = is_selected and "▶ " or "  "
    local key_text = is_selected and C.DEFAULT_TEXT.enter or item.key
    local label_color = is_selected and C.SELECTED_COLOR or C.NORMAL_COLOR

    local y = origin_y + index - 1
    local cursor_x = x
    canvas_draw_text(cursor_x, y, prefix, label_color, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(prefix)
    canvas_draw_text(cursor_x, y, key_text, C.KEY_COLOR, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(key_text)
    canvas_draw_text(cursor_x, y, " " .. item.label, label_color, nil, BOLD, nil)
  end
end

return M
