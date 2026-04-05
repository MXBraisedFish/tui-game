local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    request_exit()
end

local BOARD_W = 10 
local BOARD_H = 20 
local CELL = "██" 
local FPS = 60
local FRAME_MS = 16
local FRAME_W = BOARD_W * 2 + 2 
local FRAME_H = BOARD_H + 2     
local SIDE_GAP = 2              
local LEFT_W = 20               
local RIGHT_W = 24              

local BORDER_TL = "╔"
local BORDER_TR = "╗"
local BORDER_BL = "╚"
local BORDER_BR = "╝"
local BORDER_H = "═"
local BORDER_V = "║"

local PIECE_ORDER = { "I", "O", "T", "Z", "L", "S", "J" }

local PIECES = {
    I = {
        group = 1,
        rots = {
            { { 0, 0 }, { 0, 1 }, { 0, 2 }, { 0, 3 } },
            { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 3, 0 } },
            { { 0, 0 }, { 0, 1 }, { 0, 2 }, { 0, 3 } },
            { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 3, 0 } },
        },
    },
    O = {
        group = 1,
        rots = {
            { { 0, 0 }, { 1, 0 }, { 0, 1 }, { 1, 1 } },
            { { 0, 0 }, { 1, 0 }, { 0, 1 }, { 1, 1 } },
            { { 0, 0 }, { 1, 0 }, { 0, 1 }, { 1, 1 } },
            { { 0, 0 }, { 1, 0 }, { 0, 1 }, { 1, 1 } },
        },
    },
    T = {
        group = 1,
        rots = {
            { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 2, 1 } },
            { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 0, 2 } },
            { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 1, 1 } },
            { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 1, 2 } },
        },
    },
    Z = {
        group = 2,
        rots = {
            { { 0, 0 }, { 1, 0 }, { 1, 1 }, { 2, 1 } },
            { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 0, 2 } },
            { { 0, 0 }, { 1, 0 }, { 1, 1 }, { 2, 1 } },
            { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 0, 2 } },
        },
    },
    L = {
        group = 2,
        rots = {
            { { 0, 0 }, { 0, 1 }, { 0, 2 }, { 1, 2 } },
            { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 0, 1 } },
            { { 0, 0 }, { 1, 0 }, { 1, 1 }, { 1, 2 } },
            { { 2, 0 }, { 0, 1 }, { 1, 1 }, { 2, 1 } },
        },
    },
    S = {
        group = 3,
        rots = {
            { { 1, 0 }, { 2, 0 }, { 0, 1 }, { 1, 1 } },
            { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 1, 2 } },
            { { 1, 0 }, { 2, 0 }, { 0, 1 }, { 1, 1 } },
            { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 1, 2 } },
        },
    },
    J = {
        group = 3,
        rots = {
            { { 1, 0 }, { 1, 1 }, { 0, 2 }, { 1, 2 } },
            { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 2, 1 } },
            { { 0, 0 }, { 1, 0 }, { 0, 1 }, { 0, 2 } },
            { { 0, 0 }, { 1, 0 }, { 2, 0 }, { 2, 1 } },
        },
    },
}

local PALETTE = {
    [0] = "#545454",
    [1] = "#001e74",
    [2] = "#081090",
    [3] = "#300088",
    [4] = "#440064",
    [5] = "#5c0030",
    [6] = "#540400",
    [7] = "#3c1800",
    [8] = "#202a00",
    [9] = "#083a00",
    [10] = "#004000",
    [11] = "#003c00",
    [12] = "#00323c",
    [13] = "#989698",
    [14] = "#0072bc",
    [15] = "#3050f8",
    [16] = "#6424f4",
    [17] = "#8814b0",
    [18] = "#b41050",
    [19] = "#a82200",
    [20] = "#884000",
    [21] = "#645c00",
    [22] = "#347400",
    [23] = "#088800",
    [24] = "#008424",
    [25] = "#007868",
    [26] = "#005e8c",
    [27] = "#eceeec",
    [28] = "#00ccf0",
    [29] = "#609cfc",
    [30] = "#9c7cfc",
    [31] = "#c870dc",
    [32] = "#ec66a0",
    [33] = "#f06e48",
    [34] = "#e89e24",
    [35] = "#c8c430",
    [36] = "#84e230",
    [37] = "#30ec44",
    [38] = "#2ce684",
    [39] = "#3cd2c4",
    [40] = "#54ace8",
    [41] = "#eceeec",
    [42] = "#6ce2fc",
    [43] = "#acbefc",
    [44] = "#d4b0fc",
    [45] = "#f0aaec",
    [46] = "#f8a8b4",
    [47] = "#fcb27c",
    [48] = "#f4d270",
    [49] = "#e0e884",
    [50] = "#b4f88c",
    [51] = "#88fa98",
    [52] = "#90eeb8",
    [53] = "#9cdedc",
    [54] = "#a4c8f0",
}

