local C = load_function("memory/constants.lua")

local M = {}

function M.text_width(text)
  local width = get_text_width(text or "")
  if width == nil then
    return #(text or "")
  end
  return width
end

function M.terminal_size()
  local width, height = get_terminal_size()
  return width or 98, height or 26
end

function M.center_x(width, offset)
  return resolve_x(ANCHOR_CENTER, width, offset or 0)
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

function M.content_frame()
  local terminal_width, terminal_height = M.terminal_size()
  local width = math.min(C.MIN_TABLE_WIDTH, terminal_width - 4)
  local x = M.center_x(width, 0)
  local content_height = 12
  local y = resolve_y(ANCHOR_MIDDLE, content_height, 0)
  return {
    terminal_width = terminal_width,
    terminal_height = terminal_height,
    x = x,
    y = y,
    width = width,
    content_height = content_height,
    name_width = math.max(12, math.floor(width * 0.18)),
    size_width = math.max(12, math.floor(width * 0.16))
  }
end

return M
