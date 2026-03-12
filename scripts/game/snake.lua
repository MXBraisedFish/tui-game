-- 贪吃蛇游戏元数据
GAME_META = {
    name = "Snake",
    description = "Control the snake, eat food, and avoid biting yourself."
}

-- 游戏常量
local GRID_W = 24 -- 网格宽度
local GRID_H = 10 -- 网格高度
local FPS = 60
local FRAME_MS = 16
local BASE_MOVE_FRAMES = FPS / 2      -- 基础移动速度（每秒2格）
local BOOST_MOVE_FRAMES = FPS / 4     -- 加速移动速度（每秒4格）
local BOOST_DURATION_FRAMES = FPS * 5 -- 加速持续时间（5秒）

-- 边框字符
local BORDER_TL = "\u{2554}" -- ┌
local BORDER_TR = "\u{2557}" -- ┐
local BORDER_BL = "\u{255A}" -- └
local BORDER_BR = "\u{255D}" -- ┘
local BORDER_H = "\u{2550}"  -- ─
local BORDER_V = "\u{2551}"  -- │

-- 游戏状态表
local state = {
    -- 蛇的状态
    snake = {},         -- 蛇身坐标数组，第1个为头
    dir = "right",      -- 当前移动方向
    next_dir = "right", -- 下一个移动方向（用于缓冲）

    -- 食物
    normal_food = nil,    -- 普通食物位置
    special_food = nil,   -- 特殊食物位置
    normal_eaten = 0,     -- 已吃普通食物计数
    next_special_at = 15, -- 下一次特殊食物出现的条件（每15个普通食物）

    -- 游戏进度
    score = 0,
    won = false,
    game_over = false,
    end_frame = nil,

    -- UI状态
    confirm_mode = nil, -- 确认模式：nil, "restart", "exit"
    toast_text = nil,   -- 提示消息文本
    toast_until = 0,    -- 提示消息显示的截止帧

    -- 时间相关
    frame = 0,
    start_frame = 0,
    last_move_frame = 0,   -- 上次移动的帧号
    boost_until_frame = 0, -- 加速状态截止帧

    -- 渲染脏标记
    dirty = true,
    last_elapsed_sec = -1,
    last_boost_sec = -1,
    last_toast_visible = false,

    -- 输入防抖
    last_key = "",
    last_key_frame = -100,

    -- 启动模式
    launch_mode = "new",

    -- 最佳记录
    best_score = 0,
    best_time_sec = 0,
    result_committed = false,

    -- 尺寸警告
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
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
    local lines, current, had_token = {}, "", false

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

-- 计算已过秒数
local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

-- 格式化时间
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 是否处于加速状态
local function is_boosting()
    return state.frame < state.boost_until_frame
end

-- 获取当前移动间隔帧数
local function current_move_frames()
    if is_boosting() then
        return BOOST_MOVE_FRAMES
    end
    return BASE_MOVE_FRAMES
end

-- 获取相反方向
local function opposite_dir(dir)
    if dir == "up" then return "down" end
    if dir == "down" then return "up" end
    if dir == "left" then return "right" end
    if dir == "right" then return "left" end
    return ""
end

-- 克隆蛇身
local function clone_snake(snake)
    local out = {}
    for i = 1, #snake do
        out[i] = { x = snake[i].x, y = snake[i].y }
    end
    return out
end

-- 检查坐标是否在蛇身上
local function snake_contains(x, y, include_tail)
    local last = #state.snake
    if not include_tail and last > 0 then
        last = last - 1 -- 不包含尾部（因为移动后尾部会移开）
    end
    for i = 1, last do
        local s = state.snake[i]
        if s.x == x and s.y == y then
            return true
        end
    end
    return false
end

-- 随机获取空白格子
local function random_empty_cell(avoid_normal, avoid_special)
    local cells = {}
    for y = 1, GRID_H do
        for x = 1, GRID_W do
            if not snake_contains(x, y, true) then
                local blocked = false
                if avoid_normal and state.normal_food ~= nil and state.normal_food.x == x and state.normal_food.y == y then
                    blocked = true
                end
                if avoid_special and state.special_food ~= nil and state.special_food.x == x and state.special_food.y == y then
                    blocked = true
                end
                if not blocked then
                    cells[#cells + 1] = { x = x, y = y }
                end
            end
        end
    end

    if #cells == 0 then
        return nil
    end
    local idx = random(#cells) + 1
    return cells[idx]
end

-- 生成普通食物
local function spawn_normal_food()
    state.normal_food = random_empty_cell(false, true)
end

-- 生成或替换特殊食物
local function spawn_or_replace_special_food()
    state.special_food = random_empty_cell(true, false)
end

-- 检查是否需要生成特殊食物
local function maybe_spawn_special_food()
    while state.normal_eaten >= state.next_special_at do
        state.special_food = nil
        spawn_or_replace_special_food()
        state.next_special_at = state.next_special_at + 15
    end
end

-- 设置默认蛇身
local function set_default_snake()
    local cx = math.floor(GRID_W / 2)
    local cy = math.floor(GRID_H / 2)
    state.snake = {
        { x = cx,     y = cy },
        { x = cx - 1, y = cy },
        { x = cx - 2, y = cy },
    }
    if state.snake[3].x < 1 then
        state.snake[3].x = GRID_W
    end
end

-- 加载最佳记录
local function load_best_record()
    if type(load_data) ~= "function" then
        state.best_score = 0
        state.best_time_sec = 0
        return
    end

    local ok, data = pcall(load_data, "snake_best")
    if not ok or type(data) ~= "table" then
        state.best_score = 0
        state.best_time_sec = 0
        return
    end

    state.best_score = math.max(0, math.floor(tonumber(data.best_score) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(data.best_time_sec) or 0))
end

-- 提交游戏结果
local function commit_result_once()
    if state.result_committed then
        return
    end
    state.result_committed = true

    local elapsed = elapsed_seconds()
    local changed = false

    if state.score > state.best_score then
        state.best_score = state.score
        changed = true
    end
    if elapsed > state.best_time_sec then
        state.best_time_sec = elapsed
        changed = true
    end

    if changed and type(save_data) == "function" then
        pcall(save_data, "snake_best", {
            best_score = state.best_score,
            best_time_sec = state.best_time_sec,
        })
    end

    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "snake", state.score, elapsed)
    end
end

-- 读取启动模式
local function read_launch_mode()
    if type(get_launch_mode) ~= "function" then
        return "new"
    end
    local ok, mode = pcall(get_launch_mode)
    if not ok or type(mode) ~= "string" then
        return "new"
    end
    mode = string.lower(mode)
    if mode == "continue" then
        return "continue"
    end
    return "new"
end

-- 创建游戏快照
local function make_snapshot()
    local cooldown = math.max(0, current_move_frames() - (state.frame - state.last_move_frame))
    local boost_frames = math.max(0, state.boost_until_frame - state.frame)
    return {
        snake = clone_snake(state.snake),
        dir = state.dir,
        next_dir = state.next_dir,
        normal_food = state.normal_food and { x = state.normal_food.x, y = state.normal_food.y } or nil,
        special_food = state.special_food and { x = state.special_food.x, y = state.special_food.y } or nil,
        normal_eaten = state.normal_eaten,
        next_special_at = state.next_special_at,
        score = state.score,
        won = state.won,
        game_over = state.game_over,
        elapsed_sec = elapsed_seconds(),
        boost_remaining_frames = boost_frames,
        move_cooldown_frames = cooldown,
    }
end

-- 验证快照有效性
local function validate_snapshot(snapshot)
    if type(snapshot) ~= "table" then return false end
    if type(snapshot.snake) ~= "table" or #snapshot.snake < 1 then return false end
    for i = 1, #snapshot.snake do
        local s = snapshot.snake[i]
        if type(s) ~= "table" then return false end
        local x = math.floor(tonumber(s.x) or -1)
        local y = math.floor(tonumber(s.y) or -1)
        if x < 1 or x > GRID_W or y < 1 or y > GRID_H then return false end
    end
    return true
end

-- 恢复游戏快照
local function restore_snapshot(snapshot)
    if not validate_snapshot(snapshot) then
        return false
    end

    state.snake = clone_snake(snapshot.snake)
    state.dir = snapshot.dir or "right"
    state.next_dir = snapshot.next_dir or state.dir

    if type(snapshot.normal_food) == "table" then
        state.normal_food = { x = math.floor(snapshot.normal_food.x), y = math.floor(snapshot.normal_food.y) }
    else
        state.normal_food = nil
    end

    if type(snapshot.special_food) == "table" then
        state.special_food = { x = math.floor(snapshot.special_food.x), y = math.floor(snapshot.special_food.y) }
    else
        state.special_food = nil
    end

    state.normal_eaten = math.max(0, math.floor(tonumber(snapshot.normal_eaten) or 0))
    state.next_special_at = math.max(15, math.floor(tonumber(snapshot.next_special_at) or 15))
    state.score = math.max(0, math.floor(tonumber(snapshot.score) or 0))

    state.won = snapshot.won == true
    state.game_over = snapshot.game_over == true

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS

    local boost_remaining = math.max(0, math.floor(tonumber(snapshot.boost_remaining_frames) or 0))
    state.boost_until_frame = state.frame + boost_remaining

    local cooldown = math.max(0, math.floor(tonumber(snapshot.move_cooldown_frames) or 0))
    local interval = current_move_frames()
    local advanced = math.max(0, interval - cooldown)
    state.last_move_frame = state.frame - advanced

    state.confirm_mode = nil
    state.toast_text = nil
    state.toast_until = 0
    state.end_frame = (state.won or state.game_over) and state.frame or nil
    state.result_committed = false
    if state.won or state.game_over then
        commit_result_once()
    end

    state.dirty = true
    return true
end

-- 保存游戏状态
local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()

    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "snake", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "snake", snapshot)
        ok = s and ret ~= false
    elseif type(save_game) == "function" then
        local s, ret = pcall(save_game, snapshot)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.snake.save_success" or "game.snake.save_unavailable"
        local def = ok and "Save successful!" or "Save API unavailable."
        state.toast_text = tr(key)
        state.toast_until = state.frame + 2 * FPS
        state.dirty = true
    end
end

-- 加载游戏状态
local function load_game_state()
    local ok, snapshot = false, nil

    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "snake")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "snake")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_game) == "function" then
        local s, ret = pcall(load_game)
        ok = s and ret ~= nil
        snapshot = ret
    end

    if ok then
        return restore_snapshot(snapshot)
    end
    return false
