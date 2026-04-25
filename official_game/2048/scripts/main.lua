local State = load_function("/state.lua")
local Input = load_function("/input.lua")
local Render = load_function("/render.lua")
local Storage = load_function("/storage.lua")

function init_game(state)
  return State.init(state)
end

function handle_event(state, event)
  return Input.handle_event(state, event)
end

function render(state)
  Render.render(state)
end

function exit_game(state)
  Storage.commit_stats(state)
  return state
end

function save_best_score(state)
  return Storage.best_score_payload(state)
end

function save_game(state)
  return Storage.save_game_payload(state)
end
