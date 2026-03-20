-- 石头剪刀布游戏元数据
GAME_META = {
    name = "Rock Paper Scissors",
    description = "Challenge the computer in classic rock-paper-scissors rounds."
}

-- 帧率控制
local FRAME_MS = 16

-- 选项定义
local CHOICES = {
    [1] = { symbol = "Y", key = "game.rock_paper_scissors.choice.scissors", fallback = "Scissors" }, -- 剪刀
    [2] = { symbol = "O", key = "game.rock_paper_scissors.choice.rock", fallback = "Rock" },        -- 石头
    [3] = { symbol = "U", key = "game.rock_paper_scissors.choice.paper", fallback = "Paper" }       -- 布
}

-- 游戏状态表
local state = {
    -- 当前回合
    player_pick = nil,      -- 玩家选择（1-3）
    ai_pick = nil,          -- AI选择（1-3）

    -- 连胜记录
    current_streak = 0,     -- 当前连胜数
    best_streak = 0,        -- 历史最佳连胜
    loss_streak = 0,        -- 当前连续输给系统的次数（用于保底）

    -- 消息显示
    message = "",           -- 提示消息
    message_color = "dark_gray",

    -- 渲染相关
    dirty = true,
    last_term_w = 0,
    last_term_h = 0,

    -- 尺寸警告
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0
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

-- 规范化按键
local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
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

-- 按单词换行
local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end
    local lines = {}
    local current = ""
    local had = false
    for token in string.gmatch(text, "%S+") do
        had = true
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
    if not had then
        return { "" }
    end
    if current ~= "" then
        lines[#lines + 1] = current
    end
    return lines
end

-- 计算最小宽度
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

-- 计算文本居中位置
local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - text_width(text)) / 2)
    if x < area_x then x = area_x end
    return x
end

-- 以中心分隔符为基准绘制左右文本，保证分隔符始终居中
local function draw_center_split_line(y, left_text, right_text, fg, bg)
    local term_w = select(1, terminal_size())
    local center_x = math.floor(term_w / 2)
    local left_w = text_width(left_text)
    local left_x = center_x - 2 - left_w
    local right_x = center_x + 3
    if left_x < 1 then left_x = 1 end
    draw_text(left_x, y, left_text, fg, bg)
    draw_text(center_x, y, "|", fg, bg)
    draw_text(right_x, y, right_text, fg, bg)
end

-- 保存最佳记录
local function save_best()
    if type(save_data) == "function" then
        pcall(save_data, "rock_paper_scissors_best", { best_streak = state.best_streak })
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "rock_paper_scissors", state.best_streak, 0)
    end
end

-- 加载最佳记录
local function load_best()
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "rock_paper_scissors_best")
    if not ok or type(data) ~= "table" then
        return
    end
    local v = tonumber(data.best_streak)
    if v ~= nil and v >= 0 then
        state.best_streak = math.floor(v)
    end
end

-- 获取选项文本（符号+名称）
local function choice_text(index)
    if index == nil or CHOICES[index] == nil then
        return "-"
    end
    local info = CHOICES[index]
    return info.symbol .. " " .. tr(info.key)
end

-- 判定回合结果
-- 返回值：1=玩家胜，0=平局，-1=AI胜
local function resolve_round(player_idx, ai_idx)
    if player_idx == ai_idx then
        return 0
    end
    -- 剪刀(1)胜布(3)
    -- 石头(2)胜剪刀(1)
    -- 布(3)胜石头(2)
    if (player_idx == 1 and ai_idx == 3)
        or (player_idx == 2 and ai_idx == 1)
        or (player_idx == 3 and ai_idx == 2) then
        return 1
    end
    return -1
end

-- 获取当前连输下的玩家保底胜率
-- 第 1 次连输后开始提升，第 8 把前强制保底为 100%
local function player_win_bias(loss_streak)
    if loss_streak <= 0 then
        return 0
    end
    if loss_streak >= 7 then
        return 1
    end
    return loss_streak / 8
end

-- 根据玩家选择生成 AI 选择
-- 为了避免玩家长时间连续失败，系统会根据连输次数逐步提高“本局玩家必胜”的概率
local function pick_ai_choice(player_idx)
    local bias = player_win_bias(state.loss_streak)
    if bias > 0 then
        local roll = (random(1000) + 1) / 1000
        if roll <= bias then
            if player_idx == 1 then return 3 end -- 剪刀胜布
            if player_idx == 2 then return 1 end -- 石头胜剪刀
            return 2                             -- 布胜石头
        end
    end
    return random(3) + 1
end

