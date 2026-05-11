local state = load_function("state.lua")

local M = {}

local function clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

local function move_player(game_state, dx, dy)
  local width, height = get_terminal_size()
  game_state.player.x = clamp(game_state.player.x + dx, 1, math.max(1, width - 2))
  game_state.player.y = clamp(game_state.player.y + dy, 4, math.max(4, height - 3))
  game_state.moves = game_state.moves + 1
end

function M.handle(game_state, event)
  if event.type == "action" then
    if event.name == "up" then
      move_player(game_state, 0, -1)
    elseif event.name == "down" then
      move_player(game_state, 0, 1)
    elseif event.name == "left" then
      move_player(game_state, -1, 0)
    elseif event.name == "right" then
      move_player(game_state, 1, 0)
    elseif event.name == "collect" then
      if game_state.player.x == game_state.star.x and game_state.player.y == game_state.star.y then
        local width, height = get_terminal_size()
        game_state.score = game_state.score + 1
        game_state.star = state.spawn_star(width, height)
        game_state.message = translate("advanced_example.collected")
      else
        game_state.message = translate("advanced_example.missed")
      end
    elseif event.name == "reset" then
      local width, height = get_terminal_size()
      game_state.score = 0
      game_state.moves = 0
      game_state.star = state.spawn_star(width, height)
      game_state.message = translate("advanced_example.reset")
    elseif event.name == "save" then
      request_save_game()
      game_state.message = translate("advanced_example.saved")
    elseif event.name == "quit" then
      request_exit()
    end
  elseif event.type == "resize" then
    local width, height = get_terminal_size()
    game_state.player.x = clamp(game_state.player.x, 1, math.max(1, width - 2))
    game_state.player.y = clamp(game_state.player.y, 4, math.max(4, height - 3))
    game_state.star.x = clamp(game_state.star.x, 1, math.max(1, width - 2))
    game_state.star.y = clamp(game_state.star.y, 4, math.max(4, height - 3))
  end
  return game_state
end

return M
