-- 帧率控制
local FPS = 60
local FRAME_MS = 1000 / FPS
local MAX_LEVEL = 20            -- 最大关卡数
local EXTRA_LIFE_SCORE = 100000 -- 额外生命所需分数

-- 速度参数，单位为“秒/格”或倍率，便于后续微调。
local PLAYER_STEP_SECONDS = 0.4
local GHOST_BASE_SPEED_MULTIPLIER = 1.0         -- 普通幽灵速度倍率，与玩家一致
local BLINKY_CHASE_SPEED_MULTIPLIER = 1.15      -- Blinky 追击模式速度倍率
local FRIGHTENED_SPEED_MULTIPLIER = 0.75        -- 惊吓模式速度倍率，四个幽灵统一变为原速度的75%
local EYES_STEP_SECONDS = 0.25                  -- 被吃后眼睛回家速度，独立于普通/追击/惊吓模式

-- 地图元素字符
local PELLET_CHAR = "·" -- 普通豆子
local POWER_CHAR = "*"  -- 能量豆
local DOOR_CHAR = "-"   -- 幽灵房门

-- 地图模板（ASCII艺术）
local MAP_TEMPLATE = {
    "╔═════════╦═════════╗",
    "║·········║·········║",
    "║·╔═╗·╔═╗·║·╔═╗·╔═╗·║",
    "║*║ ║·║ ║·║·║ ║·║ ║*║",
    "║·╚═╝·╚═╝·║·╚═╝·╚═╝·║",
    "║···················║",
    "║·╔═╗·║·╔═══╗·║·╔═╗·║",
    "║·╚═╝·║·╚═╦═╝·║·╚═╝·║",
    "║·····║···║···║·····║",
    "╚═══╗·╠══ ║ ══╣·╔═══╝",
    "    ║·║       ║·║    ",
    "════╝·║ ╔═-═╗ ║·╚════",
    "<    ·  ║   ║  ·    >",
    "════╗·║ ╚═══╝ ║·╔════",
    "    ║·║       ║·║    ",
    "    ║·║ ╔═══╗ ║·║    ",
    "╔═══╝·║ ╚═╦═╝ ║·╚═══╗",
    "║·········║·········║",
    "║·══╗·═══·║·═══·╔══·║",
    "║*··║···········║··*║",
    "╠═╗·║·║·╔═══╗·║·║·╔═╣",
    "╠═╝·║·║·╚═╦═╝·║·║·╚═╣",
    "║·····║···║···║·····║",
    "║·════╩══·║·══╩════·║",
    "║···················║",
    "╚═══════════════════╝"
}

-- 墙壁字符集合
local WALL_SET = {
    ["╔"] = true,
    ["╗"] = true,
    ["╚"] = true,
    ["╝"] = true,
    ["═"] = true,
    ["║"] = true,
    ["╦"] = true,
    ["╩"] = true,
    ["╠"] = true,
    ["╣"] = true,
}

-- 方向定义
local DIRS = {
    { name = "up",    dr = -1, dc = 0 },
    { name = "left",  dr = 0,  dc = -1 },
    { name = "down",  dr = 1,  dc = 0 },
    { name = "right", dr = 0,  dc = 1 },
}

-- 水果表（按关卡）
local FRUIT_TABLE = {
    { symbol = "%", points = 100,  key = "game.pacman.fruit.cherry",     fallback = "Cherry" },
    { symbol = "U", points = 300,  key = "game.pacman.fruit.strawberry", fallback = "Strawberry" },
    { symbol = "O", points = 500,  key = "game.pacman.fruit.orange",     fallback = "Orange" },
    { symbol = "O", points = 500,  key = "game.pacman.fruit.orange",     fallback = "Orange" },
    { symbol = "Q", points = 700,  key = "game.pacman.fruit.apple",      fallback = "Apple" },
    { symbol = "Q", points = 700,  key = "game.pacman.fruit.apple",      fallback = "Apple" },
    { symbol = "§", points = 1000, key = "game.pacman.fruit.grape",      fallback = "Grape" },
    { symbol = "§", points = 1000, key = "game.pacman.fruit.grape",      fallback = "Grape" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian",   fallback = "Galaxian" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian",   fallback = "Galaxian" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell",       fallback = "Bell" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell",       fallback = "Bell" },
}

-- 游戏状态表
local state = {
    -- 地图数据
    rows = 0,
    cols = 0,
    base_map = {},         -- 基础地图（墙壁、门等）
    pellets = {},          -- 豆子状态
    total_pellets = 0,     -- 总豆子数
    remaining_pellets = 0, -- 剩余豆子数

    -- 玩家
    player = { r = 1, c = 1, dir = "left", next_dir = "left", next_step_at = 0 },
    player_start = { r = 1, c = 1 },

    -- 幽灵
    ghosts = {},
    ghost_spawn = { r = 1, c = 1 },
    global_pause_until = 0, -- 全局暂停直到该帧

    -- 隧道
    tunnel_left = nil,
    tunnel_right = nil,
    door_cells = {}, -- 幽灵房门位置

    -- 水果
    fruit = { active = false, spawned = false, r = 1, c = 1 },

    -- 游戏进度
    level = 1,
    score = 0,
    best_score = 0,
    lives = 3,
    extra_life_granted = false,

    -- 能量状态
    power_until = 0,
    power_chain = 0,  -- 连续吃幽灵计数
    power_eaten = {}, -- 各幽灵是否已被吃

    -- 游戏阶段
    phase = "playing", -- playing/won/lost
    frame = 0,
    run_start_frame = 0,
    level_start_frame = 0,
    end_frame = nil,
    stats_committed = false,
    confirm_mode = nil,

    -- 倒计时
    countdown_until = 0,
    last_countdown_sec = nil,

    -- 提示信息
    info_message = "",
    info_color = "dark_gray",
    info_message_until = nil,

    -- 收集的水果记录
    collected_fruits = {},

    -- 渲染相关
    dirty = true,
    last_area = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,

    countdown_timer_id = nil,
    power_timer_id = nil,
    info_timer_id = nil,
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
        if ok and type(w) == "number" then return w end
    end
    return #text
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
    back_tab = "BTab",
}

local function display_key_name(key)
    key = tostring(key or "")
    if key == "" then return "" end
    local mapped = KEY_DISPLAY[key]
    if mapped ~= nil then return mapped end
    if #key == 1 then return string.upper(key) end
    if string.sub(key, 1, 1) == "f" and tonumber(string.sub(key, 2)) ~= nil then
        return string.upper(key)
    end
    return key
end

