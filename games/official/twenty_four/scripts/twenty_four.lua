local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function rand_num()
  return math.random(1, 13)
end

local function fresh_numbers()
  return { rand_num(), rand_num(), rand_num(), rand_num() }
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_time_sec = math.max(0, math.floor(tonumber(best.time_sec) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", { time_sec = state.best_time_sec })
  request_refresh_best_score()
end

local function fresh_state()
  local state = {
    nums = fresh_numbers(),
    ops = { "_", "_", "_" },
    cursor = 1,
    elapsed_ms = 0,
    steps = 0,
    value = nil,
    won = false,
    message = "game.twenty_four.ready",
    best_time_sec = 0,
  }
  load_best_record(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" or type(state.nums) ~= "table" or type(state.ops) ~= "table" then
    return fresh_state()
  end
  state.cursor = math.max(1, math.min(3, math.floor(tonumber(state.cursor) or 1)))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.steps = math.max(0, math.floor(tonumber(state.steps) or 0))
  state.won = state.won == true
  state.message = state.message or "game.twenty_four.ready"
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    nums = state.nums,
    ops = state.ops,
    cursor = state.cursor,
    elapsed_ms = state.elapsed_ms,
    steps = state.steps,
    value = state.value,
    won = state.won,
    message = state.message,
  })
end

local function compute_value(state)
  for i = 1, 3 do
    if state.ops[i] == "_" then
      state.value = nil
      return
    end
  end
  local value = state.nums[1]
  for i = 1, 3 do
    local rhs = state.nums[i + 1]
    local op = state.ops[i]
    if op == "+" then
      value = value + rhs
    elseif op == "-" then
      value = value - rhs
    elseif op == "*" then
      value = value * rhs
    elseif op == "/" then
      value = rhs == 0 and math.huge or value / rhs
    end
  end
  state.value = value
  if math.abs(value - 24) < 1e-6 and not state.won then
    state.won = true
    state.message = "game.twenty_four.win_banner"
    local elapsed = math.floor(state.elapsed_ms / 1000)
    if state.best_time_sec <= 0 or elapsed < state.best_time_sec then
      state.best_time_sec = elapsed
      save_best_record(state)
    end
  end
end

local function set_op(state, op)
  if state.won then
    return state
  end
  state.ops[state.cursor] = op
  state.steps = state.steps + 1
  compute_value(state)
  save_progress(state)
  return state
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.won then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.twenty_four.runtime_resized"
    return state
  end
  if event.type == "quit" then
    request_exit()
    return state
  end
  if event.type ~= "action" then
    return state
  end

  if event.name == "quit_action" then
    request_exit()
  elseif event.name == "restart" then
    return fresh_state()
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.twenty_four.runtime_saved"
  elseif event.name == "move_left" then
    state.cursor = math.max(1, state.cursor - 1)
  elseif event.name == "move_right" then
    state.cursor = math.min(3, state.cursor + 1)
  elseif event.name == "op_add" then
    return set_op(state, "+")
  elseif event.name == "op_sub" then
    return set_op(state, "-")
  elseif event.name == "op_mul" then
    return set_op(state, "*")
  elseif event.name == "op_div" then
    return set_op(state, "/")
  end
  return state
end

local function format_duration(total_seconds)
  local h = math.floor(total_seconds / 3600)
  local m = math.floor((total_seconds % 3600) / 60)
  local s = total_seconds % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function render(state)
  canvas_clear()
  local _, term_h = get_terminal_size()
  local title = tr("game.twenty_four.name")
  canvas_draw_text(centered_x(title), 2, title, "cyan", nil)
  canvas_draw_text(4, 4, tr("game.twenty_four.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000)), "white", nil)
  canvas_draw_text(4, 5, tr("game.twenty_four.steps") .. ": " .. tostring(state.steps), "white", nil)
  local parts = {}
  for i = 1, 4 do
    parts[#parts + 1] = tostring(state.nums[i])
    if i <= 3 then
      local op = state.ops[i]
      if i == state.cursor then
        op = "[" .. op .. "]"
      end
      parts[#parts + 1] = op
    end
  end
  local expr = table.concat(parts, " ")
  canvas_draw_text(centered_x(expr), 9, expr, "yellow", nil)
  local result = "= " .. (state.value == nil and "?" or tostring(state.value))
  canvas_draw_text(centered_x(result), 11, result, state.won and "green" or "white", nil)
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.min(term_h - 3, 14), message, state.won and "green" or "white", nil)
  canvas_draw_text(centered_x(tr("game.twenty_four.controls")), term_h - 1, tr("game.twenty_four.controls"), "dark_gray", nil)
end

function best_score(state)
  local time = state.best_time_sec > 0 and format_duration(state.best_time_sec) or tr("game.twenty_four.none")
  return {
    best_string = "game.twenty_four.best_block",
    time = time,
  }
end
