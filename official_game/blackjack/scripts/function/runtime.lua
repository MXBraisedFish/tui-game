local Constants = load_function("/constants.lua")

local STARTING_FUNDS = Constants.STARTING_FUNDS
local BASE_BET = Constants.BASE_BET
local FPS = Constants.FPS
local FRAME_MS = Constants.FRAME_MS
local DEALER_REVEAL_PAUSE_MS = Constants.DEALER_REVEAL_PAUSE_MS
local DEALER_DRAW_PAUSE_MS = Constants.DEALER_DRAW_PAUSE_MS
local SETTLE_COMPARE_PAUSE_MS = Constants.SETTLE_COMPARE_PAUSE_MS

local TABLE_W = Constants.TABLE_W
local TABLE_H = Constants.TABLE_H
local CARD_W = Constants.CARD_W
local CARD_H = Constants.CARD_H
local SPINNER = Constants.SPINNER

local state = {
    funds = STARTING_FUNDS,
    initial_funds = STARTING_FUNDS,
    best_net = 0,

    dealer_cards = {},
    dealer_hidden = true,
    hands = {},
    split_mode = false,
    active_hand = 1,
    insurance = false,
    force_double_next_round = false,
    first_action_done = false,

    phase = "player",
    center_lines = {},
    confirm_mode = nil,
    await_next_round = false,
    bet_multiplier = 1.0,

    toast_text = nil,
    toast_color = "yellow",
    toast_until = 0,

    bankrupt = false,
    spinner_idx = 1,
    frame = 0,
    dirty = true,

    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    last_term_w = 0,
    last_term_h = 0
}
local function tr(key)
    if type(translate) ~= "function" then
        return key
    end

    local ok, value = pcall(translate, key)
    if not ok or value == nil or value == "" then
        return key
    end

    if type(value) == "string" and string.find(value, "[missing-i18n-key:", 1, true) ~= nil then
        return key
    end

    return value
end


local KEY_DISPLAY = {
    up = "↑",
    down = "↓",
    left = "←",
    right = "→",
    enter = "Enter",
    esc = "Esc",
    space = "Space",
    backspace = "Bksp",
    del = "Del",
    tab = "Tab",
    back_tab = "BTab"
}

local function display_key_name(key)
    key = tostring(key or "")
    if key == "" then return "" end
    if KEY_DISPLAY[key] ~= nil then return KEY_DISPLAY[key] end
    if #key == 1 then return string.upper(key) end
    if string.sub(key, 1, 1) == "f" and tonumber(string.sub(key, 2)) ~= nil then
        return string.upper(key)
    end
    return key
end

