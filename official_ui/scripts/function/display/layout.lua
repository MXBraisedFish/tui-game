local M = {}

function M.language(root_state, key, fallback)
  if type(root_state) == "table" and type(root_state.language) == "table" then
    local value = root_state.language[key]
    if type(value) == "string" and value ~= "" then
      return value
    end
  end
  return fallback or key
end

function M.text_width(value)
  return get_text_width(tostring(value or ""))
end

function M.terminal_size()
  local width, height = get_terminal_size()
  return width or 98, height or 26
end

function M.center_x(width, offset)
  local terminal_width = select(1, M.terminal_size())
  return math.max(0, math.floor((terminal_width - width) / 2) + (offset or 0))
end

function M.center_block(width, height)
  local terminal_width, terminal_height = M.terminal_size()
  return math.max(0, math.floor((terminal_width - width) / 2)), math.max(3, math.floor((terminal_height - height) / 2))
end

function M.value_text(root_state, key, fallback)
  return M.language(root_state, key, fallback)
end

return M