end

-- 重置游戏
local function reset_game()
    state.dir = "right"
    state.next_dir = "right"

    state.score = 0
    state.won = false
    state.game_over = false
    state.end_frame = nil
    state.result_committed = false

    state.confirm_mode = nil
    state.toast_text = nil
    state.toast_until = 0

    state.normal_eaten = 0
    state.next_special_at = 15
    state.special_food = nil
    state.boost_until_frame = 0

    set_default_snake()
    spawn_normal_food()

    state.start_frame = state.frame
    state.last_move_frame = state.frame
    state.last_elapsed_sec = -1
    state.last_boost_sec = -1
    state.last_toast_visible = false
    state.dirty = true
end

-- 防抖处理
local function should_debounce(key)
    if key ~= "up" and key ~= "down" and key ~= "left" and key ~= "right" then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

-- 计算棋盘几何布局
local function board_geometry()
    local w, h = terminal_size()
    local frame_w = GRID_W + 2
    local frame_h = GRID_H + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 5 then y = 5 end
    return x, y, frame_w, frame_h
end

-- 绘制外边框
local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, BORDER_TL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_TR, "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, BORDER_V, "white", "black")
        draw_text(x + frame_w - 1, y + i, BORDER_V, "white", "black")
    end
    draw_text(x, y + frame_h - 1, BORDER_BL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_BR, "white", "black")
