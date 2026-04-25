local Constants = load_function("/constants.lua")

local FPS = Constants.FPS
local FRAME_MS = Constants.FRAME_MS
local MIN_COLS = Constants.MIN_COLS
local MAX_COLS = Constants.MAX_COLS
local MIN_ROWS = Constants.MIN_ROWS
local MAX_ROWS = Constants.MAX_ROWS
local MIN_MODE = Constants.MIN_MODE
local MAX_MODE = Constants.MAX_MODE
local TILE_PATH = Constants.TILE_PATH
local TILE_WALL = Constants.TILE_WALL
local TILE_DOOR = Constants.TILE_DOOR
local TILE_KEY = Constants.TILE_KEY
local TILE_EXIT = Constants.TILE_EXIT
local WALL_GLYPH = Constants.WALL_GLYPH
local DEFAULT_COLS = Constants.DEFAULT_COLS
local DEFAULT_ROWS = Constants.DEFAULT_ROWS
local DEFAULT_MODE = Constants.DEFAULT_MODE

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
    return random(n - 1)
end

-- 游戏状态表
local state = {
    -- 迷宫配置
    cols = DEFAULT_COLS,
    rows = DEFAULT_ROWS,
    mode = DEFAULT_MODE,
    grid = {},      -- 二维网格，存储瓷砖类型
    player_r = 2,   -- 玩家行位置
    player_c = 2,   -- 玩家列位置
    exit_r = 2,     -- 出口行位置
    exit_c = 2,     -- 出口列位置
    keys_held = 0,  -- 持有的钥匙数量
    door_total = 0, -- 门的总数
    steps = 0,      -- 已走步数

    -- 时间相关
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    time_limit_sec = nil, -- 时间限制（模式3/4）
    time_timer_id = nil,  -- 限时模式计时器 ID
    won = false,
    lost = false,

    -- 游戏状态
    launch_mode = "new",
    confirm_mode = nil,
    input_mode = nil,
    input_buffer = "",
    toast_text = nil,
    toast_until = 0,
    last_auto_save_sec = 0,
    dirty = true,
    last_elapsed_sec = -1,
    last_toast_visible = false,
    last_key = "",
    last_key_frame = -100,
    last_area = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    result_committed = false,

    -- 最佳记录
    best = {
        max_area = 0,
        max_cols = 0,
        max_rows = 0,
        max_mode = 1,
        min_time_sec = nil
    }
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
    text = string.gsub(text, "%[Q%]/%[ESC%]", key_label("quit_action"))
    text = string.gsub(text, "%[ESC%]/%[Q%]", key_label("quit_action"))
    return text
end

local function controls_text()
    return table.concat({
        key_label("move_up") .. "/" .. key_label("move_down") .. "/" .. key_label("move_left") .. "/" .. key_label("move_right") .. " " .. tr("game.maze_escape.action.move_up"),
        key_label("config_input") .. " " .. tr("game.maze_escape.action.config_input"),
        key_label("save") .. " " .. tr("game.maze_escape.action.save"),
        key_label("restart") .. " " .. tr("game.maze_escape.action.restart"),
        key_label("quit_action") .. " " .. tr("game.maze_escape.action.quit")
    }, "  ")
end

local function restart_quit_controls_text()
    return key_label("restart") .. " " .. tr("game.maze_escape.action.restart")
        .. "  " .. key_label("quit_action") .. " " .. tr("game.maze_escape.action.quit")
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

-- 数值限幅
local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

-- 规范化按键
local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then
        return string.lower(key)
    end
    return tostring(key):lower()
end

local function normalize_event_key(event)
    if type(event) ~= "table" then return "" end
    if event.type == "quit" then return "quit_action" end
    if event.type == "key" then return normalize_key(event.name) end
    if event.type ~= "action" then return "" end
    local map = {
        move_up = "move_up",
        move_down = "move_down",
        move_left = "move_left",
        move_right = "move_right",
        config_input = "config_input",
        save = "save",
        confirm = "confirm",
        restart = "restart",
        quit_action = "quit_action",
        confirm_yes = "confirm_yes",
        confirm_no = "confirm_no",
    }
    return map[event.name] or normalize_key(event.name)
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

-- 清空输入缓冲区
local function flush_input_buffer()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
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

-- 停止限时模式计时器
local function stop_time_timer()
    if state.time_timer_id ~= nil and type(timer_kill) == "function" then
        pcall(timer_kill, state.time_timer_id)
    end
    state.time_timer_id = nil
end

-- 启动限时模式计时器
local function start_time_timer(limit_sec, elapsed_sec)
    stop_time_timer()
    if limit_sec == nil or type(timer_create) ~= "function" or type(timer_start) ~= "function" then
        return
    end
    local remaining = math.max(1, math.floor((limit_sec - (elapsed_sec or 0)) * 1000))
    local ok, id = pcall(timer_create, remaining, "maze_escape_limit")
    if ok and type(id) == "string" then
        state.time_timer_id = id
        pcall(timer_start, id)
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

-- 格式化持续时间
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 判断是否为计时模式
local function timed_mode(mode)
    return mode == 3 or mode == 4
end

-- 判断是否为门/钥匙模式
local function door_mode(mode)
    return mode == 2 or mode == 4
end

-- 获取模式标签
local function mode_label(mode)
    if mode == 1 then
        return tr("game.maze_escape.mode1")
    end
    if mode == 2 then
        return tr("game.maze_escape.mode2")
    end
    if mode == 3 then
        return tr("game.maze_escape.mode3")
    end
    return tr("game.maze_escape.mode4")
end

-- 获取迷宫单元格显示宽度
local function maze_cell_width()
    local w = text_width(WALL_GLYPH)
    if w < 1 then
        return 1
    end
    return w
end

-- 填充单元格文本（确保固定宽度）
local function fit_cell_text(text, cell_w)
    local w = text_width(text)
    if w >= cell_w then
        return text
    end
    return text .. string.rep(" ", cell_w - w)
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

-- 清除上次渲染的区域
local function clear_last_area()
    if state.last_area == nil then
        return
    end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

-- 强制完全刷新
local function force_full_refresh()
    clear()
    state.last_area = nil
    state.dirty = true
end

-- 创建新矩阵
local function new_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do
            m[r][c] = value
        end
    end
    return m
end

-- 复制矩阵
local function copy_matrix(source, rows, cols)
    local out = new_matrix(rows, cols, 0)
    for r = 1, rows do
        for c = 1, cols do
            out[r][c] = source[r][c]
        end
    end
    return out
end

-- 加载最佳记录
local function load_best_record()
    local ok, data = pcall(get_best_score)
    if not ok or type(data) ~= "table" then
        return
    end
    local area = tonumber(data.max_area or data.area)
    local max_cols = tonumber(data.max_cols or data.cols)
    local max_rows = tonumber(data.max_rows or data.rows)
    local mode = tonumber(data.max_mode or data.mode)
    local min_time = tonumber(data.min_time_sec or data.time_sec)
    if area ~= nil and area >= 0 then state.best.max_area = math.floor(area) end
    if max_cols ~= nil and max_cols >= 0 then state.best.max_cols = math.floor(max_cols) end
    if max_rows ~= nil and max_rows >= 0 then state.best.max_rows = math.floor(max_rows) end
    if mode ~= nil and mode >= MIN_MODE and mode <= MAX_MODE then state.best.max_mode = math.floor(mode) end
    if min_time ~= nil and min_time >= 0 then state.best.min_time_sec = math.floor(min_time) end
end

-- 打乱数组（用于随机化方向）
local function shuffle_array(arr)
    for i = #arr, 2, -1 do
        local j = random_index(i) + 1
        arr[i], arr[j] = arr[j], arr[i]
    end
end

-- 检查是否在边界内
local function in_bounds(r, c)
    return r >= 1 and r <= state.rows and c >= 1 and c <= state.cols
end

-- 检查是否在内层（非边界）
local function in_inner(r, c)
    return r >= 2 and r <= state.rows - 1 and c >= 2 and c <= state.cols - 1
end

-- 递归挖迷宫（深度优先搜索）
local function carve_from(grid, r, c)
    grid[r][c] = TILE_PATH
    local dirs = {
        { dr = -2, dc = 0 },
        { dr = 2,  dc = 0 },
        { dr = 0,  dc = -2 },
        { dr = 0,  dc = 2 }
    }
    shuffle_array(dirs)

    for _, d in ipairs(dirs) do
        local nr = r + d.dr
        local nc = c + d.dc
        if in_inner(nr, nc) and grid[nr][nc] == TILE_WALL then
            local mr = r + math.floor(d.dr / 2)
            local mc = c + math.floor(d.dc / 2)
            grid[mr][mc] = TILE_PATH
            carve_from(grid, nr, nc)
        end
    end
end

-- 平滑偶数尺寸迷宫的边缘
local function smooth_even_inner_strips(grid)
    if state.rows % 2 == 0 then
        for c = 2, state.cols - 1 do
            grid[state.rows - 1][c] = TILE_PATH
        end
    end

    if state.cols % 2 == 0 then
        for r = 2, state.rows - 1 do
            grid[r][state.cols - 1] = TILE_PATH
        end
    end
end

-- BFS计算距离和父节点
local function bfs_with_parent(grid, rows, cols, sr, sc)
    local dist = new_matrix(rows, cols, -1)
    local prev_r = new_matrix(rows, cols, 0)
    local prev_c = new_matrix(rows, cols, 0)
    local q_r = { sr }
    local q_c = { sc }
    local head = 1

    dist[sr][sc] = 0
    local far_r, far_c = sr, sc

    while head <= #q_r do
        local r = q_r[head]
        local c = q_c[head]
        head = head + 1

        if dist[r][c] > dist[far_r][far_c] then
            far_r, far_c = r, c
        end

        for _, d in ipairs({ { -1, 0 }, { 1, 0 }, { 0, -1 }, { 0, 1 } }) do
            local nr = r + d[1]
            local nc = c + d[2]
            if nr >= 1 and nr <= rows and nc >= 1 and nc <= cols then
                if dist[nr][nc] < 0 and grid[nr][nc] ~= TILE_WALL then
                    dist[nr][nc] = dist[r][c] + 1
                    prev_r[nr][nc] = r
                    prev_c[nr][nc] = c
                    q_r[#q_r + 1] = nr
                    q_c[#q_c + 1] = nc
                end
            end
        end
    end

    return far_r, far_c, dist, prev_r, prev_c
end

-- 重建路径
local function reconstruct_path(prev_r, prev_c, sr, sc, tr, tc)
    local rev = {}
    local r, c = tr, tc
    while true do
        rev[#rev + 1] = { r = r, c = c }
        if r == sr and c == sc then
            break
        end
        local pr = prev_r[r][c]
        local pc = prev_c[r][c]
        if pr == 0 or pc == 0 then
            break
        end
        r, c = pr, pc
    end

    local path = {}
    for i = #rev, 1, -1 do
        path[#path + 1] = rev[i]
    end
    return path
end

-- 计算钥匙目标数量
local function key_target_count(path_len)
    if not door_mode(state.mode) then
        return 0
    end
    -- 基于面积的钥匙数
    local by_area = math.floor((state.rows * state.cols) / 100)
    if by_area < 1 then by_area = 1 end
    -- 基于路径长度的钥匙数（每3格最多1把钥匙）
    local max_by_path = math.floor((path_len - 2) / 3)
    if max_by_path < 1 then
        return 0
    end
    if by_area > max_by_path then
        by_area = max_by_path
    end
    return by_area
end

-- 放置门和钥匙
local function place_doors_and_keys(grid, path, key_count)
    if not door_mode(state.mode) then
        return 0
    end
    local n = #path
    if key_count <= 0 then
        return 0
    end

    -- 创建路径索引映射
    local path_index = new_matrix(state.rows, state.cols, 0)
    for i = 1, #path do
        local cell = path[i]
        path_index[cell.r][cell.c] = i
    end

    -- BFS计算距离
    local _, _, dist, prev_r, prev_c = bfs_with_parent(grid, state.rows, state.cols, path[1].r, path[1].c)
    local anchor_cache = new_matrix(state.rows, state.cols, -1)
    local used_key = new_matrix(state.rows, state.cols, false)

    -- 获取锚点索引（最近路径节点的索引）
    local function anchor_index_of(r, c)
        if anchor_cache[r][c] >= 0 then
            return anchor_cache[r][c]
        end
        local cr, cc = r, c
        local guard = state.rows * state.cols + 5
        while guard > 0 do
            local idx = path_index[cr][cc]
            if idx > 0 then
                anchor_cache[r][c] = idx
                return idx
            end
            local pr = prev_r[cr][cc]
            local pc = prev_c[cr][cc]
            if pr == 0 or pc == 0 then
                anchor_cache[r][c] = 0
                return 0
            end
            cr, cc = pr, pc
            guard = guard - 1
        end
        anchor_cache[r][c] = 0
        return 0
    end

    -- 选择离路径最远的非路径节点作为钥匙位置
    local function pick_off_path_key(prev_idx, door_idx)
        local best_r, best_c = nil, nil
        local best_dist = -1
        for r = 2, state.rows - 1 do
            for c = 2, state.cols - 1 do
                if grid[r][c] == TILE_PATH and path_index[r][c] == 0 and not used_key[r][c] then
                    local anchor = anchor_index_of(r, c)
                    if anchor >= (prev_idx + 1) and anchor <= (door_idx - 1) then
                        local d = dist[r][c]
                        if d > best_dist then
                            best_dist = d
                            best_r, best_c = r, c
                        end
                    end
                end
            end
        end
        return best_r, best_c
    end

    local prev_door_idx = 1
    local placed = 0

    -- 放置门和钥匙
    for i = 1, key_count do
        -- 计算门的位置
        local door_idx = math.floor(((n - 2) * i) / (key_count + 1)) + 1
        if door_idx <= prev_door_idx + 1 then
            door_idx = prev_door_idx + 2
        end
        if door_idx >= n then
            door_idx = n - 1
        end
        if door_idx <= prev_door_idx + 1 then
            break
        end

        local key_start = prev_door_idx + 1
        local key_end = door_idx - 1
        if key_start > key_end then
            break
        end

        -- 选择钥匙位置
        local key_r, key_c = pick_off_path_key(prev_door_idx, door_idx)
        local key_cell = nil
        if key_r ~= nil and key_c ~= nil then
            key_cell = { r = key_r, c = key_c }
        else
            local key_idx = math.floor((key_start + key_end) / 2)
            key_cell = path[key_idx]
        end
        local door_cell = path[door_idx]

        if key_cell ~= nil and door_cell ~= nil then
            if grid[key_cell.r][key_cell.c] == TILE_PATH and grid[door_cell.r][door_cell.c] == TILE_PATH then
                grid[key_cell.r][key_cell.c] = TILE_KEY
                used_key[key_cell.r][key_cell.c] = true
                grid[door_cell.r][door_cell.c] = TILE_DOOR
                placed = placed + 1
                prev_door_idx = door_idx
            end
        end
    end

    return placed
end

-- 构建迷宫
local function build_maze(cols, rows, mode)
    stop_time_timer()
    state.cols = clamp(cols, MIN_COLS, MAX_COLS)
    state.rows = clamp(rows, MIN_ROWS, MAX_ROWS)
    state.mode = clamp(mode, MIN_MODE, MAX_MODE)

    -- 确保尺寸为奇数（便于迷宫生成）
    if state.cols % 2 == 0 and state.cols > 3 then
        state.cols = state.cols - 1
    end
    if state.rows % 2 == 0 and state.rows > 3 then
        state.rows = state.rows - 1
    end

    -- 初始化全墙迷宫
    local grid = new_matrix(state.rows, state.cols, TILE_WALL)
    carve_from(grid, 2, 2)

    -- 找到最长路径作为主要路线
    local start_r, start_c = 2, 2
    local exit_r, exit_c, _, prev_r, prev_c = bfs_with_parent(
        grid,
        state.rows,
        state.cols,
        start_r,
        start_c
    )
    local path = reconstruct_path(prev_r, prev_c, start_r, start_c, exit_r, exit_c)

    -- 放置门和钥匙
    local key_count = key_target_count(#path)
    state.door_total = place_doors_and_keys(grid, path, key_count)
    grid[exit_r][exit_c] = TILE_EXIT

    -- 更新状态
    state.grid = grid
    state.player_r = start_r
    state.player_c = start_c
    state.exit_r = exit_r
    state.exit_c = exit_c
    state.keys_held = 0
    state.steps = 0
    state.won = false
    state.lost = false
    state.start_frame = state.frame
    state.end_frame = nil
    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.last_auto_save_sec = 0
    state.result_committed = false

    -- 设置时间限制（计时模式）
    if timed_mode(state.mode) then
        local shortest_cells = #path
        local raw_limit = shortest_cells * 0.5 + state.door_total * 3
        state.time_limit_sec = math.max(1, math.floor(raw_limit + 0.5))
    else
        state.time_limit_sec = nil
    end
    if state.time_limit_sec ~= nil then
        start_time_timer(state.time_limit_sec, 0)
    end

    state.last_area = nil
    state.dirty = true
    flush_input_buffer()
end

-- 解析数字输入
local function parse_numbers(input)
    local nums = {}
    for token in string.gmatch(input, "%d+") do
        nums[#nums + 1] = math.floor(tonumber(token) or 0)
    end
    return nums
end

-- 解析配置输入
local function parse_config_input()
    local nums = parse_numbers(state.input_buffer)
    if #nums < 1 or #nums > 3 then
        return nil
    end
    local rows = state.rows
    local cols = state.cols
    local mode = state.mode

    if #nums == 1 then
        mode = nums[1]
    elseif #nums >= 2 then
        rows = nums[1]
        cols = nums[2]
        if #nums == 3 then
            mode = nums[3]
        end

        local rows_ok = rows >= MIN_ROWS and rows <= MAX_ROWS
        local cols_ok = cols >= MIN_COLS and cols <= MAX_COLS
        if not (rows_ok and cols_ok) then
            return nil
        end
    end

    if mode < MIN_MODE or mode > MAX_MODE then return nil end
    return cols, rows, mode
end

-- 创建游戏快照
local function make_snapshot()
    return {
        cols = state.cols,
        rows = state.rows,
        mode = state.mode,
        grid = copy_matrix(state.grid, state.rows, state.cols),
        player_r = state.player_r,
        player_c = state.player_c,
        exit_r = state.exit_r,
        exit_c = state.exit_c,
        keys_held = state.keys_held,
        door_total = state.door_total,
        steps = state.steps,
        time_limit_sec = state.time_limit_sec,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        lost = state.lost,
        last_auto_save_sec = state.last_auto_save_sec
    }
end

-- 保存游戏状态
local function save_game_state(show_toast)
    local ok = false
    if type(request_save_game) == "function" then
        local s, ret = pcall(request_save_game)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.maze_escape.save_success" or "game.maze_escape.save_unavailable"
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

    local cols = tonumber(snapshot.cols)
    local rows = tonumber(snapshot.rows)
    local mode = tonumber(snapshot.mode)
    if cols == nil or rows == nil or mode == nil then
        return false
    end
    cols = clamp(math.floor(cols), MIN_COLS, MAX_COLS)
    rows = clamp(math.floor(rows), MIN_ROWS, MAX_ROWS)
    mode = clamp(math.floor(mode), MIN_MODE, MAX_MODE)

    if type(snapshot.grid) ~= "table" then
        return false
    end

    local grid = new_matrix(rows, cols, TILE_WALL)
    for r = 1, rows do
        if type(snapshot.grid[r]) ~= "table" then
            return false
        end
        for c = 1, cols do
            local v = math.floor(tonumber(snapshot.grid[r][c]) or TILE_WALL)
            if v < TILE_PATH or v > TILE_EXIT then
                v = TILE_WALL
            end
            grid[r][c] = v
        end
    end

    state.cols = cols
    state.rows = rows
    state.mode = mode
    state.grid = grid
    state.player_r = clamp(math.floor(tonumber(snapshot.player_r) or 2), 1, rows)
    state.player_c = clamp(math.floor(tonumber(snapshot.player_c) or 2), 1, cols)
    state.exit_r = clamp(math.floor(tonumber(snapshot.exit_r) or 2), 1, rows)
    state.exit_c = clamp(math.floor(tonumber(snapshot.exit_c) or 2), 1, cols)
    state.keys_held = math.max(0, math.floor(tonumber(snapshot.keys_held) or 0))
    state.door_total = math.max(0, math.floor(tonumber(snapshot.door_total) or 0))
    state.steps = math.max(0, math.floor(tonumber(snapshot.steps) or 0))
    state.time_limit_sec = tonumber(snapshot.time_limit_sec)
    if state.time_limit_sec ~= nil then
        state.time_limit_sec = math.max(1, math.floor(state.time_limit_sec))
    end

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = math.max(0, math.floor(tonumber(snapshot.last_auto_save_sec) or elapsed))
    state.won = not not snapshot.won
    state.lost = not not snapshot.lost
    state.end_frame = nil
    if state.won or state.lost then
        state.end_frame = state.frame
    end

    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.result_committed = state.won or state.lost
    if state.time_limit_sec ~= nil and not state.won and not state.lost then
        start_time_timer(state.time_limit_sec, elapsed)
    else
        stop_time_timer()
    end
    state.last_area = nil
    state.dirty = true
    return true
end

-- 加载游戏状态
local function load_game_state(snapshot)
    return restore_snapshot(snapshot)
end

-- 提交游戏结果
local function commit_result_if_needed()
    if state.result_committed then
        return
    end
    local duration = elapsed_seconds()
    local changed = false
    if state.won then
        local area = state.rows * state.cols
        if area > state.best.max_area then
            state.best.max_area = area
            state.best.max_cols = state.cols
            state.best.max_rows = state.rows
            changed = true
        end
        if state.mode > state.best.max_mode then
            state.best.max_mode = state.mode
            changed = true
        end
        if state.best.min_time_sec == nil or duration < state.best.min_time_sec then
            state.best.min_time_sec = duration
            changed = true
        end
    end
    state.result_committed = true
    if changed and type(request_save_best_score) == "function" then
        pcall(request_save_best_score)
    end
end

-- 计算棋盘几何布局
local function board_geometry()
    local term_w, term_h = terminal_size()
    local controls_text = controls_text()
    local controls_w = min_width_for_lines(controls_text, 3, 28)

    local status_left = tr("game.maze_escape.time") .. " 00:00:00"
    local status_mid = tr("game.maze_escape.steps") .. " 9999"
    local status_right = tr("game.maze_escape.keys") .. " 99"
    local status_w = text_width(status_left) + 2 + text_width(status_mid) + 2 + text_width(status_right)

    local mode_text = tr("game.maze_escape.mode") .. ": " .. mode_label(state.mode)
    local timer_text = tr("game.maze_escape.remaining") .. " 00:00:00"
    local mode_line_w = text_width(mode_text) + 2 + text_width(timer_text)

    local message_w = math.max(
        text_width(tr("game.maze_escape.win_banner")),
        text_width(tr("game.maze_escape.lose_banner")),
        text_width(tr("game.maze_escape.input_config_hint")),
        text_width(replace_prompt_keys(tr("game.maze_escape.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.maze_escape.confirm_exit")))
    )

    local board_w = state.cols * maze_cell_width()
    local content_w = math.max(board_w, controls_w, status_w, mode_line_w, message_w)

    local top_lines = 3
    local control_lines = 3
    local total_h = top_lines + state.rows + control_lines
    local y = math.floor((term_h - total_h) / 2) + 1
    if y < 2 then y = 2 end
    local x = math.floor((term_w - content_w) / 2) + 1
    if x < 1 then x = 1 end

    return x, y, content_w
end

-- 计算文本居中位置
local function centered_x(line, area_x, area_w)
    local x = area_x + math.floor((area_w - text_width(line)) / 2)
    if x < area_x then x = area_x end
    return x
end

-- 绘制状态栏
local function draw_status(x, y, w)
    local elapsed = elapsed_seconds()
    local left = tr("game.maze_escape.time") .. " " .. format_duration(elapsed)
    local mid = tr("game.maze_escape.steps") .. " " .. tostring(state.steps)
    local right = tr("game.maze_escape.keys") .. " " .. tostring(state.keys_held)

    -- 清空状态区域
    draw_text(x, y, string.rep(" ", w), "white", "black")
    draw_text(x, y + 1, string.rep(" ", w), "white", "black")
    draw_text(x, y + 2, string.rep(" ", w), "white", "black")

    -- 计算位置（避免重叠）
    local left_x = x
    local mid_x = centered_x(mid, x, w)
    local right_x = x + w - text_width(right)
    if mid_x < left_x + text_width(left) + 1 then
        mid_x = left_x + text_width(left) + 1
    end
    if right_x <= mid_x + text_width(mid) then
        right_x = mid_x + text_width(mid) + 1
    end

    -- 绘制时间、步数、钥匙数
    draw_text(left_x, y, left, "light_cyan", "black")
    draw_text(mid_x, y, mid, "light_cyan", "black")
    draw_text(right_x, y, right, "light_cyan", "black")

    -- 绘制模式和计时信息
    local mode_text = tr("game.maze_escape.mode") .. ": " .. mode_label(state.mode)
    local timer_text = tr("game.maze_escape.timer")
    if state.time_limit_sec ~= nil then
        local remain = math.max(0, state.time_limit_sec - elapsed)
        timer_text = tr("game.maze_escape.remaining") .. ": " .. format_duration(remain)
    end
    local mode_line = mode_text .. "  " .. timer_text
    draw_text(centered_x(mode_line, x, w), y + 1, mode_line, "dark_gray", "black")

    -- 绘制提示信息
    if state.input_mode == "config" then
        if state.input_buffer == "" then
            local hint = tr("game.maze_escape.input_config_hint")
            draw_text(centered_x(hint, x, w), y + 2, hint, "dark_gray", "black")
        else
            draw_text(centered_x(state.input_buffer, x, w), y + 2, state.input_buffer, "white", "black")
        end
    elseif state.won then
        local line = tr("game.maze_escape.win_banner") .. " " .. restart_quit_controls_text()
        draw_text(centered_x(line, x, w), y + 2, line, "green", "black")
    elseif state.lost then
        local line = tr("game.maze_escape.lose_banner") .. " " .. restart_quit_controls_text()
        draw_text(centered_x(line, x, w), y + 2, line, "red", "black")
    elseif state.confirm_mode == "restart" then
        local line = replace_prompt_keys(tr("game.maze_escape.confirm_restart"))
        draw_text(centered_x(line, x, w), y + 2, line, "yellow", "black")
    elseif state.confirm_mode == "exit" then
        local line = replace_prompt_keys(tr("game.maze_escape.confirm_exit"))
        draw_text(centered_x(line, x, w), y + 2, line, "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(centered_x(state.toast_text, x, w), y + 2, state.toast_text, "green", "black")
    end
end

-- 绘制迷宫
local function draw_maze(x, y, w)
    local cell_w = maze_cell_width()
    local board_w = state.cols * cell_w
    local start_x = x + math.floor((w - board_w) / 2)
    if start_x < x then start_x = x end

    for r = 1, state.rows do
        for c = 1, state.cols do
            local tile = state.grid[r][c]
            local ch = " "
            local fg = "white"
            if tile == TILE_WALL then
                ch = WALL_GLYPH
                fg = "rgb(180,180,180)"
            elseif tile == TILE_DOOR then
                ch = "%"
                fg = "rgb(255,190,80)"
            elseif tile == TILE_KEY then
                ch = "*"
                fg = "rgb(120,255,120)"
            elseif tile == TILE_EXIT then
                ch = "&"
                fg = "light_cyan"
            end
            if r == state.player_r and c == state.player_c then
                ch = "@"
                fg = "yellow"
            end
            local draw_x = start_x + (c - 1) * cell_w
            draw_text(draw_x, y + r - 1, fit_cell_text(ch, cell_w), fg, "black")
        end
    end
end

-- 绘制控制说明
local function draw_controls(x, y, w)
    local text = controls_text()
    local term_w = terminal_size()
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(text, max_w)
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
        local line_x = math.floor((term_w - text_width(line)) / 2)
        if line_x < 1 then line_x = 1 end
        draw_text(line_x, y + offset + i - 1, line, "white", "black")
    end
end

-- 主渲染函数
local function runtime_render(state_arg)
    state = state_arg or state
    local x, y, w = board_geometry()
    local total_h = 3 + state.rows + 3
    local area = { x = x, y = y, w = w, h = total_h }

    -- 如果渲染区域变化，清除旧区域
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y
        or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    -- 绘制各组件
    draw_status(x, y, w)
    draw_maze(x, y + 3, w)
    draw_controls(x, y + 3 + state.rows, w)
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local controls_text = controls_text()
    local controls_w = min_width_for_lines(controls_text, 3, 28)

    local status_left = tr("game.maze_escape.time") .. " 00:00:00"
    local status_mid = tr("game.maze_escape.steps") .. " 9999"
    local status_right = tr("game.maze_escape.keys") .. " 99"
    local status_w = text_width(status_left) + 2 + text_width(status_mid) + 2 + text_width(status_right)

    local hint_w = math.max(
        text_width(tr("game.maze_escape.input_config_hint")),
        text_width(tr("game.maze_escape.win_banner") .. " " .. restart_quit_controls_text()),
        text_width(tr("game.maze_escape.lose_banner") .. " " .. restart_quit_controls_text()),
        text_width(replace_prompt_keys(tr("game.maze_escape.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.maze_escape.confirm_exit")))
    )

    local board_w = state.cols * maze_cell_width()
    local min_w = math.max(board_w, controls_w, status_w, hint_w) + 2
    local min_h = 3 + state.rows + 3 + 2
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
        local px = math.floor((term_w - text_width(line)) / 2)
        if px < 1 then px = 1 end
        draw_text(px, top + i - 1, line, "white", "black")
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

-- 同步终端尺寸变化
local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        force_full_refresh()
    end
end

-- 防抖处理
local function should_debounce(key)
    if not (key == "move_up" or key == "move_down" or key == "move_left" or key == "move_right") then
        return false
    end
    if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
        return true
    end
    state.last_key = key
    state.last_key_frame = state.frame
    return false
end

-- 标记胜利
local function mark_win()
    stop_time_timer()
    state.won = true
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_result_if_needed()
    state.dirty = true
end

-- 标记失败
local function mark_lost()
    stop_time_timer()
    state.lost = true
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_result_if_needed()
    state.dirty = true
end

-- 检查是否超时
local function timed_out()
    if state.time_limit_sec == nil then
        return false
    end
    if state.time_timer_id ~= nil and type(is_timer_completed) == "function" then
        local ok, done = pcall(is_timer_completed, state.time_timer_id)
        if ok and done then
            return true
        end
    end
    return elapsed_seconds() >= state.time_limit_sec
end

-- 移动玩家
local function move_player(dr, dc)
    local nr = state.player_r + dr
    local nc = state.player_c + dc
    if not in_bounds(nr, nc) then
        return "none"
    end

    local tile = state.grid[nr][nc]
    if tile == TILE_WALL then
        return "none"
    end
    if tile == TILE_DOOR then
        if state.keys_held <= 0 then
            return "none"
        end
        state.keys_held = state.keys_held - 1
        state.grid[nr][nc] = TILE_PATH
    elseif tile == TILE_KEY then
        state.keys_held = state.keys_held + 1
        state.grid[nr][nc] = TILE_PATH
    end

    state.player_r = nr
    state.player_c = nc
    state.steps = state.steps + 1
    if nr == state.exit_r and nc == state.exit_c then
        mark_win()
    else
        state.dirty = true
    end
    return "changed"
end

-- 进入输入模式
local function start_input_mode(mode)
    state.input_mode = mode
    state.input_buffer = ""
    state.dirty = true
    flush_input_buffer()
end

-- 处理输入模式下的按键
local function handle_input_mode_key(key)
    if key == "quit_action" or key == "confirm_no" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "changed"
    end

    if key == "confirm" then
        local cols, rows, mode = parse_config_input()
        state.input_mode = nil
        state.input_buffer = ""
        if cols ~= nil then
            build_maze(cols, rows, mode)
            force_full_refresh()
        else
            state.toast_text = tr("game.maze_escape.input_config_invalid")
            state.toast_until = state.frame + 2 * FPS
            state.dirty = true
        end
        flush_input_buffer()
        return "changed"
    end

    if key == "backspace" or key == "del" then
        if #state.input_buffer > 0 then
            state.input_buffer = string.sub(state.input_buffer, 1, #state.input_buffer - 1)
            state.dirty = true
            return "changed"
        end
        return "none"
    end

    -- 这些键会退出输入模式
    if key == "move_up" or key == "move_down" or key == "move_left" or key == "move_right"
        or key == "restart" or key == "save" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "pass"
    end

    -- 数字和空格输入
    if key:match("^%d$") or key == "space" then
        local token = key
        if key == "space" then token = " " end
        if #state.input_buffer < 14 then
            state.input_buffer = state.input_buffer .. token
            state.dirty = true
            return "changed"
        end
    end

    return "none"
end

-- 处理确认模式下的按键
local function handle_confirm_key(key)
    if key == "confirm_yes" then
        if state.confirm_mode == "restart" then
            build_maze(state.cols, state.rows, state.mode)
            force_full_refresh()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            return "exit"
        end
    end

    if key == "confirm_no" or key == "quit_action" then
        state.confirm_mode = nil
        state.dirty = true
        flush_input_buffer()
        return "changed"
    end
    return "none"
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
        local input_action = handle_input_mode_key(key)
        if input_action ~= "pass" then
            return input_action
        end
    end

    -- 确认模式
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 胜利/失败状态
    if state.won or state.lost then
        if key == "restart" then
            build_maze(state.cols, state.rows, state.mode)
            force_full_refresh()
            return "changed"
        end
        if key == "confirm_no" or key == "quit_action" then
            return "exit"
        end
        return "none"
    end

    -- 功能键
    if key == "restart" then
        state.confirm_mode = "restart"
        state.dirty = true
        flush_input_buffer()
        return "changed"
    end
    if key == "confirm_no" or key == "quit_action" then
        state.confirm_mode = "exit"
        state.dirty = true
        flush_input_buffer()
        return "changed"
    end
    if key == "save" then
        save_game_state(true)
        return "changed"
    end
    if key == "config_input" then
        start_input_mode("config")
        return "changed"
    end

    -- 移动
    if key == "move_up" then return move_player(-1, 0) end
    if key == "move_down" then return move_player(1, 0) end
    if key == "move_left" then return move_player(0, -1) end
    if key == "move_right" then return move_player(0, 1) end
    return "none"
end

-- 自动保存
local function auto_save_if_needed()
    if state.won or state.lost then
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

    -- 检查超时
    if not state.won and not state.lost and timed_out() then
        mark_lost()
    end
end

-- 游戏初始化
local function runtime_init_game(saved_state)
    clear()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h
    state.launch_mode = read_launch_mode()
    load_best_record()
    if state.launch_mode == "continue" and type(saved_state) == "table" then
        if not load_game_state(saved_state) then
            build_maze(DEFAULT_COLS, DEFAULT_ROWS, DEFAULT_MODE)
        end
    else
        build_maze(DEFAULT_COLS, DEFAULT_ROWS, DEFAULT_MODE)
    end
    flush_input_buffer()
    return state
end

-- 主游戏循环

local function runtime_handle_event(state_arg, event)
    state = state_arg or state
    event = event or {}
    state.frame = state.frame + 1

    local key = normalize_event_key(event)

    if event.type == "resize" then
        sync_terminal_resize()
        refresh_dirty_flags()
        return state
    end

    if event.type == "tick" then
        if ensure_terminal_size_ok() then
            sync_terminal_resize()
            auto_save_if_needed()
            refresh_dirty_flags()
        end
        return state
    end

    if not ensure_terminal_size_ok() then
        if key == "confirm_no" or key == "quit_action" then
            if type(request_exit) == "function" then pcall(request_exit) end
        end
        return state
    end

    local action = handle_input(key)
    if action == "exit" then
        if type(request_exit) == "function" then pcall(request_exit) end
    end

    sync_terminal_resize()
    auto_save_if_needed()
    refresh_dirty_flags()
    return state
end

local function runtime_save_best_score(state_arg)
    state = state_arg or state
    if state.best.max_area <= 0 then
        return { best_string = "game.maze_escape.best_none_block" }
    end
    return {
        best_string = "game.maze_escape.best_block",
        max_area = state.best.max_area,
        max_cols = state.best.max_cols,
        max_rows = state.best.max_rows,
        max_mode = state.best.max_mode,
        min_time_sec = state.best.min_time_sec,
        size = string.format("%dx%d", state.best.max_cols, state.best.max_rows),
        mode = state.best.max_mode,
        fastest = state.best.min_time_sec and format_duration(state.best.min_time_sec) or "--:--:--"
    }
end

local function runtime_save_game(state_arg)
    state = state_arg or state
    return make_snapshot()
end

local function runtime_exit_game(state_arg)
    state = state_arg or state
    if not state.won and not state.lost then
        save_game_state(false)
    end
    stop_time_timer()
    return state
end

local Runtime = {
    init_game = runtime_init_game,
    handle_event = runtime_handle_event,
    render = runtime_render,
    exit_game = runtime_exit_game,
    save_best_score = runtime_save_best_score,
    save_game = runtime_save_game,
}

_G.MAZE_ESCAPE_RUNTIME = Runtime
return Runtime
