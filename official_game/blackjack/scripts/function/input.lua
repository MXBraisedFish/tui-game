local Runtime = _G.BLACKJACK_RUNTIME or load_function("/runtime.lua")

local M = {}

function M.handle_event(state, event)
  return Runtime.handle_event(state, event)
end

return M
