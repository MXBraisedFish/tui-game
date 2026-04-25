local Runtime = _G.BLACKJACK_RUNTIME or load_function("/runtime.lua")

local M = {}

function M.render(state)
  Runtime.render(state)
end

return M
