local COLORS = {
  { label = "1", fg = "black", bg = "rgb(255,0,0)" },
  { label = "2", fg = "black", bg = "rgb(255,255,0)" },
  { label = "3", fg = "white", bg = "rgb(0,120,255)" },
  { label = "4", fg = "white", bg = "rgb(0,200,0)" },
}

local SHOW_ON_MS = 900
local SHOW_OFF_MS = 450

local function tr(key)
  return translate(key)
end

local function new_state()
  return {
    score = 0,
    round = 1,
    sequence = { math.random(1, 4) },
    input = {},
    phase = "show",
    show_index = 1,
    show_timer = SHOW_ON_MS,
    show_visible = true,
    elapsed_ms = 0,
    best_score = math.max(0, math.floor(tonumber(load_data("best_score")) or 0)),
    best_time_sec = math.max(0, math.floor(tonumber(load_data("best_time_sec")) or 0)),
    message = "game.color_memory.status_observe",
  }
end

function init_game()
  math.randomseed(os.time())
  return new_state()
end

local function center_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function save_best(state)
  save_data("best_score", state.best_score)
  save_data("best_time_sec", state.best_time_sec)
  request_refresh_best_score()
end

local function commit_best(state)
  local duration = math.floor(state.elapsed_ms / 1000)
  local changed = false
  if state.score > state.best_score then
    state.best_score = state.score
    changed = true
  end
  if duration > state.best_time_sec then
    state.best_time_sec = duration
    changed = true
  end
  if changed then
    save_best(state)
  end
end

local function begin_next_round(state)
  state.round = state.round + 1
  state.score = state.round - 1
  state.sequence[#state.sequence + 1] = math.random(1, 4)
  state.input = {}
  state.phase = "show"
  state.show_index = 1
  state.show_timer = SHOW_ON_MS
  state.show_visible = true
  state.message = "game.color_memory.status_observe"
end

local function lose_round(state)
  commit_best(state)
  state.phase = "lost"
  state.message = "game.color_memory.lose_banner"
end

local function push_input(state, choice)
  if state.phase ~= "input" then
    return state
  end
  state.input[#state.input + 1] = choice
  local idx = #state.input
  if state.sequence[idx] ~= choice then
    lose_round(state)
    return state
  end
  if idx == #state.sequence then
    begin_next_round(state)
  end
  return state
end

function handle_event(state, event)
  if event.type == "tick" then
    if state.phase ~= "lost" then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    if state.phase == "show" then
      state.show_timer = state.show_timer - (event.dt_ms or 16)
      if state.show_timer <= 0 then
        if state.show_visible then
          state.show_visible = false
          state.show_timer = SHOW_OFF_MS
        else
          state.show_index = state.show_index + 1
          if state.show_index > #state.sequence then
            state.phase = "input"
            state.message = "game.color_memory.status_input"
          else
            state.show_visible = true
            state.show_timer = SHOW_ON_MS
          end
        end
      end
    end
    return state
  end

  if event.type == "resize" then
    state.message = "game.color_memory.runtime_resized"
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
    return state
  elseif event.name == "restart" then
    return new_state()
  elseif event.name == "pick_1" then
    return push_input(state, 1)
  elseif event.name == "pick_2" then
    return push_input(state, 2)
  elseif event.name == "pick_3" then
    return push_input(state, 3)
  elseif event.name == "pick_4" then
    return push_input(state, 4)
  elseif event.name == "confirm" and state.phase == "lost" then
    return new_state()
  end

  return state
end

function render(state)
  canvas_clear()

  local width, height = get_terminal_size()
  local best_line = tr("game.color_memory.best_score") .. ": " .. tostring(state.best_score)
  local time_line = tr("game.color_memory.best_time") .. ": " .. tostring(state.best_time_sec)
  local current_line = tr("game.color_memory.score") .. ": " .. tostring(state.score)
  local round_line = tr("game.color_memory.round") .. ": " .. tostring(state.round)
  local controls = tr("game.color_memory.controls")

  canvas_draw_text(center_x(best_line), 1, best_line, "white", nil)
  canvas_draw_text(center_x(time_line), 2, time_line, "white", nil)
  canvas_draw_text(center_x(current_line), 3, current_line, "yellow", nil)
  canvas_draw_text(center_x(round_line), 4, round_line, "cyan", nil)

  local box_width = 7
  local total_width = 4 * box_width + 3 * 2
  local origin_x = resolve_x(ANCHOR_CENTER, total_width, 0)
  local origin_y = math.max(7, math.floor((height - 6) / 2))

  for i = 1, 4 do
    local cell_x = origin_x + (i - 1) * (box_width + 2)
    local active = false
    if state.phase == "show" and state.show_visible and state.sequence[state.show_index] == i then
      active = true
    end
    local color = COLORS[i]
    local text = active and " ██ " or "    "
    canvas_fill_rect(cell_x, origin_y, box_width, 3, " ", nil, active and color.bg or "black")
    canvas_draw_text(cell_x + 2, origin_y + 1, color.label, active and color.fg or "dark_gray", active and color.bg or nil)
    canvas_draw_text(cell_x + 1, origin_y + 2, text, active and color.fg or "dark_gray", active and color.bg or nil)
  end

  local input_labels = {}
  for i = 1, #state.input do
    input_labels[#input_labels + 1] = tostring(state.input[i])
  end
  local input_line = #input_labels > 0 and table.concat(input_labels, " ") or "-"
  canvas_draw_text(center_x(input_line), origin_y + 5, input_line, "white", nil)

  local msg_color = state.phase == "lost" and "red" or "white"
  canvas_draw_text(center_x(tr(state.message)), math.max(origin_y + 7, height - 3), tr(state.message), msg_color, nil)
  canvas_draw_text(center_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best_score <= 0 and state.best_time_sec <= 0 then
    return nil
  end
  return {
    best_string = "game.color_memory.best_block",
    score = state.best_score,
    time = state.best_time_sec,
  }
end
