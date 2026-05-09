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

function M.wrap(text, width)
  width = width or 72
  local lines = {}
  local source = tostring(text or "")
  for line in string.gmatch(source .. "\n", "([^\n]*)\n") do
    if line == "" then
      lines[#lines + 1] = ""
    else
      local current = ""
      local current_width = 0
      for _, code in utf8.codes(line) do
        local character = utf8.char(code)
        local character_width = M.text_width(character)
        if current_width > 0 and current_width + character_width > width then
          lines[#lines + 1] = current
          current = character
          current_width = character_width
        else
          current = current .. character
          current_width = current_width + character_width
        end
      end
      if current ~= "" then
        lines[#lines + 1] = current
      end
    end
  end
  return lines
end

return M
