local state = load_function("state.lua")
local event = load_function("event.lua")
local render = load_function("render.lua")
local persist = load_function("persist.lua")

function init_game(saved_state)
  return state.init(saved_state)
end

function handle_event(game_state, incoming_event)
  return event.handle(game_state, incoming_event)
end

function render(game_state)
  render.draw(game_state)
end

function exit_game(game_state)
  return game_state
end

function save_best_score(game_state)
  return persist.best_score(game_state)
end

function save_game(game_state)
  return persist.save_game(game_state)
end
