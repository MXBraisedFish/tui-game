local C = load_function("security/constants.lua")
local L = load_function("security/layout.lua")

local M = {}

local function selected_index(root_state, count)
  if type(root_state) == "table" and type(root_state.select) == "number" then
    return math.max(1, math.min(count or 1, math.floor(root_state.select)))
  end
  return 1
end

local function option_items(root_state)
  local safe_mode_on = root_state.default_safe_mode ~= false
  local mod_enabled = root_state.default_mod_enabled ~= false
  return {
    {
      key = C.DEFAULT_TEXT.option1,
      label = L.language(root_state, "SECURITY_DEFAULT_SAFE_MODE", C.DEFAULT_TEXT.default_safe_mode),
      value = safe_mode_on
        and L.language(root_state, "SECURITY_TOGGLE_SAFE_MODE_ON", C.DEFAULT_TEXT.safe_mode_on)
        or L.language(root_state, "SECURITY_TOGGLE_SAFE_MODE_OFF_PERMANENT", C.DEFAULT_TEXT.safe_mode_off),
      value_color = safe_mode_on and C.ON_COLOR or C.OFF_COLOR,
      is_toggle = true
    },
    {
      key = C.DEFAULT_TEXT.option2,
      label = L.language(root_state, "SECURITY_DEFAULT_MOD", C.DEFAULT_TEXT.default_mod),
      value = mod_enabled
        and L.language(root_state, "SECURITY_TOGGLE_MOD_ON", C.DEFAULT_TEXT.mod_on)
        or L.language(root_state, "SECURITY_TOGGLE_MOD_OFF", C.DEFAULT_TEXT.mod_off),
      value_color = mod_enabled and C.ON_COLOR or C.OFF_COLOR,
      is_toggle = true
    },
    {
      key = C.DEFAULT_TEXT.option3,
      label = L.language(root_state, "SECURITY_RESET_SAFE_MODE", C.DEFAULT_TEXT.reset_safe_mode),
      is_toggle = false
    },
    {
      key = C.DEFAULT_TEXT.option4,
      label = L.language(root_state, "SECURITY_RESET_MOD", C.DEFAULT_TEXT.reset_mod),
      is_toggle = false
    }
  }
end

local function item_width(item)
  local key_width = math.max(L.text_width(item.key), L.text_width(C.DEFAULT_TEXT.confirm_key))
  local width = L.text_width("▶ ") + key_width + L.text_width(" " .. item.label)
  if item.is_toggle then
    width = width + L.text_width("[ " .. item.value .. " ]")
  end
  return width
end

local function draw_toggle_value(x, y, value, color)
  canvas_draw_text(x, y, "[ ", C.BRACKET_COLOR, nil, BOLD, nil)
  x = x + L.text_width("[ ")
  canvas_draw_text(x, y, value, color, nil, BOLD, nil)
  x = x + L.text_width(value)
  canvas_draw_text(x, y, " ]", C.BRACKET_COLOR, nil, BOLD, nil)
end

local function draw_options(root_state)
  local items = option_items(root_state)
  local selected = selected_index(root_state, #items)
  local option_width = 0

  for _, item in ipairs(items) do
    option_width = math.max(option_width, item_width(item))
  end

  local option_x = L.center_x(option_width, 0)
  local y = L.content_top(#items)

  for index, item in ipairs(items) do
    local is_selected = index == selected
    local prefix = is_selected and "▶ " or "  "
    local key_text = is_selected and C.DEFAULT_TEXT.confirm_key or item.key
    local color = is_selected and C.SELECTED_COLOR or C.NORMAL_COLOR
    local cursor_x = option_x

    canvas_draw_text(cursor_x, y + index - 1, prefix, color, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(prefix)
    canvas_draw_text(cursor_x, y + index - 1, key_text, C.KEY_COLOR, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(key_text)
    canvas_draw_text(cursor_x, y + index - 1, " " .. item.label, color, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(" " .. item.label)

    if item.is_toggle then
      draw_toggle_value(cursor_x, y + index - 1, item.value, item.value_color)
    end
  end
end

local function draw_title(root_state)
  local title = L.language(root_state, "SECURITY_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function draw_action_line(root_state)
  local select_text = L.language(root_state, "SECURITY_SELECT", C.DEFAULT_TEXT.select)
  local confirm_text = L.language(root_state, "SECURITY_TOGGLE_CONFIRM", C.DEFAULT_TEXT.toggle_confirm)
  local back_text = L.language(root_state, "SECURITY_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.select_key .. " " .. select_text
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. confirm_text
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. back_text
  local terminal_width, terminal_height = L.terminal_size()
  local wrap_width = math.max(1, terminal_width - 2)
  local action_width = math.min(get_text_width(action, wrap_width), wrap_width)
  local action_height = math.max(1, get_text_height(action, wrap_width))
  local x = math.max(0, math.floor((terminal_width - action_width) / 2))
  local y = math.max(0, terminal_height - action_height)
  canvas_draw_text(x, y, action, C.KEY_COLOR, nil, nil, ALIGN_LEFT, wrap_width)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  draw_title(root_state)
  draw_options(root_state)
  draw_action_line(root_state)
end

return M
