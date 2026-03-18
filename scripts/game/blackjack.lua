-- 21点游戏元数据
GAME_META = {
    name = "Blackjack",
    description = "Play against the dealer and manage your bets to win with 21."
}

-- 游戏常量定义
local STARTING_FUNDS = 1000          -- 初始资金
local BASE_BET = 100                 -- 基础赌注
local FPS = 60                       -- 目标帧率
local FRAME_MS = 16                  -- 每帧毫秒数
local DEALER_REVEAL_PAUSE_MS = 500   -- 庄家亮牌暂停时间
local DEALER_DRAW_PAUSE_MS = 500     -- 庄家抽牌暂停时间
local SETTLE_COMPARE_PAUSE_MS = 1000 -- 结算比较暂停时间

-- 界面尺寸常量
local TABLE_W = 108                     -- 桌子宽度
local TABLE_H = 24                      -- 桌子高度
local CARD_W = 5                        -- 卡片宽度
local CARD_H = 3                        -- 卡片高度
local SPINNER = { "|", "/", "-", "\\" } -- 旋转动画

-- 游戏状态表
local state = {
    -- 资金相关
    funds = STARTING_FUNDS,            -- 当前资金
    initial_funds = STARTING_FUNDS, -- 初始资金（用于计算净收益）
    best_net = 0,                      -- 历史最佳净收益

    -- 牌局状态
    dealer_cards = {},               -- 庄家手牌
    dealer_hidden = true,            -- 庄家第二张牌是否隐藏
    hands = {},                      -- 玩家手牌数组（支持分牌）
    split_mode = false,              -- 是否处于分牌模式
    active_hand = 1,                 -- 当前活跃的手牌索引
    insurance = false,               -- 是否购买保险
    force_double_next_round = false, -- 下轮强制加倍
    first_action_done = false,       -- 是否已执行首次操作（用于保险判断）

    -- 游戏流程
    phase = "player",         -- 当前阶段：player/dealer/settle
    center_lines = {},        -- 中央提示信息
    confirm_mode = nil,       -- 确认模式：nil/restart/exit
    await_next_round = false, -- 是否等待下一轮
    bet_multiplier = 1.0,     -- 赌注倍数

    -- 提示信息
    toast_text = nil,       -- 提示文本
    toast_color = "yellow", -- 提示颜色
    toast_until = 0,        -- 提示显示截止帧

    -- 状态标志
    bankrupt = false, -- 是否破产
    spinner_idx = 1,  -- 旋转动画索引
    frame = 0,        -- 当前帧计数
    dirty = true,     -- 是否需要重新渲染

    -- 终端尺寸警告相关
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    last_term_w = 0,
    last_term_h = 0
}

-- 翻译函数（安全调用）
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

-- 获取文本显示宽度
local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
end

-- 按单词换行（保持单词完整性）
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

-- 计算在给定最大行数下所需的最小宽度
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

-- 获取终端尺寸
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

-- 规范化按键值
local function normalize_key(key)
    if key == nil then
        return ""
    end
    if type(key) == "string" then
        return string.lower(key)
    end
    if type(key) == "table" and type(key.code) == "string" then
        return string.lower(key.code)
    end
    return tostring(key):lower()
end

-- 清空输入缓冲区
local function flush_input_buffer()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

-- 填充整行（用于清空行）
local function fill_line(y, width)
    draw_text(1, y, string.rep(" ", width), "white", "black")
end

-- 随机生成牌面
local function random_rank()
    local n = random(13) + 1
    if n == 1 then return "A" end
    if n == 11 then return "J" end
    if n == 12 then return "Q" end
    if n == 13 then return "K" end
    return tostring(n)
end

-- 计算单张牌的点数
local function card_value(rank)
    if rank == "A" then return 11 end
    if rank == "J" or rank == "Q" or rank == "K" then return 10 end
    return tonumber(rank) or 0
end

-- 计算一手牌的总点数（处理Ace的软硬转换）
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
        total = total - 10 -- Ace从11转为1
        aces = aces - 1
    end
    return total
end

-- 判断是否为黑杰克（21点）
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

-- 获取手牌点数文本（黑杰克显示特殊文本）
local function hand_value_text(cards)
    if is_blackjack(cards) then
        return tr("game.blackjack.value_blackjack")
    end
    return tostring(hand_total(cards))
