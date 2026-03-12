-- 关灯游戏元数据
GAME_META = {
    name = "Lights Out",
    description = "Light all tiles by toggling cross patterns."
}

-- 游戏常量定义
local DEFAULT_SIZE = 5 -- 默认棋盘大小 5x5
local MIN_SIZE = 2     -- 最小棋盘大小
local MAX_SIZE = 10    -- 最大棋盘大小

local FPS = 60         -- 目标帧率
local FRAME_MS = 16    -- 每帧毫秒数

-- 界面尺寸常量
local CELL_W = 4      -- 单元格宽度
local CELL_H = 3      -- 单元格高度
local CELL_STEP_X = 5 -- 水平步进（包含间距）
local CELL_STEP_Y = 2 -- 垂直步进（包含间距）
local LABEL_W = 3     -- 行列标签宽度

-- 游戏状态表
local state = {
    -- 棋盘状态
    size = DEFAULT_SIZE, -- 当前棋盘大小
    board = {},          -- 二维布尔数组，true=亮，false=灭
    cursor_r = 1,        -- 光标行位置
    cursor_c = 1,        -- 光标列位置
    steps = 0,           -- 已走步数

    -- 帧相关
    frame = 0,          -- 当前帧计数
    start_frame = 0,    -- 游戏开始帧
    end_frame = nil,    -- 游戏结束帧
    won = false,        -- 是否胜利
    confirm_mode = nil, -- 确认模式：nil/restart/exit
    input_mode = nil,   -- 输入模式：nil/size/jump
    input_buffer = "",  -- 输入缓冲区

    -- 提示信息
    toast_text = nil, -- 提示文本
    toast_until = 0,  -- 提示显示截止帧

    -- 自动保存
    last_auto_save_sec = 0, -- 上次自动保存秒数

    -- 渲染相关
    dirty = true,               -- 是否需要重新渲染
    last_elapsed_sec = -1,      -- 上次记录的已过秒数
    last_toast_visible = false, -- 上次提示是否可见

    -- 输入防抖
    last_key = "",         -- 上次按键
    last_key_frame = -100, -- 上次按键帧号

    -- 启动模式
    launch_mode = "new", -- 启动模式：new/continue
    last_area = nil,     -- 上次渲染区域

    -- 最佳记录
    best = nil,             -- 最佳记录 {max_size, min_steps, min_time_sec}
    best_committed = false, -- 是否已提交最佳记录

    -- 终端尺寸
    last_term_w = 0,             -- 上次终端宽度
    last_term_h = 0,             -- 上次终端高度
    size_warning_active = false, -- 是否显示尺寸警告
    last_warn_term_w = 0,        -- 上次警告时的宽度
    last_warn_term_h = 0,        -- 上次警告时的高度
    last_warn_min_w = 0,         -- 上次警告时的最小要求宽度
    last_warn_min_h = 0          -- 上次警告时的最小要求高度
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

-- 按单词换行
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

-- 计算已过秒数
local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

-- 格式化持续时间
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 数值限幅
local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

-- 规范化按键
local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

-- 创建新棋盘
local function new_board(size, value)
    local board = {}
    for r = 1, size do
        board[r] = {}
        for c = 1, size do
            board[r][c] = value
        end
    end
    return board
end

-- 检查棋盘是否全亮
local function all_lit_board(board, size)
    for r = 1, size do
        for c = 1, size do
            if not board[r][c] then
                return false
            end
        end
    end
    return true
end

-- 检查当前棋盘是否全亮
local function all_lit()
    return all_lit_board(state.board, state.size)
end

-- 切换单个单元格
local function toggle_cell(board, size, r, c)
    if r < 1 or r > size or c < 1 or c > size then
        return
    end
    board[r][c] = not board[r][c]
end

-- 切换十字形（上下左右中）
local function toggle_cross_on(board, size, r, c)
    toggle_cell(board, size, r, c)     -- 中心
    toggle_cell(board, size, r - 1, c) -- 上
    toggle_cell(board, size, r + 1, c) -- 下
    toggle_cell(board, size, r, c - 1) -- 左
    toggle_cell(board, size, r, c + 1) -- 右
end

-- 随机生成棋盘（确保不是全亮）
local function randomize_board(size)
    local board = new_board(size, true)
    for _ = 1, size * size do
        local rr = random(size) + 1
        local cc = random(size) + 1
        toggle_cross_on(board, size, rr, cc)
    end
    -- 如果生成了全亮棋盘，再随机点一下
    if all_lit_board(board, size) then
        toggle_cross_on(board, size, random(size) + 1, random(size) + 1)
    end
    return board
end

-- 加载最佳记录
local function load_best_record()
    if type(load_data) ~= "function" then
        return nil
    end
    local ok, data = pcall(load_data, "lights_out_best")
    if not ok or type(data) ~= "table" then
        return nil
    end

    local max_size = tonumber(data.max_size)
    local min_steps = tonumber(data.min_steps)
    local min_time_sec = tonumber(data.min_time_sec)
    if max_size == nil or min_steps == nil or min_time_sec == nil then
        return nil
    end

    return {
        max_size = math.floor(max_size),
        min_steps = math.floor(min_steps),
        min_time_sec = math.floor(min_time_sec)
    }
end

-- 判断是否应该替换最佳记录
local function should_replace_best(old, new)
    if old == nil then
        return true
    end
    -- 优先比较棋盘大小（越大越好）
    if new.max_size ~= old.max_size then
        return new.max_size > old.max_size
    end
    -- 其次比较步数（越小越好）
    if new.min_steps ~= old.min_steps then
        return new.min_steps < old.min_steps
    end
    -- 最后比较时间（越小越好）
    return new.min_time_sec < old.min_time_sec
end

-- 保存最佳记录
local function save_best_record(record)
    if type(save_data) ~= "function" then
        return
    end
    pcall(save_data, "lights_out_best", record)
end

-- 提交最佳记录（如果需要）
local function commit_best_if_needed()
    if state.best_committed then
        return
    end
    local record = {
        max_size = state.size,
        min_steps = state.steps,
        min_time_sec = elapsed_seconds()
    }
    if should_replace_best(state.best, record) then
        state.best = record
        save_best_record(record)
    end
    state.best_committed = true
end

-- 标记胜利
local function mark_won()
    if state.won then
        return
    end
    state.won = true
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_best_if_needed()
    state.dirty = true
end

-- 创建游戏快照
local function make_snapshot()
    return {
        size = state.size,
        board = state.board,
        cursor_r = state.cursor_r,
        cursor_c = state.cursor_c,
        steps = state.steps,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        last_auto_save_sec = state.last_auto_save_sec
    }
end

-- 保存游戏状态
local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "lights_out", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "lights_out", snapshot)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.2048.save_success" or "game.2048.save_unavailable"
        local def = ok and "Save successful!" or "Save API unavailable."
        state.toast_text = tr(key)
        state.toast_until = state.frame + 2 * FPS
        state.dirty = true
    end
