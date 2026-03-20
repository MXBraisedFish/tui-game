-- 颜色记忆游戏元数据
GAME_META = {
    name = "Color Memory",
    description = "Repeat the color sequence exactly as the system presents it."
}

-- 游戏常量定义
local FPS = 60          -- 目标帧率
local FRAME_MS = 16     -- 每帧毫秒数
local SHOW_ON_MS = 1200 -- 颜色高亮显示时间（1.2秒）
local SHOW_OFF_MS = 800 -- 颜色熄灭间隔时间（0.8秒）

-- 界面尺寸常量
local BOX_W = 4     -- 颜色方块宽度
local BOX_H = 3     -- 颜色方块高度
local BOX_GAP = 3   -- 颜色方块间距
local INPUT_GAP = 1 -- 输入区域方块间距
local FRAME_H = 12  -- 游戏主框架高度

-- 颜色定义（四种颜色）
local COLORS = {
    { bg = "rgb(255,0,0)" },   -- 红色
    { bg = "rgb(255,255,0)" }, -- 黄色
    { bg = "rgb(0,120,255)" }, -- 蓝色
    { bg = "rgb(0,200,0)" }    -- 绿色
}

-- 游戏状态表
local state = {
    -- 游戏进度
    score = 0,         -- 当前得分
    round = 1,         -- 当前回合数
    sequence = {},     -- 系统生成的顺序序列
    input_colors = {}, -- 玩家输入的颜色序列
    highlight_idx = 0, -- 当前高亮的颜色索引

    -- 最佳记录
    best_score = 0,    -- 历史最高分
    best_time_sec = 0, -- 历史最长游戏时间

    -- 游戏状态
    phase = "input",    -- 当前阶段：show/input/lost
    lost = false,       -- 是否失败
    confirm_mode = nil, -- 确认模式：nil/restart/exit
    committed = false,  -- 是否已提交统计

    -- 帧相关
    frame = 0,       -- 当前帧计数
    start_frame = 0, -- 游戏开始的帧计数
    end_frame = nil, -- 游戏结束的帧计数
    running = true,  -- 是否运行中
    dirty = true,    -- 是否需要重新渲染

    -- 时间相关
    last_elapsed_sec = -1, -- 上次记录的已过秒数

    -- 终端尺寸相关
    last_term_w = 0, -- 上次记录的终端宽度
    last_term_h = 0, -- 上次记录的终端高度

    -- 尺寸警告相关
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
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    return tostring(key):lower()
end

-- 清空输入缓冲区
local function flush_input_buffer()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

-- 计算已过秒数
local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

-- 格式化持续时间（秒转为 HH:MM:SS）
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 填充整行（用于清空行）
local function fill_line(y, width)
    draw_text(1, y, string.rep(" ", width), "white", "black")
end

-- 填充矩形区域
local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", bg or "black")
    end
end

-- 绘制外边框
local function draw_outer_frame(x, y, w, h)
    draw_text(x, y, "╔" .. string.rep("═", w - 2) .. "╗", "white", "black")
    for i = 1, h - 2 do
        draw_text(x, y + i, "║", "white", "black")
        draw_text(x + w - 1, y + i, "║", "white", "black")
    end
    draw_text(x, y + h - 1, "╚" .. string.rep("═", w - 2) .. "╝", "white", "black")
end

-- 绘制颜色填充方块（无边框）
local function draw_color_fill_slot(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    fill_rect(x, y, BOX_W, BOX_H, "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
end

-- 绘制高亮方块（带边框）
local function draw_highlight_box(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    draw_text(x, y, "┌──┐", "white", "black")
    draw_text(x, y + 1, "│", "white", "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
    draw_text(x + 3, y + 1, "│", "white", "black")
    draw_text(x, y + 2, "└──┘", "white", "black")
end

-- 加载最佳记录
local function load_best_record()
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "color_memory_best")
    if not ok or type(data) ~= "table" then
        return
    end
    local bs = tonumber(data.best_score)
    local bt = tonumber(data.best_time_sec)
    if bs ~= nil and bs >= 0 then
        state.best_score = math.floor(bs)
    end
    if bt ~= nil and bt >= 0 then
        state.best_time_sec = math.floor(bt)
    end
end

-- 保存最佳记录
local function save_best_record()
    if type(save_data) ~= "function" then
        return
    end
    pcall(save_data, "color_memory_best", {
        best_score = state.best_score,
        best_time_sec = state.best_time_sec
    })
end

-- 提交游戏统计
local function commit_stats_if_needed()
    if state.committed then
        return
    end
    local dur = elapsed_seconds()
    if state.score > state.best_score then
        state.best_score = state.score
    end
    if dur > state.best_time_sec then
        state.best_time_sec = dur
    end
    save_best_record()
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "color_memory", state.score, dur)
    end
    state.committed = true
end

-- 推进时间帧
local function advance_time(ms)
    local delta = math.max(1, math.floor(ms / FRAME_MS))
    state.frame = state.frame + delta
end

-- 计算文本居中位置
local function centered_x(text, left_x, right_x)
    local width = text_width(text)
    local x = left_x + math.floor(((right_x - left_x + 1) - width) / 2)
    if x < left_x then x = left_x end
    if x > right_x - width + 1 then
        x = math.max(left_x, right_x - width + 1)
    end
    return x
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local controls = tr("game.color_memory.controls")
    local controls_w = min_width_for_lines(controls, 3, 40)

    local best_line = tr("game.color_memory.best_score") .. " 99999  "
        .. tr("game.color_memory.best_time") .. " 00:00:00"
    local curr_line = tr("game.color_memory.time") .. " 00:00:00  "
        .. tr("game.color_memory.score") .. " 99999"

    local info_w = math.max(
        text_width(tr("game.color_memory.confirm_restart")),
        text_width(tr("game.color_memory.confirm_exit")),
        text_width(
            tr("game.color_memory.lose_banner")
            .. " "
            .. tr("game.color_memory.lose_controls")
        )
    )

    local boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local frame_w = math.max(48, boxes_w + 10, info_w + 2)
    local min_w = math.max(frame_w + 2, controls_w + 2, text_width(best_line) + 2, text_width(curr_line) + 2)
    local min_h = FRAME_H + 7
    return min_w, min_h
end

-- 绘制终端尺寸警告
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

-- 确保终端尺寸足够
local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        local resized = (term_w ~= state.last_term_w) or (term_h ~= state.last_term_h)
        state.last_term_w = term_w
        state.last_term_h = term_h
        if state.size_warning_active or resized then
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

-- 计算游戏框架几何布局
local function frame_geometry()
    local term_w, term_h = terminal_size()
    local frame_w = math.max(48, 4 * BOX_W + 3 * BOX_GAP + 10)
    local top_h = 3
    local bottom_h = 3
    local block_h = top_h + FRAME_H + bottom_h

    local top = math.floor((term_h - block_h) / 2) + 1
    if top < 1 then top = 1 end

    local x = math.floor((term_w - frame_w) / 2)
    if x < 1 then x = 1 end

    return {
        best_y = top,                       -- 最佳记录行Y坐标
        current_y = top + 1,                -- 当前信息行Y坐标
        info_y = top + 2,                   -- 提示信息行Y坐标
        game_x = x,                         -- 游戏框架X坐标
        game_y = top + 3,                   -- 游戏框架Y坐标
        frame_w = frame_w,                  -- 游戏框架宽度
        frame_h = FRAME_H,                  -- 游戏框架高度
        controls_y = top + 3 + FRAME_H + 1, -- 控制说明Y坐标
        term_w = term_w                     -- 终端宽度
    }
end

-- 获取游戏框架内部区域
local function game_inner(g)
    return g.game_x + 1, g.game_y + 1, g.frame_w - 2
end

-- 格式化回合文本（处理语言差异）
local function format_round_text()
    local tmpl = tr("game.color_memory.round")
    if string.find(tmpl, "{n}", 1, true) ~= nil then
        return string.gsub(tmpl, "{n}", tostring(state.round))
    end
    if tmpl == "第几局" then
        return "第" .. tostring(state.round) .. "局"
    end
    return tmpl .. " " .. tostring(state.round)
end

-- 绘制显示区域（上方的高亮演示区）
local function draw_show_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    fill_rect(inner_x, inner_y, inner_w, 7, "black")

    -- 显示当前回合数
    local round_text = format_round_text()
    draw_text(centered_x(round_text, inner_x, inner_x + inner_w - 1), inner_y, round_text, "yellow", "black")

    -- 绘制四个颜色方块
    local total_boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local row_x = inner_x + math.floor((inner_w - total_boxes_w) / 2)
    local show_y = inner_y + 2
    for i = 1, 4 do
        local bx = row_x + (i - 1) * (BOX_W + BOX_GAP)
        if state.highlight_idx == i then
            draw_highlight_box(bx, show_y, i)
        else
            draw_color_fill_slot(bx, show_y, i)
        end
    end

    -- 显示当前阶段状态
    local status_text = ""
    if state.phase == "show" then
        status_text = tr("game.color_memory.status_observe")
    elseif state.phase == "input" then
        status_text = tr("game.color_memory.status_input")
    end
    draw_text(centered_x(status_text, inner_x, inner_x + inner_w - 1), inner_y + 6, status_text, "dark_gray", "black")
end

-- 绘制输入区域（下方的玩家输入区）
local function draw_input_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    local input_y = inner_y + 7
    fill_rect(inner_x, input_y, inner_w, 3, "black")

    -- 计算可见的输入方块（最近输入的优先显示）
    local max_slots = math.max(1, math.floor((inner_w + INPUT_GAP) / (BOX_W + INPUT_GAP)))
    local start_idx = 1
    if #state.input_colors > max_slots then
        start_idx = #state.input_colors - max_slots + 1
    end
    local visible = #state.input_colors - start_idx + 1
    local input_w = visible * BOX_W + math.max(0, visible - 1) * INPUT_GAP
    local input_x = inner_x + math.floor((inner_w - input_w) / 2)

    -- 绘制输入方块
    for i = start_idx, #state.input_colors do
        local slot = i - start_idx
        local bx = input_x + slot * (BOX_W + INPUT_GAP)
        draw_color_fill_slot(bx, input_y, state.input_colors[i])
    end

    -- 保持底部边框完整（部分重绘时确保边框不被覆盖）
    draw_text(
        g.game_x,
        g.game_y + g.frame_h - 1,
        "╚" .. string.rep("═", g.frame_w - 2) .. "╝",
        "white",
        "black"
    )
end

-- 绘制头部信息（最佳记录、当前时间/分数、提示）
local function draw_header(g)
    fill_line(g.best_y, g.term_w)
    fill_line(g.current_y, g.term_w)
    fill_line(g.info_y, g.term_w)

    -- 显示最佳记录
    local best_line = tr("game.color_memory.best_score") .. ": " .. tostring(state.best_score)
        .. "  "
        .. tr("game.color_memory.best_time") .. ": " .. format_duration(state.best_time_sec)
    draw_text(centered_x(best_line, 1, g.term_w), g.best_y, best_line, "dark_gray", "black")

    -- 显示当前时间和分数
    local current_line = tr("game.color_memory.time") .. ": " .. format_duration(elapsed_seconds())
        .. "  "
        .. tr("game.color_memory.score") .. ": " .. tostring(state.score)
    draw_text(centered_x(current_line, 1, g.term_w), g.current_y, current_line, "light_cyan", "black")

    -- 显示提示信息（确认或失败）
    local info = ""
    local info_color = "yellow"
    if state.confirm_mode == "restart" then
        info = tr("game.color_memory.confirm_restart")
    elseif state.confirm_mode == "exit" then
        info = tr("game.color_memory.confirm_exit")
    elseif state.lost then
        info = tr("game.color_memory.lose_banner")
            .. " "
            .. tr("game.color_memory.lose_controls")
        info_color = "red"
    end
    if info ~= "" then
        draw_text(centered_x(info, 1, g.term_w), g.info_y, info, info_color, "black")
    end
end

-- 绘制控制说明
local function draw_controls(g)
    local controls = tr("game.color_memory.controls")
    local lines = wrap_words(controls, math.max(10, g.term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 0, 2 do
        fill_line(g.controls_y + i, g.term_w)
    end

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    for i = 1, #lines do
        local line = lines[i]
        draw_text(centered_x(line, 1, g.term_w), g.controls_y + offset + i - 1, line, "white", "black")
    end
end

-- 完整渲染
local function render_full(g)
    draw_header(g)
    draw_outer_frame(g.game_x, g.game_y, g.frame_w, g.frame_h)
    local inner_x, inner_y, inner_w = game_inner(g)
    fill_rect(inner_x, inner_y, inner_w, g.frame_h - 2, "black")
    draw_show_section(g)
    draw_input_section(g)
    draw_controls(g)
end

-- 只更新头部（用于时间刷新）
local function render_header_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_header(g)
end

-- 只更新显示区域（用于序列演示）
local function render_show_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_show_section(g)
end

-- 只更新输入区域（用于玩家输入）
local function render_input_only()
    if not ensure_terminal_size_ok() then
        return
    end
    if state.dirty then
        local g_full = frame_geometry()
        state.dirty = false
        render_full(g_full)
        return
    end
    local g = frame_geometry()
    draw_input_section(g)
end

-- 按需渲染
local function render_if_needed(force)
    if not ensure_terminal_size_ok() then
        return
    end
    if force or state.dirty then
        state.dirty = false
        local g = frame_geometry()
        render_full(g)
    end
end

-- 暂停并渲染（用于序列演示的间隔）
local function pause_with_render(ms)
    render_show_only()
    sleep(ms)
    advance_time(ms)
    render_header_only()
end

-- 生成随机颜色序列
local function generate_sequence(round_no)
    local out = {}
    for _ = 1, round_no do
        out[#out + 1] = random(4) + 1 -- 生成1-4的随机数
    end
    return out
end

-- 演示序列（阻塞式动画）
local function show_sequence_blocking()
    state.phase = "show"
    state.highlight_idx = 0
    render_show_only()
    pause_with_render(SHOW_OFF_MS)

    for i = 1, #state.sequence do
        state.highlight_idx = state.sequence[i]
        pause_with_render(SHOW_ON_MS)

        state.highlight_idx = 0
        pause_with_render(SHOW_OFF_MS)
    end

    flush_input_buffer()
    state.phase = "input"
    state.highlight_idx = 0
    render_show_only()
end

-- 开始下一回合
local function start_next_round()
    state.input_colors = {}
    render_input_only()
    state.sequence = generate_sequence(state.round)
    show_sequence_blocking()
end

-- 开始新游戏
local function start_new_run()
    state.score = 0
    state.round = 1
    state.sequence = {}
    state.input_colors = {}
    state.highlight_idx = 0

    state.phase = "input"
    state.lost = false
    state.confirm_mode = nil
    state.committed = false

    state.start_frame = state.frame
    state.end_frame = nil
    state.dirty = true

    start_next_round()
end

-- 标记游戏失败
local function mark_lost()
    if state.lost then
        return
    end
    state.lost = true
    state.phase = "lost"
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_stats_if_needed()
    state.dirty = true
end

-- 回合成功处理
local function on_round_success()
    state.score = state.score + state.round
    state.round = state.round + 1
    start_next_round()
end

-- 刷新脏标记（检查时间变化）
local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        render_header_only()
    end
end

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            commit_stats_if_needed()
            start_new_run()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_stats_if_needed()
            return "exit"
        end
    elseif key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

-- 主输入处理函数
local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    -- 确认模式
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 失败状态
    if state.lost then
        if key == "r" then
            start_new_run()
            return "changed"
        end
        if key == "q" or key == "esc" then
            commit_stats_if_needed()
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

    -- 非输入阶段不能操作
    if state.phase ~= "input" then
        return "none"
    end

    -- 退格/删除：移除上一个输入
    if key == "backspace" or key == "delete" then
        if #state.input_colors > 0 then
            table.remove(state.input_colors)
            render_input_only()
        end
        return "changed"
    end

    -- 数字键1-4：输入颜色
    local color_idx = nil
    if key == "1" then color_idx = 1 end
    if key == "2" then color_idx = 2 end
    if key == "3" then color_idx = 3 end
    if key == "4" then color_idx = 4 end

    if color_idx ~= nil then
        state.input_colors[#state.input_colors + 1] = color_idx
        render_input_only()
        return "changed"
    end

    -- 回车：提交答案
    if key == "enter" then
        local ok = #state.input_colors == #state.sequence
        if ok then
            for i = 1, #state.sequence do
                if state.input_colors[i] ~= state.sequence[i] then
                    ok = false
                    break
                end
            end
        end

        if not ok then
            mark_lost()
        else
            on_round_success()
        end
        return "changed"
    end

    return "none"
end

-- 游戏初始化
local function init_game()
    clear()
    flush_input_buffer()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    load_best_record()
    start_new_run()
end

-- 主游戏循环
local function game_loop()
    while state.running do
        local key = normalize_key(get_key(false))
        if not ensure_terminal_size_ok() then
            if key == "q" or key == "esc" then
                return
            end

            state.frame = state.frame + 1
            sleep(FRAME_MS)
            goto continue
        end

        local action = handle_input(key)
        if action == "exit" then
            return
        end

        refresh_dirty_flags()
        render_if_needed(false)

        state.frame = state.frame + 1
        sleep(FRAME_MS)

        ::continue::
    end
end

-- 启动游戏
init_game()
game_loop()