end

-- 创建一手牌
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

-- 更新手牌状态（抽牌后调用）
local function update_hand_state(h)
    local total = hand_total(h.cards)
    h.bust = total > 21
    h.blackjack = is_blackjack(h.cards)
    if h.bust or (total >= 21 and not h.blackjack) then
        h.stood = true
    end
end

-- 计算净收益
local function net_value()
    return state.funds - state.initial_funds
end

-- 计算已下注总额
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

-- 计算可用资金（扣除已下注）
local function available_funds()
    return state.funds - committed_bets_total()
end

-- 加载最佳记录
local function load_best_record()
    state.best_net = 0
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "blackjack_best_net")
    if not ok then
        return
    end
    if type(data) == "number" then
        state.best_net = math.floor(data)
    elseif type(data) == "table" and type(data.value) == "number" then
        state.best_net = math.floor(data.value)
    end
end

-- 保存最佳记录
local function save_best_record()
    if type(save_data) == "function" then
        pcall(save_data, "blackjack_best_net", { value = state.best_net })
    end
end

-- 退出时提交最佳记录
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
        save_best_record()
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "blackjack", net, 0)
    end
end

-- 设置中央提示信息
local function set_center_lines(lines)
    state.center_lines = lines
end

-- 添加提示消息
local function add_toast(text, color)
    state.toast_text = text
    state.toast_color = color or "yellow"
    state.toast_until = state.frame + 2 * FPS
    state.dirty = true
end

-- 获取卡片内文字（处理10需要两个字符）
local function card_inner(rank)
    if rank == "10" then
        return " 10"
    end
    return " " .. rank .. " "
end

-- 判断是否可以要牌
local function can_hit(h)
    if h == nil or h.resolved or h.stood or h.bust or h.blackjack then return false end
    if h.doubled and h.has_hit then return false end
    return hand_total(h.cards) < 21
end

-- 判断是否可以停牌
local function can_stand(h)
    if h == nil or h.resolved then return false end
    return not h.stood
end

-- 判断是否可以加倍
local function can_double(h)
    if h == nil or h.resolved or h.stood or h.bust then return false end
    if h.has_hit or h.doubled then return false end
    return true
end

-- 判断是否可以分牌
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

-- 判断是否可以购买保险
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

-- 前向声明（因为后面会引用）
local update_player_prompt

-- 获取当前活跃的手牌
local function active_hand_ref()
    if state.split_mode then
        return state.hands[state.active_hand]
    end
    return state.hands[1]
end

-- 判断是否可以调整倍数
local function can_adjust_multiplier(h)
    if h == nil or h.resolved or h.stood or h.bust then return false end
    return not h.adj_locked
end

-- 计算有效倍数（考虑手牌倍数和调整倍数）
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

-- 计算结算因子（用于显示实际赔率）
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

-- 获取倍数显示文本
local function multiplier_display_text(h)
    local base = effective_mult(h)
    local settle = settlement_factor(h)
    if settle > 1.001 then
        return string.format("%.1fx", base * settle)
    end
    return string.format("%.1fx", base)
end

-- 调整赌注倍数
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
    next_mult = math.floor(next_mult * 2 + 0.5) / 2 -- 四舍五入到0.5
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

-- 获取手牌可用操作文本
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

-- 更新玩家提示信息
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

-- 开始新的一轮
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

-- 绘制一张牌
local function draw_card(x, y, rank, hidden, border_fg, text_fg)
    local bfg = border_fg or "white"
    local tfg = text_fg or border_fg or "white"
    -- 牌的上边框
    draw_text(x, y, "\u{250C}\u{2500}\u{2500}\u{2500}\u{2510}", bfg, "black")
    -- 牌的中部（显示牌面或隐藏）
    if hidden then
        draw_text(x, y + 1, "\u{2502}XXX\u{2502}", bfg, "black")
    else
        draw_text(x, y + 1, "\u{2502}", bfg, "black")
        draw_text(x + 1, y + 1, card_inner(rank), tfg, "black")
        draw_text(x + CARD_W - 1, y + 1, "\u{2502}", bfg, "black")
    end
    -- 牌的下边框
    draw_text(x, y + 2, "\u{2514}\u{2500}\u{2500}\u{2500}\u{2518}", bfg, "black")
