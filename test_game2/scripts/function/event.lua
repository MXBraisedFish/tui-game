local state_module = load_function("state.lua")

local M = {}

local function move_player(state, dx, dy)
  local width, height = get_terminal_size()
  state.player.x = state_module.clamp((state.player.x or 0) + dx, 0, math.max(0, width - 1))
  state.player.y = state_module.clamp((state.player.y or 0) + dy, 0, math.max(0, height - 1))
  state.moves = (state.moves or 0) + 1
end

local function collect_star(state)
  if state.star ~= nil and state.player.x == state.star.x and state.player.y == state.star.y then
    state.score = (state.score or 0) + 1
    state.best_score = math.max(state.best_score or 0, state.score)
    state.message = translate("test_game2.message.collect")
    state_module.spawn_star(state)
  else
    state.message = translate("test_game2.message.miss")
  end
end

function M.handle(state, event)
  state = state or {}
  event = event or { type = "tick" }

  if event.type == "action" then
    if event.name == "move_up" then
      move_player(state, 0, -1)
    elseif event.name == "move_down" then
      move_player(state, 0, 1)
    elseif event.name == "move_left" then
      move_player(state, -1, 0)
    elseif event.name == "move_right" then
      move_player(state, 1, 0)
    elseif event.name == "collect" then
      collect_star(state)
    elseif event.name == "reset" then
      state = state_module.reset(state)
    elseif event.name == "save" then
      state.saved_count = (state.saved_count or 0) + 1
      state.message = translate("test_game2.message.saved")
      request_save_game()
      request_save_best_score()
    elseif event.name == "debug" then
      debug_log("test_game2 debug score=" .. tostring(state.score or 0))
      state.message = translate("test_game2.message.debug")
    elseif event.name == "quit" then
      request_exit()
    end
    state.last_event = event.name
  elseif event.type == "resize" then
    state = state_module.resize(state, event.width or 0, event.height or 0)
    state.last_event = "resize"
  elseif event.type == "key" then
    state.last_event = "key:" .. tostring(event.name)
  elseif event.type == "tick" then
    state.running_ms = (state.running_ms or 0) + (event.dt_ms or 0)
    if state.blink_timer ~= nil and is_timer_exists(state.blink_timer) and is_timer_completed(state.blink_timer) then
      state.star_visible = not state.star_visible
      timer_restart(state.blink_timer)
    end
  end

  return state
end

return M