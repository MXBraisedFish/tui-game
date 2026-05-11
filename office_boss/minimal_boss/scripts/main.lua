function update(state)
  state.ticks = (state.ticks or 0) + 1
  return state
end

function render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  local title = translate("minimal_boss.title")
  local status = translate("minimal_boss.status")
  local hint = translate("minimal_boss.hint")

  canvas_draw_text(2, 1, title, "white", nil, BOLD)
  canvas_draw_text(2, 3, "> cargo check", "grey")
  canvas_draw_text(2, 4, status, "light_green")
  canvas_draw_text(2, 6, "src/main.rs", "light_cyan")
  canvas_draw_text(2, 7, "src/host_engine/runtime/event_loop.rs", "light_cyan")
  canvas_draw_text(2, 9, "Problems: 0  Warnings: 0", "grey")
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(hint)), math.max(0, height - 2), hint, "dark_grey")
  canvas_border_rect(0, 0, width, height, {
    top = "─",
    top_right = "┐",
    right = "│",
    bottom_right = "┘",
    bottom = "─",
    bottom_left = "└",
    left = "│",
    top_left = "┌"
  }, "dark_grey")
end
