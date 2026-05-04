local C = load_function("home/constants.lua")
local L = load_function("home/layout.lua")

local M = {}

local function draw_logo_line(x, y, text)
  local cursor_x = x
  for _, codepoint in utf8.codes(text) do
    local char = utf8.char(codepoint)
    local color = char == "█" and C.LOGO_COLOR or C.LOGO_EMPTY_COLOR
    canvas_draw_text(cursor_x, y, char, color, nil, BOLD, nil)
    cursor_x = cursor_x + get_text_width(char)
  end
end

function M.draw_logo(origin_y)
  local x = L.center_x(C.LOGO_WIDTH, 0)
  for index, line in ipairs(C.LOGO_LINES) do
    draw_logo_line(x, origin_y + index - 1, line)
  end
end

return M
