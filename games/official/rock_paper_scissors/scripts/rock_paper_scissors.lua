local CHOICE_SCISSORS = 1
local CHOICE_ROCK = 2
local CHOICE_PAPER = 3

local CHOICES = {
  [CHOICE_SCISSORS] = { symbol = "Y", key = "game.rock_paper_scissors.choice.scissors" },
  [CHOICE_ROCK] = { symbol = "O", key = "game.rock_paper_scissors.choice.rock" },
  [CHOICE_PAPER] = { symbol = "U", key = "game.rock_paper_scissors.choice.paper" },
}

local function tr(key)
  return translate(key)
end

local function new_state()
  return {
    player_pick = nil,
    ai_pick = nil,
    current_streak = 0,
    best_streak = 0,
    loss_streak = 0,
    message = "game.rock_paper_scissors.ready_banner",
    message_color = "dark_gray",
  }
end

local function load_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    state = new_state()
  end

  local best = load_data("best_record")
  if type(best) == "table" then
    local streak = math.floor(tonumber(best.best_streak) or 0)
    if streak > 0 then
      state.best_streak = streak
    end
  end
  return state
end

function init_game()
  math.randomseed(os.time())
  return load_state()
end

local function choice_text(choice)
  if choice == nil or CHOICES[choice] == nil then
    return "-"
  end
  local item = CHOICES[choice]
  return item.symbol .. " " .. tr(item.key)
end

local function resolve_round(player_choice, ai_choice)
  if player_choice == ai_choice then
    return 0
  end

  if (player_choice == CHOICE_SCISSORS and ai_choice == CHOICE_PAPER)
      or (player_choice == CHOICE_ROCK and ai_choice == CHOICE_SCISSORS)
      or (player_choice == CHOICE_PAPER and ai_choice == CHOICE_ROCK) then
    return 1
  end

  return -1
end

local function player_win_bias(loss_streak)
  if loss_streak <= 0 then
    return 0
  end
  if loss_streak >= 7 then
    return 1
  end
  return loss_streak / 8
end

local function pick_ai_choice(state, player_choice)
  local bias = player_win_bias(state.loss_streak)
  if bias > 0 then
    local roll = math.random()
    if roll <= bias then
      if player_choice == CHOICE_SCISSORS then
        return CHOICE_PAPER
      elseif player_choice == CHOICE_ROCK then
        return CHOICE_SCISSORS
      else
        return CHOICE_ROCK
      end
    end
  end
  return math.random(1, 3)
end

local function save_progress(state)
  save_data("state", {
    player_pick = state.player_pick,
    ai_pick = state.ai_pick,
    current_streak = state.current_streak,
    best_streak = state.best_streak,
    loss_streak = state.loss_streak,
    message = state.message,
    message_color = state.message_color,
  })
end

local function save_best_record(state)
  save_data("best_record", {
    best_streak = state.best_streak,
  })
end

local function center_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function split_positions(left_text, right_text)
  local width = select(1, get_terminal_size())
  local center = math.floor(width / 2)
  local left_width = select(1, measure_text(left_text))
  local left_x = center - 2 - left_width
  local right_x = center + 3
  if left_x < 0 then
    left_x = 0
  end
  return left_x, center, right_x
end

local function apply_round_result(state, player_choice)
  local ai_choice = pick_ai_choice(state, player_choice)
  state.player_pick = player_choice
  state.ai_pick = ai_choice

  local result = resolve_round(player_choice, ai_choice)
  if result > 0 then
    state.current_streak = state.current_streak + 1
    state.loss_streak = 0
    if state.current_streak > state.best_streak then
      state.best_streak = state.current_streak
      save_best_record(state)
      request_refresh_best_score()
      state.message = "game.rock_paper_scissors.win_banner"
      state.message_color = "green"
    else
      state.message = "game.rock_paper_scissors.win_banner"
      state.message_color = "green"
    end
  elseif result < 0 then
    state.current_streak = 0
    state.loss_streak = state.loss_streak + 1
    state.message = "game.rock_paper_scissors.lose_banner"
    state.message_color = "red"
  else
    state.current_streak = 0
    state.message = "game.rock_paper_scissors.draw_banner"
    state.message_color = "yellow"
  end
  save_progress(state)
  return state
end

local function reset_round(state)
  state.player_pick = nil
  state.ai_pick = nil
  state.current_streak = 0
  state.loss_streak = 0
  state.message = "game.rock_paper_scissors.ready_banner"
  state.message_color = "dark_gray"
  save_progress(state)
  return state
end

function handle_event(state, event)
  if event.type == "resize" then
    state.message = "game.rock_paper_scissors.msg_resized"
    state.message_color = "dark_gray"
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
    return reset_round(state)
  elseif event.name == "pick_scissors" then
    return apply_round_result(state, CHOICE_SCISSORS)
  elseif event.name == "pick_rock" then
    return apply_round_result(state, CHOICE_ROCK)
  elseif event.name == "pick_paper" then
    return apply_round_result(state, CHOICE_PAPER)
  end

  return state
end

function render(state)
  canvas_clear()

  local width, height = get_terminal_size()
  local best_line = tr("game.rock_paper_scissors.best_streak") .. ": " .. tostring(state.best_streak)
  local current_line = tr("game.rock_paper_scissors.current_streak") .. ": " .. tostring(state.current_streak)
  local controls = tr("game.rock_paper_scissors.controls")
  local left_header = tr("game.rock_paper_scissors.player")
  local right_header = tr("game.rock_paper_scissors.system")
  local left_choice = choice_text(state.player_pick)
  local right_choice = choice_text(state.ai_pick)
  local base_y = math.max(3, math.floor((height - 8) / 2))

  canvas_draw_text(center_x(best_line), 1, best_line, "green", nil)
  canvas_draw_text(center_x(current_line), 2, current_line, "yellow", nil)

  local left_x, center_x_pos, right_x = split_positions(left_header, right_header)
  canvas_draw_text(left_x, base_y, left_header, "white", nil)
  canvas_draw_text(center_x_pos, base_y, "|", "white", nil)
  canvas_draw_text(right_x, base_y, right_header, "white", nil)

  left_x, center_x_pos, right_x = split_positions(left_choice, right_choice)
  canvas_draw_text(left_x, base_y + 1, left_choice, "white", nil)
  canvas_draw_text(center_x_pos, base_y + 1, "|", "white", nil)
  canvas_draw_text(right_x, base_y + 1, right_choice, "white", nil)

  canvas_draw_text(center_x(tr(state.message)), base_y + 3, tr(state.message), state.message_color, nil)
  canvas_draw_text(center_x(controls), math.max(base_y + 5, height - 2), controls, "dark_gray", nil)
end

function best_score(state)
  if type(state.best_streak) ~= "number" or state.best_streak <= 0 then
    return nil
  end

  return {
    best_string = "game.rock_paper_scissors.best_block",
    streak = state.best_streak,
  }
end
