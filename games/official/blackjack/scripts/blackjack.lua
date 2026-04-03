local START_FUNDS = 1000
local BASE_BET = 100

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function draw_card()
  local n = math.random(13)
  if n == 1 then return "A" end
  if n == 11 then return "J" end
  if n == 12 then return "Q" end
  if n == 13 then return "K" end
  return tostring(n)
end

local function card_value(rank)
  if rank == "A" then return 11 end
  if rank == "J" or rank == "Q" or rank == "K" then return 10 end
  return tonumber(rank) or 0
end

local function hand_total(cards)
  local total, aces = 0, 0
  for i = 1, #cards do
    total = total + card_value(cards[i])
    if cards[i] == "A" then
      aces = aces + 1
    end
  end
  while total > 21 and aces > 0 do
    total = total - 10
    aces = aces - 1
  end
  return total
end

local function is_blackjack(cards)
  return #cards == 2 and hand_total(cards) == 21
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_net = math.max(0, math.floor(tonumber(best.net) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", { net = state.best_net })
  request_refresh_best_score()
end

local function update_best_record(state)
  local net = state.funds - START_FUNDS
  if net > state.best_net then
    state.best_net = net
    save_best_record(state)
  end
end

local function serialize_state(state)
  return {
    funds = state.funds,
    bet_mult = state.bet_mult,
    elapsed_ms = state.elapsed_ms,
    player = state.player,
    dealer = state.dealer,
    dealer_hidden = state.dealer_hidden,
    phase = state.phase,
    message = state.message,
    best_net = state.best_net,
  }
end

local function save_progress(state)
  save_data("state", serialize_state(state))
end

local function start_round(state)
  local bet = math.min(state.funds, BASE_BET * state.bet_mult)
  if bet <= 0 then
    state.phase = "finished"
    state.message = "game.blackjack.msg_bankrupt"
    return state
  end
  state.bet = bet
  state.player = { draw_card(), draw_card() }
  state.dealer = { draw_card(), draw_card() }
  state.dealer_hidden = true
  state.phase = "player"
  state.message = "game.blackjack.phase_player"
  save_progress(state)
  return state
end

local function fresh_state()
  local state = {
    funds = START_FUNDS,
    bet_mult = 1,
    bet = BASE_BET,
    player = {},
    dealer = {},
    dealer_hidden = true,
    phase = "player",
    elapsed_ms = 0,
    message = "game.blackjack.phase_player",
    best_net = 0,
  }
  load_best_record(state)
  return start_round(state)
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    return fresh_state()
  end
  state.funds = math.max(0, math.floor(tonumber(state.funds) or START_FUNDS))
  state.bet_mult = math.max(1, math.floor(tonumber(state.bet_mult) or 1))
  state.bet = math.max(0, math.floor(tonumber(state.bet) or BASE_BET))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.player = type(state.player) == "table" and state.player or {}
  state.dealer = type(state.dealer) == "table" and state.dealer or {}
  state.dealer_hidden = state.dealer_hidden ~= false
  state.phase = state.phase or "player"
  state.message = state.message or "game.blackjack.phase_player"
  state.best_net = math.max(0, math.floor(tonumber(state.best_net) or 0))
  load_best_record(state)
  if #state.player == 0 or #state.dealer == 0 then
    return fresh_state()
  end
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function settle_round(state)
  state.dealer_hidden = false
  local player_total = hand_total(state.player)
  local dealer_total = hand_total(state.dealer)
  local reward = 0

  while dealer_total < 17 do
    state.dealer[#state.dealer + 1] = draw_card()
    dealer_total = hand_total(state.dealer)
  end

  if player_total > 21 then
    reward = -state.bet
    state.message = "game.blackjack.msg_player_bust"
  elseif is_blackjack(state.player) and not is_blackjack(state.dealer) then
    reward = math.floor(state.bet * 1.5)
    state.message = "game.blackjack.msg_player_blackjack"
  elseif dealer_total > 21 then
    reward = state.bet
    state.message = "game.blackjack.msg_dealer_bust_win"
  elseif dealer_total > player_total then
    reward = -state.bet
    state.message = "game.blackjack.msg_dealer_higher"
  elseif dealer_total < player_total then
    reward = state.bet
    state.message = "game.blackjack.msg_player_higher"
  else
    state.message = "game.blackjack.msg_push"
  end

  state.funds = math.max(0, state.funds + reward)
  update_best_record(state)
  state.phase = "settle"
  save_progress(state)
  return state
end

local function card_line(cards, hide_second)
  local parts = {}
  for i = 1, #cards do
    local rank = cards[i]
    if hide_second and i == 2 then
      rank = "?"
    end
    parts[#parts + 1] = "[" .. rank .. "]"
  end
  return table.concat(parts, " ")
end

function handle_event(state, event)
  if event.type == "tick" then
    state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    return state
  end
  if event.type == "resize" then
    state.message = "game.blackjack.runtime_resized"
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
    return fresh_state()
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.blackjack.runtime_saved"
    return state
  elseif event.name == "bet_down" and state.phase == "player" and #state.player == 2 then
    state.bet_mult = math.max(1, state.bet_mult - 1)
    state.bet = math.min(state.funds, BASE_BET * state.bet_mult)
    return state
  elseif event.name == "bet_up" and state.phase == "player" and #state.player == 2 then
    state.bet_mult = math.min(5, state.bet_mult + 1)
    state.bet = math.min(state.funds, BASE_BET * state.bet_mult)
    return state
  elseif event.name == "hit" and state.phase == "player" then
    state.player[#state.player + 1] = draw_card()
    if hand_total(state.player) > 21 then
      return settle_round(state)
    end
    save_progress(state)
    return state
  elseif event.name == "stand" then
    if state.phase == "player" then
      return settle_round(state)
    elseif state.phase == "settle" then
      return start_round(state)
    elseif state.phase == "finished" then
      return fresh_state()
    end
  end

  return state
end

function render(state)
  canvas_clear()
  local _, height = get_terminal_size()
  local funds_line = tr("game.blackjack.bet_funds") .. ": " .. tostring(state.funds)
  local bet_line = tr("game.blackjack.bet_round") .. ": " .. tostring(state.bet)
  local net_line = tr("game.blackjack.net") .. ": " .. tostring(state.funds - START_FUNDS)
  local warning = tr("game.blackjack.warning")
  local dealer_title = tr("game.blackjack.dealer_cards") .. ": " .. tostring(hand_total(state.dealer_hidden and { state.dealer[1] or "0" } or state.dealer))
  local player_title = tr("game.blackjack.player_cards") .. ": " .. tostring(hand_total(state.player))
  local controls = tr("game.blackjack.controls")
  local message = tr(state.message)

  canvas_draw_text(centered_x(tr("game.blackjack.name")), 1, tr("game.blackjack.name"), "cyan", nil)
  canvas_draw_text(centered_x(funds_line), 3, funds_line, "white", nil)
  canvas_draw_text(centered_x(bet_line), 4, bet_line, "yellow", nil)
  canvas_draw_text(centered_x(net_line), 5, net_line, "green", nil)

  canvas_draw_text(centered_x(dealer_title), 8, dealer_title, "light_red", nil)
  canvas_draw_text(centered_x(card_line(state.dealer, state.dealer_hidden)), 9, card_line(state.dealer, state.dealer_hidden), "white", nil)
  canvas_draw_text(centered_x(player_title), 12, player_title, "light_cyan", nil)
  canvas_draw_text(centered_x(card_line(state.player, false)), 13, card_line(state.player, false), "white", nil)

  if state.phase == "settle" then
    canvas_draw_text(centered_x(tr("game.blackjack.msg_press_enter_next")), 16, tr("game.blackjack.msg_press_enter_next"), "dark_gray", nil)
  end
  canvas_draw_text(centered_x(message), math.max(18, height - 4), message, state.phase == "settle" and "green" or "white", nil)
  canvas_draw_text(centered_x(warning), math.max(19, height - 3), warning, "dark_gray", nil)
  canvas_draw_text(centered_x(controls), height - 1, controls, "dark_gray", nil)
end

function best_score(state)
  if state.best_net <= 0 then
    return nil
  end
  return {
    best_string = "game.blackjack.best_block",
    net = state.best_net,
  }
end