end

-- 恢复游戏快照
local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" then
        return false
    end

    local size = tonumber(snapshot.size)
    if size == nil then
        return false
    end
    size = clamp(math.floor(size), MIN_SIZE, MAX_SIZE)

    if type(snapshot.board) ~= "table" then
        return false
    end

    local board = new_board(size, false)
    for r = 1, size do
        if type(snapshot.board[r]) ~= "table" then
            return false
        end
        for c = 1, size do
            board[r][c] = not not snapshot.board[r][c] -- 确保是布尔值
        end
    end

    state.size = size
    state.board = board
    state.cursor_r = clamp(math.floor(tonumber(snapshot.cursor_r) or 1), 1, size)
    state.cursor_c = clamp(math.floor(tonumber(snapshot.cursor_c) or 1), 1, size)
    state.steps = math.max(0, math.floor(tonumber(snapshot.steps) or 0))

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snapshot.last_auto_save_sec) or elapsed))

    state.won = not not snapshot.won
    state.end_frame = nil
    if state.won then
        state.end_frame = state.frame
    end

    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.best_committed = state.won
    state.last_area = nil
    state.dirty = true
    return true
end

-- 加载游戏状态
local function load_game_state()
    local ok = false
    local snapshot = nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "lights_out")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "lights_out")
        ok = s and ret ~= nil
        snapshot = ret
    end

    if ok then
        return restore_snapshot(snapshot)
    end
    return false
