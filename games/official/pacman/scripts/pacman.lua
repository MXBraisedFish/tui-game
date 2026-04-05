-- 甯х巼鎺у埗
local FPS = 60
local FRAME_MS = 16
local PLAYER_STEP_FRAMES = 12   -- 鐜╁姣忔鎵€闇€甯ф暟
local GHOST_SLOW_FACTOR = 2     -- 骞界伒閫熷害鍥犲瓙锛堣秺澶ц秺鎱級
local MAX_LEVEL = 20            -- 鏈€澶у叧鍗℃暟
local EXTRA_LIFE_SCORE = 100000 -- 棰濆鐢熷懡鎵€闇€鍒嗘暟

-- 鍦板浘鍏冪礌瀛楃
local PELLET_CHAR = "路" -- 鏅€氳眴瀛?local POWER_CHAR = "*"  -- 鑳介噺璞?local DOOR_CHAR = "-"   -- 骞界伒鎴块棬

-- 鍦板浘妯℃澘锛圓SCII鑹烘湳锛?local MAP_TEMPLATE = {
    "鈺斺晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺︹晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?,
    "鈺懧仿仿仿仿仿仿仿仿封晳路路路路路路路路路鈺?,
    "鈺懧封晹鈺愨晽路鈺斺晲鈺椔封晳路鈺斺晲鈺椔封晹鈺愨晽路鈺?,
    "鈺?鈺?鈺懧封晳 鈺懧封晳路鈺?鈺懧封晳 鈺?鈺?,
    "鈺懧封暁鈺愨暆路鈺氣晲鈺澛封晳路鈺氣晲鈺澛封暁鈺愨暆路鈺?,
    "鈺懧仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿封晳",
    "鈺懧封晹鈺愨晽路鈺懧封晹鈺愨晲鈺愨晽路鈺懧封晹鈺愨晽路鈺?,
    "鈺懧封暁鈺愨暆路鈺懧封暁鈺愨暒鈺愨暆路鈺懧封暁鈺愨暆路鈺?,
    "鈺懧仿仿仿仿封晳路路路鈺懧仿仿封晳路路路路路鈺?,
    "鈺氣晲鈺愨晲鈺椔封暊鈺愨晲 鈺?鈺愨晲鈺Ｂ封晹鈺愨晲鈺愨暆",
    "    鈺懧封晳       鈺懧封晳    ",
    "鈺愨晲鈺愨晲鈺澛封晳 鈺斺晲-鈺愨晽 鈺懧封暁鈺愨晲鈺愨晲",
    "<    路  鈺?  鈺? 路    >",
    "鈺愨晲鈺愨晲鈺椔封晳 鈺氣晲鈺愨晲鈺?鈺懧封晹鈺愨晲鈺愨晲",
    "    鈺懧封晳       鈺懧封晳    ",
    "    鈺懧封晳 鈺斺晲鈺愨晲鈺?鈺懧封晳    ",
    "鈺斺晲鈺愨晲鈺澛封晳 鈺氣晲鈺︹晲鈺?鈺懧封暁鈺愨晲鈺愨晽",
    "鈺懧仿仿仿仿仿仿仿仿封晳路路路路路路路路路鈺?,
    "鈺懧封晲鈺愨晽路鈺愨晲鈺惵封晳路鈺愨晲鈺惵封晹鈺愨晲路鈺?,
    "鈺?路路鈺懧仿仿仿仿仿仿仿仿仿仿封晳路路*鈺?,
    "鈺犫晲鈺椔封晳路鈺懧封晹鈺愨晲鈺愨晽路鈺懧封晳路鈺斺晲鈺?,
    "鈺犫晲鈺澛封晳路鈺懧封暁鈺愨暒鈺愨暆路鈺懧封晳路鈺氣晲鈺?,
    "鈺懧仿仿仿仿封晳路路路鈺懧仿仿封晳路路路路路鈺?,
    "鈺懧封晲鈺愨晲鈺愨暕鈺愨晲路鈺懧封晲鈺愨暕鈺愨晲鈺愨晲路鈺?,
    "鈺懧仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿仿封晳",
    "鈺氣晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?
}

