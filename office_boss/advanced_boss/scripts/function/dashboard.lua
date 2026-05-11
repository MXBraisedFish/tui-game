local M = {}

local LOG_LINES = {
  "checking workspace metadata",
  "running incremental analysis",
  "refreshing dependency graph",
  "writing status cache",
  "waiting for file changes"
}

function M.update(state)
  state.started_ms = state.started_ms or running_time()
  state.timer = state.timer or timer_create(500, "boss dashboard update")
  state.cursor = state.cursor or 1
  state.progress = state.progress or {
    build = 82,
    tests = 96,
    deploy = 41
  }

  if get_timer_status(state.timer) == "init" then
    timer_start(state.timer)
  end

  if is_timer_completed(state.timer) then
    timer_restart(state.timer)
    state.cursor = state.cursor + 1
    if state.cursor > #LOG_LINES then
      state.cursor = 1
    end
    state.progress.build = 70 + random(0, 25)
    state.progress.tests = 80 + random(0, 19)
    state.progress.deploy = 25 + random(0, 60)
  end

  state.logs = LOG_LINES
  return state
end

return M