-- 进行一回合
local function play_round(player_idx)
    -- AI 选择：基础随机 + 连输保底修正
    local ai_idx = pick_ai_choice(player_idx)
    state.player_pick = player_idx
    state.ai_pick = ai_idx

    local result = resolve_round(player_idx, ai_idx)
    local controls = tr("game.rock_paper_scissors.result_controls")
    if result > 0 then
        -- 玩家胜
        state.current_streak = state.current_streak + 1
        state.loss_streak = 0
        if state.current_streak > state.best_streak then
            state.best_streak = state.current_streak
            save_best()
        end
        state.message = tr("game.rock_paper_scissors.win_banner") .. " " .. controls
        state.message_color = "green"
    elseif result < 0 then
        -- AI胜
        state.current_streak = 0
        state.loss_streak = state.loss_streak + 1
        state.message = tr("game.rock_paper_scissors.lose_banner") .. " " .. controls
        state.message_color = "red"
    else
        -- 平局
        state.current_streak = 0
        -- 平局不影响连输保底
        state.message = tr("game.rock_paper_scissors.draw_banner") .. " " .. controls
        state.message_color = "yellow"
    end

    state.dirty = true
end

-- 重置回合（清空选择）
local function reset_round()
    state.player_pick = nil
    state.ai_pick = nil
    state.current_streak = 0
    state.loss_streak = 0
    state.message = tr("game.rock_paper_scissors.ready_banner")
    state.message_color = "dark_gray"
    state.dirty = true
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local top1 = tr("game.rock_paper_scissors.best_streak") .. ": 9999"
    local top2 = tr("game.rock_paper_scissors.current_streak") .. ": 9999"
    local header = tr("game.rock_paper_scissors.player") .. "   |   " .. tr("game.rock_paper_scissors.system")
    local picks = "Y " .. tr("game.rock_paper_scissors.choice.scissors") .. "   |   O " .. tr("game.rock_paper_scissors.choice.rock")
    local msg = tr("game.rock_paper_scissors.win_banner") .. " "
        .. tr("game.rock_paper_scissors.result_controls")
    local controls = tr("game.rock_paper_scissors.controls")
    local controls_w = min_width_for_lines(controls, 3, 24)
    local min_w = math.max(text_width(top1), text_width(top2), text_width(header), text_width(picks), text_width(msg), controls_w) + 2
    local min_h = 10
    return min_w, min_h
end

-- 绘制终端尺寸警告
local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
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
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, line, "white", "black")
    end
end

-- 确保终端尺寸足够
local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then
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
        clear()
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end

    state.size_warning_active = true
    return false
end

-- 绘制控制说明
local function draw_controls(y)
    local controls = tr("game.rock_paper_scissors.controls")
    local term_w = terminal_size()
    local lines = wrap_words(controls, math.max(10, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    -- 清空控制区域
    for i = 1, 3 do
        draw_text(1, y + i - 1, string.rep(" ", term_w), "white", "black")
    end

    -- 垂直居中
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    -- 绘制控制说明
    for i = 1, #lines do
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, y + offset + i - 1, line, "white", "black")
    end
end

-- 主渲染函数
local function render()
    local term_w, term_h = terminal_size()
    local total_h = 8
    local y0 = math.floor((term_h - total_h) / 2) + 1
    if y0 < 1 then y0 = 1 end

    clear()

    -- 显示连胜记录
    local top1 = tr("game.rock_paper_scissors.best_streak") .. ": " .. tostring(state.best_streak)
    local top2 = tr("game.rock_paper_scissors.current_streak") .. ": " .. tostring(state.current_streak)
    draw_text(centered_x(top1, 1, term_w), y0, top1, "dark_gray", "black")
    draw_text(centered_x(top2, 1, term_w), y0 + 1, top2, "light_cyan", "black")

    -- 显示提示消息
    if state.message ~= "" then
        draw_text(centered_x(state.message, 1, term_w), y0 + 2, state.message, state.message_color, "black")
    end

    -- 显示双方选择
    local left_header = tr("game.rock_paper_scissors.player")
    local right_header = tr("game.rock_paper_scissors.system")
    local left_pick = choice_text(state.player_pick)
    local right_pick = choice_text(state.ai_pick)
    draw_center_split_line(y0 + 4, left_header, right_header, "white", "black")
    draw_center_split_line(y0 + 5, left_pick, right_pick, "white", "black")

    -- 绘制控制说明
    draw_controls(y0 + 7)
end

-- 主输入处理函数
local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    -- 退出
    if key == "q" or key == "esc" then
        return "exit"
    end

    -- 重置
    if key == "r" then
        reset_round()
        return "changed"
    end

    -- 选择 1-3
    if key == "1" or key == "2" or key == "3" then
        play_round(tonumber(key))
        return "changed"
    end

    return "none"
end

-- 游戏初始化
local function init_game()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h
    load_best()
    reset_round()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

-- 同步终端尺寸变化
local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        state.dirty = true
    end
end

-- 主游戏循环
local function game_loop()
    while true do
        local key = normalize_key(get_key(false))

        if ensure_terminal_size_ok() then
            local action = handle_input(key)
            if action == "exit" then
                save_best()
                return
            end

            sync_terminal_resize()
            if state.dirty then
                render()
                state.dirty = false
            end
        else
            if key == "q" or key == "esc" then
                save_best()
                return
            end
        end

        sleep(FRAME_MS)
    end
end

-- 启动游戏
init_game()
game_loop()
