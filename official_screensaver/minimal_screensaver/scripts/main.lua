function update(state)
  local width, height = get_terminal_size()
  state.x = (state.x or 0) + 1
  state.y = state.y or math.floor(height / 2)
  if state.x >= width then
    state.x = 0
  end
  return state
end

function render(state)
  canvas_clear()
  local width, height = get_terminal_size()
  local title = translate("minimal_screensaver.title")
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(title)), 2, title, "yellow", nil, BOLD)
  canvas_draw_text(state.x or 0, state.y or math.floor(height / 2), "*", "light_cyan", nil, BOLD)
  local hint = translate("minimal_screensaver.hint")
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(hint)), math.max(0, height - 2), hint, "grey")
end
