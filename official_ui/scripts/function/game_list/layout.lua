local C = load_function("game_list/constants.lua")

local M = {}

function M.text_width(text)
  local width = get_text_width(text or "")
  if width == nil then
    return #(text or "")
  end
  return width
end

function M.text_height(text, wrap_width)
  local height = get_text_height(text or "", wrap_width)
  if height == nil then
    return 1
  end
  return height
end

function M.terminal_size()
  local width, height = get_terminal_size()
  return width or 98, height or 26
end

function M.language(root_state, key, fallback)
  if type(root_state) == "table" and type(root_state.language) == "table" then
    local value = root_state.language[key]
    if value ~= nil and tostring(value) ~= "" then
      return tostring(value)
    end
  end
  return fallback
end

function M.layout()
  local terminal_width, terminal_height = M.terminal_size()
  local content_height = math.max(3, terminal_height - 1)
  local left_width = math.max(C.MIN_PANEL_WIDTH, math.floor(terminal_width * C.LEFT_RATIO))
  left_width = math.min(left_width, terminal_width - C.MIN_PANEL_WIDTH)
  local right_width = math.max(C.MIN_PANEL_WIDTH, terminal_width - left_width)

  return {
    terminal_width = terminal_width,
    terminal_height = terminal_height,
    content_height = content_height,
    left_x = 0,
    left_y = 0,
    left_width = left_width,
    right_x = left_width,
    right_y = 0,
    right_width = right_width,
    right_height = content_height,
    list_capacity = math.max(1, content_height - 4),
    info_width = math.max(1, right_width - 2),
    info_height = math.max(1, content_height - 2)
  }
end

return M
