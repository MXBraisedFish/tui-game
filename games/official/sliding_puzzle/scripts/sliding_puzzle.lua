local host_random = random

local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text, fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    request_exit()
end

local SIZE = 4
local FPS = 60
local FRAME_MS = 16
local CELL_W = 8 -- 姣忎釜鍗曞厓鏍煎搴?
local CELL_H = 4 -- 姣忎釜鍗曞厓鏍奸珮搴?

-- 杈规瀛楃
local BORDER_TL = "\u{2554}" -- 鈹?
local BORDER_TR = "\u{2557}" -- 鈹?
local BORDER_BL = "\u{255A}" -- 鈹?
local BORDER_BR = "\u{255D}" -- 鈹?
local BORDER_H = "\u{2550}"  -- 鈹€
local BORDER_V = "\u{2551}"  -- 鈹?

-- 娓告垙鐘舵€佽〃
local state = {
    -- 妫嬬洏
    board = {},
    steps = 0,
    won = false,
    confirm_mode = nil,
    move_mode = "blank", -- "blank" 鎴?"number"

    -- 鏃堕棿鐩稿叧
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    last_auto_save_sec = 0,

    -- 鎻愮ず娑堟伅
    toast_text = nil,
    toast_until = 0,

    -- 娓叉煋鐩稿叧
    dirty = true,
    last_elapsed_sec = -1,
    last_toast_visible = false,

    -- 杈撳叆闃叉姈
    last_key = "",
    last_key_frame = -100,

    -- 鍚姩妯″紡
    launch_mode = "new",
    last_area = nil,
    last_term_w = 0,
    last_term_h = 0,

    -- 灏哄璀﹀憡
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,

    -- 鏈€浣宠褰?
    best_steps = 0,
    best_time_sec = 0,
    result_committed = false,
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

-- 鑾峰彇鏂囨湰鏄剧ず瀹藉害
local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
end

-- 鑾峰彇缁堢灏哄
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

-- 娣卞害澶嶅埗妫嬬洏
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

-- 鐢熸垚宸茶В鍐崇殑妫嬬洏
local function solved_board()
    local board = {}
    local v = 1
    for r = 1, SIZE do
        board[r] = {}
        for c = 1, SIZE do
            if r == SIZE and c == SIZE then
                board[r][c] = 0 -- 绌烘牸
            else
                board[r][c] = v
                v = v + 1
            end
        end
    end
    return board
end

-- 鏌ユ壘绌烘牸浣嶇疆
local function find_blank(board)
    for r = 1, SIZE do
        for c = 1, SIZE do
            if board[r][c] == 0 then
                return r, c
            end
        end
    end
    return SIZE, SIZE
end

-- 妫€鏌ユ槸鍚﹀彲浠ョЩ鍔ㄧ┖鏍?
local function can_move_blank(board, dir)
    local br, bc = find_blank(board)
    if dir == "up" then return br > 1 end
    if dir == "down" then return br < SIZE end
    if dir == "left" then return bc > 1 end
    if dir == "right" then return bc < SIZE end
    return false
end

-- 绉诲姩绌烘牸
local function move_blank(board, dir)
    local br, bc = find_blank(board)
    local tr, tc = br, bc

    if dir == "up" then
        tr = br - 1
    elseif dir == "down" then
        tr = br + 1
    elseif dir == "left" then
        tc = bc - 1
    elseif dir == "right" then
        tc = bc + 1
    else
        return false
    end

    if tr < 1 or tr > SIZE or tc < 1 or tc > SIZE then
        return false
    end

    -- 浜ゆ崲绌烘牸鍜岀浉閭绘暟瀛?
    board[br][bc], board[tr][tc] = board[tr][tc], board[br][bc]
    return true
end

-- 鑾峰彇鐩稿弽鏂瑰悜
local function opposite_dir(dir)
    if dir == "up" then return "down" end
    if dir == "down" then return "up" end
    if dir == "left" then return "right" end
    if dir == "right" then return "left" end
    return ""
end

-- 闅忔満鏁存暟 [1, n]
local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then
        return 0
    end
    return host_random(n)
end

-- 闅忔満鎵撲贡妫嬬洏
local function scramble_board(board)
    local steps = 80 + rand_int(41) -- 80~120姝?
    local dirs = { "up", "down", "left", "right" }
    local prev = ""

    for _ = 1, steps do
        -- 鏀堕泦鍙敤鐨勬柟鍚?
        local available = {}
        for i = 1, #dirs do
            local d = dirs[i]
            if can_move_blank(board, d) then
                -- 浼樺厛閬垮厤绔嬪嵆寰€鍥炶蛋
                if #available == 0 or d ~= opposite_dir(prev) then
                    available[#available + 1] = d
                end
            end
        end

        -- 濡傛灉娌℃湁閬垮紑寰€鍥炶蛋鐨勬柟鍚戯紝鎺ュ彈浠讳綍鍙敤鏂瑰悜
        if #available == 0 then
            for i = 1, #dirs do
                local d = dirs[i]
                if can_move_blank(board, d) then
                    available[#available + 1] = d
                end
            end
        end

        local pick = available[rand_int(#available) + 1]
        if pick ~= nil then
            move_blank(board, pick)
            prev = pick
        end
    end
end

-- 妫€鏌ユ槸鍚﹀凡瑙ｅ喅
local function is_solved(board)
    local expected = 1
    for r = 1, SIZE do
        for c = 1, SIZE do
            if r == SIZE and c == SIZE then
                if board[r][c] ~= 0 then
                    return false
                end
            else
                if board[r][c] ~= expected then
                    return false
                end
                expected = expected + 1
            end
        end
    end
    return true
end

-- 璁＄畻宸茶繃绉掓暟
local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

-- 鏍煎紡鍖栨椂闂?
local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

-- 瑙勮寖鍖栨寜閿?
local function normalize_key(key)
    if key == nil then
        return ""
    end
    if type(key) == "string" then
        return string.lower(key)
    end
    if type(key) == "table" then
        if type(key.type) == "string" then
            local event_type = string.lower(key.type)
            if event_type == "quit" then
                return "esc"
            end
            if event_type == "key" and type(key.name) == "string" then
                return string.lower(key.name)
            end
            if event_type == "action" and type(key.name) == "string" then
                local map = {
                    move_up = "up",
                    move_down = "down",
                    move_left = "left",
                    move_right = "right",
                    toggle_mode = "x",
                    save = "s",
                    restart = "r",
                    quit_action = "q",
                    confirm_yes = "enter",
                    confirm_no = "esc",
                }
                local mapped = map[string.lower(key.name)]
                if mapped ~= nil then
                    return mapped
                end
                return string.lower(key.name)
            end
        end
        if type(key.code) == "string" then
            return string.lower(key.code)
        end
    end
    return tostring(key):lower()
end
-- 鎸夊崟璇嶆崲琛?
local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end

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

    if not had then
        return { "" }
    end
    if current ~= "" then
        lines[#lines + 1] = current
    end
    return lines
end

-- 璁＄畻鏈€灏忓搴?
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

-- 濉厖鐭╁舰鍖哄煙
local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
end

-- 缁樺埗澶栬竟妗?
local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, BORDER_TL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_TR, "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, BORDER_V, "white", "black")
        draw_text(x + frame_w - 1, y + i, BORDER_V, "white", "black")
    end
    draw_text(x, y + frame_h - 1, BORDER_BL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_BR, "white", "black")
end

-- 璁＄畻妫嬬洏鍑犱綍甯冨眬
local function board_geometry()
    local w, h = terminal_size()
    local grid_w = SIZE * CELL_W
    local grid_h = SIZE * CELL_H

    local status_w = text_width(tr("game.sliding_puzzle.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.sliding_puzzle.steps") .. " 99999")
    local best_w = text_width(
        tr("game.sliding_puzzle.best_title")
        .. "  "
        .. tr("game.sliding_puzzle.best_steps")
        .. " "
        .. tostring(math.max(0, state.best_steps))
        .. "  "
        .. tr("game.sliding_puzzle.best_time")
        .. " "
        .. format_duration(math.max(0, state.best_time_sec))
    )

    local frame_w = math.max(grid_w, status_w, best_w) + 2
    local frame_h = grid_h + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 5 then y = 5 end
    return x, y, frame_w, frame_h
end

-- 缁樺埗鍗曚釜鏂瑰潡
local function draw_tile(tile_x, tile_y, value)
    local bg = (value == 0) and "rgb(80,80,80)" or "rgb(255,255,255)"
    local fg = "black"

    -- 缁樺埗鑳屾櫙
    for row = 0, CELL_H - 1 do
        draw_text(tile_x, tile_y + row, string.rep(" ", CELL_W), fg, bg)
    end

    -- 缁樺埗鏁板瓧
    if value ~= 0 then
        local text = tostring(value)
        local tx = tile_x + math.floor((CELL_W - #text) / 2)
        local ty = tile_y + math.floor(CELL_H / 2)
        draw_text(tx, ty, text, fg, bg)
    end
end

-- 鑾峰彇鏈€浣宠褰曟枃鏈?
local function best_line_text()
    if state.best_steps <= 0 then
        return tr("game.sliding_puzzle.best_none")
    end
    return tr("game.sliding_puzzle.best_title")
        .. "  "
        .. tr("game.sliding_puzzle.best_steps")
        .. " " .. tostring(state.best_steps)
        .. "  "
        .. tr("game.sliding_puzzle.best_time")
        .. " " .. format_duration(state.best_time_sec)
end

-- 鑾峰彇绉诲姩妯″紡鏂囨湰
local function move_mode_text()
    if state.move_mode == "number" then
        return tr("game.sliding_puzzle.mode_number")
    end
    return tr("game.sliding_puzzle.mode_blank")
end

-- 缁樺埗鐘舵€佹爮
local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local left = tr("game.sliding_puzzle.time") .. " " .. format_duration(elapsed)
    local right = tr("game.sliding_puzzle.steps") .. " " .. tostring(state.steps)

    local term_w = terminal_size()
    local right_x = x + frame_w - text_width(right)
    if right_x < 1 then right_x = 1 end

    -- 娓呯┖鐘舵€佸尯鍩?
    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    -- 鏄剧ず鏈€浣宠褰曘€佹椂闂淬€佹鏁?
    draw_text(x, y - 3, best_line_text(), "dark_gray", "black")
    draw_text(x, y - 2, left, "light_cyan", "black")
    draw_text(right_x, y - 2, right, "light_cyan", "black")

    -- 鏄剧ず鎻愮ず淇℃伅
    if state.won then
        local line = tr("game.sliding_puzzle.win_banner")
            .. tr("game.sliding_puzzle.win_controls")
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, tr("game.sliding_puzzle.confirm_restart"), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, tr("game.sliding_puzzle.confirm_exit"), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

-- 缁樺埗鎺у埗璇存槑
local function draw_controls(y_bottom)
    local term_w = terminal_size()
    local controls = tr("game.sliding_puzzle.controls")
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(controls, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    -- 娓呯┖鎺у埗鍖哄煙
    draw_text(1, y_bottom + 1, string.rep(" ", term_w), "white", "black")
    draw_text(1, y_bottom + 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y_bottom + 3, string.rep(" ", term_w), "white", "black")

    -- 鍨傜洿灞呬腑
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    -- 缁樺埗鎺у埗璇存槑
    for i = 1, #lines do
        local line = lines[i]
        local cx = math.floor((term_w - text_width(line)) / 2)
        if cx < 1 then cx = 1 end
        draw_text(cx, y_bottom + 1 + offset + i - 1, line, "white", "black")
    end
end

-- 缁樺埗绉诲姩妯″紡鎻愮ず
local function draw_move_mode(y_bottom)
    local term_w = terminal_size()
    local line = tr("game.sliding_puzzle.mode_label")
        .. ": "
        .. move_mode_text()
    local cx = math.floor((term_w - text_width(line)) / 2)
    if cx < 1 then cx = 1 end
    draw_text(1, y_bottom, string.rep(" ", term_w), "white", "black")
    draw_text(cx, y_bottom, line, "dark_gray", "black")
end

-- 娓呴櫎涓婃娓叉煋鐨勫尯鍩?
local function clear_last_area()
    if state.last_area == nil then return end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

-- 涓绘覆鏌撳嚱鏁?
local function render_frame()
    local x, y, frame_w, frame_h = board_geometry()
    local area = { x = x, y = y - 3, w = frame_w, h = frame_h + 7 }

    -- 濡傛灉娓叉煋鍖哄煙鍙樺寲锛屾竻闄ゆ棫鍖哄煙
    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y
        or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    -- 缁樺埗鍚勭粍浠?
    draw_status(x, y, frame_w)
    draw_outer_frame(x, y, frame_w, frame_h)

    -- 缁樺埗妫嬬洏
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

    draw_move_mode(y + frame_h)
    draw_controls(y + frame_h)
end

-- 鎻愪氦娓告垙缁撴灉
local function commit_result_once()
    if state.result_committed then return end
    state.result_committed = true

    if not state.won then
        return
    end

    local elapsed = elapsed_seconds()
    local improved = false

    -- 鍒ゆ柇鏄惁鍒锋柊鏈€浣宠褰?
    if state.best_steps <= 0 or state.steps < state.best_steps then
        improved = true
    elseif state.steps == state.best_steps and (state.best_time_sec <= 0 or elapsed < state.best_time_sec) then
        improved = true
    end

    if improved then
        state.best_steps = state.steps
        state.best_time_sec = elapsed
        if type(save_data) == "function" then
            pcall(save_data, "sliding_puzzle_best", { steps = state.best_steps, time_sec = state.best_time_sec })
        end
    end

    -- 鏇存柊鍏ㄥ眬缁熻
    if type(update_game_stats) == "function" then
        local score = math.max(0, 100000 - state.steps * 100 - elapsed)
        pcall(update_game_stats, "sliding_puzzle", score, elapsed)
    end
end

-- 鍔犺浇鏈€浣宠褰?
local function load_best_record()
    if type(load_data) ~= "function" then
        state.best_steps = 0
        state.best_time_sec = 0
        return
    end

    local ok, data = pcall(load_data, "sliding_puzzle_best")
    if not ok or type(data) ~= "table" then
        state.best_steps = 0
        state.best_time_sec = 0
        return
    end

    state.best_steps = math.max(0, math.floor(tonumber(data.steps) or 0))
    state.best_time_sec = math.max(0, math.floor(tonumber(data.time_sec) or 0))
end

-- 楠岃瘉妫嬬洏鍊兼槸鍚︽湁鏁?
local function validate_board_values(board)
    local seen = {}
    for r = 1, SIZE do
        if type(board[r]) ~= "table" then return false end
        for c = 1, SIZE do
            local v = math.floor(tonumber(board[r][c]) or -1)
            if v < 0 or v > 15 then return false end
            if seen[v] then return false end
            seen[v] = true
        end
    end
    return true
end

-- 鍒涘缓娓告垙蹇収
local function make_snapshot()
    return {
        board = deep_copy_board(state.board),
        steps = state.steps,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        move_mode = state.move_mode,
    }
end

-- 鎭㈠娓告垙蹇収
local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" or type(snapshot.board) ~= "table" then
        return false
    end
    if not validate_board_values(snapshot.board) then
        return false
    end

    state.board = deep_copy_board(snapshot.board)
    state.steps = math.max(0, math.floor(tonumber(snapshot.steps) or 0))

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = elapsed

    state.won = snapshot.won == true or is_solved(state.board)
    if snapshot.move_mode == "number" then
        state.move_mode = "number"
    else
        state.move_mode = "blank"
    end
    state.confirm_mode = nil
    state.toast_text = nil
    state.toast_until = 0
    state.end_frame = state.won and state.frame or nil
    state.result_committed = false
    if state.won then commit_result_once() end

    state.dirty = true
    return true
end

-- 淇濆瓨娓告垙鐘舵€?
local function save_game_state(show_toast)
    local ok = false
    local snapshot = make_snapshot()

    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "sliding_puzzle", snapshot)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "sliding_puzzle", snapshot)
        ok = s and ret ~= false
    elseif type(save_game) == "function" then
        local s, ret = pcall(save_game, snapshot)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.sliding_puzzle.save_success" or "game.sliding_puzzle.save_unavailable"
        local def = ok and "Save successful!" or "Save API unavailable."
        state.toast_text = tr(key)
        state.toast_until = state.frame + 2 * FPS
        state.dirty = true
    end
end

-- 鍔犺浇娓告垙鐘舵€?
local function load_game_state()
    local ok, snapshot = false, nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "sliding_puzzle")
        ok = s and ret ~= nil
        snapshot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "sliding_puzzle")
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

-- 璇诲彇鍚姩妯″紡
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

-- 閲嶇疆娓告垙
local function reset_game()
    state.board = solved_board()
    repeat
        scramble_board(state.board)
    until not is_solved(state.board)
    state.steps = 0
    state.won = false
    state.confirm_mode = nil
    state.move_mode = "blank"
    state.start_frame = state.frame
    state.end_frame = nil
    state.last_auto_save_sec = 0
    state.toast_text = nil
    state.toast_until = 0
    state.result_committed = false
    state.dirty = true
end

-- 闃叉姈澶勭悊
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

-- 澶勭悊纭妯″紡涓嬬殑鎸夐敭
local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            reset_game()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_result_once()
            return "exit"
        end
    end

    if key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

-- 涓昏緭鍏ュ鐞嗗嚱鏁?
local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end
    if should_debounce(key) then
        return "none"
    end

    -- 纭妯″紡
    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    -- 鍒囨崲绉诲姩妯″紡
    if key == "x" then
        if state.move_mode == "blank" then
            state.move_mode = "number"
        else
            state.move_mode = "blank"
        end
        state.dirty = true
        return "changed"
    end

    -- 鑳滃埄鐘舵€?
    if state.won then
        if key == "r" then
            reset_game()
            return "changed"
        end
        if key == "q" or key == "esc" then
            commit_result_once()
            return "exit"
        end
        if key == "s" then
            save_game_state(true)
            return "changed"
        end
        return "none"
    end

    -- 鍔熻兘閿?
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

    -- 鏂瑰悜閿Щ鍔?
    if key == "up" or key == "down" or key == "left" or key == "right" then
        local move_dir = key
        if state.move_mode == "number" then
            -- 鏁板瓧妯″紡锛氭柟鍚戦敭绉诲姩鏁板瓧锛屽疄闄呬笂鏄Щ鍔ㄧ┖鏍肩殑鐩稿弽鏂瑰悜
            move_dir = opposite_dir(key)
        end
        local moved = move_blank(state.board, move_dir)
        if moved then
            state.steps = state.steps + 1
            if is_solved(state.board) then
                state.won = true
                state.end_frame = state.frame
                commit_result_once()
            end
            state.dirty = true
            return "changed"
        end
    end

    return "none"
end

-- 鑷姩淇濆瓨
local function auto_save_if_needed()
    local elapsed = elapsed_seconds()
    if elapsed - state.last_auto_save_sec >= 60 then
        save_game_state(false)
        state.last_auto_save_sec = elapsed
    end
end

-- 鍒锋柊鑴忔爣璁?
local function refresh_dirty_flags()
    local elapsed = math.floor((state.frame - state.start_frame) / FPS)
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

-- 鍚屾缁堢灏哄鍙樺寲
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

-- 璁＄畻鏈€灏忔墍闇€缁堢灏哄
local function minimum_required_size()
    local frame_w = SIZE * CELL_W + 2
    local frame_h = SIZE * CELL_H + 2

    local controls_w = min_width_for_lines(
        tr("game.sliding_puzzle.controls"),
        3,
        26
    )

    local status_w = text_width(tr("game.sliding_puzzle.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.sliding_puzzle.steps") .. " 99999")

    local best_w = text_width(
        tr("game.sliding_puzzle.best_title")
        .. "  "
        .. tr("game.sliding_puzzle.best_steps")
        .. " 99999  "
        .. tr("game.sliding_puzzle.best_time")
        .. " 00:00:00"
    )

    local tip_w = math.max(
        text_width(tr("game.sliding_puzzle.confirm_restart")),
        text_width(tr("game.sliding_puzzle.confirm_exit")),
        text_width(tr("game.sliding_puzzle.win_banner") .. tr("game.sliding_puzzle.win_controls"))
    )
    local mode_w = text_width(tr("game.sliding_puzzle.mode_label") .. ": " .. tr("game.sliding_puzzle.mode_number"))
    local min_w = math.max(frame_w, controls_w, status_w, best_w, tip_w, mode_w) + 2
    local min_h = frame_h + 8
    return min_w, min_h
end

-- 缁樺埗缁堢灏哄璀﹀憡
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

-- 纭繚缁堢灏哄瓒冲
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

-- 娓告垙鍒濆鍖?
local function bootstrap_game()
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

    state.dirty = true
end

-- 涓绘父鎴忓惊鐜?
function init_game()
    bootstrap_game()
    return state
end

function handle_event(state_arg, event)
    state = state_arg or state
    sync_terminal_resize()
    local key = normalize_key(event)

    if ensure_terminal_size_ok() then
        if event == nil or event.type ~= "tick" then
            local action = handle_input(key)
            if action == "exit" then
                exit_game()
                return state
            end
        end

        auto_save_if_needed()
        refresh_dirty_flags()

        if event ~= nil and event.type == "tick" then
            state.frame = state.frame + 1
        end
    else
        if key == "q" or key == "esc" then
            exit_game()
        end
    end

    return state
end

function render(state_arg)
    state = state_arg or state
    sync_terminal_resize()
    if ensure_terminal_size_ok() then
        render_frame()
        state.dirty = false
    end
end

function best_score(state_arg)
    state = state_arg or state
    if state.best_steps <= 0 then
        return nil
    end
    return {
        best_string = "game.sliding_puzzle.best_block",
        steps = state.best_steps,
        time = format_duration(state.best_time_sec),
    }
end
