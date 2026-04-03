local WORDS = { "apple", "brace", "clock", "dream", "earth", "flame", "grape", "house", "light", "sound", "table", "water" }
local MAX_ATTEMPTS = 6

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function pick_word()
  return WORDS[math.random(1, #WORDS)]
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_streak = math.max(0, math.floor(tonumber(best.streak) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(best.time_sec) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", {
    streak = state.best_streak,
    time_sec = state.best_time_sec,
  })
  request_refresh_best_score()
end

local function fresh_state()
  local state = {
    secret = pick_word(),
    guesses = {},
    marks = {},
    input = "",
    mode = "input",
    settled = false,
    won = false,
    streak = 0,
    best_streak = 0,
    best_time_sec = 0,
    elapsed_ms = 0,
    message = "game.wordle.mode_input",
  }
  load_best_record(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" or type(state.secret) ~= "string" then
    return fresh_state()
  end
  state.guesses = type(state.guesses) == "table" and state.guesses or {}
  state.marks = type(state.marks) == "table" and state.marks or {}
  state.input = type(state.input) == "string" and state.input or ""
  state.mode = state.mode == "action" and "action" or "input"
  state.settled = state.settled == true
  state.won = state.won == true
  state.streak = math.max(0, math.floor(tonumber(state.streak) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.message = state.message or "game.wordle.mode_input"
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    secret = state.secret,
    guesses = state.guesses,
    marks = state.marks,
    input = state.input,
    mode = state.mode,
    settled = state.settled,
    won = state.won,
    streak = state.streak,
    elapsed_ms = state.elapsed_ms,
    message = state.message,
  })
end

local function evaluate_guess(secret, guess)
  local marks = {}
  local counts = {}
  for i = 1, #secret do
    local ch = secret:sub(i, i)
    counts[ch] = (counts[ch] or 0) + 1
  end
  for i = 1, #guess do
    local ch = guess:sub(i, i)
    if ch == secret:sub(i, i) then
      marks[i] = "correct"
      counts[ch] = counts[ch] - 1
    end
  end
  for i = 1, #guess do
    if not marks[i] then
      local ch = guess:sub(i, i)
      if (counts[ch] or 0) > 0 then
        marks[i] = "present"
        counts[ch] = counts[ch] - 1
      else
        marks[i] = "absent"
      end
    end
  end
  return marks
end

local function finish_round(state, won)
  state.settled = true
  state.won = won
  local elapsed = math.floor(state.elapsed_ms / 1000)
  if won then
    state.message = "game.wordle.win"
    state.streak = state.streak + 1
    if state.streak > state.best_streak then
      state.best_streak = state.streak
    end
    if state.best_time_sec <= 0 or elapsed < state.best_time_sec then
      state.best_time_sec = elapsed
    end
    save_best_record(state)
  else
    state.message = "game.wordle.lose"
    state.streak = 0
  end
  save_progress(state)
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.settled then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.wordle.runtime_resized"
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
    state.message = "game.wordle.runtime_saved"
  elseif event.name == "toggle_mode" then
    state.mode = state.mode == "input" and "action" or "input"
    state.message = state.mode == "input" and "game.wordle.mode_input" or "game.wordle.mode_action"
  elseif state.mode == "input" and not state.settled then
    if event.name == "backspace" then
      state.input = state.input:sub(1, #state.input - 1)
    elseif event.name == "submit" then
      if #state.input < #state.secret then
        state.message = "game.wordle.need_letters"
      else
        local guess = state.input:sub(1, #state.secret)
        state.guesses[#state.guesses + 1] = guess
        state.marks[#state.marks + 1] = evaluate_guess(state.secret, guess)
        state.input = ""
        if guess == state.secret then
          finish_round(state, true)
        elseif #state.guesses >= MAX_ATTEMPTS then
          finish_round(state, false)
        end
      end
    else
      local prefix = "letter_"
      if event.name:sub(1, #prefix) == prefix and #state.input < #state.secret then
        state.input = state.input .. event.name:sub(#prefix + 1, #prefix + 1)
      end
    end
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
  local title = tr("game.wordle.name")
  canvas_draw_text(centered_x(title), 2, title, "cyan", nil)
  canvas_draw_text(4, 4, tr("game.wordle.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000)), "white", nil)
  canvas_draw_text(4, 5, tr("game.wordle.streak") .. ": " .. tostring(state.streak), "white", nil)
  local start_y = 8
  for i = 1, MAX_ATTEMPTS do
    local guess = state.guesses[i] or ""
    local line = guess
    if line == "" and i == #state.guesses + 1 and not state.settled then
      line = state.input .. string.rep("_", #state.secret - #state.input)
    elseif line == "" then
      line = string.rep("_", #state.secret)
    end
    local fg = "white"
    if state.marks[i] then
      local has_correct, has_present = false, false
      for j = 1, #state.marks[i] do
        if state.marks[i][j] == "correct" then has_correct = true end
        if state.marks[i][j] == "present" then has_present = true end
      end
      if has_correct and not has_present then
        fg = "green"
      elseif has_present then
        fg = "yellow"
      else
        fg = "dark_gray"
      end
    end
    canvas_draw_text(centered_x(line), start_y + i - 1, line, fg, nil)
  end
  if state.settled then
    local answer = tr("game.wordle.name") .. ": " .. state.secret
    canvas_draw_text(centered_x(answer), start_y + MAX_ATTEMPTS + 1, answer, state.won and "green" or "red", nil)
  end
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.min(term_h - 3, start_y + MAX_ATTEMPTS + 3), message, state.settled and (state.won and "green" or "red") or "white", nil)
  local controls_key = state.mode == "input" and "game.wordle.controls_input" or "game.wordle.controls_action"
  canvas_draw_text(centered_x(tr(controls_key)), term_h - 1, tr(controls_key), "dark_gray", nil)
end

function best_score(state)
  local best_time = state.best_time_sec > 0 and format_duration(state.best_time_sec) or "--:--:--"
  return {
    best_string = "game.wordle.best_block",
    streak = state.best_streak,
    time = best_time,
  }
end