end

-- 结算手牌结果
local function resolve_hand(h, outcome, payout_mult, text_key, fallback, color)
    h.resolved = true
    h.outcome = outcome
    h.payout_mult = payout_mult
    h.result_text = tr(text_key)
    h.result_color = color
end

-- 渲染函数（完整绘制一次）
local function render_once()
    clear()
    local w, h = terminal_size()
    local controls = tr("game.blackjack.controls")
    local ctrl_lines = wrap_words(controls, math.max(10, w - 2))
    if #ctrl_lines > 3 then
        ctrl_lines = { ctrl_lines[1], ctrl_lines[2], ctrl_lines[3] }
    end
    local ctrl_h = #ctrl_lines
    if ctrl_h < 1 then ctrl_h = 1 end

    -- 计算布局位置
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

    -- 居中计算辅助函数
    local function centered_x(text, left_x, right_x)
        local width = text_width(text)
        local cx = left_x + math.floor(((right_x - left_x + 1) - width) / 2)
        if cx < left_x then cx = left_x end
        if cx > right_x - width + 1 then
            cx = math.max(left_x, right_x - width + 1)
        end
        return cx
    end

    -- 绘制状态行（最佳记录和净收益）
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

    -- 绘制提示行（确认信息、提示消息等）
    fill_line(alert_y, w)
    local alert_text = ""
    local alert_color = "red"
    if state.confirm_mode == "restart" then
        alert_text = tr("game.blackjack.confirm_restart")
        alert_color = "yellow"
    elseif state.confirm_mode == "exit" then
        alert_text = tr("game.blackjack.confirm_exit")
        alert_color = "yellow"
    elseif state.bankrupt then
        alert_text = tr("game.blackjack.msg_bankrupt")
            .. "  "
            .. tr("game.blackjack.bankrupt_controls")
        alert_color = "red"
    elseif state.await_next_round then
        alert_text = tr("game.blackjack.msg_press_enter_next")
        alert_color = "yellow"
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        alert_text = state.toast_text
        alert_color = state.toast_color
    end
    if alert_text ~= "" then
        local ax = centered_x(alert_text, 1, w)
        draw_text(ax, alert_y, alert_text, alert_color, "black")
    end

    -- 绘制桌子边框
    draw_text(table_x, table_y, "\u{2554}" .. string.rep("\u{2550}", TABLE_W - 2) .. "\u{2557}", "white", "black")
    for i = 1, TABLE_H - 2 do
        draw_text(table_x, table_y + i, "\u{2551}", "white", "black")
        draw_text(table_x + TABLE_W - 1, table_y + i, "\u{2551}", "white", "black")
    end
    draw_text(table_x, table_y + TABLE_H - 1, "\u{255A}" .. string.rep("\u{2550}", TABLE_W - 2) .. "\u{255D}", "white",
        "black")

    -- 绘制标签
    local dealer_label = " " .. tr("game.blackjack.dealer_cards") .. " "
    local player_label = " " .. tr("game.blackjack.player_cards") .. " "
    draw_text(centered_x(dealer_label, table_x + 2, table_right - 2), table_y, dealer_label, "white", "black")
    draw_text(centered_x(player_label, table_x + 2, table_right - 2), table_y + TABLE_H - 1, player_label, "white",
        "black")

    -- 内部区域
    local inner_left = table_x + 2
    local inner_right = table_right - 2
    local inner_w = inner_right - inner_left + 1
    local table_bottom = table_y + TABLE_H - 1

    -- 计算卡片组宽度
    local function card_group_width(count)
        if count <= 0 then return 0 end
        return count * CARD_W + (count - 1)
    end

    -- 绘制点数行
    local function draw_points_line(text, y, color)
        if y <= table_y or y >= table_bottom then return end
        local x = centered_x(text, inner_left, inner_right)
        draw_text(x, y, text, color, "black")
    end

    -- 绘制庄家手牌
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

    -- 显示庄家点数
    local dealer_points = "?"
    if not state.dealer_hidden then
        dealer_points = hand_value_text(state.dealer_cards)
    end
    draw_points_line(tr("game.blackjack.msg_dealer_points") .. ": " .. dealer_points, dealer_y + CARD_H, "dark_gray")

    -- 绘制牌堆装饰
    local deck_x = table_x + TABLE_W - 16
    local deck_y = table_y + math.floor(TABLE_H / 2) - 2
    draw_text(deck_x, deck_y + 1, "\u{250C}\u{2500}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}\u{250C}",
        "white", "black")
    draw_text(deck_x, deck_y + 2, "\u{2502}X\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}\u{2502}", "white",
        "black")
    draw_text(deck_x, deck_y + 3, "\u{2514}\u{2500}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}\u{2514}",
        "white", "black")

    -- 绘制资金和赌注信息
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

    -- 中央信息区域
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

    -- 清空中央区域
    local center_w = center_right - center_left + 1
    for y = center_top, center_bottom do
        draw_text(center_left, y, string.rep(" ", center_w), "white", "black")
    end

    -- 绘制中央信息
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

    -- 绘制玩家手牌
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
        -- 绘制当前活跃手牌的指示线
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

    -- 绘制警告信息
    fill_line(warn_y, w)
    local warning = tr("game.blackjack.warning")
    local wx = centered_x(warning, 1, w)
    draw_text(wx, warn_y, warning, "dark_gray", "black")

    -- 绘制控制说明
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

