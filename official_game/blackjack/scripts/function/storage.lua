local Runtime = _G.BLACKJACK_RUNTIME or load_function("/runtime.lua")

local M = {}

function M.save_best_score(state)
  return Runtime.save_best_score(state)
end

return M
