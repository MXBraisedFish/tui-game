local Runtime = _G.MAZE_ESCAPE_RUNTIME or load_function("/runtime.lua")

local M = {}

function M.render(state)
    Runtime.render(state)
end

return M
