local M = {}

local function progress_bar(value, width)
  local filled = math.floor(width * value / 100)
  local empty = width - filled
  return string.rep("█", filled) .. string.rep("░", empty)
end

local function draw_metric(y, label, value, width)
  canvas_draw_text(4, y, label .. ": ", "white", nil, BOLD)
  canvas_draw_text(18, y, progress_bar(value, width), "light_green")
  canvas_draw_text(20 + width, y, tostring(value) .. "%", "grey")
end

function M.render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  local title = translate("advanced_boss.title")
  local subtitle = translate("advanced_boss.subtitle")
  local hint = translate("advanced_boss.hint")
  local bar_width = math.max(10, math.min(42, width - 30))

  canvas_border_rect(0, 0, width, height, {
    top = "═",
    top_right = "╗",
    right = "║",
    bottom_right = "╝",
    bottom = "═",
    bottom_left = "╚",
    left = "║",
    top_left = "╔"
  }, "dark_grey")

  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(title)), 2, title, "light_cyan", nil, BOLD)
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(subtitle)), 3, subtitle, "grey")

  draw_metric(6, translate("advanced_boss.build"), state.progress.build, bar_width)
  draw_metric(8, translate("advanced_boss.tests"), state.progress.tests, bar_width)
  draw_metric(10, translate("advanced_boss.deploy"), state.progress.deploy, bar_width)

  canvas_draw_text(4, 13, translate("advanced_boss.logs"), "yellow", nil, BOLD)
  for index, line in ipairs(state.logs or {}) do
    local prefix = index == state.cursor and "> " or "  "
    local color = index == state.cursor and "light_green" or "grey"
    canvas_draw_text(4, 14 + index, prefix .. line, color)
  end

  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(hint)), math.max(0, height - 2), hint, "dark_grey")
end

return M