-- 澧欏瀛楃闆嗗悎
local WALL_SET = {
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
    ["鈺?] = true,
}

-- 鏂瑰悜瀹氫箟
local DIRS = {
    { name = "up",    dr = -1, dc = 0 },
    { name = "left",  dr = 0,  dc = -1 },
    { name = "down",  dr = 1,  dc = 0 },
    { name = "right", dr = 0,  dc = 1 },
}

-- 姘存灉琛紙鎸夊叧鍗★級
local FRUIT_TABLE = {
    { symbol = "%", points = 100,  key = "game.pacman.fruit.cherry",     fallback = "Cherry" },
    { symbol = "U", points = 300,  key = "game.pacman.fruit.strawberry", fallback = "Strawberry" },
    { symbol = "O", points = 500,  key = "game.pacman.fruit.orange",     fallback = "Orange" },
    { symbol = "O", points = 500,  key = "game.pacman.fruit.orange",     fallback = "Orange" },
    { symbol = "Q", points = 700,  key = "game.pacman.fruit.apple",      fallback = "Apple" },
    { symbol = "Q", points = 700,  key = "game.pacman.fruit.apple",      fallback = "Apple" },
    { symbol = "搂", points = 1000, key = "game.pacman.fruit.grape",      fallback = "Grape" },
    { symbol = "搂", points = 1000, key = "game.pacman.fruit.grape",      fallback = "Grape" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian",   fallback = "Galaxian" },
    { symbol = "W", points = 2000, key = "game.pacman.fruit.galaxian",   fallback = "Galaxian" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell",       fallback = "Bell" },
    { symbol = "?", points = 3000, key = "game.pacman.fruit.bell",       fallback = "Bell" },
}

-- 娓告垙鐘舵€佽〃
local state = {
    -- 鍦板浘鏁版嵁
    rows = 0,
    cols = 0,
    base_map = {},         -- 鍩虹鍦板浘锛堝澹併€侀棬绛夛級
    pellets = {},          -- 璞嗗瓙鐘舵€?    total_pellets = 0,     -- 鎬昏眴瀛愭暟
    remaining_pellets = 0, -- 鍓╀綑璞嗗瓙鏁?
    -- 鐜╁
    player = { r = 1, c = 1, dir = "left", next_dir = "left", next_step_at = 0 },
    player_start = { r = 1, c = 1 },

    -- 骞界伒
    ghosts = {},
    ghost_spawn = { r = 1, c = 1 },
    global_pause_until = 0, -- 鍏ㄥ眬鏆傚仠鐩村埌璇ュ抚

    -- 闅ч亾
    tunnel_left = nil,
    tunnel_right = nil,
    door_cells = {}, -- 骞界伒鎴块棬浣嶇疆

    -- 姘存灉
    fruit = { active = false, spawned = false, r = 1, c = 1 },

    -- 娓告垙杩涘害
    level = 1,
    score = 0,
    best_score = 0,
    lives = 3,
    extra_life_granted = false,

    -- 鑳介噺鐘舵€?    power_until = 0,
    power_chain = 0,  -- 杩炵画鍚冨菇鐏佃鏁?    power_eaten = {}, -- 鍚勫菇鐏垫槸鍚﹀凡琚悆

    -- 娓告垙闃舵
    phase = "playing", -- playing/won/lost
    frame = 0,
    run_start_frame = 0,
    level_start_frame = 0,
    end_frame = nil,
    stats_committed = false,
    confirm_mode = nil,

    -- 鍊掕鏃?    countdown_until = 0,
    last_countdown_sec = nil,

    -- 鎻愮ず淇℃伅
    info_message = "",
    info_color = "dark_gray",
    info_message_until = nil,

    -- 鏀堕泦鐨勬按鏋滆褰?    collected_fruits = {},

    -- 娓叉煋鐩稿叧
    dirty = true,
    last_area = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
}

-- 缈昏瘧鍑芥暟锛堝畨鍏ㄨ皟鐢級
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

-- 閼惧嘲褰囬弬鍥ㄦ拱閺勫墽銇氱€硅棄瀹?local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then return w end
    end
    return #text
end

-- 鐟欏嫯瀵栭崠鏍ㄥ瘻闁?
local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    if type(key) == "table" then
        if key.type == "quit" then return "esc" end
        if key.type == "key" and type(key.name) == "string" then
            return string.lower(key.name)
        end
        if key.type == "action" and type(key.name) == "string" then
            local map = {
                move_up = "up",
                move_down = "down",
                move_left = "left",
                move_right = "right",
                restart = "r",
                quit_action = "q",
                confirm_yes = "enter",
                confirm_no = "esc",
            }
            return map[key.name] or ""
        end
    end
    return tostring(key):lower()
end

-- 閼惧嘲褰囩紒鍫㈩伂鐏忓搫顕?local function terminal_size()
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

local function exit_game(reason)
    if type(request_exit) == "function" then
        pcall(request_exit)
    end
end

-- 閹稿宕熺拠宥嗗床鐞?
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

-- 鐠侊紕鐣婚張鈧亸蹇擃啍鎼?
local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local width = hard_min
    while width <= full do
        if #wrap_words(text, width) <= max_lines then return width end
        width = width + 1
    end
    return full
end

-- 閺佹澘鈧ジ妾洪獮?
local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

-- 鐠侊紕鐣诲鑼剁箖缁夋帗鏆?local function elapsed_seconds()
    local ending = state.end_frame
    if ending == nil then ending = state.frame end
    return math.floor((ending - state.run_start_frame) / FPS)
end

-- 閺嶇厧绱￠崠鏍ㄥ瘮缂侇厽妞傞梻?
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 鐏忓棗鐡х粭锔胯鏉烆兛璐烾TF-8鐎涙顑侀弫鎵矋
local function utf8_chars(str)
    local out = {}
    for _, code in utf8.codes(str) do out[#out + 1] = utf8.char(code) end
    return out
end

-- 閸掓稑缂撶粚铏规閻晠妯€
local function blank_matrix(rows, cols, value)
    local m = {}
    for r = 1, rows do
        m[r] = {}
        for c = 1, cols do m[r][c] = value end
    end
    return m
end

-- 鐟欙絾鐎介崷鏉挎禈濡剝婢?local function parse_map()
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

-- 鏉堝湱鏅Λ鈧弻?
local function in_bounds(r, c)
    return r >= 1 and r <= state.rows and c >= 1 and c <= state.cols
end

-- 閺勵垰鎯佹稉鍝勵暰婢?
local function is_wall(r, c)
    return (not in_bounds(r, c)) or WALL_SET[state.base_map[r][c]] == true
end

-- 閺勵垰鎯佹稉鐑樺煣闂?
local function is_door(r, c)
    return in_bounds(r, c) and state.base_map[r][c] == DOOR_CHAR
end

-- 閺勵垰鎯侀崣顖濐攽鐠у府绱欓棃鐐差暰婢逛緤绱?local function is_walkable(r, c)
    return in_bounds(r, c) and (not is_wall(r, c))
end

-- 鎼存梻鏁ら梾褔浜炬导鐘烩偓?
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

-- 閸掋倖鏌囬悳鈺侇啀閺勵垰鎯侀崣顖欎簰缁夎濮╅崚鐗堝瘹鐎规矮缍呯純?
local function can_move_player(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) or is_door(r, c) then
        return false, r, c
    end
    return true, r, c
end

-- 閸掋倖鏌囬獮鐣屼紥閺勵垰鎯侀崣顖欎簰缁夎濮╅崚鐗堝瘹鐎规矮缍呯純?
local function can_move_ghost(r, c)
    r, c = apply_tunnel(r, c)
    if not in_bounds(r, c) or is_wall(r, c) then
        return false, r, c
    end
    return true, r, c
end

-- 閼惧嘲褰囬弬鐟版倻鐎电懓绨查惃鍕攽閸掓褰夐崠?
local function direction_delta(dir)
    if dir == "up" then return -1, 0 end
    if dir == "down" then return 1, 0 end
    if dir == "left" then return 0, -1 end
    return 0, 1
end

-- 閼惧嘲褰囬惄绋垮冀閺傜懓鎮?local function opposite_dir(dir)
    if dir == "up" then return "down" end
    if dir == "down" then return "up" end
    if dir == "left" then return "right" end
    return "left"
end

-- 缂佺喕顓哥挒鍡楃摍閹粯鏆?local function pellet_counts()
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

-- 閼惧嘲褰囬懗浠嬪櫤閹镐胶鐢婚弮鍫曟？閿涘牏顫楅敍?
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

-- 閼惧嘲褰囬獮鐣屼紥婢跺秵妞块弮鍫曟？閿涘牏顫楅敍?
local function ghost_revive_sec(level)
    if level >= 11 and level <= 16 then
        return 5
    end
    return 3
end

-- 閼惧嘲褰囪ぐ鎾冲閸忓啿宕遍惃鍕寜閺?
local function fruit_for_level(level)
    if level >= 13 then
        return { symbol = "!", points = 5000, key = "game.pacman.fruit.key", fallback = "Key" }
    end
    return FRUIT_TABLE[level] or FRUIT_TABLE[#FRUIT_TABLE]
end

-- 閼惧嘲褰囬弫锝呯殸/鏉╀粙鈧劖妞傞梻纾嬨€?local function scatter_schedule(level)
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

-- 閼惧嘲褰囪ぐ鎾冲鏉╀粙鈧劖膩瀵?
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

-- 閼惧嘲褰囬獮鐣屼紥闁插﹥鏂佸鎯扮箿
local function ghost_release_delays(level)
    if level <= 1 then
        return { blinky = 0, pinky = 2, inky = 4, clyde = 6 }
    elseif level <= 4 then
        return { blinky = 0, pinky = 1, inky = 3, clyde = 5 }
    end
    return { blinky = 0, pinky = 1, inky = 2, clyde = 3 }
end

-- 鐎电粯澹橀張鈧潻鎴犳畱閸欘垵顢戠挧棰佺秴缂?
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

-- 閺嬪嫬缂撻悳鈺侇啀閸欘垵鎻崠鍝勭厵閹衡晝鐖?local function build_player_reachable_mask()
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

-- 閺嬪嫬缂撻獮鐣屼紥鐏炲灏崺鐔稿负閻?
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

-- 娴犲骸婀撮崶鎯у灥婵瀵查悳鈺侇啀閸滃苯鑿囬悘鍏哥秴缂?
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

-- 娑撳搫缍嬮崜宥呭彠閸楋繝娈㈤張鍝勫濮樺瓨鐏夋担宥囩枂
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
        idx = random(#candidates) + 1
    end
    local pick = candidates[idx] or candidates[1]
    state.fruit.r, state.fruit.c = pick.r, pick.c
end

-- 閸旂姾娴囬張鈧担鍐插瀻閺?
local function load_best_record()
    state.best_score = 0
    if type(load_best_score) ~= "function" then return end
    local ok, v = pcall(load_best_score)
    if not ok then return end
    if type(v) == "table" and type(v.score) == "number" then
        state.best_score = math.floor(v.score)
    elseif type(v) == "number" then
        state.best_score = math.floor(v)
    elseif type(v) == "table" and type(v.value) == "number" then
        state.best_score = math.floor(v.value)
    end
end

-- 閹绘劒姘﹀〒鍛婂灆缂佺喕顓?local function commit_stats_once()
    if state.stats_committed then return end
    if state.score > state.best_score then
        state.best_score = state.score
        if type(request_refresh_best_score) == "function" then
            pcall(request_refresh_best_score)
        end
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "pacman", state.score, elapsed_seconds())
    end
    state.stats_committed = true
end

-- 鐠佸墽鐤嗛幓鎰仛娣団剝浼?local function set_info_message(text, color, duration_sec)
    state.info_message, state.info_color = text, (color or "dark_gray")
    if duration_sec ~= nil and duration_sec > 0 then
        state.info_message_until = state.frame + math.floor(duration_sec * FPS)
    else
        state.info_message_until = nil
    end
end

-- 瀵偓婵娲栭崥鍫濃偓鎺曨吀閺?
local function start_round_countdown(seconds)
    local sec = math.max(0, math.floor(seconds or 0))
    state.countdown_until = state.frame + sec * FPS
    state.last_countdown_sec = nil
    if state.global_pause_until < state.countdown_until then
        state.global_pause_until = state.countdown_until
    end
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
    return state
end

-- 閼惧嘲褰囬崜鈺€缍戦崐鎺曨吀閺冨墎顫楅弫?
local function countdown_seconds_left()
    if state.phase ~= "playing" then return 0 end
    if state.countdown_until <= state.frame then return 0 end
    return math.max(1, math.ceil((state.countdown_until - state.frame) / FPS))
end

-- 閼惧嘲褰囪ぐ鎾冲濡亜绠欏☉鍫熶紖
local function current_banner_message()
    if state.confirm_mode == "restart" then
        return tr("game.pacman.confirm_restart"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.pacman.confirm_exit"), "yellow"
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

-- 闁插秶鐤嗛懗浠嬪櫤閻樿埖鈧?
local function reset_power_cycle()
    state.power_until, state.power_chain, state.power_eaten = 0, 0, {}
    for i = 1, #state.ghosts do
        if state.ghosts[i].state == "frightened" then
            state.ghosts[i].state = "normal"
        end
    end
end

-- 濠碘偓濞叉槒鍏橀柌蹇曞Ц閹?
local function activate_power_cycle()
    state.power_until = state.frame + power_duration_sec(state.level) * FPS
    state.power_chain, state.power_eaten = 0, {}
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        state.power_eaten[g.id] = false
        if g.state ~= "eyes" and g.state ~= "house" then
            g.state = "frightened"
        end
    end
end

-- 婢х偛濮為崚鍡樻殶
local function add_score(delta)
    if delta <= 0 then return end
    state.score = state.score + delta
    if (not state.extra_life_granted) and state.score >= EXTRA_LIFE_SCORE then
        state.extra_life_granted = true
        state.lives = state.lives + 1
    end
end

-- 閸掓稑缂撻獮鐣屼紥
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

-- 闁插秶鐤嗛悳鈺侇啀閼奉亜濮╅弬鐟版倻
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

-- 闁插秶鐤嗙€圭偘缍嬮崚鏉跨秼閸撳秴鍙ч崡?
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

-- 瀵偓婵鏌婇崗鍐插幢
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

-- 瀵偓婵鏌婂〒鍛婂灆
local function start_new_run()
    state.score, state.lives, state.extra_life_granted = 0, 3, false
    state.phase, state.run_start_frame, state.end_frame = "playing", state.frame, nil
    state.stats_committed = false
    state.confirm_mode = nil
    state.collected_fruits = {}
    start_level(1)
end

-- 濡偓閺屻儴鍏橀柌蹇旀Ц閸氾附绺哄ú?
local function is_power_active()
    return state.power_until > state.frame
end

-- 閺囧瓨鏌婇懗浠嬪櫤閻樿埖鈧?
local function update_power_state()
    if state.power_until > 0 and state.frame >= state.power_until then
        reset_power_cycle()
        state.dirty = true
    end
end

-- 闂団偓鐟曚焦妞傞悽鐔稿灇濮樺瓨鐏?local function spawn_fruit_if_needed()
    if state.fruit.spawned then return end
    if state.remaining_pellets <= math.floor(state.total_pellets * 0.7) then
        state.fruit.spawned = true
        state.fruit.active = true
        state.dirty = true
    end
end

-- 閸氬啯甯€瑜版挸澧犻弽鐓庣摍娑撳﹦娈戦悧鈺佹惂
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
            if type(clear_input_buffer) == "function" then
                pcall(clear_input_buffer)
            end
            set_info_message(tr("game.pacman.win_banner") .. " " .. tr("game.pacman.result_controls"), "green")
            state.dirty = true
        else
            start_level(state.level + 1)
            set_info_message(tr("game.pacman.status_level_clear") .. " " .. tostring(state.level), "green", 3)
        end
    end
end

-- 鐏忔繆鐦粔璇插З閻溾晛顔?local function try_move_player()
    if state.frame < state.player.next_step_at then return end
    state.player.next_step_at = state.frame + PLAYER_STEP_FRAMES

    -- 鐏忔繆鐦潪顒€鎮?    local dr, dc = direction_delta(state.player.next_dir)
    local can_turn = can_move_player(state.player.r + dr, state.player.c + dc)
    if can_turn then
        state.player.dir = state.player.next_dir
    end

    -- 瑜版挸澧犻弬鐟版倻缁夎濮?    local mdr, mdc = direction_delta(state.player.dir)
    local can_move, nr, nc = can_move_player(state.player.r + mdr, state.player.c + mdc)
    if can_move then
        state.player.r, state.player.c = nr, nc
        consume_current_cell()
        state.dirty = true
    end
end

-- BFS 閸欘垵顢戠挧鏉垮灲閺?
local function walkable_for_bfs(r, c)
    return in_bounds(r, c) and (not is_wall(r, c))
end

-- BFS 鐠烘繄顬囩拋锛勭暬
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

-- 闁瀚ㄩ獮鐣屼紥閻ㄥ嫪绗呮稉鈧?
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
        return candidates[random(#candidates) + 1]
    end

    local best, best_dist = nil, 9999
    for i = 1, #candidates do
        local cand = candidates[i]
        local d = bfs_distance(cand.r, cand.c, target_r, target_c)
        if d < best_dist then
            best_dist, best = d, cand
        end
    end
    return best or candidates[random(#candidates) + 1]
end

-- 妫板嫭绁撮悳鈺侇啀閺堫亝娼垫担宥囩枂
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

-- Blinky 閺勵垰鎯侀悪鍌涙瘹
local function blinky_enraged()
    if state.level < 5 then return false end
    return state.remaining_pellets <= math.max(20, math.floor(state.total_pellets * 0.15))
end

-- 閼惧嘲褰囬獮鐣屼紥閻╊喗鐖ｆ担宥囩枂
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

-- 楠炵晫浼掑銉ㄧ箻闂傛挳娈?local function ghost_step_interval(g)
    if g.state == "eyes" then
        return math.max(1, math.floor(4 * GHOST_SLOW_FACTOR + 0.5))
    end
    local base = (state.level >= 11 and 6) or (state.level >= 5 and 7) or 8
    if g.id == "blinky" and blinky_enraged() then
        base = math.max(4, base - 1)
    end
    if g.state == "frightened" and (not state.power_eaten[g.id]) then
        base = base + 3
    end
    return math.max(1, math.floor(base * GHOST_SLOW_FACTOR + 0.5))
end

-- 楠炵晫浼掓潻娑樺弳楠炵晫浼掔仦?
local function ghost_enter_house(g)
    g.r, g.c = state.ghost_spawn.r, state.ghost_spawn.c
    g.state = "house"
    g.release_at = state.frame + ghost_revive_sec(state.level) * FPS
    g.next_step_at = state.frame + FPS
end

-- 閸氬啯甯€楠炵晫浼?local function eat_ghost(g)
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

-- 閻溾晛顔嶅璁抽閸氬酣鍣哥純?
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

-- 閻溾晛顔嶆径鍗炲箵娑撯偓閺夆€虫嚒
local function player_lost_life()
    state.lives = state.lives - 1
    if state.lives <= 0 then
        state.phase = "lost"
        state.end_frame = state.frame
        commit_stats_once()
        if type(clear_input_buffer) == "function" then
            pcall(clear_input_buffer)
        end
        set_info_message(tr("game.pacman.lose_banner") .. " " .. tr("game.pacman.result_controls"), "red")
    else
        reset_after_player_death()
    end
end

-- 濡偓閺屻儳甯虹€规湹绗岄獮鐣屼紥閻ㄥ嫮顫幘?
local function check_collision_with_ghost(g)
    if state.player.r ~= g.r or state.player.c ~= g.c then return end
    if g.state == "eyes" or g.state == "house" then return end
    if is_power_active() and g.state == "frightened" and (not state.power_eaten[g.id]) then
        eat_ghost(g)
    else
        player_lost_life()
    end
end

-- 缁夎濮╅獮鐣屼紥
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

-- 閺囧瓨鏌婇幍鈧張澶婅弴閻?
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

-- 閺囧瓨鏌婇悳鈺侇啀閸滃瞼顫幘鐐搭梾濞?
local function update_player_and_collisions()
    if state.phase ~= "playing" or countdown_seconds_left() > 0 then return end
    try_move_player()
    for i = 1, #state.ghosts do
        check_collision_with_ghost(state.ghosts[i])
        if state.phase ~= "playing" then return end
    end
end

-- 婢跺嫮鎮婄涵顔款吇濡€崇础娑撳娈戞潏鎾冲弳
local function handle_confirm_input(key)
    if state.confirm_mode == nil then return false end
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            start_new_run()
        else
            commit_stats_once()
            exit_game("confirm_exit_yes")
        end
        return true
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return true
    end
    return true
end

-- 婢跺嫮鎮婂〒鍛婂灆娑擃厾娈戞潏鎾冲弳
local function handle_playing_input(key)
    if handle_confirm_input(key) then return end
    if key == "up" or key == "down" or key == "left" or key == "right" then
        state.player.next_dir = key
        return
    end
    if key == "r" then
        state.confirm_mode = "restart"
        if type(clear_input_buffer) == "function" then
            pcall(clear_input_buffer)
        end
        state.dirty = true
        return
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        if type(clear_input_buffer) == "function" then
            pcall(clear_input_buffer)
        end
        state.dirty = true
        return
    end
end

-- 婢跺嫮鎮婂〒鍛婂灆缂佹挻娼崥搴ｆ畱鏉堟挸鍙?local function handle_result_input(key)
    if key == "r" then
        start_new_run()
        return
    end
    if key == "q" or key == "esc" then
        commit_stats_once()
        exit_game("result_input")
    end
end

-- 閼惧嘲褰囬幐鍥х暰娴ｅ秶鐤嗛惃鍕弴閻?
local function ghost_at(r, c)
    for i = 1, #state.ghosts do
        local g = state.ghosts[i]
        if g.r == r and g.c == c then
            return g
        end
    end
    return nil
end

-- 閼惧嘲褰囬崡鏇炲帗閺嶅吋妯夌粈鍝勭摟缁楋箑鎷版０婊嗗
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

-- 鐠侊紕鐣婚弬鍥ㄦ拱鐏炲懍鑵戞担宥囩枂
local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - text_width(text)) / 2)
    if x < area_x then x = area_x end
    return x
end

-- 婵夘偄鍘栭惌鈺佽埌閸栧搫鐓?local function fill_rect(x, y, w, h)
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", "black")
    end
end

-- 閼惧嘲褰囬弨鍫曟肠閻ㄥ嫭鎸夐弸婊咁儊閸欏嘲鐡х粭锔胯
local function collected_fruit_symbols()
    if #state.collected_fruits == 0 then
        return "-"
    end
    return table.concat(state.collected_fruits, " ")
end

-- 閺嬪嫬缂撻崣鍏呮櫠娣団剝浼呴崠鍝勭厵閻ㄥ嫬顔旀惔?
local function build_info_width()
    local best_line = tr("game.pacman.best_score") .. ": " .. tostring(state.best_score)
    local score_line = tr("game.pacman.current_score") .. ": " .. tostring(state.score)
    local time_line = tr("game.pacman.game_time") .. ": " .. format_duration(elapsed_seconds())
    local level_line = tr("game.pacman.level") .. ": " .. tostring(state.level)
    local power_left = is_power_active() and math.max(0, math.ceil((state.power_until - state.frame) / FPS)) or 0
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

-- 鐠侊紕鐣婚悾宀勬桨閸戠姳缍嶇敮鍐ㄧ湰
local function board_geometry()
    local term_w, term_h = terminal_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local gap = 4

    local content_w = map_w + gap + info_w
    local content_h = map_h

    local controls = tr("game.pacman.controls")
    local controls_w = min_width_for_lines(controls, 3, 24)
    local result_w = text_width(tr("game.pacman.lose_banner") .. " " .. tr("game.pacman.result_controls"))
    local confirm_w = math.max(
        text_width(tr("game.pacman.confirm_restart")),
        text_width(tr("game.pacman.confirm_exit"))
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

-- 缂佹ê鍩楅崷鏉挎禈
local function draw_map(layout)
    for r = 1, state.rows do
        for c = 1, state.cols do
            local ch, fg = cell_visual(r, c)
            draw_text(layout.map_x + c - 1, layout.map_y + r - 1, ch, fg, "black")
        end
    end
end

-- 缂佹ê鍩楅崣鍏呮櫠娣団剝浼?local function draw_info(layout)
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

    local remain = is_power_active() and math.max(0, math.ceil((state.power_until - state.frame) / FPS)) or 0
    local power_line = tr("game.pacman.power_left") .. " " .. tostring(remain) .. tr("game.pacman.seconds_unit")
    draw_text(layout.info_x, mid_y + 2, power_line, "white", "black")

    draw_text(layout.info_x, layout.info_fruits_y, collected_fruit_symbols(), "light_magenta", "black")
end

-- 缂佹ê鍩楀☉鍫熶紖鐞?
local function draw_message(layout)
    local term_w, _ = terminal_size()
    draw_text(1, layout.message_y, string.rep(" ", term_w), "white", "black")

    local msg, color = current_banner_message()
    if msg ~= "" then
        draw_text(centered_x(msg, 1, term_w), layout.message_y, msg, color or "dark_gray", "black")
    end
end

-- 缂佹ê鍩楅幒褍鍩楃拠瀛樻
local function draw_controls(layout)
    local controls = tr("game.pacman.controls")
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

-- 濞撳懘娅庢稉濠冾偧濞撳弶鐓嬮惃鍕隘閸╃噦绱欐俊鍌涚亯闂団偓鐟曚緤绱?local function clear_last_area_if_needed(layout)
    local area = { x = layout.x, y = layout.y, w = layout.total_w, h = layout.total_h }
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h)
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h)
        fill_rect(area.x, area.y, area.w, area.h)
    end
    state.last_area = area
end

-- 娑撶粯瑕嗛弻鎾冲毐閺?
local function render_scene()
    local layout = board_geometry()
    clear_last_area_if_needed(layout)
    draw_map(layout)
    draw_info(layout)
    draw_message(layout)
    draw_controls(layout)
end

-- 鐠侊紕鐣婚張鈧亸蹇斿闂団偓缂佸牏顏亸鍝勵嚟
local function minimum_required_size()
    local map_w, map_h = state.cols, state.rows
    local info_w = build_info_width()
    local content_w = map_w + 4 + info_w

    local controls_text = tr("game.pacman.controls")
    local controls_w = min_width_for_lines(controls_text, 3, 24)
    local result_w = text_width(tr("game.pacman.lose_banner") .. " " .. tr("game.pacman.result_controls"))
    local confirm_w = math.max(
        text_width(tr("game.pacman.confirm_restart")),
        text_width(tr("game.pacman.confirm_exit"))
    )

    local min_w = math.max(content_w, controls_w, result_w, confirm_w) + 2
    local min_h = map_h + 1 + 3 + 2
    return min_w, min_h
end

-- 缂佹ê鍩楃紒鍫㈩伂鐏忓搫顕拃锕€鎲?local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
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

-- 绾喕绻氱紒鍫㈩伂鐏忓搫顕搾鍐差檮
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
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
        state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
    end
    state.size_warning_active = true
    return false
end

-- 閸氬本顒炵紒鍫㈩伂鐏忓搫顕崣妯哄
local function sync_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w, state.last_term_h = w, h
        clear()
        state.last_area = nil
        state.dirty = true
    end
end

-- 濞撳憡鍨欓崚婵嗩潗閸?
function init_game()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    clear()
    load_best_record()
    parse_map()
    init_positions_from_map()
    start_new_run()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
    return state
end

-- 閺囧瓨鏌婂〒鍛婂灆闁槒绶?local function update_logic(key)
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

    if state.info_message_until ~= nil and state.frame >= state.info_message_until then
        state.info_message = ""
        state.info_message_until = nil
        state.dirty = true
    end

    if state.confirm_mode ~= nil then return end
    if state.phase ~= "playing" then return end

    update_power_state()
    spawn_fruit_if_needed()
    update_player_and_collisions()
    update_ghosts()
end

-- 娑撶粯鐖堕幋蹇撴儕閻?
function handle_event(state_arg, event)
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
    if event ~= nil and event.type == "tick" then
        key = ""
    end

    if ensure_terminal_size_ok() then
        update_logic(key)
        sync_resize()
        state.frame = state.frame + 1
    else
        if key == "q" or key == "esc" then
            commit_stats_once()
            exit_game("size_warning")
        end
    end

    return state
end

function render(state_arg)
    state = state_arg or state
    if ensure_terminal_size_ok() then
        render_scene()
        state.dirty = false
    end
end

function best_score(state_arg)
    state = state_arg or state
    return {
        best_string = "game.pacman.best_block",
        score = state.best_score,
        level = state.level,
    }
end