local RANDOM_COLOR_POOL = {}
for i = 0, 54 do
    if i ~= 0 and i ~= 1 and i ~= 12 and i ~= 13 then
        RANDOM_COLOR_POOL[#RANDOM_COLOR_POOL + 1] = i
    end
end

local FIXED_COLOR_ROWS = {
    [0] = { 27, 27, 27 },
    [1] = { 19, 19, 19 },
    [2] = { 40, 40, 40 },
    [3] = { 35, 30, 30 },
    [4] = { 14, 34, 23 },
    [5] = { 38, 15, 32 },
    [6] = { 26, 34, 19 },
    [7] = { 34, 38, 31 },
    [8] = { 2, 15, 23 },
    [9] = { 32, 19, 29 },
}

local DROP_FRAMES_0_9 = { [0] = 48, [1] = 43, [2] = 38, [3] = 33, [4] = 28, [5] = 23, [6] = 18, [7] = 13, [8] = 8, [9] = 6 }

local state = {
    
    board = {}, 

    
    active = nil,  
    next_kind = 1, 

    
    level = 0,            
    reincarnated = false, 
    phase = 1,            
    lines_total = 0,      
    lines_for_level = 0,  
    score = 0,            

    
    counters = { single = 0, double = 0, triple = 0, tetris = 0 },
    piece_used = { I = 0, O = 0, T = 0, Z = 0, L = 0, S = 0, J = 0 },

    
    color_group = { 27, 27, 27 }, 
    color_level = -999,           

    
    game_over = false,
    end_frame = nil,
    result_committed = false,
    confirm_mode = nil,
    input_mode = nil,
    input_buffer = "",
    toast_text = nil,
    toast_until = 0,

    
    frame = 0,
    start_frame = 0,
    drop_accum = 0, 
    last_auto_save_sec = 0,

    
    best_score = 0,

    
    launch_mode = "new",
    start_level = 0, 

    
    dirty = true,
    last_elapsed_sec = -1,
    last_toast_visible = false,

    
    last_key = "",
    last_key_frame = -100,

    
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    last_layout = nil,
}

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

local function normalize_key(event)
    if event == nil then return "" end
    if type(event) == "string" then return string.lower(event) end
    if type(event) ~= "table" then return tostring(event):lower() end
    if event.type == "quit" then return "esc" end
    if event.type == "key" and type(event.name) == "string" then return string.lower(event.name) end
    if event.type ~= "action" or type(event.name) ~= "string" then return "" end
    local map = {
        move_left = "left",
        move_right = "right",
        soft_drop = "down",
        rotate_left = "z",
        rotate_right = "x",
        hard_drop = "space",
        set_level = "p",
        save = "s",
        restart = "r",
        quit_action = "q",
        confirm_yes = "enter",
        confirm_no = "esc",
    }
    return map[event.name] or ""
end

local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then return w end
    end
    return #text
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

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

local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local w = hard_min
    while w <= full do
        if #wrap_words(text, w) <= max_lines then return w end
        w = w + 1
    end
    return full
end

local function rand_int(max)
    if max <= 0 or type(random) ~= "function" then return 0 end
    return random(max)
end

local function clone_board(src)
    local out = {}
    for y = 1, BOARD_H do
        out[y] = {}
        for x = 1, BOARD_W do out[y][x] = src[y][x] end
    end
    return out
end

local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function threshold_for_level(level)
    if level == 235 then return 810 end
    return 10
end

local function frames_per_drop(level)
    if level >= 29 then return 1 end
    if level >= 19 then return 2 end
    if level >= 16 then return 3 end
    if level >= 13 then return 4 end
    if level >= 10 then return 5 end
    return DROP_FRAMES_0_9[level] or 48
end

local function stage_label_key()
    if state.level == 255 then return "game.tetris.stage.dawn" end
    if state.level == 235 then return "game.tetris.stage.marathon" end
    if state.level >= 155 and state.level <= 157 then return "game.tetris.stage.crash" end
    if state.level == 148 then return "game.tetris.stage.darkness" end
    if state.level == 146 then return "game.tetris.stage.dusk" end
    if state.level >= 29 then return "game.tetris.stage.challenge" end
    if state.reincarnated then return "game.tetris.stage.rebirth" end
    return "game.tetris.stage.classic"
end

local function stage_label()
    local key = stage_label_key()
    if key == "game.tetris.stage.dawn" then return tr(key) end
    if key == "game.tetris.stage.marathon" then return tr(key) end
    if key == "game.tetris.stage.crash" then return tr(key) end
    if key == "game.tetris.stage.darkness" then return tr(key) end
    if key == "game.tetris.stage.dusk" then return tr(key) end
    if key == "game.tetris.stage.challenge" then return tr(key) end
    if key == "game.tetris.stage.rebirth" then return tr(key) end
    return tr(key)
end

local function stage_color()
    local key = stage_label_key()
    if key == "game.tetris.stage.classic" then return "white" end
    if key == "game.tetris.stage.challenge" then return "red" end
    if key == "game.tetris.stage.dusk" then return "rgb(232,158,36)" end
    if key == "game.tetris.stage.darkness" then return "gray" end
    if key == "game.tetris.stage.marathon" then return "green" end
    if key == "game.tetris.stage.dawn" then return "light_red" end
    if key == "game.tetris.stage.rebirth" then return "light_cyan" end
    if key == "game.tetris.stage.crash" then return "yellow" end
    return "white"
end

local function fmt5(n)
    n = math.max(0, math.floor(n or 0))
    if n < 100000 then return string.format("%05d", n) end
    return tostring(n)
end

local function piece_name(kind_id)
    return PIECE_ORDER[kind_id] or "I"
end

local function random_piece_kind()
    return rand_int(#PIECE_ORDER) + 1
end

local function pick_random_color_index()
    return RANDOM_COLOR_POOL[rand_int(#RANDOM_COLOR_POOL) + 1]
end

local function update_level_colors(force)
    if (not force) and state.color_level == state.level then return end
    local a, b, c
    if state.level == 146 then
        a, b, c = 13, 12, 1
    elseif state.level == 148 then
        a, b, c = 13, 27, 0
    elseif state.level >= 0 and state.level <= 29 then
        local row = FIXED_COLOR_ROWS[state.level % 10] or FIXED_COLOR_ROWS[0]
        a, b, c = row[1], row[2], row[3]
    else
        a, b, c = pick_random_color_index(), pick_random_color_index(), pick_random_color_index()
    end
    state.color_group[1], state.color_group[2], state.color_group[3] = a, b, c
    state.color_level = state.level
end

local function color_for_kind(kind_id)
    local piece = PIECES[piece_name(kind_id)]
    local group = 1
    if piece ~= nil and type(piece.group) == "number" then group = piece.group end
    local idx = state.color_group[group] or 27
    return PALETTE[idx] or "#eceeec"
end

local function refresh_board_colors_by_kind()
    for y = 1, BOARD_H do
        for x = 1, BOARD_W do
            local v = state.board[y][x]
            if type(v) == "number" then
                if v ~= 0 then
                    local kind = math.floor(v)
                    state.board[y][x] = { kind = kind, color = color_for_kind(kind) }
                end
            elseif type(v) == "table" then
                local kind = tonumber(v.kind)
                if kind ~= nil and kind > 0 then
                    kind = math.floor(kind)
                    v.kind = kind
                    v.color = color_for_kind(kind)
                else
                    state.board[y][x] = 0
                end
            end
        end
    end
end

local function init_board()
    state.board = {}
    for y = 1, BOARD_H do
        state.board[y] = {}
        for x = 1, BOARD_W do state.board[y][x] = 0 end
    end
end

local function get_cells(kind_id, rot)
    local piece = PIECES[piece_name(kind_id)] or PIECES.I
    return piece.rots[((rot or 0) % 4) + 1]
end

local function collides(kind_id, rot, px, py)
    local cells = get_cells(kind_id, rot)
    for i = 1, #cells do
        local cx = px + cells[i][1]
        local cy = py + cells[i][2]
        if cx < 1 or cx > BOARD_W or cy > BOARD_H then return true end
        if cy >= 1 and state.board[cy][cx] ~= 0 then return true end
    end
    return false
end

local function make_active(kind_id)
    return {
        kind = kind_id,
        rot = 0,
        x = 4,
        y = 1,
        color = color_for_kind(kind_id),
    }
end

local function spawn_active()
    local kind_id = state.next_kind
    state.next_kind = random_piece_kind()
    state.active = make_active(kind_id)
    state.piece_used[piece_name(kind_id)] = (state.piece_used[piece_name(kind_id)] or 0) + 1

    if collides(state.active.kind, state.active.rot, state.active.x, state.active.y) then
        state.game_over = true
        state.end_frame = state.frame
    end
    state.dirty = true
end

local function try_move(dx, dy)
    if state.active == nil then return false end
    local nx, ny = state.active.x + dx, state.active.y + dy
    if collides(state.active.kind, state.active.rot, nx, ny) then return false end
    state.active.x, state.active.y = nx, ny
    state.dirty = true
    return true
end

local function try_rotate(delta)
    if state.active == nil then return false end
    local nr = (state.active.rot + delta) % 4
    if not collides(state.active.kind, nr, state.active.x, state.active.y) then
        state.active.rot = nr
        state.dirty = true
        return true
    end

    
    local kicks = { 1, -1, 2, -2 }
    for i = 1, #kicks do
        local nx = state.active.x + kicks[i]
        if not collides(state.active.kind, nr, nx, state.active.y) then
            state.active.rot = nr
            state.active.x = nx
            state.dirty = true
            return true
        end
    end
    return false
end

local function lock_active_to_board()
    if state.active == nil then return end
    local cells = get_cells(state.active.kind, state.active.rot)
    for i = 1, #cells do
        local cx = state.active.x + cells[i][1]
        local cy = state.active.y + cells[i][2]
        if cy < 1 then
            state.game_over = true
            state.end_frame = state.frame
        elseif cx >= 1 and cx <= BOARD_W and cy <= BOARD_H then
            state.board[cy][cx] = { kind = state.active.kind, color = color_for_kind(state.active.kind) }
        end
    end
    state.active = nil
end

local function clear_full_rows()
    local keep = {}
    local cleared = 0
    for y = BOARD_H, 1, -1 do
        local full = true
        for x = 1, BOARD_W do
            if state.board[y][x] == 0 then
                full = false
                break
            end
        end
        if full then
            cleared = cleared + 1
        else
            table.insert(keep, 1, state.board[y])
        end
    end
    while #keep < BOARD_H do
        local row = {}
        for x = 1, BOARD_W do row[x] = 0 end
        table.insert(keep, 1, row)
    end
    state.board = keep
    return cleared
end

local function apply_level_up()
    if state.level >= 255 then
        state.level = 0
        state.reincarnated = true
        state.phase = state.phase + 1
    else
        state.level = state.level + 1
    end
    update_level_colors(true)
    refresh_board_colors_by_kind()
end

local function apply_line_result(cleared)
    if cleared <= 0 then return end

    if cleared == 1 then
        state.counters.single = state.counters.single + 1
    elseif cleared == 2 then
        state.counters.double = state.counters.double + 1
    elseif cleared == 3 then
        state.counters.triple = state.counters.triple + 1
    else
        state.counters.tetris = state.counters.tetris + 1
    end

    state.lines_total = state.lines_total + cleared
    state.lines_for_level = state.lines_for_level + cleared

    local base = 0
    if cleared == 1 then
        base = 40
    elseif cleared == 2 then
        base = 100
    elseif cleared == 3 then
        base = 300
    else
        base = 1200
    end
    state.score = state.score + (base * (state.level + 1))

    while state.lines_for_level >= threshold_for_level(state.level) do
        state.lines_for_level = state.lines_for_level - threshold_for_level(state.level)
        apply_level_up()
    end
end

local function settle_and_respawn()
    lock_active_to_board()
    if state.game_over then return end
    local cleared = clear_full_rows()
    apply_line_result(cleared)
    spawn_active()
end

local function can_soft_drop()
    return state.level <= 28
end

local function do_soft_drop()
    if state.active == nil then return false end
    if try_move(0, 1) then
        state.score = state.score + 1
        return true
    end
    settle_and_respawn()
    return false
end

local function do_hard_drop()
    if state.active == nil then return end
    while try_move(0, 1) do
    end
    settle_and_respawn()
end

local function clone_active(a)
    if a == nil then return nil end
    return { kind = a.kind, rot = a.rot, x = a.x, y = a.y, color = a.color }
end

local function read_launch_mode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, mode = pcall(get_launch_mode)
    if ok and type(mode) == "string" then
        mode = string.lower(mode)
        if mode == "continue" then return "continue" end
    end
    return "new"
end

local function load_best_record()
    local data = nil
    if type(load_best_score) == "function" then
        local ok, ret = pcall(load_best_score)
        if ok then data = ret end
    end
    if type(data) == "table" and type(data.score) == "number" then
        state.best_score = math.max(0, math.floor(data.score))
    else
        state.best_score = 0
    end
end

local function save_snapshot(manual)
    local snapshot = {
        level = state.level,
        reincarnated = state.reincarnated,
        phase = state.phase,
        lines_total = state.lines_total,
        lines_for_level = state.lines_for_level,
        score = state.score,
        counters = state.counters,
        piece_used = state.piece_used,
        board = state.board,
        active = clone_active(state.active),
        next_kind = state.next_kind,
        elapsed = elapsed_seconds(),
        color_group = state.color_group,
        color_level = state.color_level,
        start_level = state.start_level,
    }

    local ok = false
    if type(save_continue) == "function" then
        local s, ret = pcall(save_continue, snapshot)
        ok = s and ret ~= false
    end

    if manual then
        if ok then
            state.toast_text = tr("game.tetris.save_success")
            state.toast_until = state.frame + FPS * 2
        else
            state.toast_text = tr("game.tetris.save_unavailable")
            state.toast_until = state.frame + FPS * 2
        end
        state.dirty = true
    end
end

local function load_snapshot()
    local snap = nil
    if type(load_continue) == "function" then
        local s, ret = pcall(load_continue)
        if s and type(ret) == "table" then snap = ret end
    end
    if type(snap) ~= "table" then return false end

    init_board()
    if type(snap.board) == "table" then
        for y = 1, BOARD_H do
            if type(snap.board[y]) == "table" then
                for x = 1, BOARD_W do
                    local v = snap.board[y][x]
                    state.board[y][x] = v == nil and 0 or v
                end
            end
        end
    end

    state.level = math.max(0, math.floor(tonumber(snap.level) or 0))
    state.reincarnated = snap.reincarnated == true
    state.phase = math.max(1, math.floor(tonumber(snap.phase) or 1))
    state.lines_total = math.max(0, math.floor(tonumber(snap.lines_total) or 0))
    state.lines_for_level = math.max(0, math.floor(tonumber(snap.lines_for_level) or 0))
    state.score = math.max(0, math.floor(tonumber(snap.score) or 0))
    state.start_level = math.max(0, math.floor(tonumber(snap.start_level) or 0))

    state.counters = { single = 0, double = 0, triple = 0, tetris = 0 }
    if type(snap.counters) == "table" then
        state.counters.single = math.max(0, math.floor(tonumber(snap.counters.single) or 0))
        state.counters.double = math.max(0, math.floor(tonumber(snap.counters.double) or 0))
        state.counters.triple = math.max(0, math.floor(tonumber(snap.counters.triple) or 0))
        state.counters.tetris = math.max(0, math.floor(tonumber(snap.counters.tetris) or 0))
    end

    state.piece_used = { I = 0, O = 0, T = 0, Z = 0, L = 0, S = 0, J = 0 }
    if type(snap.piece_used) == "table" then
        for _, k in ipairs(PIECE_ORDER) do
            state.piece_used[k] = math.max(0, math.floor(tonumber(snap.piece_used[k]) or 0))
        end
    end

    state.color_group = { 27, 27, 27 }
    if type(snap.color_group) == "table" then
        state.color_group[1] = tonumber(snap.color_group[1]) or 27
        state.color_group[2] = tonumber(snap.color_group[2]) or 27
        state.color_group[3] = tonumber(snap.color_group[3]) or 27
    end
    state.color_level = tonumber(snap.color_level) or -999
    update_level_colors(true)
    refresh_board_colors_by_kind()

    state.active = nil
    if type(snap.active) == "table" then
        local a = snap.active
        if type(a.kind) == "number" and type(a.rot) == "number" and type(a.x) == "number" and type(a.y) == "number" then
            state.active = {
                kind = math.floor(a.kind),
                rot = math.floor(a.rot) % 4,
                x = math.floor(a.x),
                y = math.floor(a.y),
                color = type(a.color) == "string" and a.color or color_for_kind(math.floor(a.kind)),
            }
        end
    end
    state.next_kind = math.max(1, math.min(#PIECE_ORDER, math.floor(tonumber(snap.next_kind) or random_piece_kind())))

    local elapsed = math.max(0, math.floor(tonumber(snap.elapsed) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = elapsed
    if state.active == nil then spawn_active() end
    state.toast_text = tr("game.tetris.continue_loaded")
    state.toast_until = state.frame + FPS * 2
    state.dirty = true
    return true
end

local function reset_run(start_level)
    state.level = start_level or 0
    state.start_level = state.level
    state.reincarnated = false
    state.phase = 1
    state.lines_total = 0
    state.lines_for_level = 0
    state.score = 0
    state.counters = { single = 0, double = 0, triple = 0, tetris = 0 }
    state.piece_used = { I = 0, O = 0, T = 0, Z = 0, L = 0, S = 0, J = 0 }
    state.game_over = false
    state.end_frame = nil
    state.result_committed = false
    state.confirm_mode = nil
    state.input_mode = nil
    state.input_buffer = ""
    state.toast_text = nil
    state.toast_until = 0
    state.drop_accum = 0
    state.start_frame = state.frame
    state.last_auto_save_sec = 0
    state.last_elapsed_sec = -1
    state.last_toast_visible = false
    state.color_level = -999
    update_level_colors(true)
    init_board()
    state.next_kind = random_piece_kind()
    spawn_active()
    state.dirty = true
end

local function centered_x(text, min_x, max_x)
    min_x = min_x or 1
    max_x = max_x or terminal_size()
    local span = (max_x - min_x + 1)
    if span <= 1 then return min_x end
    local w = text_width(text)
    local x = min_x + math.floor((span - w) / 2)
    if x < min_x then x = min_x end
    return x
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local row = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, row, "white", bg or "black")
    end
end

local function draw_padded(x, y, w, text, fg, bg, align)
    local blank = string.rep(" ", w)
    draw_text(x, y, blank, "white", bg or "black")
    if text == nil or text == "" then return end
    local tx = x
    local tw = text_width(text)
    if align == "right" then
        tx = x + math.max(0, w - tw)
    elseif align == "center" then
        tx = x + math.max(0, math.floor((w - tw) / 2))
    end
    draw_text(tx, y, text, fg or "white", bg or "black")
end

local function build_layout()
    local term_w, term_h = terminal_size()
    local board_x = math.floor((term_w - FRAME_W) / 2) + 1
    if board_x < 1 then board_x = 1 end
    local board_y = math.floor((term_h - (FRAME_H + 4)) / 2) + 1
    if board_y < 2 then board_y = 2 end

    local left_x = board_x - 1 - LEFT_W
    local right_x = board_x + FRAME_W + SIDE_GAP
    local msg_y = board_y + FRAME_H
    local controls_y = msg_y + 1

    local total_l = left_x
    local total_r = right_x + RIGHT_W - 1
    local area_x = total_l
    local area_y = board_y
    local area_w = total_r - total_l + 1
    local area_h = controls_y - board_y + 2

    return {
        term_w = term_w,
        term_h = term_h,
        board_x = board_x,
        board_y = board_y,
        left_x = left_x,
        left_y = board_y,
        right_x = right_x,
        right_y = board_y,
        msg_y = msg_y,
        controls_y = controls_y,
        area_x = area_x,
        area_y = area_y,
        area_w = area_w,
        area_h = area_h,
    }
end

local function controls_text()
    return tr("game.tetris.controls")
end

local function minimum_required_size()
    local controls_w = min_width_for_lines(controls_text(), 3, 60)
    local content_w = LEFT_W + SIDE_GAP + FRAME_W + SIDE_GAP + RIGHT_W
    local msg_w = math.max(
        text_width(tr("game.tetris.confirm_exit")),
        text_width(tr("game.tetris.confirm_restart")),
        text_width(tr("game.tetris.lose_banner") .. " " .. tr("game.tetris.result_controls"))
    )
    local min_w = math.max(content_w, controls_w, msg_w) + 2
    local min_h = FRAME_H + 6
    return min_w, min_h
end

local function draw_size_warning(term_w, term_h, min_w, min_h)
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint"),
        tr("warning.back_to_game_list_hint"),
    }

    clear()
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end
    for i = 1, #lines do
        local line = lines[i]
        draw_text(centered_x(line, 1, term_w), top + i - 1, line, "white", "black")
    end
end

local function ensure_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()
    state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
    state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
    state.size_warning_active = not (term_w >= min_w and term_h >= min_h)
    return not state.size_warning_active
end

local function force_full_refresh()
    clear()
    state.last_layout = nil
    state.dirty = true
end

local function draw_frame(layout)
    local x, y = layout.board_x, layout.board_y
    draw_text(x, y, BORDER_TL .. string.rep(BORDER_H, FRAME_W - 2) .. BORDER_TR, "blue", "black")
    for r = 1, FRAME_H - 2 do
        draw_text(x, y + r, BORDER_V, "blue", "black")
        draw_text(x + FRAME_W - 1, y + r, BORDER_V, "blue", "black")
    end
    draw_text(x, y + FRAME_H - 1, BORDER_BL .. string.rep(BORDER_H, FRAME_W - 2) .. BORDER_BR, "blue", "black")
end

local function build_active_map()
    local map = {}
    if state.active == nil then return map end
    local cells = get_cells(state.active.kind, state.active.rot)
    for i = 1, #cells do
        local cx = state.active.x + cells[i][1]
        local cy = state.active.y + cells[i][2]
        if cx >= 1 and cx <= BOARD_W and cy >= 1 and cy <= BOARD_H then
            map[cy .. "," .. cx] = state.active.kind
        end
    end
    return map
end

local function draw_board_content(layout)
    local inner_x = layout.board_x + 1
    local inner_y = layout.board_y + 1
    local active_map = build_active_map()

    for y = 1, BOARD_H do
        draw_text(inner_x, inner_y + y - 1, string.rep(" ", BOARD_W * 2), "white", "black")
        for x = 1, BOARD_W do
            local v = state.board[y][x]
            local color = nil
            if type(v) == "table" then
                if type(v.kind) == "number" then
                    color = v.color or color_for_kind(v.kind)
                end
            elseif type(v) == "number" and v ~= 0 then
                color = color_for_kind(v)
            elseif type(v) == "string" and v ~= "0" and v ~= "" then
                color = v
            end
            local a = active_map[y .. "," .. x]
            if a ~= nil then color = color_for_kind(a) end
            if color ~= nil and color ~= 0 then
                draw_text(inner_x + (x - 1) * 2, inner_y + y - 1, CELL, color, "black")
            end
        end
    end
end

local function draw_next_preview(layout)
    local kind = state.next_kind
    local cells = get_cells(kind, 0)
    local min_x, min_y, max_x, max_y = 99, 99, -99, -99
    for i = 1, #cells do
        local c = cells[i]
        if c[1] < min_x then min_x = c[1] end
        if c[2] < min_y then min_y = c[2] end
        if c[1] > max_x then max_x = c[1] end
        if c[2] > max_y then max_y = c[2] end
    end
    local w = (max_x - min_x + 1) * 2
    local h = (max_y - min_y + 1)
    local box_x = layout.right_x
    local box_y = layout.right_y + 10
    local center_x = box_x + 1
    local center_y = box_y + math.floor((5 - h) / 2)

    fill_rect(box_x, box_y, 8, 5, "black")
    local color = color_for_kind(kind)
    for i = 1, #cells do
        local cx = (cells[i][1] - min_x)
        local cy = (cells[i][2] - min_y)
        draw_text(center_x + cx * 2, center_y + cy, CELL, color, "black")
    end
end

local function draw_left_panel(layout)
    local x, y = layout.left_x, layout.left_y
    local lines = {
        string.format("SINGLE %s", fmt5(state.counters.single)),
        string.format("DOUBLE %s", fmt5(state.counters.double)),
        string.format("TRIPLE %s", fmt5(state.counters.triple)),
        string.format("TETRIS %s", fmt5(state.counters.tetris)),
        string.format("LINES  %s", fmt5(state.lines_total)),
        "",
        string.format("I %s", fmt5(state.piece_used.I)),
        string.format("O %s", fmt5(state.piece_used.O)),
        string.format("T %s", fmt5(state.piece_used.T)),
        string.format("Z %s", fmt5(state.piece_used.Z)),
        string.format("L %s", fmt5(state.piece_used.L)),
        string.format("S %s", fmt5(state.piece_used.S)),
        string.format("J %s", fmt5(state.piece_used.J)),
    }
    local block_h = #lines
    local start_y = y + math.floor((FRAME_H - block_h) / 2)
    if start_y < y then start_y = y end
    for i = 1, #lines do
        local t = lines[i]
        if t ~= "" then
            draw_padded(x, start_y + i - 1, LEFT_W, t, "white", "black", "right")
        else
            draw_padded(x, start_y + i - 1, LEFT_W, "", "white", "black", "right")
        end
    end
end

local function draw_right_panel(layout)
    local x, y = layout.right_x, layout.right_y
    draw_padded(x, y + 0, RIGHT_W, tr("game.tetris.best_score"), "dark_gray", "black", "left")
    draw_padded(x, y + 1, RIGHT_W, tostring(state.best_score), "white", "black", "left")

    draw_padded(x, y + 3, RIGHT_W, tr("game.tetris.current_score"), "dark_gray", "black", "left")
    draw_padded(x, y + 4, RIGHT_W, tostring(state.score), "white", "black", "left")

    draw_padded(x, y + 6, RIGHT_W, tr("game.tetris.time"), "dark_gray", "black", "left")
    draw_padded(x, y + 7, RIGHT_W, format_duration(elapsed_seconds()), "light_cyan", "black", "left")

    draw_padded(x, y + 9, RIGHT_W, tr("game.tetris.next"), "white", "black", "left")
    draw_next_preview(layout)

    draw_padded(x, y + 18, RIGHT_W, string.format("LV %d", state.level), "white", "black", "left")
    draw_padded(x, y + 19, RIGHT_W, "", "white", "black", "left")
    local prefix = tr("game.tetris.stage") .. " "
    draw_text(x, y + 19, prefix, "white", "black")
    draw_text(x + text_width(prefix), y + 19, stage_label(), stage_color(), "black")
end

local function current_message()
    if state.game_over then
        return tr("game.tetris.lose_banner") .. " "
            .. tr("game.tetris.result_controls"), "red"
    end
    if state.confirm_mode == "restart" then
        return tr("game.tetris.confirm_restart"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.tetris.confirm_exit"), "yellow"
    end
    if state.input_mode == "level" then
        local p = tr("game.tetris.input_level")
        return p .. state.input_buffer, "yellow"
    end
    if state.toast_text ~= nil and state.frame < state.toast_until then
        return state.toast_text, "light_green"
    end
    return "", "white"
end

local function draw_message_and_controls(layout)
    local msg, msg_color = current_message()
    draw_text(1, layout.msg_y, string.rep(" ", layout.term_w), "white", "black")
    if msg ~= "" then
        draw_text(centered_x(msg, 1, layout.term_w), layout.msg_y, msg, msg_color, "black")
    end

    for i = 0, 2 do
        draw_text(1, layout.controls_y + i, string.rep(" ", layout.term_w), "white", "black")
    end
    local lines = wrap_words(controls_text(), layout.term_w - 2)
    local offset = math.max(0, 3 - #lines)
    for i = 1, #lines do
        draw_text(centered_x(lines[i], 1, layout.term_w), layout.controls_y + offset + i - 1, lines[i], "white", "black")
    end
end

local function render_frame()
    local layout = build_layout()
    local l = state.last_layout
    if l == nil
        or l.board_x ~= layout.board_x
        or l.board_y ~= layout.board_y
        or l.left_x ~= layout.left_x
        or l.right_x ~= layout.right_x
        or l.term_w ~= layout.term_w
        or l.term_h ~= layout.term_h then
        clear()
        state.last_layout = {
            board_x = layout.board_x,
            board_y = layout.board_y,
            left_x = layout.left_x,
            right_x = layout.right_x,
            term_w = layout.term_w,
            term_h = layout.term_h,
        }
    end
    draw_frame(layout)
    draw_board_content(layout)
    draw_left_panel(layout)
    draw_right_panel(layout)
    draw_message_and_controls(layout)
end

local function commit_result_once()
    if state.result_committed then return end
    if state.score > state.best_score then
        state.best_score = state.score
        if type(request_refresh_best_score) == "function" then
            pcall(request_refresh_best_score)
        end
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "tetris", state.score, elapsed_seconds())
    end
    state.result_committed = true
end

local function update_toast_timer()
    local visible = (state.toast_text ~= nil and state.frame < state.toast_until)
    if state.last_toast_visible ~= visible then
        state.last_toast_visible = visible
        state.dirty = true
    end
end

local function refresh_dirty_time()
    local sec = elapsed_seconds()
    if sec ~= state.last_elapsed_sec then
        state.last_elapsed_sec = sec
        state.dirty = true
    end
end

local function sync_resize()
    local w, h = terminal_size()
    if w ~= state.last_warn_term_w or h ~= state.last_warn_term_h then
        state.last_warn_term_w, state.last_warn_term_h = w, h
        force_full_refresh()
    end
end

local function gameplay_update()
    if state.game_over then return end
    if state.confirm_mode ~= nil or state.input_mode ~= nil then return end
    if state.active == nil then spawn_active() end
    if state.active == nil then return end

    state.drop_accum = state.drop_accum + 1
    if state.drop_accum >= frames_per_drop(state.level) then
        state.drop_accum = 0
        if not try_move(0, 1) then
            settle_and_respawn()
        end
    end

    local sec = elapsed_seconds()
    if sec - state.last_auto_save_sec >= 60 then
        state.last_auto_save_sec = sec
        save_snapshot(false)
    end
end

local function handle_confirm_key(key)
    if state.confirm_mode == nil then return false end
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            commit_result_once()
            reset_run(state.start_level)
        else
            commit_result_once()
            exit_game()
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

local function handle_input_mode(key)
    if state.input_mode ~= "level" then return false end
    if key == "esc" or key == "q" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return true
    end
    if key == "backspace" or key == "delete" then
        local n = #state.input_buffer
        if n > 0 then
            state.input_buffer = string.sub(state.input_buffer, 1, n - 1)
            state.dirty = true
        end
        return true
    end
    if key == "enter" then
        local v = tonumber(state.input_buffer)
        if v ~= nil and v >= 0 and v <= 28 then
            state.input_mode = nil
            state.input_buffer = ""
            reset_run(math.floor(v))
        else
            state.toast_text = tr("game.tetris.input_invalid")
            state.toast_until = state.frame + FPS * 2
            state.dirty = true
        end
        return true
    end
    if #key == 1 and key >= "0" and key <= "9" and #state.input_buffer < 2 then
        state.input_buffer = state.input_buffer .. key
        state.dirty = true
        return true
    end
    return true
end

local function handle_input(key)
    if key == nil or key == "" then return end

    if state.confirm_mode ~= nil then
        handle_confirm_key(key)
        return
    end
    if state.input_mode ~= nil then
        handle_input_mode(key)
        return
    end

    if state.game_over then
        if key == "r" then
            reset_run(state.start_level)
            return
        end
        if key == "q" or key == "esc" then
            commit_result_once()
            exit_game()
            return
        end
        return
    end

    if key == "left" then
        try_move(-1, 0)
        return
    end
    if key == "right" then
        try_move(1, 0)
        return
    end
    if key == "z" then
        try_rotate(-1)
        return
    end
    if key == "x" then
        try_rotate(1)
        return
    end
    if key == "down" and can_soft_drop() then
        do_soft_drop()
        return
    end
    if key == "space" then
        do_hard_drop()
        return
    end
    if key == "s" then
        save_snapshot(true)
        return
    end
    if key == "p" then
        state.input_mode = "level"
        state.input_buffer = ""
        state.dirty = true
        return
    end
    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return
    end
end

local function bootstrap_game()
    clear()
    state.launch_mode = read_launch_mode()
    load_best_record()
    state.frame = 0
    state.last_warn_term_w, state.last_warn_term_h = terminal_size()

    if state.launch_mode == "continue" then
        if not load_snapshot() then
            reset_run(0)
        end
    else
        reset_run(0)
    end

    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

function init_game()
    bootstrap_game()
    return state
end

function handle_event(state_arg, event)
    state = state_arg or state
    local key = normalize_key(event)

    if ensure_size_ok() then
        if key ~= "" then
            handle_input(key)
        end
        if type(event) == "table" and event.type == "tick" then
            gameplay_update()
            update_toast_timer()
            refresh_dirty_time()
            sync_resize()
            state.frame = state.frame + 1
        else
            update_toast_timer()
            refresh_dirty_time()
            sync_resize()
        end
    else
        if key == "q" or key == "esc" then
            commit_result_once()
            exit_game()
            return state
        end
        if type(event) == "table" and event.type == "tick" then
            state.frame = state.frame + 1
        end
    end

    return state
end

function render(state_arg)
    state = state_arg or state
    if state.size_warning_active then
        clear()
        draw_size_warning(
            state.last_warn_term_w or 80,
            state.last_warn_term_h or 24,
            state.last_warn_min_w or 0,
            state.last_warn_min_h or 0
        )
        return
    end
    render_frame()
end

function best_score(state_arg)
    state = state_arg or state
    if state.best_score <= 0 then return nil end
    return {
        best_string = "game.tetris.best_block",
        score = state.best_score,
    }
end
