local C = load_function("warning_needed_size/constants.lua")
local L = load_function("warning_needed_size/layout.lua")

local M = {}

local function size_text(size)
  if type(size) ~= "table" then
    return "0 x 0"
  end
  return tostring(size.width or 0) .. " x " .. tostring(size.height or 0)
end

local function draw_line(x, y, parts)
  local cursor_x = x
  for _, part in ipairs(parts) do
    canvas_draw_text(cursor_x, y, part.text, part.color, nil, part.style, nil)
    cursor_x = cursor_x + L.text_width(part.text)
  end
end

local function center_x_for_text(text)
  return L.center_x(L.text_width(text), 0)
end

function M.render(root_state)
  canvas_clear()

  local actual_label = L.language(root_state, "WARNING_SIZE_ACTUAL", C.DEFAULT_TEXT.actual)
  local needed_label = L.language(root_state, "WARNING_SIZE_NEEDED", C.DEFAULT_TEXT.needed)
  local hint = L.language(root_state, "WARNING_SIZE_HINT", C.DEFAULT_TEXT.hint)
  local exit_text = L.language(root_state, "WARNING_SIZE_ACTION_EXIT", C.DEFAULT_TEXT.exit_action)
  local return_text = L.language(root_state, "WARNING_SIZE_ACTION_RETURN", C.DEFAULT_TEXT.return_action)
  local return_key_name = L.language(root_state, "KEY_SIZE_RETURN", C.DEFAULT_TEXT.return_key_name)
  local action_text = root_state.mode == "game" and return_text or exit_text
  local action_line_text = C.DEFAULT_TEXT.return_key .. " " .. action_text

  local lines = {
    { text = needed_label .. size_text(root_state.needed), color = C.WARNING_COLOR },
    { text = actual_label .. size_text(root_state.actual), color = C.TEXT_COLOR },
    { text = hint, color = C.HINT_COLOR },
    { text = action_line_text, color = C.HINT_COLOR },
  }

  local content_height = #lines + 2
  local y = L.center_y(content_height, 0)

  draw_line(center_x_for_text(lines[1].text), y, {
    { text = lines[1].text, color = C.WARNING_COLOR, style = BOLD },
  })
  draw_line(center_x_for_text(lines[1].text), y + 2, {
    { text = actual_label, color = C.TEXT_COLOR, style = BOLD },
    { text = size_text(root_state.actual), color = C.VALUE_COLOR, style = BOLD },
  })
  draw_line(center_x_for_text(lines[3].text), y + 3, {
    { text = hint, color = C.HINT_COLOR, style = nil },
  })
  draw_line(center_x_for_text(lines[4].text), y + 5, {
    { text = action_line_text, color = C.HINT_COLOR, style = nil },
  })
end

return M
