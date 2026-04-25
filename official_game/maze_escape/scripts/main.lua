local Runtime = load_function("/runtime.lua")
local Input = load_function("/input.lua")
local Render = load_function("/render.lua")
local Storage = load_function("/storage.lua")

function init_game(state)
    return Runtime.init_game(state)
end

function handle_event(state, event)
    return Input.handle_event(state, event)
end

function render(state)
    Render.render(state)
end

function exit_game(state)
    return Runtime.exit_game(state)
end

function save_best_score(state)
    return Storage.save_best_score(state)
end

function save_game(state)
    return Storage.save_game(state)
end