end

-- 绘制状态栏
local function draw_status(x, y)
    local term_w = terminal_size()
    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    -- 截断长文本函数（确保文本不超过终端宽度）
    local function fit_line(line, max_w)
        if text_width(line) <= max_w then
            return line
        end
        local suffix = "..."
        if max_w <= text_width(suffix) then
            return suffix
        end

        local out = ""
        for _, cp in utf8.codes(line) do
            local ch = utf8.char(cp)
            if text_width(out .. ch .. suffix) > max_w then
                break
            end
            out = out .. ch
        end
        return out .. suffix
    end

    -- 绘制居中行
    local function draw_centered_line(y_line, text, fg)
        local line = fit_line(text, math.max(1, term_w - 2))
        local x_line = math.floor((term_w - text_width(line)) / 2)
        if x_line < 1 then x_line = 1 end
        draw_text(x_line, y_line, line, fg, "black")
    end

    -- 显示最佳记录
    local best = tr("game.snake.best_score")
        .. " " .. tostring(state.best_score)
        .. "  "
        .. tr("game.snake.best_time")
        .. " " .. format_duration(state.best_time_sec)
    draw_centered_line(y - 3, best, "dark_gray")

    -- 显示当前时间和分数
    local middle = tr("game.snake.time")
        .. " " .. format_duration(elapsed_seconds())
        .. "  "
        .. tr("game.snake.score")
        .. " " .. tostring(state.score)
    draw_centered_line(y - 2, middle, "light_cyan")

    -- 显示提示信息
    if state.won then
        local line = tr("game.snake.win_banner")
            .. " "
            .. tr("game.snake.result_controls")
        draw_centered_line(y - 1, line, "yellow")
    elseif state.game_over then
        local line = tr("game.snake.lose_banner")
            .. " "
            .. tr("game.snake.result_controls")
        draw_centered_line(y - 1, line, "red")
    elseif state.confirm_mode == "restart" then
        draw_centered_line(y - 1, tr("game.snake.confirm_restart"), "yellow")
    elseif state.confirm_mode == "exit" then
        draw_centered_line(y - 1, tr("game.snake.confirm_exit"), "yellow")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_centered_line(y - 1, state.toast_text, "green")
    elseif is_boosting() then
        local sec = math.max(0, math.ceil((state.boost_until_frame - state.frame) / FPS))
        local line = tr("game.snake.boosting") .. " " .. tostring(sec) .. "s"
        draw_centered_line(y - 1, line, "light_cyan")
    end