end

-- 重置游戏
local function reset_game(new_size)
    if new_size ~= nil then
        state.size = clamp(new_size, MIN_SIZE, MAX_SIZE)
    end

    -- 从全灭状态开始（而不是随机状态）
    state.board = new_board(state.size, false)
    state.cursor_r = 1
    state.cursor_c = 1
    state.steps = 0
    state.start_frame = state.frame
    state.end_frame = nil
    state.won = false
    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.last_auto_save_sec = 0
    state.best_committed = false
    state.last_area = nil
    state.dirty = true
end

-- 游戏初始化
local function init_game()
    clear()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then
            w, h = tw, th
        end
    end
    state.last_term_w, state.last_term_h = w, h
    state.best = load_best_record()
    state.launch_mode = read_launch_mode()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_game(DEFAULT_SIZE)
        end
    else
        reset_game(DEFAULT_SIZE)
    end
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

-- 计算棋盘几何布局
local function board_geometry()
    local w, h = terminal_size()
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H

    local status_w = text_width(tr("game.lights_out.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.lights_out.steps") .. " 9999")
    local win_line_w = text_width(
        tr("game.lights_out.win_banner")
        .. tr("game.lights_out.win_controls")
    )
    local content_w = math.max(LABEL_W + grid_w, status_w, win_line_w)
    local content_h = 1 + grid_h
    local frame_w = content_w + 2
    local frame_h = content_h + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 6 then y = 6 end

    return x, y, frame_w, frame_h, content_w, content_h
end

-- 填充矩形区域
local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then
        return
    end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
end

-- 绘制外边框
local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, "╔" .. string.rep("═", frame_w - 2) .. "╗", "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, "║", "white", "black")
        draw_text(x + frame_w - 1, y + i, "║", "white", "black")
    end
    draw_text(x, y + frame_h - 1, "╚" .. string.rep("═", frame_w - 2) .. "╝", "white", "black")
end

-- 绘制单个灯泡
local function draw_lamp(x, y, lit, selected)
    local lamp_color = lit and "rgb(255,255,0)" or "rgb(210,210,210)" -- 亮黄色，灭灰色

    if selected then
        -- 选中状态：带绿色边框
        draw_text(x, y, "┌──┐", "green", "black")
        draw_text(x, y + 1, "│", "green", "black")
        draw_text(x + 1, y + 1, "██", lamp_color, "black")
        draw_text(x + 3, y + 1, "│", "green", "black")
        draw_text(x, y + 2, "└──┘", "green", "black")
    else
        -- 非选中状态：无边框
        draw_text(x, y, "    ", "white", "black")
        draw_text(x, y + 1, " ██ ", lamp_color, "black")
        draw_text(x, y + 2, "    ", "white", "black")
    end
end