-- 暂停并重新渲染
local function pause_with_render(ms)
    state.dirty = true
    if state.dirty then
        state.dirty = false
        render_once()
    end
    sleep(ms)
end

-- 检查所有玩家手牌是否都已停牌
local function all_player_hands_done()
    for i = 1, #state.hands do
        local h = state.hands[i]
        if not h.stood and not h.resolved then
            return false
        end
    end
    return true
end

-- 确保活跃手牌有效
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

-- 庄家阶段和结算
local function dealer_phase_and_settle()
    state.phase = "dealer"
    state.await_next_round = false
    state.dealer_hidden = false
    set_center_lines({
        { text = tr("game.blackjack.msg_player_stand_dealer"), color = "light_cyan" }
    })
    pause_with_render(DEALER_REVEAL_PAUSE_MS)

    -- 先处理黑杰克情况
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

    -- 庄家抽牌（如果还有未结算的手牌且庄家没有黑杰克）
    local unresolved = false
    for i = 1, #state.hands do
        if not state.hands[i].resolved then
            unresolved = true
            break
        end
    end

    if unresolved and not dealer_bj then
        while hand_total(state.dealer_cards) <= 16 do
            set_center_lines({
                { text = tr("game.blackjack.msg_dealer_drawing") .. " " .. SPINNER[state.spinner_idx], color = "light_cyan" }
            })
            state.spinner_idx = state.spinner_idx + 1
            if state.spinner_idx > #SPINNER then state.spinner_idx = 1 end
            pause_with_render(DEALER_DRAW_PAUSE_MS)
            state.dealer_cards[#state.dealer_cards + 1] = random_rank()
            pause_with_render(DEALER_DRAW_PAUSE_MS)
        end
    end

    -- 结算阶段
    set_center_lines({
        { text = tr("game.blackjack.phase_settle"), color = "rgb(255,165,0)" }
    })
    pause_with_render(SETTLE_COMPARE_PAUSE_MS)

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
            text = tr("game.blackjack.msg_player_points") ..
            ": " .. hand_value_text(state.hands[1].cards),
            color = "white"
        }
        lines[#lines + 1] = { text = state.hands[1].result_text, color = state.hands[1].result_color }
    end
    set_center_lines(lines)

    -- 结算资金
    for i = 1, #state.hands do
        local h = state.hands[i]
        if h.outcome == "win" then
            local gain = math.floor(h.bet * h.payout_mult + 0.5)
            state.funds = state.funds + gain
        elseif h.outcome == "push" then
            -- 平局不改变资金
        elseif h.outcome == "lose" and h.payout_mult < 0 then
            state.funds = state.funds - h.bet
            local extra = math.floor(h.bet * math.abs(h.payout_mult) + 0.5)
            state.funds = state.funds - extra
        else
            state.funds = state.funds - h.bet
        end
    end

    pause_with_render(1200)
    if state.funds <= 0 then
        state.bankrupt = true
        state.phase = "player"
        state.await_next_round = false
        set_center_lines({})
        state.dirty = true
    else
        state.await_next_round = true
        flush_input_buffer()
        state.dirty = true
    end
end

-- 当前手牌要牌
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

-- 当前手牌停牌
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

-- 当前手牌加倍
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

-- 分牌
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

-- 购买保险
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

-- 重新开始游戏
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

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            restart_session()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            maybe_commit_best_on_exit()
            return "exit"
        end
    elseif key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        update_player_prompt()
        state.dirty = true
        return "changed"
    end
    return "none"
end

-- 主输入处理函数
local function handle_input(key)
    if key == nil or key == "" then return "none" end

    -- 确认模式
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 破产状态
    if state.bankrupt then
        if key == "r" then
            restart_session()
            return "changed"
        end
        if key == "q" or key == "esc" then
            maybe_commit_best_on_exit()
            return "exit"
        end
        return "none"
    end

    -- 全局功能键
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end

    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end

    -- 等待下一轮状态
    if state.await_next_round then
        if key == "enter" then
            begin_round()
            return "changed"
        end
        return "none"
    end

    -- 非玩家阶段不能操作
    if state.bankrupt or state.phase ~= "player" then
        return "none"
    end

    -- 调整赌注倍数
    if key == "+" or key == "=" or key == "add" then
        adjust_bet_multiplier(0.5)
        return "changed"
    end
    if key == "-" or key == "subtract" then
        adjust_bet_multiplier(-0.5)
        return "changed"
    end

    -- 分牌模式切换活跃手牌
    if key == "left" and state.split_mode then
        state.active_hand = 1
        update_player_prompt()
        state.dirty = true
        return "changed"
    end
    if key == "right" and state.split_mode then
        state.active_hand = 2
        update_player_prompt()
        state.dirty = true
        return "changed"
    end

    -- 游戏操作
    if key == "space" then
        hit_current(); return "changed"
    end
    if key == "enter" then
        stand_current(); return "changed"
    end
    if key == "z" then
        double_current(); return "changed"
    end
    if key == "x" then
        split_current(); return "changed"
    end
    if key == "c" then
        insurance_current(); return "changed"
    end
    return "none"
end

-- 刷新脏标记
local function refresh_dirty_flags()
    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if not toast_visible and state.toast_text ~= nil then
        state.toast_text = nil
        state.dirty = true
    end
end

-- 绘制终端尺寸警告
local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint")
    }
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end
    for i = 1, #lines do
        local x = math.floor((term_w - text_width(lines[i])) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, lines[i], "white", "black")
    end
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local controls_w = min_width_for_lines(tr("game.blackjack.controls"), 3, 40)
    local warning_w = text_width(tr("game.blackjack.warning"))
    local status_w = text_width(tr("game.blackjack.best") .. ": -999999")
        + 3
        + text_width(tr("game.blackjack.net") .. ": -999999")
    local alert_w = math.max(
        text_width(tr("game.blackjack.confirm_restart")),
        text_width(tr("game.blackjack.confirm_exit")),
        text_width(tr("game.blackjack.msg_bankrupt"))
    )
    local min_w = math.max(TABLE_W + 2, controls_w + 2, warning_w + 2, status_w + 2, alert_w + 2)
    local min_h = TABLE_H + 6
    return min_w, min_h
end

-- 确保终端尺寸足够
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

-- 游戏初始化
local function init_game()
    clear()
    flush_input_buffer()
    load_best_record()
    restart_session()
end

-- 主游戏循环
local function game_loop()
    while true do
        local key = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            local action = handle_input(key)
            if action == "exit" then
                return
            end

            -- 自动进入庄家阶段（如果所有玩家手牌都已停牌）
            if (not state.bankrupt)
                and state.confirm_mode == nil
                and (not state.await_next_round)
                and state.phase == "player"
                and all_player_hands_done()
            then
                dealer_phase_and_settle()
            end

            refresh_dirty_flags()
            if state.dirty then
                state.dirty = false
                render_once()
            end
            state.frame = state.frame + 1
        else
            if key == "q" or key == "esc" then
                return
            end
        end
        sleep(FRAME_MS)
    end
end

-- 启动游戏
init_game()
game_loop()
