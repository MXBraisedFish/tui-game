local C = load_function("home/constants.lua")
local L = load_function("home/layout.lua")

local M = {}

local HOME_PLAY

local function menu_texts(root_state)
  local has_continue = type(root_state) == "table" and type(root_state.continue) == "table" and next(root_state.continue) ~= nil
  HOME_PLAY = L.language(root_state, "HOME_PLAY", C.DEFAULT_TEXT.play)
  return {
    {
      key = C.DEFAULT_TEXT.option1,
      label = HOME_PLAY,
      disabled = false,
    },
    {
      key = C.DEFAULT_TEXT.option2,
      label = L.language(root_state, "HOME_CONTINUE", C.DEFAULT_TEXT.continue_game),
      disabled = not has_continue,
    },
    {
      key = C.DEFAULT_TEXT.option3,
      label = L.language(root_state, "HOME_SETTINGS", C.DEFAULT_TEXT.settings),
      disabled = false,
    },
    {
      key = C.DEFAULT_TEXT.option4,
      label = L.language(root_state, "HOME_ABOUT", C.DEFAULT_TEXT.about),
      disabled = false,
    },
    {
      key = C.DEFAULT_TEXT.option5,
      label = L.language(root_state, "HOME_QUIT", C.DEFAULT_TEXT.quit),
      disabled = false,
    },
  }
end

function get_max_text_width(data_table)
    if not data_table or #data_table == 0 then
        return 0
    end
    
    local max_width = 0

    for i = 1, #data_table do
        local label = data_table[i].label
        if label then
            local width = get_text_width(label)
            if width > max_width then
                max_width = width
            end
        end
    end
    
    return max_width
end

local function selected_index(root_state)
  if type(root_state) == "table" and type(root_state.select) == "number" then
    return math.max(1, math.min(5, math.floor(root_state.select)))
  end
  return 1
end

local function alignment_width(items)
  return get_max_text_width(items)
end

function M.draw_menu(root_state, origin_y)
  local items = menu_texts(root_state)
  local selected = selected_index(root_state)
  local x = L.center_x(alignment_width(items), -3)
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
