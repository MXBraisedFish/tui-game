-- Wordle游戏元数据
GAME_META = {
    name = "Wordle",
    description = "Guess the hidden word using color hints from each attempt."
}

-- 游戏常量
local FPS, FRAME_MS = 60, 16
local MAX_ATTEMPTS = 6 -- 最大尝试次数

-- 游戏状态表
local S = {
    -- 单词数据
    words = {},   -- 所有可用单词列表
    secret = "",  -- 当前要猜的秘密单词
    word_len = 5, -- 单词长度

    -- 游戏状态
    guesses = {},    -- 已尝试的单词列表
    marks = {},      -- 每个猜测的标记结果（correct/present/absent）
    input = "",      -- 当前输入的字母
    mode = "input",  -- 输入模式："input" 或 "action"
    confirm = nil,   -- 确认模式
    settled = false, -- 是否已结束（胜利或失败）
    won = false,     -- 是否获胜

    -- 统计记录
    streak = 0,        -- 当前连胜次数
    best_time_sec = 0, -- 最快完成时间

    -- 时间相关
    frame = 0,
    start_frame = 0,
    end_frame = nil,

    -- 提示消息
    toast = nil,
    toast_color = "green",
    toast_until = 0,

    -- 渲染脏标记
    dirty = true,
    time_dirty = false,
    last_elapsed = -1,
    last_time_line = "",

    -- 终端尺寸
    tw = 0,
    th = 0,
    warn = false,
    lw = 0,
    lh = 0,
    lmw = 0,
    lmh = 0,
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

-- 规范化按键
local function normalize_key(k)
    if k == nil then return "" end
    if type(k) == "string" then return string.lower(k) end
    if type(k) == "table" and type(k.code) == "string" then return string.lower(k.code) end
    return tostring(k):lower()
end

-- 获取文本显示宽度
local function text_width(t)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, t)
        if ok and type(w) == "number" then return w end
    end
    return #t
end

-- 获取终端尺寸
local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

-- 计算已过秒数
local function elapsed_seconds()
    local ef = S.end_frame or S.frame
    return math.max(0, math.floor((ef - S.start_frame) / FPS))
end