end

-- 绘制棋盘
local function draw_board(x, y)
    draw_outer_frame(x, y, GRID_W + 2, GRID_H + 2)

    -- 清空内部
    for yy = 1, GRID_H do
        draw_text(x + 1, y + yy, string.rep(" ", GRID_W), "white", "black")
    end

    -- 绘制食物
    if state.normal_food ~= nil then
        draw_text(x + state.normal_food.x, y + state.normal_food.y, "$", "rgb(255,165,0)", "black")
    end
    if state.special_food ~= nil then
        draw_text(x + state.special_food.x, y + state.special_food.y, "%", "light_cyan", "black")
    end

    -- 绘制蛇（从尾到头，让头在最上层）
    for i = #state.snake, 1, -1 do
        local part = state.snake[i]
        local color = "green"
        if i == 1 then
            color = "yellow" -- 蛇头用黄色
        end
        draw_text(x + part.x, y + part.y, "\u{2588}", color, "black")
    end
end

-- 绘制控制说明
local function draw_controls(y_bottom)
    local term_w = terminal_size()
    local controls = tr("game.snake.controls")

    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(controls, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    -- 清空控制区域
    draw_text(1, y_bottom + 1, string.rep(" ", term_w), "white", "black")
    draw_text(1, y_bottom + 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y_bottom + 3, string.rep(" ", term_w), "white", "black")

    -- 垂直居中
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    -- 绘制控制说明
    for i = 1, #lines do
        local line = lines[i]
        local cx = math.floor((term_w - text_width(line)) / 2)
        if cx < 1 then cx = 1 end
        draw_text(cx, y_bottom + 1 + offset + i - 1, line, "white", "black")
    end
end

-- 主渲染函数
local function render()
    local x, y = board_geometry()
    draw_status(x, y)
    draw_board(x, y)
    draw_controls(y + GRID_H + 2)
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local frame_w = GRID_W + 2
    local frame_h = GRID_H + 2

    local controls_w = min_width_for_lines(
        tr("game.snake.controls"),
        3,
        32
    )

    local best_w = text_width(
        tr("game.snake.best_score") .. " 999999"
        .. "  "
        .. tr("game.snake.best_time")
        .. " 00:00:00"
    )

    local score_w = text_width(tr("game.snake.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.snake.score") .. " 999999")

    local tip_w = math.max(
        text_width(tr("game.snake.win_banner") .. " " .. tr("game.snake.result_controls")),
        text_width(tr("game.snake.lose_banner") .. " " .. tr("game.snake.result_controls")),
        text_width(tr("game.snake.confirm_restart")),
        text_width(tr("game.snake.confirm_exit")),
        text_width(tr("game.snake.save_success")),
        text_width(tr("game.snake.save_unavailable")),
        text_width(tr("game.snake.boosting") .. " 99s")
    )

    local min_w = math.max(frame_w, controls_w, best_w, score_w, tip_w) + 2
    local min_h = frame_h + 8
    return min_w, min_h
end

-- 绘制终端尺寸警告
local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint")
    }

    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end

    for i = 1, #lines do
        local line = lines[i]
        local xx = math.floor((term_w - text_width(line)) / 2)
        if xx < 1 then xx = 1 end
        draw_text(xx, top + i - 1, line, "white", "black")
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

