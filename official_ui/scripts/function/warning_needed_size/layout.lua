local M = {}

function M.text_width(text)
  local width = get_text_width(text or "")
  if width == nil then
    return #(text or "")
  end
  return width
end

function M.center_x(width, offset)
  return resolve_x(ANCHOR_CENTER, width, offset or 0)
end

function M.center_y(height, offset)
  return resolve_y(ANCHOR_MIDDLE, height, offset or 0)
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

return M
