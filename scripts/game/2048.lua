-- 2048 for TUI GAME
-- 游戏元数据，供启动器显示
GAME_META = {
    name = "2048",
    description = "Merge equal tiles to reach 131072!"
}

-- 游戏常量定义
local SIZE = 4              -- 棋盘大小 4x4
local TARGET_TILE = 131072  -- 目标数字（2^17）
local MAX_TILE = 2147483647 -- 最大允许数字（防止溢出）
local FPS = 60              -- 目标帧率
local FRAME_MS = 16         -- 每帧毫秒数（1000/60≈16）
local CELL_W = 8            -- 每个单元格宽度（字符数）
local CELL_H = 4            -- 每个单元格高度（行数）

-- 边框字符（使用 Unicode 制表符）
local BORDER_TL = "┌" -- 左上角 ┌
local BORDER_TR = "┐" -- 右上角 ┐
local BORDER_BL = "└" -- 左下角 └
local BORDER_BR = "┘" -- 右下角 ┘
local BORDER_H = "─" -- 水平线 ─
local BORDER_V = "│" -- 垂直线 │

-- 游戏状态表
local state = {
    board = {},                  -- 4x4 棋盘，存储每个格子的数值
    score = 0,                   -- 当前得分
    game_over = false,           -- 游戏是否结束
    won = false,                 -- 是否达到目标
    confirm_mode = nil,          -- 确认模式：nil, "game_over", "restart", "exit"
    frame = 0,                   -- 当前帧计数
    start_frame = 0,             -- 游戏开始的帧计数（用于计时）
    win_message_until = 0,       -- 胜利消息显示到第几帧
    last_auto_save_sec = 0,      -- 上次自动保存的秒数
    toast_text = nil,            -- 提示消息文本
    toast_until = 0,             -- 提示消息显示到第几帧
    dirty = true,                -- 是否需要重新渲染
    last_elapsed_sec = -1,       -- 上次记录的已过秒数（用于检测变化）
    last_win_visible = false,    -- 上次胜利消息是否可见
    last_toast_visible = false,  -- 上次提示消息是否可见
    last_key = "",               -- 上次按下的键
    last_key_frame = -100,       -- 上次按键的帧号（用于防抖）
    launch_mode = "new",         -- 启动模式："new" 或 "continue"
    last_area = nil,             -- 上次渲染的区域（用于局部刷新）
    end_frame = nil,             -- 游戏结束时的帧号
    last_term_w = 0,             -- 上次记录的终端宽度
    last_term_h = 0,             -- 上次记录的终端高度
    size_warning_active = false, -- 是否正在显示尺寸警告
    last_warn_term_w = 0,        -- 上次警告时的终端宽度
    last_warn_term_h = 0,        -- 上次警告时的终端高度
    last_warn_min_w = 0,         -- 上次警告时的最小要求宽度
    last_warn_min_h = 0,         -- 上次警告时的最小要求高度
    best_score = 0,              -- 历史最高分
    best_time_sec = 0            -- 历史最高分所用时间（秒）
}

-- 翻译函数封装
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

-- 深度复制棋盘
local function deep_copy_board(board)
    local out = {}
    for r = 1, SIZE do
        out[r] = {}
        for c = 1, SIZE do
            out[r][c] = board[r][c]
        end
    end
    return out
end

-- 初始化空棋盘（全部为0）
local function init_empty_board()
    local board = {}
    for r = 1, SIZE do
        board[r] = {}
        for c = 1, SIZE do
            board[r][c] = 0
        end
    end
    return board
end

-- 随机生成新方块的值（90%概率2，10%概率4）
local function random_tile_value()
    if random(10) == 0 then
        return 4
    end
    return 2
end