-- 格式化时间
local function format_duration(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

-- 随机整数 [0, n-1]
local function rand_int(n)
    if n <= 0 then return 0 end
    if type(random) == "function" then return random(n) end
    return math.random(0, n - 1)
end

-- 计算文本居中位置
local function centered_x(text, l, r)
    local x = l + math.floor(((r - l + 1) - text_width(text)) / 2)
    if x < l then x = l end
    return x
end

-- 清空一行
local function clear_line(y, tw)
    draw_text(1, y, string.rep(" ", tw), "white", "black")
end

-- 读取文件内容
local function read_file(path)
    if not io or not io.open then return nil end
    local f = io.open(path, "r")
    if not f then return nil end
    local data = f:read("*a")
    f:close()
    return data
end

-- 加载单词列表
local function load_words()
    local raw = read_file("assets/wordle/word.json")
    local words, seen = {}, {}

    if type(raw) == "string" then
        for w in raw:gmatch('"([A-Za-z]+)"') do
            local lw = string.lower(w)
            if #lw >= 2 and not seen[lw] then
                seen[lw] = true
                words[#words + 1] = lw
            end
        end
    end

    -- 备用单词列表
    if #words == 0 then
        words = { "apple", "water", "green", "house", "sound", "light", "story", "music", "table", "clock" }
    end

    S.words = words
end

-- 随机选择一个秘密单词
local function pick_word()
    if #S.words == 0 then load_words() end
    local idx = rand_int(#S.words) + 1
    local w = S.words[idx]
    S.secret = string.lower(w)
    S.word_len = #S.secret
end

-- 获取字符串指定位置的字符
local function char_at(str, i)
    return string.sub(str, i, i)
end

-- 评估猜测结果
local function evaluate_guess(secret, guess)
    local n = #secret
    local marks, pool = {}, {}

    -- 统计秘密单词中每个字母的出现次数
    for i = 1, n do
        local c = char_at(secret, i)
        pool[c] = (pool[c] or 0) + 1
        marks[i] = "absent"
    end

    -- 先标记位置完全正确的字母
    for i = 1, n do
        local g = char_at(guess, i)
        local s = char_at(secret, i)
        if g == s then
            marks[i] = "correct"
            pool[g] = (pool[g] or 0) - 1
        end
    end

    -- 再标记存在但位置错误的字母
    for i = 1, n do
        if marks[i] ~= "correct" then
            local g = char_at(guess, i)
            local cnt = pool[g] or 0
            if cnt > 0 then
                marks[i] = "present"
                pool[g] = cnt - 1
            else
                marks[i] = "absent"
            end
        end
    end

    return marks
end

-- 保存最佳时间
local function save_best_time()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_best_time_sec", S.best_time_sec)
    end
end

-- 保存连胜次数
local function save_streak()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_streak", S.streak)
    end
end

-- 加载元数据（最佳时间和连胜）
local function load_meta()
    if type(load_data) ~= "function" then return end

    local ok1, bt = pcall(load_data, "wordle_best_time_sec")
    if ok1 and type(bt) == "number" and bt > 0 then
        S.best_time_sec = math.floor(bt)
    end

    local ok2, st = pcall(load_data, "wordle_streak")
    if ok2 and type(st) == "number" and st >= 0 then
        S.streak = math.floor(st)
    end
end

-- 保存游戏进度
local function save_slot()
    if type(save_game_slot) ~= "function" then return end
    local payload = {
        secret = S.secret,
        guesses = S.guesses,
        input = S.input,
        mode = S.mode,
        streak = S.streak,
        best_time_sec = S.best_time_sec,
        elapsed_sec = elapsed_seconds(),
        settled = S.settled,
        won = S.won,
    }
    pcall(save_game_slot, "wordle", payload)
    S.toast = tr("game.wordle.saved")
    S.toast_color = "green"
    S.toast_until = S.frame + FPS * 2
end

-- 加载游戏进度（如果是继续模式）
local function load_slot_if_continue()
    if type(get_launch_mode) ~= "function" or type(load_game_slot) ~= "function" then return false end
    local mode = string.lower(tostring(get_launch_mode()))
    if mode ~= "continue" then return false end

    local ok, slot = pcall(load_game_slot, "wordle")
    if not ok or type(slot) ~= "table" then return false end
    if type(slot.secret) ~= "string" or slot.secret == "" then return false end
    if slot.settled then return false end -- 已结束的游戏不继续

    S.secret = string.lower(slot.secret)
    S.word_len = #S.secret
    S.guesses = {}
    S.marks = {}

    -- 恢复猜测记录
    if type(slot.guesses) == "table" then
        for i = 1, #slot.guesses do
            local g = tostring(slot.guesses[i]):lower()
            if #g == S.word_len then
                S.guesses[#S.guesses + 1] = g
                S.marks[#S.marks + 1] = evaluate_guess(S.secret, g)
            end
        end
    end

    -- 恢复当前输入
    S.input = type(slot.input) == "string" and string.lower(slot.input) or ""
    if #S.input > S.word_len then
        S.input = string.sub(S.input, 1, S.word_len)
    end

    S.mode = (slot.mode == "action") and "action" or "input"
    if type(slot.streak) == "number" and slot.streak >= 0 then
        S.streak = math.floor(slot.streak)
    end
    if type(slot.best_time_sec) == "number" and slot.best_time_sec > 0 then
        S.best_time_sec = math.floor(slot.best_time_sec)
    end

    local elapsed = 0
    if type(slot.elapsed_sec) == "number" and slot.elapsed_sec >= 0 then
        elapsed = math.floor(slot.elapsed_sec)
    end
    S.start_frame = S.frame - elapsed * FPS
    S.end_frame = nil
    S.settled = false
    S.won = false
    return true
end

-- 开始新的一轮
local function new_round(preserve_streak)
    if not preserve_streak then
        S.streak = 0
        save_streak()
    end

    pick_word()
    S.guesses = {}
    S.marks = {}
    S.input = ""
    S.mode = "input"
    S.confirm = nil
    S.settled = false
    S.won = false
    S.start_frame = S.frame
    S.end_frame = nil
    S.toast = nil
    S.dirty = true
end

-- 结束当前回合
local function settle(win)
    S.settled = true
    S.won = win
    S.end_frame = S.frame

    if win then
        S.streak = S.streak + 1
        local t = elapsed_seconds()
        if S.best_time_sec <= 0 or t < S.best_time_sec then
            S.best_time_sec = t
            save_best_time()
        end
        save_streak()
        if type(update_game_stats) == "function" then
            pcall(update_game_stats, "wordle", S.streak, t)
        end
    else
        S.streak = 0
        save_streak()
    end
end

-- 获取状态文本
local function status_text()
    if S.confirm == "restart" then
        return tr("game.wordle.confirm_restart"), "yellow"
    end
    if S.confirm == "exit" then
        return tr("game.wordle.confirm_exit"), "yellow"
    end
    if S.settled then
        if S.won then
            return tr("game.wordle.win") .. "  " .. tr("game.wordle.result_controls"), "green"
        end
        return tr("game.wordle.lose") .. "  " .. tr("game.wordle.result_controls"), "red"
    end
    if S.toast and S.frame <= S.toast_until then
        return S.toast, S.toast_color
    end
    if S.mode == "action" then
        return tr("game.wordle.mode_action"), "yellow"
    end
    return tr("game.wordle.mode_input"), "dark_gray"
end

-- 获取控制说明文本
local function controls_text()
    if S.settled then
        return tr("game.wordle.controls_result")
    end
    if S.mode == "action" then
        return tr("game.wordle.controls_action")
    end
    return tr("game.wordle.controls_input")
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local cw = text_width(controls_text())
    local row_w = text_width("-> ") + S.word_len * 2 + 2
    local top_w = math.max(
        text_width(tr("game.wordle.best_time") .. " " .. format_duration(0)),
        text_width(tr("game.wordle.time") .. " " .. format_duration(0) .. "  " .. tr("game.wordle.streak") .. " 999")
    )
    local need_w = math.max(60, cw + 2, row_w + 8, top_w + 2)
    return need_w, 14
end

-- 绘制终端尺寸警告
local function draw_terminal_size_warning(tw, th, mw, mh)
    clear()
    local ls = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), mw, mh),
        string.format("%s: %dx%d", tr("warning.current"), tw, th),
        tr("warning.enlarge_hint")
    }
    local top = math.floor((th - #ls) / 2)
    if top < 1 then top = 1 end
    for i = 1, #ls do
        draw_text(centered_x(ls[i], 1, tw), top + i - 1, ls[i], "white", "black")
    end
end

-- 确保终端尺寸足够
local function ensure_terminal_size_ok()
    local tw, th = terminal_size()
    local mw, mh = minimum_required_size()
    if tw >= mw and th >= mh then
        if S.warn then
            clear()
            S.dirty = true
        end
        if tw ~= S.tw or th ~= S.th then
            clear()
            S.dirty = true
        end
        S.tw, S.th, S.warn = tw, th, false
        return true
    end
    local changed = (not S.warn) or S.lw ~= tw or S.lh ~= th or S.lmw ~= mw or S.lmh ~= mh
    if changed then
        draw_terminal_size_warning(tw, th, mw, mh)
        S.lw, S.lh, S.lmw, S.lmh = tw, th, mw, mh
    end
    S.warn = true
    return false
end

-- 顶部时间行文本
local function top_time_line()
    return tr("game.wordle.time") ..
    " " .. format_duration(elapsed_seconds()) .. "  " .. tr("game.wordle.streak") .. " " .. tostring(S.streak)
end

-- 绘制单行猜测
local function draw_guess_row(y, tw, idx)
    local prefix = "-> "
    local guess = S.guesses[idx]
    local marks = S.marks[idx]

    local width = text_width(prefix) + S.word_len * 2
    local x = centered_x(string.rep(" ", width), 1, tw)

    draw_text(x, y, prefix, "white", "black")
    x = x + text_width(prefix)

    for i = 1, S.word_len do
        local ch = " "
        local fg, bg = "white", "black"

        if type(guess) == "string" then
            ch = string.upper(char_at(guess, i))
            local mark = marks and marks[i] or "absent"
            if mark == "correct" then
                fg, bg = "black", "green"
            elseif mark == "present" then
                fg, bg = "black", "yellow"
            else
                fg, bg = "dark_gray", "black"
            end
        end

        draw_text(x, y, ch, fg, bg)
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

-- 绘制当前输入行
local function draw_input_row(y, tw)
    local prefix = "  "
    local width = text_width(prefix) + S.word_len * 2
    local x = centered_x(string.rep(" ", width), 1, tw)

    draw_text(x, y, prefix, "white", "black")
    x = x + text_width(prefix)

    local show = S.input
    if S.settled then
        show = S.secret
    end

    for i = 1, S.word_len do
        local ch, fg = "_", "dark_gray"
        if i <= #show then
            ch = string.upper(char_at(show, i))
            if S.settled then
                fg = S.won and "green" or "red"
            else
                fg = "white"
            end
        end

        draw_text(x, y, ch, fg, "black")
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

-- 主渲染函数
local function render()
    local tw, th = terminal_size()
    local top = math.floor((th - 12) / 2)
    if top < 1 then top = 1 end

    local best = tr("game.wordle.best_time") ..
        " " .. ((S.best_time_sec > 0) and format_duration(S.best_time_sec) or tr("game.twenty_four.none"))
    local tline = top_time_line()
    local msg, mc = status_text()

    -- 顶部区域
    for i = 0, 2 do clear_line(top + i, tw) end
    draw_text(centered_x(best, 1, tw), top, best, "dark_gray", "black")
    draw_text(centered_x(tline, 1, tw), top + 1, tline, "light_cyan", "black")
    S.last_time_line = tline
    draw_text(centered_x(msg, 1, tw), top + 2, msg, mc, "black")

    -- 猜测区域
    local y0 = top + 4
    for i = 0, MAX_ATTEMPTS do clear_line(y0 + i, tw) end
    for i = 1, MAX_ATTEMPTS do
        draw_guess_row(y0 + i - 1, tw, i)
    end
    draw_input_row(y0 + MAX_ATTEMPTS, tw)

    -- 控制说明
    local controls = controls_text()
    clear_line(y0 + MAX_ATTEMPTS + 2, tw)
    draw_text(centered_x(controls, 1, tw), y0 + MAX_ATTEMPTS + 2, controls, "white", "black")
end

-- 仅更新时间显示（优化）
local function render_time_only()
    local tw, th = terminal_size()
    local top = math.floor((th - 12) / 2)
    if top < 1 then top = 1 end
    local tline = top_time_line()

    local cw = math.max(text_width(S.last_time_line or ""), text_width(tline))
    local x = centered_x(string.rep(" ", cw), 1, tw)
    draw_text(x, top + 1, string.rep(" ", cw), "white", "black")
    draw_text(centered_x(tline, 1, tw), top + 1, tline, "light_cyan", "black")
    S.last_time_line = tline
end

-- 应用当前猜测
local function apply_guess()
    if #S.input ~= S.word_len then
        S.toast = tr("game.wordle.need_letters")
        S.toast_color = "red"
        S.toast_until = S.frame + FPS * 2
        S.dirty = true
        return
    end

    local guess = string.lower(S.input)
    local marks = evaluate_guess(S.secret, guess)
    S.guesses[#S.guesses + 1] = guess
    S.marks[#S.marks + 1] = marks
    S.input = ""

    if guess == S.secret then
        settle(true)
    elseif #S.guesses >= MAX_ATTEMPTS then
        settle(false)
    end

    S.dirty = true
end

-- 刷新脏标记
local function refresh_dirty_flags()
    local e = elapsed_seconds()
    if e ~= S.last_elapsed then
        S.last_elapsed = e
        S.time_dirty = true
    end

    local tv = S.toast ~= nil and S.frame <= S.toast_until
    if (not tv) and S.toast ~= nil then
        S.toast = nil
        S.dirty = true
    end
end

-- 处理确认模式按键
local function handle_confirm_key(k)
    if k == "y" or k == "enter" then
        if S.confirm == "restart" then
            S.confirm = nil
            new_round(false)
            return "changed"
        end
        if S.confirm == "exit" then
            return "exit"
        end
    end

    if k == "n" or k == "q" or k == "esc" then
        S.confirm = nil
        S.dirty = true
        return "changed"
    end
    return "none"
end

-- 处理游戏进行中的按键
local function handle_playing_key(k)
    if S.mode == "input" then
        if k == "tab" then
            S.mode = "action"
            S.dirty = true
            return "changed"
        end
        if k == "backspace" or k == "delete" then
            if #S.input > 0 then
                S.input = string.sub(S.input, 1, #S.input - 1)
                S.dirty = true
            end
            return "changed"
        end
        if k == "enter" then
            apply_guess()
            return "changed"
        end
        if k:match("^[a-z]$") then
            if #S.input < S.word_len then
                S.input = S.input .. k
                S.dirty = true
            end
            return "changed"
        end
        return "none"
    end

    -- action模式
    if k == "tab" then
        S.mode = "input"
        S.dirty = true
        return "changed"
    end
    if k == "s" then
        save_slot()
        S.dirty = true
        return "changed"
    end
    if k == "r" then
        S.confirm = "restart"
        S.dirty = true
        return "changed"
    end
    if k == "q" or k == "esc" then
        S.confirm = "exit"
        S.dirty = true
        return "changed"
    end
    return "none"
end

-- 处理游戏结束后的按键
local function handle_settled_key(k)
    if k == "r" then
        new_round(S.won)
        return "changed"
    end
    if k == "q" or k == "esc" then
        return "exit"
    end
    if k == "tab" then
        S.mode = (S.mode == "input") and "action" or "input"
        S.dirty = true
        return "changed"
    end
    return "none"
end

-- 游戏初始化
local function init_game()
    clear()
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end

    load_meta()
    load_words()

    if not load_slot_if_continue() then
        new_round(true)
    end

    S.frame = 0
    S.last_elapsed = elapsed_seconds()
    S.time_dirty = false
    S.dirty = true
end

-- 主游戏循环
local function game_loop()
    while true do
        if not ensure_terminal_size_ok() then
            sleep(FRAME_MS)
            S.frame = S.frame + 1
        else
            local k = normalize_key(get_key(false))
            local a = "none"

            if k ~= "" then
                if S.confirm then
                    a = handle_confirm_key(k)
                elseif S.settled then
                    a = handle_settled_key(k)
                else
                    a = handle_playing_key(k)
                end

                if a == "exit" then
                    exit_game()
                    return
                end
            end

            refresh_dirty_flags()
            if S.dirty then
                render()
                S.dirty = false
                S.time_dirty = false
            elseif S.time_dirty then
                render_time_only()
                S.time_dirty = false
            end

            sleep(FRAME_MS)
            S.frame = S.frame + 1
        end
    end
end

-- 启动游戏
init_game()
game_loop()
