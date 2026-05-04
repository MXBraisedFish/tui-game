local C = load_function("home/constants.lua")
local L = load_function("home/layout.lua")

local M = {}

local function menu_texts(state)
  return {
    {
      key = L.state_text(state, "key", "home_option1", C.DEFAULT_TEXT.option1),
      label = L.state_text(state, "home", "play", C.DEFAULT_TEXT.play),
      disabled = false,
    },
    {
      key = L.state_text(state, "key", "home_option2", C.DEFAULT_TEXT.option2),
      label = L.state_text(state, "home", "continue_game", C.DEFAULT_TEXT.continue_game),
      disabled = state.can_continue == false,
    },
    {
      key = L.state_text(state, "key", "home_option3", C.DEFAULT_TEXT.option3),
      label = L.state_text(state, "home", "settings", C.DEFAULT_TEXT.settings),
      disabled = false,
    },
    {
      key = L.state_text(state, "key", "home_option4", C.DEFAULT_TEXT.option4),
      label = L.state_text(state, "home", "about", C.DEFAULT_TEXT.about),
      disabled = false,
    },
    {
      key = L.state_text(state, "key", "home_option5", C.DEFAULT_TEXT.option5),
      label = L.state_text(state, "home", "quit", C.DEFAULT_TEXT.quit),
      disabled = false,
    },
  }
end

local function selected_index(state)
  if type(state) == "table" and type(state.selected_index) == "number" then
    return math.max(1, math.min(5, math.floor(state.selected_index)))
  end
  return 1
end

function M.draw_menu(state, origin_y)
  local items = menu_texts(state)
  local selected = selected_index(state)
  local content_width = 0
  for index, item in ipairs(items) do
    local key = index == selected and C.DEFAULT_TEXT.enter or item.key
    local content = "▶ " .. key .. " " .. item.label
    content_width = math.max(content_width, L.text_width(content))
  end

  local x = L.center_x(content_width, 0)
  for index, item in ipairs(items) do
    local is_selected = index == selected
    local prefix = is_selected and "▶ " or "  "
    local key_text = is_selected and C.DEFAULT_TEXT.enter or item.key
    local label_color = C.NORMAL_COLOR
    if item.disabled then
      label_color = C.DISABLED_COLOR
    elseif is_selected then
      label_color = C.SELECTED_COLOR
    end

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