local function key_label(action)
    if type(get_key) ~= "function" then return "[]" end
    local ok, info = pcall(get_key, action)
    if not ok or type(info) ~= "table" then return "[]" end
    if type(info[action]) == "table" then info = info[action] end
    local keys = info.key_user or info.key
    if type(keys) ~= "table" then keys = { keys } end
    local parts = {}
    for i = 1, #keys do
        local label = display_key_name(keys[i])
        if label ~= "" then parts[#parts + 1] = "[" .. label .. "]" end
    end
    if #parts == 0 then return "[]" end
    return table.concat(parts, "/")
end

local function replace_prompt_keys(text)
    text = tostring(text or "")
    text = string.gsub(text, "%[Y%]", key_label("confirm_yes"))
    text = string.gsub(text, "%[N%]", key_label("confirm_no"))
    text = string.gsub(text, "%[Q%]/%[ESC%]", key_label("quit_action"))
    text = string.gsub(text, "%[ESC%]/%[Q%]", key_label("quit_action"))
    text = string.gsub(text, "%[R%]", key_label("restart"))
    return text
end

local function controls_text()
    return table.concat({
        key_label("move_up") .. "/" .. key_label("move_down") .. "/" .. key_label("move_left") .. "/" .. key_label("move_right") .. " " .. tr("game.pacman.action.move_up"),
        key_label("restart") .. " " .. tr("game.pacman.action.restart"),
        key_label("quit_action") .. " " .. tr("game.pacman.action.quit"),
    }, "  ")
end

local function restart_quit_controls_text()
    return key_label("restart") .. " " .. tr("game.pacman.action.restart")
        .. "  " .. key_label("quit_action") .. " " .. tr("game.pacman.action.quit")
end

local function random_index(count)
    count = math.floor(tonumber(count) or 0)
    if count <= 1 then return 1 end
    local ok, value = pcall(_G.random, count - 1)
    if ok and type(value) == "number" then
        return math.floor(value) + 1
    end
    return 1
end

local function kill_timer_field(field)
    local id = state[field]
    if id ~= nil and type(timer_kill) == "function" then
        pcall(timer_kill, id)
    end
    state[field] = nil
end

local function start_timer_field(field, duration_ms, note)
    kill_timer_field(field)
    if type(timer_create) ~= "function" or type(timer_start) ~= "function" then
        return nil
    end
    local ok, id = pcall(timer_create, math.max(1, math.floor(duration_ms or 1)), note or field)
    if ok and type(id) == "string" then
        state[field] = id
        pcall(timer_start, id)
        return id
    end
    return nil
end

local function timer_remaining_ms(field)
    local id = state[field]
    if id ~= nil and type(get_timer_remaining) == "function" then
        local ok, remaining = pcall(get_timer_remaining, id)
        if ok and type(remaining) == "number" then
            return math.max(0, remaining)
        end
    end
    return nil
end

local function timer_completed(field)
    local id = state[field]
    if id ~= nil and type(is_timer_completed) == "function" then
        local ok, done = pcall(is_timer_completed, id)
        if ok then return done end
    end
    return false
end

-- 规范化按键
local function request_exit_game()
    kill_timer_field("countdown_timer_id")
    kill_timer_field("power_timer_id")
    kill_timer_field("info_timer_id")
    if type(request_exit) == "function" then
        pcall(request_exit)
    end
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    if type(key) == "table" then
        if key.type == "quit" then return "quit_action" end
        if key.type == "key" and type(key.name) == "string" then return string.lower(key.name) end
        if key.type == "action" and type(key.name) == "string" then return string.lower(key.name) end
    end
    return tostring(key):lower()
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

local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

-- 按单词换行
local function wrap_words(text, max_width)
    if max_width <= 1 then return { text } end
    local lines, current, had = {}, "", false
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
    if not had then return { "" } end
    if current ~= "" then lines[#lines + 1] = current end
    return lines
end

-- 计算最小宽度
local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then return width end
        width = width + 1
    end
    return full
end

-- 数值限幅
local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

local function seconds_to_frames(seconds)
    return math.max(1, math.floor(seconds * FPS + 0.5))
end

local function step_frames_for_speed_multiplier(multiplier)
    local speed = multiplier or 1.0
    if speed <= 0 then speed = 1.0 end
    return seconds_to_frames(PLAYER_STEP_SECONDS / speed)
end

local function frames_from_dt(dt_ms)
    local ms = tonumber(dt_ms) or FRAME_MS
    if ms < 0 then ms = 0 end
    return ms / FRAME_MS
end

-- 计算已过秒数
local function elapsed_seconds()
    local ending = state.end_frame
    if ending == nil then ending = state.frame end
    return math.floor((ending - state.run_start_frame) / FPS)
end

-- 格式化持续时间
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 将字符串转为UTF-8字符数组
local function utf8_chars(str)
    local out = {}
    for _, code in utf8.codes(str) do out[#out + 1] = utf8.char(code) end
    return out
end

-- 创建空白矩阵
local function blank_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do m[r][c] = value end
    end
    return m
end

-- 解析地图模板
local function parse_map()
    local lines, max_cols = {}, 0
    for i = 1, #MAP_TEMPLATE do
        lines[i] = utf8_chars(MAP_TEMPLATE[i])
        if #lines[i] > max_cols then max_cols = #lines[i] end
    end

    state.rows, state.cols = #lines, max_cols
    state.base_map = blank_matrix(state.rows, state.cols, " ")
    state.pellets = blank_matrix(state.rows, state.cols, " ")
    state.door_cells, state.tunnel_left, state.tunnel_right = {}, nil, nil

    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch = lines[r][c] or " "
            state.base_map[r][c] = ch
            state.pellets[r][c] = (ch == PELLET_CHAR or ch == POWER_CHAR) and ch or " "
            if ch == DOOR_CHAR then
                state.door_cells[#state.door_cells + 1] = { r = r, c = c }
            elseif ch == "<" then
                state.tunnel_left = { r = r, c = c }
            elseif ch == ">" then
                state.tunnel_right = { r = r, c = c }
            end
        end
    end
end

-- 边界检查
local function in_bounds(r, c)
    return r >= 1 and r <= state.rows and c >= 1 and c <= state.cols
end

-- 是否为墙壁
local function is_wall(r, c)
    return (not in_bounds(r, c)) or WALL_SET[state.base_map[r][c]] == true
end

-- 是否为房门
local function is_door(r, c)
    return in_bounds(r, c) and state.base_map[r][c] == DOOR_CHAR
end

-- 是否可行走（非墙壁）
local function is_walkable(r, c)
    return in_bounds(r, c) and (not is_wall(r, c))
end

-- 应用隧道传送
local function apply_tunnel(r, c)
    if state.tunnel_left and state.tunnel_right then
        if r == state.tunnel_left.r and c < 1 then
            return state.tunnel_right.r, state.tunnel_right.c
        end
        if r == state.tunnel_right.r and c > state.cols then
            return state.tunnel_left.r, state.tunnel_left.c
        end
    end
    return r, c
end

-- 判断玩家是否可以移动到指定位置
local function can_move_player(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) or is_door(r, c) then
        return false, r, c
    end
    return true, r, c
end

-- 判断幽灵是否可以移动到指定位置
local function can_move_ghost(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) then
        return false, r, c
    end
    return true, r, c
end

-- 获取方向对应的行列变化
local function direction_delta(dir)
    if dir == "up" then return -1, 0 end
    if dir == "down" then return 1, 0 end
    if dir == "left" then return 0, -1 end
    return 0, 1
end

-- 获取相反方向
local function opposite_dir(dir)
    if dir == "up" then return "down" end
    if dir == "down" then return "up" end
    if dir == "left" then return "right" end
    return "left"
end

-- 统计豆子总数
local function pellet_counts()
    local total = 0
    for r = 1, state.rows do
        for c = 1, state.cols do
            if state.pellets[r][c] == PELLET_CHAR or state.pellets[r][c] == POWER_CHAR then
                total = total + 1
            end
        end
    end
    return total
end

-- 获取能量持续时间（秒）
local function power_duration_sec(level)
    if level <= 1 then
        return 6
    elseif level == 2 then
        return 5
    elseif level <= 10 then
        return 4
    elseif level <= 16 then
        return 3
    else
        return 2
    end
end

-- 获取幽灵复活时间（秒）
local function ghost_revive_sec(level)
    if level >= 11 and level <= 16 then
        return 5
    end
    return 3
end

-- 获取当前关卡的水果
local function fruit_for_level(level)
    if level >= 13 then
        return { symbol = "!", points = 5000, key = "game.pacman.fruit.key", fallback = "Key" }
    end
    return FRUIT_TABLE[level] or FRUIT_TABLE[#FRUIT_TABLE]
end

-- 获取散射/追逐时间表
local function scatter_schedule(level)
    if level >= 17 then
        return {
            { mode = "scatter", sec = 1 },
            { mode = "chase",   sec = 20 },
            { mode = "scatter", sec = 1 },
            { mode = "chase",   sec = 20 },
            { mode = "chase",   sec = 9999 }
        }
    elseif level >= 5 then
        return {
            { mode = "scatter", sec = 5 },
            { mode = "chase",   sec = 20 },
            { mode = "scatter", sec = 5 },
            { mode = "chase",   sec = 20 },
            { mode = "scatter", sec = 5 },
            { mode = "chase",   sec = 9999 }
        }
    end
    return {
        { mode = "scatter", sec = 7 },
        { mode = "chase",   sec = 20 },
        { mode = "scatter", sec = 7 },
        { mode = "chase",   sec = 20 },
        { mode = "scatter", sec = 5 },
        { mode = "chase",   sec = 20 },
        { mode = "scatter", sec = 5 },
        { mode = "chase",   sec = 9999 }
    }
end

-- 获取当前追逐模式
local function current_chase_mode()
    local t = math.floor((state.frame - state.level_start_frame) / FPS)
    local schedule = scatter_schedule(state.level)
    for i = 1, #schedule do
        local seg = schedule[i]
        if t < seg.sec then
            return seg.mode
        end
        t = t - seg.sec
    end
    return "chase"
end

-- 获取幽灵释放延迟
local function ghost_release_delays(level)
    if level <= 1 then
        return { blinky = 0, pinky = 2, inky = 4, clyde = 6 }
    elseif level <= 4 then
        return { blinky = 0, pinky = 1, inky = 3, clyde = 5 }
    end
    return { blinky = 0, pinky = 1, inky = 2, clyde = 3 }
end

-- 寻找最近的可行走位置
local function find_nearest_walkable(start_r, start_c)
    if is_walkable(start_r, start_c) and (not is_door(start_r, start_c)) then
        return start_r, start_c
    end
    local q_r, q_c, head = { start_r }, { start_c }, 1
    local visited = blank_matrix(state.rows, state.cols, false)
    if in_bounds(start_r, start_c) then
        visited[start_r][start_c] = true
    end
    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local nr, nc = apply_tunnel(r + d.dr, c + d.dc)
            if in_bounds(nr, nc) and not visited[nr][nc] then
                visited[nr][nc] = true
                if is_walkable(nr, nc) and (not is_door(nr, nc)) then
                    return nr, nc
                end
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end
    return 2, 2
end

-- 构建玩家可达区域掩码
local function build_player_reachable_mask()
    local reachable = blank_matrix(state.rows, state.cols, false)
    local sr, sc = state.player_start.r, state.player_start.c
    if not in_bounds(sr, sc) then return reachable end

    local q_r, q_c, head = { sr }, { sc }, 1
    reachable[sr][sc] = true

    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local ok, nr, nc = can_move_player(r + d.dr, c + d.dc)
            if ok and (not reachable[nr][nc]) then
                reachable[nr][nc] = true
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end

    return reachable
end

-- 构建幽灵屋区域掩码
local function build_ghost_house_mask()
    local house = blank_matrix(state.rows, state.cols, false)
    local sr, sc = state.ghost_spawn.r, state.ghost_spawn.c
    if not in_bounds(sr, sc) or is_wall(sr, sc) then return house end

    local q_r, q_c, head = { sr }, { sc }, 1
    house[sr][sc] = true

    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local nr, nc = apply_tunnel(r + d.dr, c + d.dc)
            if in_bounds(nr, nc)
                and (not house[nr][nc])
                and (not is_wall(nr, nc))
                and (not is_door(nr, nc))
            then
                house[nr][nc] = true
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end

    return house
end

-- 从地图初始化玩家和幽灵位置
local function init_positions_from_map()
    local center_r, center_c = math.floor(state.rows * 0.75), math.floor(state.cols / 2)
    local pr, pc = find_nearest_walkable(center_r, center_c)
    state.player_start = { r = pr, c = pc }

    if #state.door_cells > 0 then
        local d = state.door_cells[math.floor((#state.door_cells + 1) / 2)]
        local sr, sc = find_nearest_walkable(d.r + 1, d.c)
        local fr, fc = find_nearest_walkable(d.r + 3, d.c)
        state.ghost_spawn, state.fruit.r, state.fruit.c = { r = sr, c = sc }, fr, fc
    else
        local sr, sc = find_nearest_walkable(math.floor(state.rows / 2), math.floor(state.cols / 2))
        state.ghost_spawn, state.fruit.r, state.fruit.c = { r = sr, c = sc }, sr, sc
    end
end

-- 为当前关卡随机化水果位置
local function randomize_fruit_spawn_for_level()
    local candidates = {}
    local reachable = build_player_reachable_mask()
    local ghost_house = build_ghost_house_mask()

    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch = state.base_map[r][c]
            local walkable = (not is_wall(r, c)) and (not is_door(r, c))
            local on_pellet_track = (ch == PELLET_CHAR or ch == POWER_CHAR)
            if walkable and on_pellet_track and reachable[r][c] and (not ghost_house[r][c]) and ch ~= "<" and ch ~= ">" then
                local is_player_spawn = (r == state.player_start.r and c == state.player_start.c)
                local is_ghost_spawn = (r == state.ghost_spawn.r and c == state.ghost_spawn.c)
                local no_pellet = state.pellets[r][c] == " "
                if (not is_player_spawn) and (not is_ghost_spawn) and no_pellet then
                    candidates[#candidates + 1] = { r = r, c = c }
                end
            end
        end
    end

    if #candidates == 0 then return end

    local idx = 1
    if type(random) == "function" then
        idx = random_index(#candidates)
    end
    local pick = candidates[idx] or candidates[1]
    state.fruit.r, state.fruit.c = pick.r, pick.c
end

-- 加载最佳分数
local function load_best_record()
    state.best_score = 0
    if type(get_best_score) ~= "function" then return end
    local ok, v = pcall(get_best_score)
    if not ok then return end
    if type(v) == "table" and type(v.score) == "number" then
        state.best_score = math.floor(v.score)
    elseif type(v) == "number" then
        state.best_score = math.floor(v)
    elseif type(v) == "table" and type(v.value) == "number" then
        state.best_score = math.floor(v.value)
    end
end

-- 保存最佳分数
local function save_best_score()
    if type(request_save_best_score) == "function" then
        pcall(request_save_best_score)
    end
end

-- 提交游戏统计
local function commit_stats_once()
    if state.stats_committed then return end
    if state.score > state.best_score then
        state.best_score = state.score
        save_best_score()
    end
    state.stats_committed = true
end

-- 设置提示信息
local function set_info_message(text, color, duration_sec)
    state.info_message, state.info_color = text, (color or "dark_gray")
    if duration_sec ~= nil and duration_sec > 0 then
        state.info_message_until = state.frame + math.floor(duration_sec * FPS)
        start_timer_field("info_timer_id", duration_sec * 1000, "pacman_info")
    else
        state.info_message_until = nil
        kill_timer_field("info_timer_id")
    end
end

-- 开始回合倒计时
local function start_round_countdown(seconds)
    local sec = math.max(0, math.floor(seconds or 0))
    state.countdown_until = state.frame + sec * FPS
    state.last_countdown_sec = nil
    if sec > 0 then
        start_timer_field("countdown_timer_id", sec * 1000, "pacman_countdown")
    else
        kill_timer_field("countdown_timer_id")
    end
    if state.global_pause_until < state.countdown_until then
        state.global_pause_until = state.countdown_until
    end
end

-- 获取剩余倒计时秒数
local function countdown_seconds_left()
    if state.phase ~= "playing" then return 0 end
    local remaining = timer_remaining_ms("countdown_timer_id")
    if remaining ~= nil then
        return remaining > 0 and math.max(1, math.ceil(remaining / 1000)) or 0
    end
    if state.countdown_until <= state.frame then return 0 end
    return math.max(1, math.ceil((state.countdown_until - state.frame) / FPS))
end

-- 获取当前横幅消息
local function current_banner_message()
    if state.confirm_mode == "restart" then
        return replace_prompt_keys(tr("game.pacman.confirm_restart")), "yellow"
    end
    if state.confirm_mode == "exit" then
        return replace_prompt_keys(tr("game.pacman.confirm_exit")), "yellow"
    end

    local countdown = countdown_seconds_left()
    if countdown > 0 then
        return tr("game.pacman.countdown") .. " " .. tostring(countdown), "yellow"
    end

    if state.info_message == "" then
        return "", state.info_color or "dark_gray"
    end
    return state.info_message, state.info_color or "dark_gray"
end

-- 重置能量状态
local function reset_power_cycle()
    state.power_until, state.power_chain, state.power_eaten = 0, 0, {}
    kill_timer_field("power_timer_id")
    for i = 1, #state.ghosts do
        if state.ghosts[i].state == "frightened" then
            state.ghosts[i].state = "normal"
        end
    end
end

-- 激活能量状态
local function activate_power_cycle()
    local duration = power_duration_sec(state.level)
    state.power_until = state.frame + duration * FPS
    start_timer_field("power_timer_id", duration * 1000, "pacman_power")
    state.power_chain, state.power_eaten = 0, {}
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        state.power_eaten[g.id] = false
        if g.state ~= "eyes" and g.state ~= "house" then
            g.state = "frightened"
        end
    end
end

-- 增加分数
local function add_score(delta)
    if delta <= 0 then return end
    state.score = state.score + delta
    if (not state.extra_life_granted) and state.score >= EXTRA_LIFE_SCORE then
        state.extra_life_granted = true
        state.lives = state.lives + 1
    end
end

-- 创建幽灵
local function create_ghost(id, color, home_r, home_c)
    return {
        id = id,
        color = color,
        r = state.ghost_spawn.r,
        c = state.ghost_spawn.c,
        dir = "left",
        state = "house",
        release_at = state.frame,
        next_step_at = state.frame,
        home_r = home_r,
        home_c = home_c,
    }
end

-- 重置玩家自动方向
local function reset_player_auto_direction()
    local order = { "left", "up", "right", "down" }
    for i = 1, #order do
        local dir = order[i]
        local dr, dc = direction_delta(dir)
        local can_move = can_move_player(state.player.r + dr, state.player.c + dc)
        if can_move then
            state.player.dir = dir
            state.player.next_dir = dir
            return
        end
    end
    state.player.dir = "left"
    state.player.next_dir = "left"
end

-- 重置实体到当前关卡
local function reset_entities_for_level()
    state.player.r, state.player.c = state.player_start.r, state.player_start.c
    state.player.next_step_at = state.frame
    reset_player_auto_direction()

    local top, left, right, bottom = 2, 2, state.cols - 1, state.rows - 1
    state.ghosts = {
        create_ghost("blinky", "red", top, right),
        create_ghost("pinky", "magenta", top, left),
        create_ghost("inky", "cyan", bottom, right),
        create_ghost("clyde", "rgb(255,165,0)", bottom, left),
    }

    local delays = ghost_release_delays(state.level)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.id == "blinky" then
            g.state, g.release_at = "normal", state.frame
        else
            g.state, g.release_at = "house", state.frame + math.floor((delays[g.id] or 0) * FPS)
        end
    end

    state.global_pause_until = state.frame + FPS
    reset_power_cycle()
    state.fruit.active, state.fruit.spawned = false, false
end

-- 开始新关卡
local function start_level(level)
    state.level, state.level_start_frame = level, state.frame
    parse_map()
    init_positions_from_map()
    state.total_pellets = pellet_counts()
    state.remaining_pellets = state.total_pellets
    reset_entities_for_level()
    randomize_fruit_spawn_for_level()
    start_round_countdown(3)
    set_info_message(tr("game.pacman.status_ready"), "yellow", 3)
    state.dirty = true
end

-- 开始新游戏
local function start_new_run()
    state.score, state.lives, state.extra_life_granted = 0, 3, false
    state.phase, state.run_start_frame, state.end_frame = "playing", state.frame, nil
    state.stats_committed = false
    state.confirm_mode = nil
    state.collected_fruits = {}
    start_level(1)
end

-- 检查能量是否激活
local function is_power_active()
    if state.phase ~= "playing" then return false end
    local remaining = timer_remaining_ms("power_timer_id")
    if remaining ~= nil then return remaining > 0 end
    return state.power_until > state.frame
end

local function power_remaining_seconds()
    local remaining = timer_remaining_ms("power_timer_id")
    if remaining ~= nil then return math.max(0, math.ceil(remaining / 1000)) end
    return math.max(0, math.ceil((state.power_until - state.frame) / FPS))
end

-- 更新能量状态
local function update_power_state()
    if state.power_until > 0 and state.frame >= state.power_until then
        reset_power_cycle()
        state.dirty = true
    end
end

-- 需要时生成水果
local function spawn_fruit_if_needed()
    if state.fruit.spawned then return end
    if state.remaining_pellets <= math.floor(state.total_pellets * 0.7) then
        state.fruit.spawned = true
        state.fruit.active = true
        state.dirty = true
    end
end

-- 吃掉当前格子上的物品
local function consume_current_cell()
    local r, c = state.player.r, state.player.c
    local pellet = state.pellets[r][c]
    if pellet == PELLET_CHAR then
        state.pellets[r][c] = " "
        state.remaining_pellets = state.remaining_pellets - 1
        add_score(10)
        state.dirty = true
    elseif pellet == POWER_CHAR then
        state.pellets[r][c] = " "
        state.remaining_pellets = state.remaining_pellets - 1
        add_score(50)
        activate_power_cycle()
        set_info_message(tr("game.pacman.status_power"), "cyan", 3)
        state.dirty = true
    end

    if state.fruit.active and r == state.fruit.r and c == state.fruit.c then
        local fruit = fruit_for_level(state.level)
        add_score(fruit.points)
        state.fruit.active = false
        state.collected_fruits[#state.collected_fruits + 1] = fruit.symbol
        set_info_message(tr("game.pacman.status_fruit"), "magenta", 3)
        state.dirty = true
    end

    if state.remaining_pellets <= 0 then
        if state.level >= MAX_LEVEL then
            state.phase = "won"
            state.end_frame = state.frame
            commit_stats_once()
            set_info_message(tr("game.pacman.win_banner") .. " " .. restart_quit_controls_text(), "green")
            state.dirty = true
        else
            start_level(state.level + 1)
            set_info_message(tr("game.pacman.status_level_clear") .. " " .. tostring(state.level), "green", 3)
        end
    end
end

-- 尝试移动玩家
local function try_move_player()
    if state.frame < state.player.next_step_at then return end
    state.player.next_step_at = state.frame + seconds_to_frames(PLAYER_STEP_SECONDS)

    -- 尝试转向
    local dr, dc = direction_delta(state.player.next_dir)
    local can_turn = can_move_player(state.player.r + dr, state.player.c + dc)
    if can_turn then
        state.player.dir = state.player.next_dir
    end

    -- 当前方向移动
    local mdr, mdc = direction_delta(state.player.dir)
    local can_move, nr, nc = can_move_player(state.player.r + mdr, state.player.c + mdc)
    if can_move then
        state.player.r, state.player.c = nr, nc
        consume_current_cell()
        state.dirty = true
    end
end

-- BFS 可行走判断
local function walkable_for_bfs(r, c)
    return in_bounds(r, c) and (not is_wall(r, c))
end

-- BFS 距离计算
local function bfs_distance(sr, sc, tr, tc)
    if sr == tr and sc == tc then return 0 end
    if not in_bounds(sr, sc) then return 9999 end
    local visited = blank_matrix(state.rows, state.cols, false)
    local dist = blank_matrix(state.rows, state.cols, -1)
    local q_r, q_c, head = { sr }, { sc }, 1
    visited[sr][sc], dist[sr][sc] = true, 0
    while head <= #q_r do
        local r, c = q_r[head], q_c[head]
        head = head + 1
        for i = 1, #DIRS do
            local d = DIRS[i]
            local nr, nc = apply_tunnel(r + d.dr, c + d.dc)
            if in_bounds(nr, nc) and (not visited[nr][nc]) and walkable_for_bfs(nr, nc) then
                visited[nr][nc], dist[nr][nc] = true, dist[r][c] + 1
                if nr == tr and nc == tc then return dist[nr][nc] end
                q_r[#q_r + 1], q_c[#q_c + 1] = nr, nc
            end
        end
    end
    return 9999
end

-- 选择幽灵的下一步
local function choose_step_towards(g, target_r, target_c)
    local candidates = {}
    for i = 1, #DIRS do
        local d = DIRS[i]
        local ok, nr, nc = can_move_ghost(g.r + d.dr, g.c + d.dc)
        if ok then
            candidates[#candidates + 1] = { dir = d.name, r = nr, c = nc }
        end
    end
    if #candidates == 0 then return nil end

    if #candidates > 1 then
        local opp = opposite_dir(g.dir)
        local filtered = {}
        for i = 1, #candidates do
            if candidates[i].dir ~= opp then
                filtered[#filtered + 1] = candidates[i]
            end
        end
        if #filtered > 0 then
            candidates = filtered
        end
    end

    if g.state == "frightened" and (not state.power_eaten[g.id]) then
        return candidates[random_index(#candidates)]
    end

    local best, best_dist = nil, 9999
    for i = 1, #candidates do
        local cand = candidates[i]
        local d = bfs_distance(cand.r, cand.c, target_r, target_c)
        if d < best_dist then
            best_dist, best = d, cand
        end
    end
    return best or candidates[random_index(#candidates)]
end

-- 预测玩家未来位置
local function projected_player_pos(steps)
    local r, c = state.player.r, state.player.c
    local dr, dc = direction_delta(state.player.dir)
    for _ = 1, steps do
        local nr, nc = apply_tunnel(r + dr, c + dc)
        if not in_bounds(nr, nc) or is_wall(nr, nc) then break end
        r, c = nr, nc
    end
    return r, c
end

-- Blinky 是否狂暴
local function blinky_enraged()
    if state.level < 5 then return false end
    return state.remaining_pellets <= math.max(20, math.floor(state.total_pellets * 0.15))
end

-- 获取幽灵目标位置
local function ghost_target(g)
    if g.state == "eyes" then
        return state.ghost_spawn.r, state.ghost_spawn.c
    end

    local mode = current_chase_mode()
    if g.id == "blinky" and blinky_enraged() then
        mode = "chase"
    end
    if mode == "scatter" then
        return g.home_r, g.home_c
    end

    if g.id == "blinky" then
        return state.player.r, state.player.c
    elseif g.id == "pinky" then
        return projected_player_pos(4)
    elseif g.id == "inky" then
        local ar, ac = projected_player_pos(2)
        local br, bc = state.player.r, state.player.c
        for i = 1, #state.ghosts do
            if state.ghosts[i].id == "blinky" then
                br = state.ghosts[i].r
                bc = state.ghosts[i].c
                break
            end
        end
        local tx = clamp(ar + (ar - br), 2, state.rows - 1)
        local ty = clamp(ac + (ac - bc), 2, state.cols - 1)
        return tx, ty
    else -- clyde
        local dist = math.abs(state.player.r - g.r) + math.abs(state.player.c - g.c)
        if dist > 8 then
            return state.player.r, state.player.c
        end
        return g.home_r, g.home_c
    end
end

-- 幽灵步进间隔
local function ghost_step_interval(g)
    if g.state == "eyes" then
        return seconds_to_frames(EYES_STEP_SECONDS)
    end

    local multiplier = GHOST_BASE_SPEED_MULTIPLIER
    if g.state == "frightened" and (not state.power_eaten[g.id]) then
        multiplier = multiplier * FRIGHTENED_SPEED_MULTIPLIER
    elseif g.id == "blinky" and current_chase_mode() == "chase" then
        multiplier = multiplier * BLINKY_CHASE_SPEED_MULTIPLIER
    end
    return step_frames_for_speed_multiplier(multiplier)
end

-- 幽灵进入幽灵屋
local function ghost_enter_house(g)
    g.r, g.c = state.ghost_spawn.r, state.ghost_spawn.c
    g.state = "house"
    g.release_at = state.frame + ghost_revive_sec(state.level) * FPS
    g.next_step_at = state.frame + FPS
end

-- 吃掉幽灵
local function eat_ghost(g)
    if g.state ~= "frightened" or state.power_eaten[g.id] then
        return false
    end
    local reward = { 200, 400, 800, 1600 }
    local idx = math.min(4, state.power_chain + 1)
    add_score(reward[idx])
    state.power_chain = state.power_chain + 1
    state.power_eaten[g.id] = true
    g.state = "eyes"
    g.next_step_at = state.frame
    set_info_message(tr("game.pacman.status_ghost_eaten"), "cyan", 3)
    state.dirty = true
    return true
end

-- 玩家死亡后重置
local function reset_after_player_death()
    state.player.r, state.player.c = state.player_start.r, state.player_start.c
    state.player.next_step_at = state.frame
    reset_player_auto_direction()

    local delays = ghost_release_delays(state.level)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        g.r, g.c = state.ghost_spawn.r, state.ghost_spawn.c
        g.dir, g.state = "left", "house"
        g.release_at = state.frame + 4 * FPS + math.floor((delays[g.id] or 0) * FPS)
        g.next_step_at = g.release_at
    end

    state.global_pause_until = state.frame + 4 * FPS
    reset_power_cycle()
    set_info_message(tr("game.pacman.status_wait"), "yellow", 4)
    state.dirty = true
end

-- 玩家失去一条命
local function player_lost_life()
    state.lives = state.lives - 1
    if state.lives <= 0 then
        state.phase = "lost"
        state.end_frame = state.frame
        commit_stats_once()
        set_info_message(tr("game.pacman.lose_banner") .. " " .. restart_quit_controls_text(), "red")
    else
        reset_after_player_death()
    end
end

-- 检查玩家与幽灵的碰撞
local function check_collision_with_ghost(g)
    if state.player.r ~= g.r or state.player.c ~= g.c then return end
    if g.state == "eyes" or g.state == "house" then return end
    if is_power_active() and g.state == "frightened" and (not state.power_eaten[g.id]) then
        eat_ghost(g)
    else
        player_lost_life()
    end
end

-- 移动幽灵
local function move_ghost(g)
    if g.state == "house" then
        if state.frame >= g.release_at and state.frame >= state.global_pause_until then
            g.state = (is_power_active() and (not state.power_eaten[g.id])) and "frightened" or "normal"
            g.next_step_at = state.frame
        end
        return
    end

    if state.frame < g.next_step_at or state.frame < state.global_pause_until then return end
    g.next_step_at = state.frame + ghost_step_interval(g)

    local target_r, target_c = ghost_target(g)
    local step = choose_step_towards(g, target_r, target_c)
    if step then
        g.r, g.c, g.dir = step.r, step.c, step.dir
        state.dirty = true
    end

    if g.state == "eyes" and g.r == state.ghost_spawn.r and g.c == state.ghost_spawn.c then
        ghost_enter_house(g)
    end
end

-- 更新所有幽灵
local function update_ghosts()
    if state.phase ~= "playing" then return end

    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.state ~= "eyes" then
            if is_power_active() and (not state.power_eaten[g.id]) and g.state ~= "house" then
                g.state = "frightened"
            elseif g.state == "frightened" and ((not is_power_active()) or state.power_eaten[g.id]) then
                g.state = "normal"
            end
        end
    end

    for i = 1, #state.ghosts do
        move_ghost(state.ghosts[i])
        check_collision_with_ghost(state.ghosts[i])
        if state.phase ~= "playing" then return end
    end
end

-- 更新玩家和碰撞检测
local function update_player_and_collisions()
    if state.phase ~= "playing" or countdown_seconds_left() > 0 then return end
    try_move_player()
    for i = 1, #state.ghosts do
        check_collision_with_ghost(state.ghosts[i])
        if state.phase ~= "playing" then return end
    end
end

-- 处理确认模式下的输入
local function handle_confirm_input(key)
    if state.confirm_mode == nil then return false end
    if key == "confirm_yes" or key == "y" then
        if state.confirm_mode == "restart" then
            start_new_run()
        else
            commit_stats_once()
            request_exit_game()
        end
        return true
    end
    if key == "confirm_no" or key == "n" then
        state.confirm_mode = nil
        state.dirty = true
        return true
    end
    return true
end

-- 处理游戏中的输入
local function handle_playing_input(key)
    if handle_confirm_input(key) then return end
    if key == "move_up" or key == "move_down" or key == "move_left" or key == "move_right" or key == "up" or key == "down" or key == "left" or key == "right" then
        state.player.next_dir = ({ move_up = "up", move_down = "down", move_left = "left", move_right = "right" })[key] or key
        return
    end
    if key == "restart" or key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return
    end
    if key == "quit_action" or key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return
    end
end

-- 处理游戏结束后的输入
local function handle_result_input(key)
    if key == "restart" or key == "r" then
        start_new_run()
        return
    end
    if key == "quit_action" or key == "q" or key == "esc" then
        commit_stats_once()
        request_exit_game()
    end
end

-- 获取指定位置的幽灵
local function ghost_at(r, c)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.r == r and g.c == c then
            return g
        end
    end
    return nil
end

-- 获取单元格显示字符和颜色
local function cell_visual(r, c)
    if state.player.r == r and state.player.c == c then
        return "@", (is_power_active() and "light_cyan" or "light_yellow")
    end

    local g = ghost_at(r, c)
    if g then
        if g.state == "eyes" then
            return "&", "white"
        end
        if g.state == "frightened" and (not state.power_eaten[g.id]) then
            return "&", "light_blue"
        end
        return "&", g.color
    end

    if state.fruit.active and r == state.fruit.r and c == state.fruit.c then
        return fruit_for_level(state.level).symbol, "magenta"
    end

    local pellet = state.pellets[r][c]
    if pellet == PELLET_CHAR then
        return PELLET_CHAR, "yellow"
    end
    if pellet == POWER_CHAR then
        return POWER_CHAR, "rgb(255,165,0)"
    end

    local ch = state.base_map[r][c]
    if ch == PELLET_CHAR or ch == POWER_CHAR then
        return " ", "white"
    end
    if WALL_SET[ch] then
        return ch, "blue"
    end
    if ch == DOOR_CHAR then
        return DOOR_CHAR, "white"
    end
    if ch == "<" or ch == ">" then
        return " ", "white"
    end
    return ch, "white"
end

-- 计算文本居中位置
local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - text_width(text)) / 2)
    if x < area_x then x = area_x end
    return x
end

-- 填充矩形区域
local function fill_rect(x, y, w, h)
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", "black")
    end
end

-- 获取收集的水果符号字符串
local function collected_fruit_symbols()
    if #state.collected_fruits == 0 then
        return "-"
    end
    return table.concat(state.collected_fruits, " ")
end

-- 构建右侧信息区域的宽度
local function build_info_width()
    local best_line = tr("game.pacman.best_score") .. ": " .. tostring(state.best_score)
    local score_line = tr("game.pacman.current_score") .. ": " .. tostring(state.score)
    local time_line = tr("game.pacman.game_time") .. ": " .. format_duration(elapsed_seconds())
    local level_line = tr("game.pacman.level") .. ": " .. tostring(state.level)
    local power_left = is_power_active() and power_remaining_seconds() or 0
    local power_line = tr("game.pacman.power_left") .. " " .. tostring(power_left) .. tr("game.pacman.seconds_unit")
    local lives_label = tr("game.pacman.lives") .. ": "
    local lives_icons = string.rep("@", math.max(0, state.lives))
    if lives_icons == "" then lives_icons = "-" end
    local fruits_line = collected_fruit_symbols()

    local max_w = 18
    local candidates = { best_line, score_line, time_line, level_line, power_line, fruits_line, lives_label ..
    lives_icons }
    for i = 1, #candidates do
        local w = text_width(candidates[i])
        if w > max_w then max_w = w end
    end
    return max_w + 1
end

-- 计算界面几何布局
local function board_geometry()
    local term_w, term_h = terminal_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local gap = 4

    local content_w = map_w + gap + info_w
    local content_h = map_h

    local controls = controls_text()
    local controls_w = min_width_for_lines(controls, 3, 24)
    local result_w = text_width(tr("game.pacman.lose_banner") .. " " .. restart_quit_controls_text())
    local confirm_w = math.max(
        text_width(replace_prompt_keys(tr("game.pacman.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.pacman.confirm_exit")))
    )
    local countdown_w = text_width(tr("game.pacman.countdown") .. " 3")

    local total_w = math.max(content_w, controls_w, result_w, confirm_w, countdown_w)
    local total_h = content_h + 1 + 3

    local x = math.floor((term_w - total_w) / 2) + 1
    local y = math.floor((term_h - total_h) / 2) + 1
    if x < 1 then x = 1 end
    if y < 1 then y = 1 end

    local map_x = x
    local map_y = y
    local info_x = map_x + map_w + gap
    if info_x + info_w - 1 > x + total_w - 1 then
        info_x = x + total_w - info_w
    end

    local mid_y = map_y + math.floor(map_h / 2) - 1
    if mid_y < map_y then mid_y = map_y end
    if mid_y > map_y + map_h - 3 then mid_y = map_y + map_h - 3 end

    local fruits_y = mid_y + 5
    if fruits_y > map_y + map_h - 1 then fruits_y = map_y + map_h - 1 end

    return {
        x = x,
        y = y,
        total_w = total_w,
        total_h = total_h,
        map_x = map_x,
        map_y = map_y,
        map_w = map_w,
        map_h = map_h,
        info_x = info_x,
        info_y = map_y,
        info_w = info_w,
        info_mid_y = mid_y,
        info_fruits_y = fruits_y,
        message_y = y + content_h,
        controls_y = y + content_h + 1,
    }
end

-- 绘制地图
local function draw_map(layout)
    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch, fg = cell_visual(r, c)
            draw_text(layout.map_x + c - 1, layout.map_y + r - 1, ch, fg, "black")
        end
    end
end

-- 绘制右侧信息
local function draw_info(layout)
    fill_rect(layout.info_x, layout.info_y, layout.info_w, layout.map_h)

    local top_y = layout.info_y
    local mid_y = layout.info_mid_y

    local best_line = tr("game.pacman.best_score") .. ": " .. tostring(state.best_score)
    local score_line = tr("game.pacman.current_score") .. ": " .. tostring(state.score)
    local time_line = tr("game.pacman.game_time") .. ": " .. format_duration(elapsed_seconds())

    draw_text(layout.info_x, top_y, best_line, "dark_gray", "black")
    draw_text(layout.info_x, top_y + 1, score_line, "white", "black")
    draw_text(layout.info_x, top_y + 2, time_line, "light_cyan", "black")

    local level_line = tr("game.pacman.level") .. ": " .. tostring(state.level)
    draw_text(layout.info_x, mid_y, level_line, "white", "black")

    local lives_label = tr("game.pacman.lives") .. ": "
    local lives_icons = string.rep("@", math.max(0, state.lives))
    if lives_icons == "" then lives_icons = "-" end
    draw_text(layout.info_x, mid_y + 1, lives_label, "white", "black")
    draw_text(layout.info_x + text_width(lives_label), mid_y + 1, lives_icons, "yellow", "black")

    local remain = is_power_active() and power_remaining_seconds() or 0
    local power_line = tr("game.pacman.power_left") .. " " .. tostring(remain) .. tr("game.pacman.seconds_unit")
    draw_text(layout.info_x, mid_y + 2, power_line, "white", "black")

    draw_text(layout.info_x, layout.info_fruits_y, collected_fruit_symbols(), "light_magenta", "black")
end

-- 绘制消息行
local function draw_message(layout)
    local term_w, _ = terminal_size()
    draw_text(1, layout.message_y, string.rep(" ", term_w), "white", "black")

    local msg, color = current_banner_message()
    if msg ~= "" then
        draw_text(centered_x(msg, 1, term_w), layout.message_y, msg, color or "dark_gray", "black")
    end
end

-- 绘制控制说明
local function draw_controls(layout)
    local controls = controls_text()
    local term_w, _ = terminal_size()
    local lines = wrap_words(controls, math.max(10, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 0, 2 do
        draw_text(1, layout.controls_y + i, string.rep(" ", term_w), "white", "black")
    end
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        draw_text(centered_x(lines[i], 1, term_w), layout.controls_y + offset + i - 1, lines[i], "white", "black")
    end
end

-- 清除上次渲染的区域（如果需要）
local function clear_last_area_if_needed(layout)
    local area = { x = layout.x, y = layout.y, w = layout.total_w, h = layout.total_h }
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h)
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h)
        fill_rect(area.x, area.y, area.w, area.h)
    end
    state.last_area = area
end

-- 主渲染函数
local function render_scene()
    local layout = board_geometry()
    clear_last_area_if_needed(layout)
    draw_map(layout)
    draw_info(layout)
    draw_message(layout)
    draw_controls(layout)
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local content_w = map_w + 4 + info_w

    local controls_text = controls_text()
    local controls_w = min_width_for_lines(controls_text, 3, 24)
    local result_w = text_width(tr("game.pacman.lose_banner") .. " " .. restart_quit_controls_text())
    local confirm_w = math.max(
        text_width(replace_prompt_keys(tr("game.pacman.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.pacman.confirm_exit")))
    )

    local min_w = math.max(content_w, controls_w, result_w, confirm_w) + 2
    local min_h = map_h + 1 + 3 + 2
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
        tr("warning.back_to_game_list_hint"),
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
        if state.size_warning_active then
            clear()
            state.last_area = nil
            state.dirty = true
        end
        state.size_warning_active = false
        return true
    end

    draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
    state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
    state.size_warning_active = true
    return false
end

-- 同步终端尺寸变化
local function sync_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w, state.last_term_h = w, h
        clear()
        state.last_area = nil
        state.dirty = true
    end
end

-- 游戏初始化
local function runtime_init_game(saved_state)
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    clear()
    load_best_record()
    parse_map()
    init_positions_from_map()
    start_new_run()
    return state
end

-- 更新游戏逻辑
local function update_logic(key)
    if state.phase == "playing" then
        handle_playing_input(key)
    else
        handle_result_input(key)
    end

    local countdown = countdown_seconds_left()
    if countdown > 0 then
        if state.last_countdown_sec ~= countdown then
            state.last_countdown_sec = countdown
            state.dirty = true
        end
    elseif state.last_countdown_sec ~= nil then
        state.last_countdown_sec = nil
        state.dirty = true
    end

    if state.info_message_until ~= nil and (state.frame >= state.info_message_until or timer_completed("info_timer_id")) then
        state.info_message = ""
        state.info_message_until = nil
        kill_timer_field("info_timer_id")
        state.dirty = true
    end

    if state.confirm_mode ~= nil then return end
    if state.phase ~= "playing" then return end

    update_power_state()
    spawn_fruit_if_needed()
    update_player_and_collisions()
    update_ghosts()
end


local function runtime_handle_event(state_arg, event)
    state = state_arg or state

    if event ~= nil and event.type == "resize" then
        state.last_term_w = event.width or state.last_term_w
        state.last_term_h = event.height or state.last_term_h
        clear()
        state.last_area = nil
        state.dirty = true
        return state
    end

    local key = normalize_key(event)
    local is_tick = event ~= nil and event.type == "tick"
    if is_tick then
        state.frame = state.frame + frames_from_dt(event.dt_ms)
        key = ""
    end

    if ensure_terminal_size_ok() then
        update_logic(key)
        sync_resize()
    else
        if key == "quit_action" or key == "q" or key == "esc" then
            commit_stats_once()
            request_exit_game()
        end
    end

    return state
end

local function runtime_render(state_arg)
    state = state_arg or state
    if ensure_terminal_size_ok() then
        render_scene()
        state.dirty = false
    end
end

local function runtime_save_best_score(state_arg)
    state = state_arg or state
    return {
        best_string = "game.pacman.best_block",
        score = state.best_score,
        level = state.level,
    }
end

local function runtime_exit_game(state_arg)
    state = state_arg or state
    commit_stats_once()
    kill_timer_field("countdown_timer_id")
    kill_timer_field("power_timer_id")
    kill_timer_field("info_timer_id")
    return state
end

local Runtime = {
    init_game = runtime_init_game,
    handle_event = runtime_handle_event,
    render = runtime_render,
    exit_game = runtime_exit_game,
    save_best_score = runtime_save_best_score,
}

_G.PACMAN_RUNTIME = Runtime
return Runtime