-- 绘制棋盘
local function draw_board(x, y, frame_w, frame_h)
    draw_outer_frame(x, y, frame_w, frame_h)
    local inner_x = x + 1
    local inner_y = y + 1

    draw_text(inner_x, inner_y, string.rep(" ", frame_w - 2), "white", "black")

    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_total_w = LABEL_W + grid_w
    local pad_x = math.floor((frame_w - 2 - grid_total_w) / 2)
    if pad_x < 0 then pad_x = 0 end
    local grid_block_x = inner_x + pad_x
    local grid_x = grid_block_x + LABEL_W

    -- 绘制列号
    for c = 1, state.size do
        local cx = grid_x + (c - 1) * CELL_STEP_X + 1
        draw_text(cx, inner_y, string.format("%2d", c), "dark_gray", "black")
    end

    -- 绘制行号和灯泡
    for r = 1, state.size do
        local row_base = inner_y + 1 + (r - 1) * CELL_STEP_Y
        -- 行号
        draw_text(grid_block_x, row_base + 1, string.format("%2d", r), "dark_gray", "black")

        -- 该行的灯泡
        for c = 1, state.size do
            local cx = grid_x + (c - 1) * CELL_STEP_X
            local selected = (r == state.cursor_r and c == state.cursor_c)
            draw_lamp(cx, row_base, state.board[r][c], selected)
        end
    end

    -- 最后重新绘制选中的灯泡，确保其边框不被覆盖
    local sr = state.cursor_r
    local sc = state.cursor_c
    if sr >= 1 and sr <= state.size and sc >= 1 and sc <= state.size then
        local sel_y = inner_y + 1 + (sr - 1) * CELL_STEP_Y
        local sel_x = grid_x + (sc - 1) * CELL_STEP_X
        draw_lamp(sel_x, sel_y, state.board[sr][sc], true)
    end
end

-- 获取最佳记录显示文本
local function best_line()
    if state.best == nil then
        return tr("game.lights_out.best_none")
    end

    return string.format(
        "%s %dx%d  %s %d  %s %s",
        tr("game.lights_out.best_size"),
        state.best.max_size,
        state.best.max_size,
        tr("game.lights_out.best_steps"),
        state.best.min_steps,
        tr("game.lights_out.best_time"),
        format_duration(state.best.min_time_sec)
    )
end

-- 绘制状态栏
local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local time_text = tr("game.lights_out.time") .. " " .. format_duration(elapsed)
    local steps_text = tr("game.lights_out.steps") .. " " .. tostring(state.steps)
    local term_w = terminal_size()
    local right_x = x + frame_w - text_width(steps_text)
    if right_x < 1 then right_x = 1 end

    -- 清空状态区域
    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    -- 显示最佳记录、时间、步数
    draw_text(x, y - 3, best_line(), "dark_gray", "black")
    draw_text(x, y - 2, time_text, "light_cyan", "black")
    draw_text(right_x, y - 2, steps_text, "light_cyan", "black")

    -- 显示输入提示或状态信息
    if state.input_mode == "size" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.lights_out.input_size_hint"), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.input_mode == "jump" then
        if state.input_buffer == "" then
            draw_text(x, y - 1, tr("game.lights_out.input_jump_hint"), "dark_gray", "black")
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.won then
        local line = tr("game.lights_out.win_banner")
            .. tr("game.lights_out.win_controls")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.2048.confirm_restart"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.2048.confirm_exit"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

