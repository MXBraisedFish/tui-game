local M = {}

function M.draw(game_state)
  canvas_clear()
  local width, height = get_terminal_size()
  local title = translate("advanced_example.title")
  canvas_draw_text(resolve_x(ANCHOR_CENTER, get_text_width(title)), 1, title, "yellow", nil, BOLD)
  canvas_draw_text(2, 3, "Score: " .. tostring(game_state.score), "light_green")
  canvas_draw_text(18, 3, "Moves: " .. tostring(game_state.moves), "light_cyan")
  canvas_draw_text(2, 4, game_state.message or "", "white", nil, nil, ALIGN_LEFT, math.max(1, width - 4))
  canvas_draw_text(game_state.star.x, game_state.star.y, "*", "yellow", nil, BOLD)
  canvas_draw_text(game_state.player.x, game_state.player.y, "@", "light_cyan", nil, BOLD)
  canvas_draw_rich_text(2, math.max(0, height - 2), translate("advanced_example.help"), "grey", nil, ALIGN_LEFT, math.max(1, width - 4))
end

return M
