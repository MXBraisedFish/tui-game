local Runtime = _G.MEMORY_FLIP_RUNTIME or load_function("/runtime.lua")

local M = {}

function M.save_best_score(state)
    return Runtime.save_best_score(state)
end

function M.save_game(state)
    return Runtime.save_game(state)
end

return M