-- 绘制控制说明
local function draw_controls(x, y, frame_h)
    local term_w = terminal_size()
    local text = tr("game.lights_out.controls")
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(text, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    -- 清空控制区域
    for i = 1, 3 do
        draw_text(1, y + frame_h + i, string.rep(" ", term_w), "white", "black")
    end

    -- 垂直居中
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    -- 绘制控制说明
    for i = 1, #lines do
        local line = lines[i]
        local line_x = math.floor((term_w - text_width(line)) / 2)
        if line_x < 1 then line_x = 1 end
        draw_text(line_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
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
    draw_board(x, y, frame_w, frame_h)
    draw_controls(x, y, frame_h)
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
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H
    local frame_w = LABEL_W + grid_w + 2
    local frame_h = 1 + grid_h + 2

    local controls_w = min_width_for_lines(
        tr("game.lights_out.controls"),
        3,
        24
    )
    local status_w = text_width(tr("game.lights_out.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.lights_out.steps") .. " 9999")
    local hint_w = text_width(tr("game.lights_out.input_jump_hint"))

    local min_w = math.max(frame_w, controls_w, status_w, hint_w) + 2
    -- 渲染范围是 [y-3, y+frame_h+3]，且 y 最小为6
    -- 所以最小高度至少 frame_h + 9
    local min_h = frame_h + 9
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

-- 确保终端尺寸足够
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

-- 进入输入模式
local function start_input_mode(mode)
    state.input_mode = mode
    state.input_buffer = ""
    state.dirty = true
end

-- 解析尺寸输入
local function parse_size_input()
    local value = tonumber(state.input_buffer)
    if value == nil then
        return nil
    end
    value = math.floor(value)
    if value < MIN_SIZE or value > MAX_SIZE then
        return nil
    end
    return value
end

-- 解析跳转输入
local function parse_jump_input()
    local a, b = state.input_buffer:match("^(%d+)%s+(%d+)$")
    if a == nil or b == nil then
        return nil, nil
    end
    local r = math.floor(tonumber(a) or 0)
    local c = math.floor(tonumber(b) or 0)
    if r < 1 or r > state.size or c < 1 or c > state.size then
        return nil, nil
    end
    return r, c
end

-- 处理输入模式下的按键
local function handle_input_mode_key(key)
    if key == "esc" or key == "q" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "changed"
    end

    if key == "enter" then
        if state.input_mode == "size" then
            local size = parse_size_input()
            state.input_mode = nil
            state.input_buffer = ""
            if size ~= nil then
                if size ~= state.size then
                    clear()
                    state.last_area = nil
                end
                reset_game(size)
            else
                state.dirty = true
            end
            return "changed"
        end

        if state.input_mode == "jump" then
            local r, c = parse_jump_input()
            state.input_mode = nil
            state.input_buffer = ""
            if r ~= nil and c ~= nil then
                state.cursor_r = r
                state.cursor_c = c
            end
            state.dirty = true
            return "changed"
        end
    end

    if key == "backspace" then
        if #state.input_buffer > 0 then
            state.input_buffer = string.sub(state.input_buffer, 1, #state.input_buffer - 1)
            state.dirty = true
            return "changed"
        end
        return "none"
    end

    if state.input_mode == "size" then
        if key:match("^%d$") then
            if #state.input_buffer < 2 then
                state.input_buffer = state.input_buffer .. key
                state.dirty = true
                return "changed"
            end
        end
        return "none"
    end

    if state.input_mode == "jump" then
        if key:match("^%d$") or key == "space" then
            local token = key
            if key == "space" then
                token = " "
            end
            if #state.input_buffer < 6 then
                state.input_buffer = state.input_buffer .. token
                state.dirty = true
                return "changed"
            end
        end
        return "none"
    end

    return "none"
end

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            reset_game(state.size)
            return "changed"
        end
        if state.confirm_mode == "exit" then
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

-- 防抖处理
local function should_debounce(key)
    if not (key == "up" or key == "down" or key == "left" or key == "right") then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

-- 主输入处理函数
local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    if should_debounce(key) then
        return "none"
    end

    -- 输入模式
    if state.input_mode ~= nil then
        return handle_input_mode_key(key)
    end

    -- 确认模式
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 胜利状态
    if state.won then
        if key == "r" then
            reset_game(state.size)
            return "changed"
        end
        if key == "q" or key == "esc" then
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

    if key == "p" then
        start_input_mode("size")
        return "changed"
    end

    if key == "d" then
        start_input_mode("jump")
        return "changed"
    end

    -- 光标移动
    if key == "up" then
        state.cursor_r = clamp(state.cursor_r - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "down" then
        state.cursor_r = clamp(state.cursor_r + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "left" then
        state.cursor_c = clamp(state.cursor_c - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "right" then
        state.cursor_c = clamp(state.cursor_c + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    -- 空格切换
    if key == "space" then
        toggle_cross_on(state.board, state.size, state.cursor_r, state.cursor_c)
        state.steps = state.steps + 1
        if all_lit() then
            mark_won()
        else
            state.dirty = true
        end
        return "changed"
    end

    return "none"
end

-- 自动保存
local function auto_save_if_needed()
    if state.won then
        return
    end
    local elapsed = elapsed_seconds()
    if elapsed - state.last_auto_save_sec >= 60 then
        save_game_state(false)
        state.last_auto_save_sec = elapsed
    end
end

-- 刷新脏标记
local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end

    local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
    if toast_visible ~= state.last_toast_visible then
        state.last_toast_visible = toast_visible
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