-- 处理方向键输入
local function handle_direction_key(key)
    local requested = nil
    if key == "up" or key == "k" then requested = "up" end
    if key == "down" or key == "j" then requested = "down" end
    if key == "left" or key == "h" then requested = "left" end
    if key == "right" or key == "l" then requested = "right" end

    if requested == nil then
        return false
    end

    -- 不能直接掉头
    if requested == opposite_dir(state.dir) then
        return false
    end

    state.next_dir = requested
    return true
end

-- 计算下一个头部位置（支持穿越边界）
local function next_head_position()
    local head = state.snake[1]
    local dx, dy = 0, 0
    if state.dir == "up" then dy = -1 end
    if state.dir == "down" then dy = 1 end
    if state.dir == "left" then dx = -1 end
    if state.dir == "right" then dx = 1 end

    local nx = ((head.x - 1 + dx) % GRID_W) + 1
    local ny = ((head.y - 1 + dy) % GRID_H) + 1
    return nx, ny
end

-- 更新游戏逻辑（每帧调用）
local function update_tick()
    if state.game_over or state.won or state.confirm_mode ~= nil then
        return
    end

    local interval = current_move_frames()
    if state.frame - state.last_move_frame < interval then
        return
    end

    state.last_move_frame = state.frame
    state.dir = state.next_dir

    local nx, ny = next_head_position()

    local eat_normal = state.normal_food ~= nil and nx == state.normal_food.x and ny == state.normal_food.y
    local eat_special = state.special_food ~= nil and nx == state.special_food.x and ny == state.special_food.y
    local growing = eat_normal or eat_special

    -- 检查是否撞到自己
    if snake_contains(nx, ny, growing) then
        state.game_over = true
        state.end_frame = state.frame
        commit_result_once()
        state.dirty = true
        return
    end

    -- 移动蛇
    table.insert(state.snake, 1, { x = nx, y = ny })

    if eat_normal then
        state.score = state.score + 10
        state.normal_eaten = state.normal_eaten + 1
        spawn_normal_food()
        maybe_spawn_special_food()
    elseif eat_special then
        state.score = state.score + 25
        state.special_food = nil
        state.boost_until_frame = state.frame + BOOST_DURATION_FRAMES
    else
        table.remove(state.snake)
    end

    -- 检查是否占满整个网格（胜利）
    if #state.snake >= GRID_W * GRID_H then
        state.won = true
        state.end_frame = state.frame
        state.normal_food = nil
        state.special_food = nil
        commit_result_once()
    end

    state.dirty = true
end

-- 刷新脏标记
local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end

    local boost_sec = 0
    if is_boosting() then
        boost_sec = math.max(0, math.ceil((state.boost_until_frame - state.frame) / FPS))
    end
    if boost_sec ~= state.last_boost_sec then
        state.last_boost_sec = boost_sec
        state.dirty = true
    end

    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if toast_visible ~= state.last_toast_visible then
        state.last_toast_visible = toast_visible
        state.dirty = true
    end
end

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            commit_result_once()
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_result_once()
            return "exit"
        end
    end

    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end

    return "none"
end

-- 游戏初始化
local function init_game()
    clear()
    load_best_record()

    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_game()
        end
    else
        reset_game()
    end
end

-- 主游戏循环
local function game_loop()
    while true do
        local key = normalize_key(get_key(false))

        if ensure_terminal_size_ok() then
            if key ~= "" and not should_debounce(key) then
                if state.confirm_mode ~= nil then
                    local action = handle_confirm_key(key)
                    if action == "exit" then
                        return
                    end
                elseif state.won or state.game_over then
                    if key == "r" then
                        reset_game()
                    elseif key == "q" or key == "esc" then
                        return
                    elseif key == "s" then
                        save_game_state(true)
                    else
                        handle_direction_key(key)
                    end
                else
                    if key == "r" then
                        state.confirm_mode = "restart"
                        state.dirty = true
                    elseif key == "q" or key == "esc" then
                        state.confirm_mode = "exit"
                        state.dirty = true
                    elseif key == "s" then
                        save_game_state(true)
                    else
                        handle_direction_key(key)
                    end
                end
            end

            update_tick()
            refresh_dirty_flags()

            if state.dirty then
                render()
                state.dirty = false
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
