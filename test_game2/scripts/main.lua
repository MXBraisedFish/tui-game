local state_module = load_function("state.lua")
local event_module = load_function("event.lua")
local render_module = load_function("render.lua")
local persistence = load_function("persistence.lua")

function init_game(state)
  return state_module.init(state)
end

function handle_event(state, event)
  return event_module.handle(state, event)
end

function render(state)
  render_module.render(state)
end

function exit_game(state)
  state.message = translate("test_game2.message.exit")
  state.exited_at = now()
  return state
end

function save_best_score(state)
  return persistence.best_score(state)
end

function save_game(state)
  return persistence.save_state(state)
end