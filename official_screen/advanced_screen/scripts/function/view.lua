local M = {}

function M.render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  local title = translate("advanced_screen.title")
  local hint = translate("advanced_screen.hint")
  local time_text = translate("advanced_screen.time") .. ": " .. tostring(math.floor(running_time() / 1000)) .. "s"

  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(title)), 1, title, "#ffa500", nil, BOLD)
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(time_text)), 3, time_text, "light_cyan")

  for _, particle in ipairs(state.particles or {}) do
    canvas_draw_text(particle.x, particle.y, particle.char, "light_yellow", nil, BOLD)
  end

  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(hint)), math.max(0, height - 2), hint, "grey")
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
end

return M