-- 列出所有空格子的坐标
local function list_empty_cells(board)
    local cells = {}
    for r = 1, SIZE do
        for c = 1, SIZE do
            if board[r][c] == 0 then
                cells[#cells + 1] = { r = r, c = c }
            end
        end
    end
    return cells
end

-- 在随机空位生成一个新方块
local function spawn_tile(board)
    local empty = list_empty_cells(board)
    if #empty == 0 then
        return false
    end
    local pick = empty[random(#empty) + 1]
    if pick == nil then
        return false
    end
    board[pick.r][pick.c] = random_tile_value()
    return true
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

-- 格式化持续时间（秒转为 HH:MM:SS）
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 格式化单元格数值（大数字缩写）
local function format_cell_value(v)
    if v == 0 then
        return "."
    end
    local text = tostring(v)
    if #text > 4 then
        if v >= 1000000000 then
            text = tostring(math.floor(v / 1000000000)) .. "g" -- 十亿级用g
        elseif v >= 1000000 then
            text = tostring(math.floor(v / 1000000)) .. "m"    -- 百万级用m
        elseif v >= 1000 then
            text = tostring(math.floor(v / 1000)) .. "k"       -- 千级用k
        end
    end
    if #text > 4 then
        text = string.sub(text, 1, 4) -- 最多显示4个字符
    end
    return text
end

-- 根据数值返回背景色
local function tile_bg_color(v)
    if v == 0 then return "rgb(90,90,90)" end      -- 空格子灰色
    if v == 2 then return "rgb(255,255,255)" end   -- 2白色
    if v == 4 then return "rgb(255,229,229)" end   -- 4浅粉
    if v == 8 then return "rgb(255,204,204)" end   -- 8粉色
    if v == 16 then return "rgb(255,178,178)" end  -- 16浅红
    if v == 32 then return "rgb(255,153,153)" end  -- 32粉红
    if v == 64 then return "rgb(255,127,127)" end  -- 64肉红
    if v == 128 then return "rgb(255,102,102)" end -- 128亮红
    if v == 256 then return "rgb(255,76,76)" end   -- 256红
    if v == 512 then return "rgb(255,50,50)" end   -- 512深红
    if v == 1024 then return "rgb(255,25,25)" end  -- 1024更红
    if v == 2048 then return "rgb(255,0,0)" end    -- 2048纯红
    if v == 4096 then return "rgb(212,0,0)" end    -- 4096暗红
    if v == 8192 then return "rgb(170,0,0)" end    -- 8192深红
    if v == 16384 then return "rgb(127,0,0)" end   -- 16384褐红
    if v == 32768 then return "rgb(85,0,0)" end    -- 32768深褐
    if v == 65536 then return "rgb(42,0,0)" end    -- 65536近黑
    return "rgb(0,0,0)"                            -- 其他黑色
end

-- 获取文本显示宽度（使用API）
local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text -- 后备方案：按字符数计算
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

-- 根据数值返回文字颜色（深色背景用亮色文字）
local function text_color_for_value(v)
    if v == 0 then
        return "black"
    end
    if v <= 2048 then
        return "black"
    end
    return "white"
end

-- 合并一行（核心游戏逻辑）
-- 输入一行数值，返回合并后的行和增加的分数
local function merge_line(values)
    -- 1. 移除零，压缩到 compact 数组
    local compact = {}
    for i = 1, #values do
        if values[i] ~= 0 then
            compact[#compact + 1] = values[i]
        end
    end

    -- 2. 合并相邻相同数字
    local out = {}
    local gained = 0
    local i = 1
    while i <= #compact do
        if i < #compact and compact[i] == compact[i + 1] then
            local merged = compact[i] * 2
            if merged > MAX_TILE then merged = MAX_TILE end -- 防止溢出
            out[#out + 1] = merged
            gained = gained + merged
            i = i + 2
        else
            out[#out + 1] = compact[i]
            i = i + 1
        end
    end

    -- 3. 补齐到SIZE长度（末尾补0）
    while #out < SIZE do
        out[#out + 1] = 0
    end
    return out, gained
end

-- 获取指定行
local function get_row(board, r)
    local line = {}
    for c = 1, SIZE do line[c] = board[r][c] end
    return line
end

-- 设置指定行
local function set_row(board, r, line)
    for c = 1, SIZE do board[r][c] = line[c] end
end

-- 获取指定列
local function get_col(board, c)
    local line = {}
    for r = 1, SIZE do line[r] = board[r][c] end
    return line
end

-- 设置指定列
local function set_col(board, c, line)
    for r = 1, SIZE do board[r][c] = line[r] end
end

-- 反转一行
local function reverse_line(line)
    local out = {}
    for i = 1, SIZE do out[i] = line[SIZE - i + 1] end
    return out
end

-- 比较两行是否相等
local function lines_equal(a, b)
    for i = 1, SIZE do
        if a[i] ~= b[i] then return false end
    end
    return true
end

-- 应用移动方向
local function apply_move(dir)
    local moved = false
    local gained = 0

    if dir == "left" or dir == "right" then
        -- 处理左右移动（按行处理）
        for r = 1, SIZE do
            local old = get_row(state.board, r)
            local line = old
            local gained_line = 0
            if dir == "right" then line = reverse_line(line) end -- 右移先反转
            line, gained_line = merge_line(line)
            if dir == "right" then line = reverse_line(line) end -- 合并后再反转回来
            set_row(state.board, r, line)
            if not lines_equal(old, line) then moved = true end
            gained = gained + gained_line
        end
    else
        -- 处理上下移动（按列处理）
        for c = 1, SIZE do
            local old = get_col(state.board, c)
            local line = old
            local gained_line = 0
            if dir == "down" then line = reverse_line(line) end -- 下移先反转
            line, gained_line = merge_line(line)
            if dir == "down" then line = reverse_line(line) end -- 合并后再反转
            set_col(state.board, c, line)
            if not lines_equal(old, line) then moved = true end
            gained = gained + gained_line
        end
    end

    if moved then state.score = state.score + gained end
    return moved
end

-- 检查是否还有任何合法移动
local function can_move_any()
    -- 有空位就可以移动
    if #list_empty_cells(state.board) > 0 then return true end
    -- 检查相邻相同数字
    for r = 1, SIZE do
        for c = 1, SIZE do
            local v = state.board[r][c]
            if r < SIZE and state.board[r + 1][c] == v then return true end
            if c < SIZE and state.board[r][c + 1] == v then return true end
        end
    end
    return false
end

-- 更新胜利状态
local function update_win_and_loss()
    local was_won = state.won
    state.won = false
    for r = 1, SIZE do
        for c = 1, SIZE do
            if state.board[r][c] >= TARGET_TILE then
                state.won = true
                state.win_message_until = state.frame + 3 * FPS -- 显示3秒
                if not was_won then
                    state.end_frame = state.frame
                    commit_stats() -- 达到目标时提交统计
                end
                return
            end
        end
    end
end

-- 创建游戏快照（用于保存）
local function make_snapshot()
    return {
        board = deep_copy_board(state.board),
        score = state.score,
        elapsed_sec = math.floor((state.frame - state.start_frame) / FPS)
    }
end

-- 计算已过秒数
local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

-- 提交游戏统计
local function commit_stats()
    local score = tonumber(state.score) or 0
    local duration = elapsed_seconds()
    -- 更新最佳记录
    if score > state.best_score or (score == state.best_score and score > 0 and (state.best_time_sec == 0 or duration < state.best_time_sec)) then
        state.best_score = score
        state.best_time_sec = duration
        if type(save_data) == "function" then
            pcall(save_data, "2048_best", { score = state.best_score, time_sec = state.best_time_sec })
        end
    end

    -- 更新全局统计
    if type(update_game_stats) ~= "function" then
        return
    end
    pcall(update_game_stats, "2048", score, duration)
end

-- 加载最佳记录
local function load_best_record()
    local data = nil
    if type(load_data) == "function" then
        local ok, ret = pcall(load_data, "2048_best")
        if ok and type(ret) == "table" then
            data = ret
        end
    end

    if data == nil then
        state.best_score = 0
        state.best_time_sec = 0
        return
    end

    state.best_score = math.max(0, math.floor(tonumber(data.score) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(data.time_sec) or 0))
end

-- 恢复游戏快照
local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" or type(snapshot.board) ~= "table" then
        return false
    end
    local board = init_empty_board()
    for r = 1, SIZE do
        if type(snapshot.board[r]) ~= "table" then
            return false
        end
        for c = 1, SIZE do
            board[r][c] = tonumber(snapshot.board[r][c]) or 0
        end
    end

    state.board = board
    state.score = tonumber(snapshot.score) or 0
    local elapsed = tonumber(snapshot.elapsed_sec) or 0
    state.start_frame = state.frame - math.floor(elapsed * FPS)
    state.last_auto_save_sec = elapsed
    state.game_over = false
    state.won = false
    state.confirm_mode = nil
    state.win_message_until = 0
    state.toast_text = nil
    state.toast_until = 0
    state.end_frame = nil
    state.dirty = true
    return true
end

-- 保存游戏状态
local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()
    -- 尝试多种保存API（兼容不同版本）
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "2048", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "2048", snapshot)
        ok = s and ret ~= false
    elseif type(save_game) == "function" then
        local s, ret = pcall(save_game, snapshot)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.2048.save_success" or "game.2048.save_unavailable"
        local def = ok and "Save successful!" or "Save API unavailable."
        state.toast_text = tr(key)
        state.toast_until = state.frame + 2 * FPS -- 显示2秒
        state.dirty = true
    end
end

-- 加载游戏状态
local function load_game_state()
    local ok = false
    local snapshot = nil
    -- 尝试多种加载API
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "2048")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "2048")
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
    state.board = init_empty_board()
    state.score = 0
    state.game_over = false
    state.won = false
    state.confirm_mode = nil
    state.start_frame = state.frame
    state.last_auto_save_sec = 0
    state.toast_text = nil
    state.toast_until = 0
    state.win_message_until = 0
    state.end_frame = nil
    spawn_tile(state.board) -- 初始两个方块
    spawn_tile(state.board)
    state.dirty = true
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

-- 计算棋盘几何布局
local function board_geometry()
    -- 获取终端尺寸
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end

    -- 计算各元素尺寸
    local grid_w = SIZE * CELL_W
    local grid_h = SIZE * CELL_H
    local status_w = text_width(tr("game.2048.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.2048.score") .. " 999999999")
    local best_w = text_width(
        tr("game.2048.best_title")
        .. "  "
        .. tr("game.2048.best_score")
        .. " "
        .. tostring(math.max(0, state.best_score))
        .. "  "
        .. tr("game.2048.best_time")
        .. " "
        .. format_duration(math.max(0, state.best_time_sec))
    )
    local frame_w = math.max(grid_w, status_w, best_w) + 2
    local frame_h = grid_h + 2

    -- 计算居中位置
    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 5 then y = 5 end -- 留出顶部空间显示状态
    return x, y, frame_w, frame_h
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

-- 填充矩形区域
local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
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

-- 绘制单个方块
local function draw_tile(tile_x, tile_y, value)
    local bg = tile_bg_color(value)
    local fg = text_color_for_value(value)

    -- 绘制背景
    for row = 0, CELL_H - 1 do
        draw_text(tile_x, tile_y + row, string.rep(" ", CELL_W), fg, bg)
    end

    -- 绘制数值文字（居中）
    local text = format_cell_value(value)
    local text_x = tile_x + math.floor((CELL_W - #text) / 2)
    local text_y = tile_y + math.floor(CELL_H / 2)
    draw_text(text_x, text_y, text, fg, bg)
end

-- 绘制状态栏（分数、时间、最佳记录等）
local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local left = tr("game.2048.time") .. " " .. format_duration(elapsed)
    local right = tr("game.2048.score") .. " " .. tostring(state.score)
    local term_w = terminal_size()
    local right_x = x + frame_w - text_width(right)
    if right_x < 1 then right_x = 1 end

    -- 清除状态区域
    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    -- 显示最佳记录
    local best_line = tr("game.2048.best_title")
        .. "  "
        .. tr("game.2048.best_score")
        .. " "
        .. tostring(math.max(0, state.best_score))
        .. "  "
        .. tr("game.2048.best_time")
        .. " "
        .. format_duration(math.max(0, state.best_time_sec))
    draw_text(x, y - 3, best_line, "dark_gray", "black")

    -- 显示时间和分数
    draw_text(x, y - 2, left, "light_cyan", "black")
    draw_text(right_x, y - 2, right, "light_cyan", "black")

    -- 显示提示信息（根据当前状态）
    if state.won then
        local line = tr("game.2048.win_banner")
            .. tr("game.2048.win_controls")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "game_over" then
        draw_text(x, y - 1, tr("game.2048.game_over"), "red", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.2048.confirm_restart"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.2048.confirm_exit"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

-- 绘制控制说明
local function draw_controls(x, y, frame_h, frame_w)
    local term_w = terminal_size()
    local controls = tr("game.2048.controls")
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(controls, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] } -- 最多显示3行
    end

    -- 清除控制区域
    draw_text(1, y + frame_h + 1, string.rep(" ", term_w), "white", "black")
    draw_text(1, y + frame_h + 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y + frame_h + 3, string.rep(" ", term_w), "white", "black")

    -- 垂直居中
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        local line = lines[i]
        local controls_x = math.floor((term_w - text_width(line)) / 2)
        if controls_x < 1 then controls_x = 1 end
        draw_text(controls_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
    end
end

-- 清除上次渲染的区域
local function clear_last_area()
    if state.last_area == nil then
        return
    end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

-- 主渲染函数
local function render()
    local x, y, frame_w, frame_h = board_geometry()
    local area = { x = x, y = y - 3, w = frame_w, h = frame_h + 7 }

    -- 如果渲染区域变化，清除旧区域
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or
        state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    -- 绘制各组件
    draw_status(x, y, frame_w)
    draw_outer_frame(x, y, frame_w, frame_h)

    -- 绘制棋盘格子
    local pad_x = math.floor((frame_w - 2 - SIZE * CELL_W) / 2)
    if pad_x < 0 then pad_x = 0 end
    local inner_x = x + 1 + pad_x
    local inner_y = y + 1
    for r = 1, SIZE do
        for c = 1, SIZE do
            local tx = inner_x + (c - 1) * CELL_W
            local ty = inner_y + (r - 1) * CELL_H
            draw_tile(tx, ty, state.board[r][c])
        end
    end

    draw_controls(x, y, frame_h, frame_w)
end

-- 转换方向键
local function apply_direction_key(key)
    if key == "up" or key == "down" or key == "left" or key == "right" then
        return key
    end
    return nil
end

-- 判断是否为移动键
local function is_move_key(key)
    return key == "up" or key == "down" or key == "left" or key == "right"
end

-- 防抖处理（防止同一方向键被连续快速触发）
local function should_debounce(key)
    if not is_move_key(key) then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "game_over" then
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "restart" then
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_stats()
            return "exit"
        end
    end

    if state.confirm_mode == "game_over" and key == "n" then
        commit_stats()
        return "exit"
    end

    if state.confirm_mode == "game_over" then
        return "none"
    end

    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

-- 协调游戏结束状态（如果又有可移动位置，自动退出游戏结束）
local function reconcile_game_over_state()
    if state.confirm_mode == "game_over" and can_move_any() then
        state.game_over = false
        state.confirm_mode = nil
        state.end_frame = nil
        state.dirty = true
    end
end

-- 处理输入
local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end
    if should_debounce(key) then
        return "none"
    end

    reconcile_game_over_state()

    -- 确认模式
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 胜利状态
    if state.won then
        if key == "r" then
            reset_game()
            return "changed"
        end
        if key == "q" or key == "esc" then
            commit_stats()
            return "exit"
        end
        return "none"
    end

    -- 功能键
    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end
    if key == "s" then
        save_game_state(true)
        return "changed"
    end

    -- 游戏结束状态不能移动
    if state.game_over then
        return "none"
    end

    -- 移动
    local dir = apply_direction_key(key)
    if dir == nil then
        return "none"
    end

    local moved = apply_move(dir)
    if moved then
        spawn_tile(state.board)
        update_win_and_loss()
        state.dirty = true
        return "changed"
    end

    -- 无法移动，游戏结束
    if not can_move_any() and not state.game_over then
        state.game_over = true
        state.confirm_mode = "game_over"
        state.end_frame = state.frame
        state.dirty = true
        commit_stats()
        return "changed"
    end

    return "none"
end

-- 自动保存（每分钟一次）
local function auto_save_if_needed()
    local elapsed = elapsed_seconds()
    if elapsed - state.last_auto_save_sec >= 60 then
        save_game_state(false)
        state.last_auto_save_sec = elapsed
    end
end

-- 刷新脏标记（检测需要更新的内容）
local function refresh_dirty_flags()
    -- 时间变化
    local elapsed = math.floor((state.frame - state.start_frame) / FPS)
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end

    -- 胜利消息可见性变化
    local win_visible = state.frame <= state.win_message_until
    if win_visible ~= state.last_win_visible then
        state.last_win_visible = win_visible
        state.dirty = true
    end

    -- 提示消息可见性变化
    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if toast_visible ~= state.last_toast_visible then
        state.last_toast_visible = toast_visible
        state.dirty = true
    end
end

-- 同步终端尺寸变化
local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        clear()
        state.last_area = nil
        state.dirty = true
    end
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local frame_w = SIZE * CELL_W + 2
    local frame_h = SIZE * CELL_H + 2

    local controls_w = min_width_for_lines(
        tr("game.2048.controls"),
        3,
        24
    )
    local status_w = text_width(tr("game.2048.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.2048.score") .. " 999999999")
    local best_w = text_width(
        tr("game.2048.best_title")
        .. "  "
        .. tr("game.2048.best_score")
        .. " 999999999  "
        .. tr("game.2048.best_time")
        .. " 00:00:00"
    )
    local win_line_w = text_width(
        tr("game.2048.win_banner")
        .. tr("game.2048.win_controls")
    )
    local tip_w = math.max(
        text_width(tr("game.2048.game_over")),
        text_width(tr("game.2048.confirm_restart")),
        text_width(tr("game.2048.confirm_exit")),
        win_line_w
    )

    local min_w = math.max(frame_w, controls_w, status_w, best_w, tip_w) + 2
    -- 渲染范围是 [y-3, y+frame_h+3]，且 y 最小为5
    -- 所以最小高度至少 frame_h + 8
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
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, line, "white", "black")
    end
end

-- 确保终端尺寸足够，否则显示警告
local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then
            clear()
            state.last_area = nil
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

-- 初始化游戏
local function init_game()
    clear()
    state.last_term_w, state.last_term_h = terminal_size()
    state.last_area = nil
    load_best_record()
    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_game()
        end
    else
        reset_game()
    end
    update_win_and_loss()
    state.dirty = true
end

-- 主游戏循环
local function game_loop()
    while true do
        -- 非阻塞获取按键
        local key = normalize_key(get_key(false))

        -- 检查终端尺寸
        if ensure_terminal_size_ok() then
            local action = handle_input(key)
            if action == "exit" then
                return
            end

            sync_terminal_resize()
            auto_save_if_needed()
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