local function key_label(action)
    if type(get_key) ~= "function" then
        return "[]"
    end
    local ok, info = pcall(get_key, action)
    if not ok or type(info) ~= "table" then
        return "[]"
    end
    if info[action] ~= nil and type(info[action]) == "table" then
        info = info[action]
    end
    local keys = info.key_user or info.key
    if type(keys) ~= "table" then
        keys = { keys }
    end
    local out = {}
    for i = 1, #keys do
        local label = display_key_name(keys[i])
        if label ~= "" then
            out[#out + 1] = "[" .. label .. "]"
        end
    end
    if #out == 0 then return "[]" end
    return table.concat(out, "/")
end

local function replace_prompt_keys(text)
    text = tostring(text or "")
    text = string.gsub(text, "%[Y%]", key_label("confirm_yes"))
    text = string.gsub(text, "%[N%]", key_label("confirm_no"))
    text = string.gsub(text, "%[Enter%]", key_label("stand"))
    text = string.gsub(text, "%[Q%]/%[ESC%]", key_label("quit_action"))
    return text
end

local function controls_text()
    return table.concat({
        key_label("adjust_up") .. "/" .. key_label("adjust_down") .. " " .. tr("game.blackjack.ops_adjust_multiplier"),
        key_label("switch_left") .. "/" .. key_label("switch_right") .. " " .. tr("game.blackjack.action.switch_left"),
        key_label("hit") .. " " .. tr("game.blackjack.action.hit"),
        key_label("stand") .. " " .. tr("game.blackjack.action.stand"),
        key_label("double_down") .. " " .. tr("game.blackjack.action.double_down"),
        key_label("split_hand") .. " " .. tr("game.blackjack.action.split_hand"),
        key_label("insurance") .. " " .. tr("game.blackjack.action.insurance"),
        key_label("restart") .. " " .. tr("game.blackjack.action.restart"),
        key_label("quit_action") .. " " .. tr("game.blackjack.action.quit")
    }, "  ")
end

local function restart_quit_controls_text()
    return key_label("restart") .. " " .. tr("game.blackjack.action.restart")
        .. "  " .. key_label("quit_action") .. " " .. tr("game.blackjack.action.quit")
end


local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
end


local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end

    local lines = {}
    local current = ""
    local had_token = false
    for token in string.gmatch(text, "%S+") do
        had_token = true
        if current == "" then
            current = token
        else
            local candidate = current .. " " .. token
            if text_width(candidate) <= max_width then
                current = candidate
            else
                lines[#lines + 1] = current
                current = token
            end
        end
    end

    if not had_token then
        return { "" }
    end
    if current ~= "" then
        lines[#lines + 1] = current
    end
    return lines
end


local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then
            return width
        end
        width = width + 1
    end
    return full
end


local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then
            w, h = tw, th
        end
    end
    return w, h
end

local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function random_index(n)
    if type(n) ~= "number" or n <= 0 then
        return 0
    end
    return _G.random(n - 1)
end


local function normalize_key(key)
    if key == nil then
        return ""
    end
    if type(key) == "string" then
        return string.lower(key)
    end
    if type(key) == "table" then
        if key.type == "quit" then
            return "quit_action"
        end
        if key.type == "key" and type(key.name) == "string" then
            return string.lower(key.name)
        end
        if key.type == "action" and type(key.name) == "string" then
            local map = {
                adjust_up = "adjust_up",
                adjust_down = "adjust_down",
                confirm_yes = "confirm_yes",
                switch_left = "switch_left",
                switch_right = "switch_right",
                hit = "hit",
                stand = "stand",
                double_down = "double_down",
                split_hand = "split_hand",
                insurance = "insurance",
                restart = "restart",
                quit_action = "quit_action",
                confirm_no = "confirm_no"
            }
            return map[key.name] or ""
        end
    end
    return tostring(key):lower()
end

local function flush_input_buffer()
end


local function fill_line(y, width)
    draw_text(1, y, string.rep(" ", width), "white", "black")
end


local function fill_rect(x, y, width, height)
    if width <= 0 or height <= 0 then
        return
    end
    local blank = string.rep(" ", width)
    for row = 0, height - 1 do
        draw_text(x, y + row, blank, "white", "black")
    end
end


local function random_rank()
    local n = random_index(13) + 1
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
    local total = 0
    local aces = 0
    for i = 1, #cards do
        local r = cards[i]
        total = total + card_value(r)
        if r == "A" then
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
    if #cards ~= 2 then
        return false
    end
    local has_a = false
    local has_ten = false
    for i = 1, 2 do
        local r = cards[i]
        if r == "A" then has_a = true end
        if r == "10" or r == "J" or r == "Q" or r == "K" then has_ten = true end
    end
    return has_a and has_ten
end


local function hand_value_text(cards)
    if is_blackjack(cards) then
        return tr("game.blackjack.value_blackjack")
    end
    return tostring(hand_total(cards))
end


local function make_hand(cards, bet, adj_mult)
    local h = {
        cards = cards,
        bet = bet,
        mult = 1.0,
        adj_mult = adj_mult or 1.0,
        adj_locked = false,
        has_hit = false,
        doubled = false,
        stood = false,
        bust = false,
        blackjack = false,
        resolved = false,
        outcome = nil,
        payout_mult = 0,
        result_text = "",
        result_color = "white",
        insured_skip = false,
    }
    local total = hand_total(h.cards)
    h.bust = total > 21
    h.blackjack = is_blackjack(h.cards)
    if h.bust then
        h.stood = true
    end
    return h
end


local function update_hand_state(h)
    local total = hand_total(h.cards)
    h.bust = total > 21
    h.blackjack = is_blackjack(h.cards)
    if h.bust or (total >= 21 and not h.blackjack) then
        h.stood = true
    end
end


local function net_value()
    return state.funds - state.initial_funds
end


local function committed_bets_total()
    if state.hands == nil then
        return 0
    end
    local total = 0
    for i = 1, #state.hands do
        local h = state.hands[i]
        if h ~= nil and type(h.bet) == "number" then
            total = total + h.bet
        end
    end
    return total
end


local function available_funds()
    return state.funds - committed_bets_total()
end


local function load_best_record()
    state.best_net = 0
    local data = get_best_score()
    if type(data) ~= "table" then
        return
    end
    if type(data.net) == "number" then
        state.best_net = math.floor(data.net)
    elseif type(data.value) == "number" then
        state.best_net = math.floor(data.value)
    end
end


local function maybe_commit_best_on_exit()
    if state.bankrupt then
        return
    end
    local net = net_value()
    if net <= 0 then
        return
    end
    if net > state.best_net then
        state.best_net = net
        request_save_best_score()
    end
end


local function set_center_lines(lines)
    state.center_lines = lines
end


local function add_toast(text, color)
    state.toast_text = text
    state.toast_color = color or "yellow"
    state.toast_until = state.frame + 2 * FPS
    state.dirty = true
end


local function card_inner(rank)
    if rank == "10" then
        return " 10"
    end
    return " " .. rank .. " "
end


local function can_hit(h)
    if h == nil or h.resolved or h.stood or h.bust or h.blackjack then return false end
    if h.doubled and h.has_hit then return false end
    return hand_total(h.cards) < 21
end


local function can_stand(h)
    if h == nil or h.resolved then return false end
    return not h.stood
end


local function can_double(h)
    if h == nil or h.resolved or h.stood or h.bust then return false end
    if h.has_hit or h.doubled then return false end
    return true
end


local function can_split()
    if state.split_mode then return false end
    local h = state.hands[1]
    if h == nil or #h.cards ~= 2 or h.has_hit or h.blackjack or h.doubled then return false end
    local c1 = h.cards[1]
    local c2 = h.cards[2]
    local same_rank = (c1 == c2)
    local same_ten_value = (card_value(c1) == 10 and card_value(c2) == 10)
    if not same_rank and not same_ten_value then return false end
    return available_funds() >= h.bet
end


local function can_insurance()
    if state.insurance then return false end
    if state.phase ~= "player" then return false end
    if state.split_mode then return false end
    if state.first_action_done then return false end

    local h = state.hands[1]
    if h == nil or h.resolved or h.stood or h.bust then return false end
    if #h.cards ~= 2 or h.has_hit or h.doubled or h.blackjack then return false end
    return hand_total(h.cards) < 17
end


local update_player_prompt


local function active_hand_ref()
    if state.split_mode then
        return state.hands[state.active_hand]
    end
    return state.hands[1]
end


local function can_adjust_multiplier(h)
    if h == nil or h.resolved or h.stood or h.bust then return false end
    return not h.adj_locked
end


local function effective_mult(h)
    local hand_mult = 1.0
    if h ~= nil and type(h.mult) == "number" then
        hand_mult = h.mult
    end
    local adjust_mult = state.bet_multiplier
    if h ~= nil and type(h.adj_mult) == "number" then
        adjust_mult = h.adj_mult
    end
    return adjust_mult * hand_mult
end


local function settlement_factor(h)
    if h == nil or not h.resolved then
        return 1.0
    end
    if h.outcome == "win" then
        return h.payout_mult or 1.0
    end
    if h.outcome == "lose" and (h.payout_mult or 0) < 0 then
        return 1.0 + math.abs(h.payout_mult)
    end
    return 1.0
end


local function multiplier_display_text(h)
    local base = effective_mult(h)
    local settle = settlement_factor(h)
    if settle > 1.001 then
        return string.format("%.1fx", base * settle)
    end
    return string.format("%.1fx", base)
end


local function adjust_bet_multiplier(delta)
    local h = active_hand_ref()
    if not can_adjust_multiplier(h) then
        add_toast(tr("game.blackjack.action_unavailable"), "dark_gray")
        return
    end

    local current_mult = h.adj_mult or state.bet_multiplier
    local next_mult = current_mult + delta
    if next_mult < 0.5 then next_mult = 0.5 end
    if next_mult > 3.0 then next_mult = 3.0 end
    next_mult = math.floor(next_mult * 2 + 0.5) / 2
    if math.abs(next_mult - current_mult) < 0.001 then
        return
    end

    local old_bet = h.bet or 0
    local target_bet = math.floor(BASE_BET * next_mult + 0.5)
    if target_bet < 1 then target_bet = 1 end
    local projected_total = committed_bets_total() - old_bet + target_bet
    if projected_total > state.funds then
        add_toast(tr("game.blackjack.action_need_funds"), "red")
        return
    end

    h.adj_mult = next_mult
    h.bet = target_bet
    if not state.split_mode then
        state.bet_multiplier = next_mult
    end
    state.first_action_done = true
    update_player_prompt()
    state.dirty = true
end


local function hand_actions_text(h)
    if h.stood or h.resolved or h.bust then
        return {
            text = tr("game.blackjack.msg_stood"),
            color = "red"
        }
    end
    local ops = {}
    if can_adjust_multiplier(h) then
        ops[#ops + 1] = tr("game.blackjack.ops_adjust_multiplier")
    end
    if can_hit(h) then ops[#ops + 1] = tr("game.blackjack.ops_hit") end
    if can_double(h) then ops[#ops + 1] = tr("game.blackjack.ops_double") end
    if can_split() then ops[#ops + 1] = tr("game.blackjack.ops_split") end
    if can_stand(h) then ops[#ops + 1] = tr("game.blackjack.ops_stand") end
    if can_insurance() then ops[#ops + 1] = tr("game.blackjack.ops_insurance") end
    return { text = table.concat(ops, "  "), color = "white" }
end


update_player_prompt = function()
    local lines = {}
    if state.split_mode then
        local left = hand_actions_text(state.hands[1])
        local right = hand_actions_text(state.hands[2])
        lines[#lines + 1] = {
            text = tr("game.blackjack.bet_round_left") .. ": " .. (left.text or ""),
            color = left.color or "white"
        }
        lines[#lines + 1] = {
            text = tr("game.blackjack.bet_round_right") .. ": " .. (right.text or ""),
            color = right.color or "white"
        }
    else
        lines[#lines + 1] = hand_actions_text(state.hands[1])
    end
    set_center_lines(lines)
end


local function begin_round()
    if state.funds <= 0 then
        state.bankrupt = true
        set_center_lines({})
        state.dirty = true
        return
    end

    local forced_double = state.force_double_next_round == true
    state.bet_multiplier = forced_double and 2.0 or 1.0

    local bet = math.floor(BASE_BET * state.bet_multiplier + 0.5)
    if bet < 1 then bet = 1 end
    if state.funds < bet then
        bet = state.funds
    end

    state.dealer_cards = { random_rank(), random_rank() }
    state.dealer_hidden = true
    state.hands = { make_hand({ random_rank(), random_rank() }, bet, state.bet_multiplier) }
    state.split_mode = false
    state.active_hand = 1
    state.insurance = false
    state.phase = "player"
    state.bankrupt = false
    state.confirm_mode = nil
    state.await_next_round = false
    state.first_action_done = false

    if forced_double then
        local h = state.hands[1]
        if h ~= nil then
            h.doubled = true
            h.adj_locked = true
        end
        state.force_double_next_round = false
        add_toast(tr("game.blackjack.msg_forced_double_applied"), "yellow")
    end

    update_player_prompt()
    state.dirty = true
    flush_input_buffer()
end


local function draw_card(x, y, rank, hidden, border_fg, text_fg)
    local bfg = border_fg or "white"
    local tfg = text_fg or border_fg or "white"

    draw_text(x, y, "\u{250C}\u{2500}\u{2500}\u{2500}\u{2510}", bfg, "black")

    if hidden then
        draw_text(x, y + 1, "\u{2502}XXX\u{2502}", bfg, "black")
    else
        draw_text(x, y + 1, "\u{2502}", bfg, "black")
        draw_text(x + 1, y + 1, card_inner(rank), tfg, "black")
        draw_text(x + CARD_W - 1, y + 1, "\u{2502}", bfg, "black")
    end

    draw_text(x, y + 2, "\u{2514}\u{2500}\u{2500}\u{2500}\u{2518}", bfg, "black")
end


local function resolve_hand(h, outcome, payout_mult, text_key, fallback, color)
    h.resolved = true
    h.outcome = outcome
    h.payout_mult = payout_mult
    h.result_text = tr(text_key)
    h.result_color = color
end


local function render_once()
    local w, h = terminal_size()
    local controls = controls_text()
    local ctrl_lines = wrap_words(controls, math.max(10, w - 2))
    if #ctrl_lines > 3 then
        ctrl_lines = { ctrl_lines[1], ctrl_lines[2], ctrl_lines[3] }
    end
    local ctrl_h = #ctrl_lines
    if ctrl_h < 1 then ctrl_h = 1 end


    local block_h = TABLE_H + 3 + ctrl_h
    local top_y = math.floor((h - block_h) / 2) + 1
    if top_y < 1 then top_y = 1 end

    local status_y = top_y
    local alert_y = top_y + 1
    local table_y = top_y + 2
    local warn_y = table_y + TABLE_H
    local controls_y = warn_y + 1

    local table_x = math.floor((w - TABLE_W) / 2)
    if table_x < 1 then table_x = 1 end
    local table_right = table_x + TABLE_W - 1




    local function centered_x(text, left_x, right_x)
        local width = text_width(text)
        local cx = left_x + math.floor(((right_x - left_x + 1) - width) / 2)
        if cx < left_x then cx = left_x end
        if cx > right_x - width + 1 then
            cx = math.max(left_x, right_x - width + 1)
        end
        return cx
    end


    local best_text = tr("game.blackjack.best") .. ": " .. tostring(state.best_net)
    local net = net_value()
    local net_color = "dark_gray"
    if net > 0 then
        net_color = "green"
    elseif net < 0 then
        net_color = "red"
    end
    local net_text = tr("game.blackjack.net") .. ": " .. tostring(net)

    fill_line(status_y, w)
    local status_sep = "    "
    local best_w = text_width(best_text)
    local sep_w = text_width(status_sep)
    local sx = centered_x(best_text .. status_sep .. net_text, 1, w)
    draw_text(sx, status_y, best_text, "dark_gray", "black")
    draw_text(sx + best_w + sep_w, status_y, net_text, net_color, "black")


    fill_line(alert_y, w)
    local alert_text = ""
    local alert_color = "red"
    if state.confirm_mode == "restart" then
        alert_text = replace_prompt_keys(tr("game.blackjack.confirm_restart"))
        alert_color = "yellow"
    elseif state.confirm_mode == "exit" then
        alert_text = replace_prompt_keys(tr("game.blackjack.confirm_exit"))
        alert_color = "yellow"
    elseif state.bankrupt then
        alert_text = tr("game.blackjack.msg_bankrupt")
            .. "  "
            .. restart_quit_controls_text()
        alert_color = "red"
    elseif state.await_next_round then
        alert_text = replace_prompt_keys(tr("game.blackjack.msg_press_enter_next"))
        alert_color = "yellow"
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        alert_text = state.toast_text
        alert_color = state.toast_color
    end
    if alert_text ~= "" then
        local ax = centered_x(alert_text, 1, w)
        draw_text(ax, alert_y, alert_text, alert_color, "black")
    end


    draw_text(table_x, table_y, "\u{2554}" .. string.rep("\u{2550}", TABLE_W - 2) .. "\u{2557}", "white", "black")
    for i = 1, TABLE_H - 2 do
        draw_text(table_x, table_y + i, "\u{2551}", "white", "black")
        draw_text(table_x + TABLE_W - 1, table_y + i, "\u{2551}", "white", "black")
    end
    draw_text(table_x, table_y + TABLE_H - 1, "\u{255A}" .. string.rep("\u{2550}", TABLE_W - 2) .. "\u{255D}", "white",
        "black")


    local dealer_label = " " .. tr("game.blackjack.dealer_cards") .. " "
    local player_label = " " .. tr("game.blackjack.player_cards") .. " "
    draw_text(centered_x(dealer_label, table_x + 2, table_right - 2), table_y, dealer_label, "white", "black")
    draw_text(centered_x(player_label, table_x + 2, table_right - 2), table_y + TABLE_H - 1, player_label, "white",
        "black")


    local inner_left = table_x + 2
    local inner_right = table_right - 2
    local inner_w = inner_right - inner_left + 1
    local table_bottom = table_y + TABLE_H - 1


    local function card_group_width(count)
        if count <= 0 then return 0 end
        return count * CARD_W + (count - 1)
    end


    local function draw_points_line(text, y, color)
        if y <= table_y or y >= table_bottom then return end
        local x = centered_x(text, inner_left, inner_right)
        draw_text(x, y, text, color, "black")
    end


    local dealer_group_w = card_group_width(#state.dealer_cards)
    local dealer_x = inner_left + math.floor((inner_w - dealer_group_w) / 2)
    local dealer_y = table_y + 2
    for i = 1, #state.dealer_cards do
        draw_card(
            dealer_x + (i - 1) * (CARD_W + 1),
            dealer_y,
            state.dealer_cards[i],
            state.dealer_hidden and i == 2,
            "rgb(255,165,0)",
            "rgb(255,165,0)"
        )
    end


    local dealer_points = "?"
    if not state.dealer_hidden then
        dealer_points = hand_value_text(state.dealer_cards)
    end
    draw_points_line(tr("game.blackjack.msg_dealer_points") .. ": " .. dealer_points, dealer_y + CARD_H, "dark_gray")


    local deck_x = table_x + TABLE_W - 16
    local deck_y = table_y + math.floor(TABLE_H / 2) - 2
    draw_text(deck_x, deck_y + 1, "\u{250C}\u{2500}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}",
        "white", "black")
    draw_text(deck_x, deck_y + 2, "\u{2502}X\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}", "white",
        "black")
    draw_text(deck_x, deck_y + 3, "\u{2514}\u{2500}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}",
        "white", "black")


    local info_x = table_x + 3
    local info_y = table_y + math.floor(TABLE_H / 2) - 1
    draw_text(info_x, info_y, "[$] " .. tostring(state.funds), "white", "black")
    if state.split_mode then
        draw_text(
            info_x,
            info_y + 1,
            tr("game.blackjack.bet_round_left") ..
            " -[$] " .. tostring(state.hands[1].bet) .. " " .. multiplier_display_text(state.hands[1]),
            "white",
            "black"
        )
        draw_text(
            info_x,
            info_y + 2,
            tr("game.blackjack.bet_round_right") ..
            " -[$] " .. tostring(state.hands[2].bet) .. " " .. multiplier_display_text(state.hands[2]),
            "white",
            "black"
        )
    else
        draw_text(
            info_x,
            info_y + 1,
            "-[$] " .. tostring(state.hands[1].bet) .. " " .. multiplier_display_text(state.hands[1]),
            "white",
            "black"
        )
    end


    local phase_key = "game.blackjack.phase_player"
    local phase_color = "yellow"
    if state.phase == "dealer" then
        phase_key = "game.blackjack.phase_dealer"
        phase_color = "light_cyan"
    elseif state.phase == "settle" then
        phase_key = "game.blackjack.phase_settle"
        phase_color = "rgb(255,165,0)"
    end
    local center_left = table_x + 27
    local center_right = table_x + TABLE_W - 28
    if center_right - center_left < 20 then
        center_left = table_x + 20
        center_right = table_x + TABLE_W - 21
    end
    local center_top = dealer_y + CARD_H + 2
    local player_cards_y = table_bottom - 4
    local player_points_y = player_cards_y - 1
    local center_bottom = player_points_y - 2
    if center_bottom < center_top then
        center_bottom = center_top
    end


    local center_w = center_right - center_left + 1
    for y = center_top, center_bottom do
        draw_text(center_left, y, string.rep(" ", center_w), "white", "black")
    end


    local lines_to_draw = { { text = tr(phase_key), color = phase_color } }
    for i = 1, #state.center_lines do
        lines_to_draw[#lines_to_draw + 1] = state.center_lines[i]
    end
    local visible_count = #lines_to_draw
    local area_h = center_bottom - center_top + 1
    local first_y = center_top + math.floor((area_h - visible_count) / 2)
    if first_y < center_top then first_y = center_top end
    for i = 1, visible_count do
        local ln = lines_to_draw[i]
        local txt = (ln and ln.text) or ""
        local clr = (ln and ln.color) or "white"
        local lx = centered_x(txt, center_left, center_right)
        draw_text(lx, first_y + i - 1, txt, clr, "black")
    end


    if state.split_mode then
        local mid_x = table_x + math.floor(TABLE_W / 2)
        local left_zone_l = inner_left
        local left_zone_r = mid_x - 2
        local right_zone_l = mid_x + 2
        local right_zone_r = inner_right
        local left_w = card_group_width(#state.hands[1].cards)
        local right_w = card_group_width(#state.hands[2].cards)
        local left_x = left_zone_l + math.floor((left_zone_r - left_zone_l + 1 - left_w) / 2)
        local right_x = right_zone_l + math.floor((right_zone_r - right_zone_l + 1 - right_w) / 2)
        for i = 1, #state.hands[1].cards do
            draw_card(left_x + (i - 1) * (CARD_W + 1), player_cards_y, state.hands[1].cards[i], false, "white", "white")
        end
        for i = 1, #state.hands[2].cards do
            draw_card(right_x + (i - 1) * (CARD_W + 1), player_cards_y, state.hands[2].cards[i], false, "white", "white")
        end
        draw_points_line(
            tr("game.blackjack.msg_left_points") .. ": " .. hand_value_text(state.hands[1].cards),
            player_points_y,
            "dark_gray"
        )
        draw_points_line(
            tr("game.blackjack.msg_right_points") .. ": " .. hand_value_text(state.hands[2].cards),
            player_points_y + 1,
            "dark_gray"
        )

        local indicator_y = player_cards_y + CARD_H
        if state.active_hand == 1 then
            draw_text(left_x, indicator_y, string.rep("\u{2500}", math.max(5, left_w)), "green", "black")
        else
            draw_text(right_x, indicator_y, string.rep("\u{2500}", math.max(5, right_w)), "green", "black")
        end
    else
        local group_w = card_group_width(#state.hands[1].cards)
        local px = inner_left + math.floor((inner_w - group_w) / 2)
        for i = 1, #state.hands[1].cards do
            draw_card(px + (i - 1) * (CARD_W + 1), player_cards_y, state.hands[1].cards[i], false, "white", "white")
        end
        draw_points_line(
            tr("game.blackjack.msg_player_points") .. ": " .. hand_value_text(state.hands[1].cards),
            player_points_y,
            "dark_gray"
        )
    end


    fill_line(warn_y, w)
    local warning = tr("game.blackjack.warning")
    local wx = centered_x(warning, 1, w)
    draw_text(wx, warn_y, warning, "dark_gray", "black")


    for i = 0, 2 do
        fill_line(controls_y + i, w)
    end
    local offset = math.floor((3 - ctrl_h) / 2)
    if offset < 0 then offset = 0 end
    for i = 1, #ctrl_lines do
        local lx = centered_x(ctrl_lines[i], 1, w)
        draw_text(lx, controls_y + offset + i - 1, ctrl_lines[i], "white", "black")
    end
end


local function all_player_hands_done()
    for i = 1, #state.hands do
        local h = state.hands[i]
        if not h.stood and not h.resolved then
            return false
        end
    end
    return true
end


local function ensure_active_hand()
    if not state.split_mode then
        state.active_hand = 1
        return
    end
    local h = state.hands[state.active_hand]
    if h == nil or h.stood or h.resolved then
        if not state.hands[1].stood and not state.hands[1].resolved then
            state.active_hand = 1
        elseif not state.hands[2].stood and not state.hands[2].resolved then
            state.active_hand = 2
        end
    end
end


local function has_unresolved_hands()
    for i = 1, #state.hands do
        if not state.hands[i].resolved then
            return true
        end
    end
    return false
end

local function apply_initial_dealer_results()
    local dealer_bj = is_blackjack(state.dealer_cards)
    for i = 1, #state.hands do
        local h = state.hands[i]
        local player_bj = h.blackjack
        if h.bust then
            resolve_hand(h, "lose", 0, "game.blackjack.msg_player_bust", "Player bust! Lose bet.", "red")
        elseif h.insured_skip and (not dealer_bj) then
            resolve_hand(h, "push", 0, "game.blackjack.msg_insurance_skip_round", "Player insurance: round skipped.",
                "light_cyan")
        elseif player_bj and not dealer_bj then
            if state.insurance then
                resolve_hand(h, "win", 2.0, "game.blackjack.msg_player_blackjack_insured",
                    "Player blackjack with insurance! Win bet.", "green")
            else
                resolve_hand(h, "win", 1.5, "game.blackjack.msg_player_blackjack", "Player blackjack! Win bet.", "green")
            end
        elseif dealer_bj and not player_bj then
            resolve_hand(h, "lose", -0.5, "game.blackjack.msg_dealer_blackjack", "Dealer blackjack! Lose bet.", "red")
        elseif dealer_bj and player_bj then
            resolve_hand(h, "push", 0, "game.blackjack.msg_both_blackjack", "Both blackjack. Push.", "dark_gray")
        end
    end
    return dealer_bj
end

local function queue_settle_phase()
    state.phase = "settle"
    set_center_lines({
        { text = tr("game.blackjack.phase_settle"), color = "rgb(255,165,0)" }
    })
    state.anim = { kind = "settle_compare_wait", timer_ms = SETTLE_COMPARE_PAUSE_MS }
    state.dirty = true
end

local function finish_settle_phase()
    local dealer_total = hand_total(state.dealer_cards)
    for i = 1, #state.hands do
        local h = state.hands[i]
        if not h.resolved then
            local player_total = hand_total(h.cards)
            if dealer_total > 21 then
                resolve_hand(h, "win", 1.0, "game.blackjack.msg_dealer_bust_win", "Dealer bust! Win bet.", "green")
            elseif player_total > dealer_total then
                resolve_hand(h, "win", 1.0, "game.blackjack.msg_player_higher", "Player higher, win bet!", "green")
            elseif player_total < dealer_total then
                resolve_hand(h, "lose", 0, "game.blackjack.msg_dealer_higher", "Dealer higher, lose bet!", "red")
            else
                resolve_hand(h, "push", 0, "game.blackjack.msg_push", "Push.", "dark_gray")
            end
        end
    end

    state.phase = "settle"
    local lines = {
        { text = tr("game.blackjack.msg_dealer_points") .. ": " .. hand_value_text(state.dealer_cards), color = "rgb(255,165,0)" }
    }
    if state.split_mode then
        lines[#lines + 1] = {
            text = tr("game.blackjack.msg_left_points") ..
            ": " .. hand_value_text(state.hands[1].cards) .. "  " .. state.hands[1].result_text,
            color = state.hands[1].result_color
        }
        lines[#lines + 1] = {
            text = tr("game.blackjack.msg_right_points") ..
            ": " .. hand_value_text(state.hands[2].cards) .. "  " .. state.hands[2].result_text,
            color = state.hands[2].result_color
        }
    else
        lines[#lines + 1] = {
            text = tr("game.blackjack.msg_player_points") .. ": " .. hand_value_text(state.hands[1].cards),
            color = "white"
        }
        lines[#lines + 1] = { text = state.hands[1].result_text, color = state.hands[1].result_color }
    end
    set_center_lines(lines)

    for i = 1, #state.hands do
        local h = state.hands[i]
        if h.outcome == "win" then
            local gain = math.floor(h.bet * h.payout_mult + 0.5)
            state.funds = state.funds + gain
        elseif h.outcome == "lose" and h.payout_mult < 0 then
            state.funds = state.funds - h.bet
            local extra = math.floor(h.bet * math.abs(h.payout_mult) + 0.5)
            state.funds = state.funds - extra
        elseif h.outcome ~= "push" then
            state.funds = state.funds - h.bet
        end
    end

    state.anim = { kind = "results_hold", timer_ms = 1200 }
    state.dirty = true
end

local function finalize_settle_phase()
    if state.funds <= 0 then
        state.bankrupt = true
        state.phase = "player"
        state.await_next_round = false
        set_center_lines({})
    else
        state.await_next_round = true
        flush_input_buffer()
    end
    state.anim = nil
    state.dirty = true
end

local function advance_dealer_animation(dt_ms)
    local anim = state.anim
    if anim == nil then
        return
    end

    anim.timer_ms = anim.timer_ms - dt_ms
    if anim.timer_ms > 0 then
        return
    end

    if anim.kind == "dealer_reveal" then
        local dealer_bj = apply_initial_dealer_results()
        if has_unresolved_hands() and not dealer_bj and hand_total(state.dealer_cards) <= 16 then
            set_center_lines({
                { text = tr("game.blackjack.msg_dealer_drawing") .. " " .. SPINNER[state.spinner_idx], color = "light_cyan" }
            })
            state.spinner_idx = state.spinner_idx + 1
            if state.spinner_idx > #SPINNER then state.spinner_idx = 1 end
            state.anim = { kind = "dealer_draw_wait", timer_ms = DEALER_DRAW_PAUSE_MS }
        else
            queue_settle_phase()
        end
        return
    end

    if anim.kind == "dealer_draw_wait" then
        state.dealer_cards[#state.dealer_cards + 1] = random_rank()
        state.anim = { kind = "dealer_post_draw", timer_ms = DEALER_DRAW_PAUSE_MS }
        state.dirty = true
        return
    end

    if anim.kind == "dealer_post_draw" then
        if hand_total(state.dealer_cards) <= 16 then
            set_center_lines({
                { text = tr("game.blackjack.msg_dealer_drawing") .. " " .. SPINNER[state.spinner_idx], color = "light_cyan" }
            })
            state.spinner_idx = state.spinner_idx + 1
            if state.spinner_idx > #SPINNER then state.spinner_idx = 1 end
            state.anim = { kind = "dealer_draw_wait", timer_ms = DEALER_DRAW_PAUSE_MS }
        else
            queue_settle_phase()
        end
        return
    end

    if anim.kind == "settle_compare_wait" then
        finish_settle_phase()
        return
    end

    if anim.kind == "results_hold" then
        finalize_settle_phase()
    end
end

local function dealer_phase_and_settle()
    state.phase = "dealer"
    state.await_next_round = false
    state.dealer_hidden = false
    set_center_lines({
        { text = tr("game.blackjack.msg_player_stand_dealer"), color = "light_cyan" }
    })
    state.anim = { kind = "dealer_reveal", timer_ms = DEALER_REVEAL_PAUSE_MS }
    state.dirty = true
end

local function hit_current()
    local h = state.hands[state.active_hand]
    if not can_hit(h) then
        add_toast(tr("game.blackjack.action_unavailable"), "red")
        return
    end
    h.adj_locked = true
    state.first_action_done = true
    h.cards[#h.cards + 1] = random_rank()
    h.has_hit = true
    update_hand_state(h)
    update_player_prompt()
    state.dirty = true
    if all_player_hands_done() then
        dealer_phase_and_settle()
    end
end


local function stand_current()
    local h = state.hands[state.active_hand]
    if not can_stand(h) then
        add_toast(tr("game.blackjack.action_unavailable"), "red")
        return
    end
    h.adj_locked = true
    state.first_action_done = true
    h.stood = true
    ensure_active_hand()
    update_player_prompt()
    state.dirty = true
    if all_player_hands_done() then
        dealer_phase_and_settle()
    end
end


local function double_current()
    local h = state.hands[state.active_hand]
    if not can_double(h) then
        add_toast(tr("game.blackjack.action_need_funds"), "red")
        return
    end
    h.adj_locked = true
    state.first_action_done = true
    h.bet = h.bet * 2
    h.mult = 2.0
    h.doubled = true
    update_player_prompt()
    state.dirty = true
end


local function split_current()
    if not can_split() then
        add_toast(tr("game.blackjack.action_need_funds"), "red")
        return
    end
    local h = state.hands[1]
    local adj = h.adj_mult or state.bet_multiplier
    local left = make_hand({ h.cards[1] }, h.bet, adj)
    local right = make_hand({ h.cards[2] }, h.bet, adj)
    state.hands = { left, right }
    state.first_action_done = true
    state.split_mode = true
    state.active_hand = 1
    update_player_prompt()
    state.dirty = true
    if all_player_hands_done() then
        dealer_phase_and_settle()
    end
end


local function insurance_current()
    if not can_insurance() then
        add_toast(tr("game.blackjack.action_unavailable"), "red")
        return
    end

    local h = state.hands[state.active_hand]
    if h ~= nil then
        h.adj_locked = true
        h.stood = true
        h.insured_skip = true
    end

    state.first_action_done = true
    state.insurance = true
    state.force_double_next_round = true
    state.confirm_mode = nil

    dealer_phase_and_settle()
end


local function restart_session()
    state.funds = STARTING_FUNDS
    state.initial_funds = STARTING_FUNDS
    state.bankrupt = false
    state.confirm_mode = nil
    state.await_next_round = false
    state.bet_multiplier = 1.0
    state.first_action_done = false
    state.insurance = false
    state.force_double_next_round = false
    state.toast_text = nil
    state.toast_until = 0
    begin_round()
end


local function handle_confirm_key(key)
    if key == "confirm_yes" then
        if state.confirm_mode == "restart" then
            restart_session()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            maybe_commit_best_on_exit()
            return "exit"
        end
    elseif key == "confirm_no" or key == "quit_action" then
        state.confirm_mode = nil
        update_player_prompt()
        state.dirty = true
        return "changed"
    end
    return "none"
end


local function handle_input(key)
    if key == nil or key == "" then return "none" end


    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end


    if state.bankrupt then
        if key == "restart" then
            restart_session()
            return "changed"
        end
        if key == "quit_action" then
            maybe_commit_best_on_exit()
            return "exit"
        end
        return "none"
    end


    if key == "quit_action" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end

    if key == "restart" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end


    if state.await_next_round then
        if key == "stand" then
            begin_round()
            return "changed"
        end
        return "none"
    end


    if state.bankrupt or state.phase ~= "player" then
        return "none"
    end


    if key == "adjust_up" then
        adjust_bet_multiplier(0.5)
        return "changed"
    end
    if key == "adjust_down" then
        adjust_bet_multiplier(-0.5)
        return "changed"
    end


    if key == "switch_left" and state.split_mode then
        state.active_hand = 1
        update_player_prompt()
        state.dirty = true
        return "changed"
    end
    if key == "switch_right" and state.split_mode then
        state.active_hand = 2
        update_player_prompt()
        state.dirty = true
        return "changed"
    end


    if key == "hit" then
        hit_current(); return "changed"
    end
    if key == "stand" then
        stand_current(); return "changed"
    end
    if key == "double_down" then
        double_current(); return "changed"
    end
    if key == "split_hand" then
        split_current(); return "changed"
    end
    if key == "insurance" then
        insurance_current(); return "changed"
    end
    return "none"
end


local function refresh_dirty_flags()
    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if not toast_visible and state.toast_text ~= nil then
        state.toast_text = nil
        state.dirty = true
    end
end


local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint"),
        tr("warning.back_to_game_list_hint")
    }
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end
    for i = 1, #lines do
        local x = math.floor((term_w - text_width(lines[i])) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, lines[i], "white", "black")
    end
end


local function minimum_required_size()
    local controls_w = min_width_for_lines(controls_text(), 3, 40)
    local warning_w = text_width(tr("game.blackjack.warning"))
    local status_w = text_width(tr("game.blackjack.best") .. ": -999999")
        + 3
        + text_width(tr("game.blackjack.net") .. ": -999999")
    local alert_w = math.max(
        text_width(replace_prompt_keys(tr("game.blackjack.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.blackjack.confirm_exit"))),
        text_width(tr("game.blackjack.msg_bankrupt"))
    )
    local min_w = math.max(TABLE_W + 2, controls_w + 2, warning_w + 2, status_w + 2, alert_w + 2)
    local min_h = TABLE_H + 6
    return min_w, min_h
end


local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()
    if term_w >= min_w and term_h >= min_h then
        local resized = (term_w ~= state.last_term_w) or (term_h ~= state.last_term_h)
        state.last_term_w = term_w
        state.last_term_h = term_h
        if state.size_warning_active then
            clear()
            state.dirty = true
        end
        if resized then
            clear()
            state.dirty = true
        end
        state.size_warning_active = false
        return true
    end

    local changed = (not state.size_warning_active)
        or state.last_warn_term_w ~= term_w
        or state.last_warn_term_h ~= term_h
        or state.last_warn_min_w ~= min_w
        or state.last_warn_min_h ~= min_h
    if changed then
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end
    state.size_warning_active = true
    return false
end


local function runtime_init_game(saved_state)
    clear()
    flush_input_buffer()
    load_best_record()
    restart_session()
    if type(saved_state) == "table" then
        if type(saved_state.funds) == "number" then state.funds = saved_state.funds end
        if type(saved_state.initial_funds) == "number" then state.initial_funds = saved_state.initial_funds end
    end
    state.anim = nil
    state.frame = 0
    state.dirty = true
    return state
end

local function handle_tick(dt_ms)
    if not ensure_terminal_size_ok() then
        state.frame = state.frame + 1
        refresh_dirty_flags()
        return
    end

    if (not state.bankrupt)
        and state.confirm_mode == nil
        and (not state.await_next_round)
        and state.phase == "player"
        and all_player_hands_done()
        and state.anim == nil
    then
        dealer_phase_and_settle()
    end

    advance_dealer_animation(dt_ms or FRAME_MS)
    refresh_dirty_flags()
    state.frame = state.frame + 1
end

local function handle_runtime_event(event)
    local key = normalize_key(event)

    if not ensure_terminal_size_ok() then
        if key == "quit_action" then
            maybe_commit_best_on_exit()
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
        end
        return
    end

    if event ~= nil and event.type == "resize" then
        state.dirty = true
        return
    end

    if event ~= nil and event.type == "tick" then
        handle_tick(event.dt_ms)
        return
    end

    local action = handle_input(key)
    if action == "exit" then
        if type(request_exit) == "function" then
            pcall(request_exit)
        end
        return
    end

    if (not state.bankrupt)
        and state.confirm_mode == nil
        and (not state.await_next_round)
        and state.phase == "player"
        and all_player_hands_done()
        and state.anim == nil
    then
        dealer_phase_and_settle()
    end

    refresh_dirty_flags()
end

local function runtime_render(state_arg)
    state = state_arg or state
    if not ensure_terminal_size_ok() then
        return
    end
    render_once()
end

local function runtime_handle_event(state_arg, event)
    state = state_arg or state
    handle_runtime_event(event)
    return state
end

local function runtime_save_best_score(state_arg)
    state = state_arg or state
    if type(state.best_net) == "number" and state.best_net > 0 then
        return {
            best_string = "game.blackjack.best_block",
            net = state.best_net
        }
    end
    return {
        best_string = "game.blackjack.best_none_block"
    }
end



local function runtime_exit_game(state_arg)
    state = state_arg or state
    maybe_commit_best_on_exit()
    return state
end

local Runtime = {
    init_game = runtime_init_game,
    handle_event = runtime_handle_event,
    render = runtime_render,
    exit_game = runtime_exit_game,
    save_best_score = runtime_save_best_score,
}

_G.BLACKJACK_RUNTIME = Runtime
return Runtime
