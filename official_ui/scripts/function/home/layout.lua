local M = {}

function M.text_width(text)
  local width = get_text_width(text or "")
  if width == nil then
    return #(text or "")
  end
  return width
end

function M.center_x(width, offset)
  local terminal_width = 98
  local x = math.floor((terminal_width - width) / 2)
  return math.max(0, x + (offset or 0))
end

function M.content_top(content_height)
  local terminal_height = 26
  return math.max(0, math.floor((terminal_height - content_height) / 2))
end

function M.value_at(root, first_key, second_key, fallback)
  if type(root) == "table" and type(root[first_key]) == "table" then
    local value = root[first_key][second_key]
    if value ~= nil and tostring(value) ~= "" then
      return tostring(value)
    end
  end
  return fallback
end

function M.state_text(state, group, key, fallback)
  if type(state) ~= "table" then
    return fallback
  end
  if type(state.text) == "table" then
    return M.value_at(state.text, group, key, fallback)
  end
  return M.value_at(state, group, key, fallback)
end

return M