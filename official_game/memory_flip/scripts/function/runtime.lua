local Constants = load_function("/constants.lua")

local DEFAULT_DIFFICULTY = Constants.DEFAULT_DIFFICULTY
local MIN_DIFFICULTY = Constants.MIN_DIFFICULTY
local MAX_DIFFICULTY = Constants.MAX_DIFFICULTY
local DIFFICULTY_TO_SIZE = Constants.DIFFICULTY_TO_SIZE
local FPS = Constants.FPS
local FRAME_MS = Constants.FRAME_MS
local CELL_W = Constants.CELL_W
local CELL_H = Constants.CELL_H
local CELL_STEP_X = Constants.CELL_STEP_X
local CELL_STEP_Y = Constants.CELL_STEP_Y
local LABEL_W = Constants.LABEL_W
local HIDE_DELAY_MS = Constants.HIDE_DELAY_MS
local SYMBOLS = Constants.SYMBOLS
local PALETTE = Constants.PALETTE

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

local state = {
    difficulty = DEFAULT_DIFFICULTY,
    size = DIFFICULTY_TO_SIZE[DEFAULT_DIFFICULTY],

    board = {},                        -- 存储每张卡片的配对ID
    revealed = {},                     -- 是否已翻开（临时）
    matched = {},                      -- 是否已匹配（永久翻开）

    cursor_r = 1,
    cursor_c = 1,
    steps = 0,                         -- 翻牌步数

    frame = 0,
    start_frame = 0,
    end_frame = nil,
    won = false,
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
    launch_mode = "new",
    last_area = nil,

    best = nil,
    best_committed = false,

    first_pick = nil,                  -- 第一次翻开的卡片位置
    pending_hide = nil,                -- 等待隐藏的不匹配卡片
    pending_hide_timer_id = nil,       -- 等待盖回计时器 ID

    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0
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
    text = string.gsub(text, "%[R%]", key_label("restart"))
    return text
end

local function controls_text()
    return table.concat({
        key_label("move_up") .. "/" .. key_label("move_down") .. "/" .. key_label("move_left") .. "/" .. key_label("move_right") .. " " .. tr("game.memory_flip.action.move_up"),
        key_label("flip") .. " " .. tr("game.memory_flip.action.flip"),
        key_label("difficulty_input") .. " " .. tr("game.memory_flip.action.difficulty_input"),
        key_label("quick_jump") .. " " .. tr("game.memory_flip.action.quick_jump"),
        key_label("save") .. " " .. tr("game.memory_flip.action.save"),
        key_label("restart") .. " " .. tr("game.memory_flip.action.restart"),
        key_label("quit_action") .. " " .. tr("game.memory_flip.action.quit")
    }, "  ")
end

local function restart_quit_controls_text()
    return key_label("restart") .. " " .. tr("game.memory_flip.action.restart")
        .. "  " .. key_label("quit_action") .. " " .. tr("game.memory_flip.action.quit")
end

local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
end

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

local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
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
        flip = "flip",
        difficulty_input = "difficulty_input",
        quick_jump = "quick_jump",
        save = "save",
        confirm = "confirm",
        restart = "restart",
        quit_action = "quit_action",
        confirm_yes = "confirm_yes",
        confirm_no = "confirm_no",
        remove_last = "remove_last",
    }
    return map[event.name] or normalize_key(event.name)
end

local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function difficulty_to_size(difficulty)
    local d = clamp(difficulty, MIN_DIFFICULTY, MAX_DIFFICULTY)
    return DIFFICULTY_TO_SIZE[d]
end

local function size_to_difficulty(size)
    for difficulty = MIN_DIFFICULTY, MAX_DIFFICULTY do
        if DIFFICULTY_TO_SIZE[difficulty] == size then
            return difficulty
        end
    end
    return DEFAULT_DIFFICULTY
end

local function new_matrix(size, value)
    local matrix = {}
    for r = 1, size do
        matrix[r] = {}
        for c = 1, size do
            matrix[r][c] = value
        end
    end
    return matrix
end

local function copy_matrix(source, size)
    local matrix = new_matrix(size, false)
    for r = 1, size do
        for c = 1, size do
            matrix[r][c] = source[r][c]
        end
    end
    return matrix
end

local function pair_symbol(pair_id)
    local idx = ((pair_id - 1) % #SYMBOLS) + 1
    return SYMBOLS[idx]
end

local function pair_bg_color(pair_id)
    local idx = ((pair_id - 1) % #PALETTE) + 1
    return PALETTE[idx]
end

local function color_brightness(rgb)
    local r, g, b = rgb:match("^rgb%((%d+),(%d+),(%d+)%)$")
    if r == nil or g == nil or b == nil then
        return 0
    end
    local rr = tonumber(r) or 0
    local gg = tonumber(g) or 0
    local bb = tonumber(b) or 0
    return rr * 0.299 + gg * 0.587 + bb * 0.114
end

local function pair_text_color(pair_id)
    if color_brightness(pair_bg_color(pair_id)) >= 150 then
        return "black"
    end
    return "white"
end

local function shuffle_list(items)
    for i = #items, 2, -1 do
        local j = random_index(i) + 1
        items[i], items[j] = items[j], items[i]
    end
end

local function generate_board(size)
    local pair_count = (size * size) / 2
    local deck = {}
    for pair_id = 1, pair_count do
        deck[#deck + 1] = pair_id
        deck[#deck + 1] = pair_id
    end
    shuffle_list(deck)

    local board = new_matrix(size, 0)
    local index = 1
    for r = 1, size do
        for c = 1, size do
            board[r][c] = deck[index]
            index = index + 1
        end
    end
    return board
end

local function all_matched()
    for r = 1, state.size do
        for c = 1, state.size do
            if not state.matched[r][c] then
                return false
            end
        end
    end
    return true
end

local function stop_pending_hide_timer()
    if state.pending_hide_timer_id ~= nil and type(timer_kill) == "function" then
        pcall(timer_kill, state.pending_hide_timer_id)
    end
    state.pending_hide_timer_id = nil
end

local function start_pending_hide_timer()
    stop_pending_hide_timer()
    if type(timer_create) ~= "function" or type(timer_start) ~= "function" then
        return
    end
    local ok, id = pcall(timer_create, HIDE_DELAY_MS, "memory_flip_hide_pair")
    if ok and type(id) == "string" then
        state.pending_hide_timer_id = id
        pcall(timer_start, id)
    end
end

local function pending_hide_completed()
    if state.pending_hide == nil then
        return false
    end
    if state.pending_hide_timer_id ~= nil and type(is_timer_completed) == "function" then
        local ok, done = pcall(is_timer_completed, state.pending_hide_timer_id)
        if ok and done then return true end
    end
    return state.frame >= (state.pending_hide.until_frame or 0)
end

local function make_snapshot()
    local snapshot = {
        difficulty = state.difficulty,
        size = state.size,
        board = copy_matrix(state.board, state.size),
        revealed = copy_matrix(state.revealed, state.size),
        matched = copy_matrix(state.matched, state.size),
        cursor_r = state.cursor_r,
        cursor_c = state.cursor_c,
        steps = state.steps,
        elapsed_sec = elapsed_seconds(),
        won = state.won,
        last_auto_save_sec = state.last_auto_save_sec
    }

    if state.first_pick ~= nil then
        snapshot.first_pick = {
            r = state.first_pick.r,
            c = state.first_pick.c
        }
    end
    return snapshot
end

local function save_game_state(show_toast)
    local ok = false
    if type(request_save_game) == "function" then
        local s, ret = pcall(request_save_game)
        ok = s and ret ~= false
    end

    if show_toast then
        local key = ok and "game.memory_flip.save_success" or "game.memory_flip.save_unavailable"
        state.toast_text = tr(key)
        state.toast_until = state.frame + 2 * FPS
        state.dirty = true
    end
end

local function parse_saved_matrix(snapshot, key, size, default_value)
    if type(snapshot[key]) ~= "table" then
        return nil
    end
    local matrix = new_matrix(size, default_value)
    for r = 1, size do
        if type(snapshot[key][r]) ~= "table" then
            return nil
        end
        for c = 1, size do
            matrix[r][c] = snapshot[key][r][c]
        end
    end
    return matrix
end

local function restore_snapshot(snapshot)
    if type(snapshot) ~= "table" then
        return false
    end

    local difficulty = tonumber(snapshot.difficulty)
    local size = tonumber(snapshot.size)

    if difficulty == nil and size ~= nil then
        difficulty = size_to_difficulty(math.floor(size))
    end
    if difficulty == nil then
        return false
    end

    difficulty = clamp(math.floor(difficulty), MIN_DIFFICULTY, MAX_DIFFICULTY)
    size = difficulty_to_size(difficulty)

    local board = parse_saved_matrix(snapshot, "board", size, 0)
    local revealed = parse_saved_matrix(snapshot, "revealed", size, false)
    local matched = parse_saved_matrix(snapshot, "matched", size, false)
    if board == nil or revealed == nil or matched == nil then
        return false
    end

    state.difficulty = difficulty
    state.size = size
    state.board = board
    state.revealed = new_matrix(size, false)
    state.matched = new_matrix(size, false)

    for r = 1, size do
        for c = 1, size do
            state.matched[r][c] = not not matched[r][c]
            state.revealed[r][c] = state.matched[r][c] or not not revealed[r][c]
        end
    end

    state.cursor_r = clamp(math.floor(tonumber(snapshot.cursor_r) or 1), 1, size)
    state.cursor_c = clamp(math.floor(tonumber(snapshot.cursor_c) or 1), 1, size)
    state.steps = math.max(0, math.floor(tonumber(snapshot.steps) or 0))

    local elapsed = math.max(0, math.floor(tonumber(snapshot.elapsed_sec) or 0))
    state.start_frame = state.frame - elapsed * FPS
    state.last_auto_save_sec = math.max(
        0,
        math.floor(tonumber(snapshot.last_auto_save_sec) or elapsed)
    )

    state.won = not not snapshot.won
    state.end_frame = nil
    if state.won then
        state.end_frame = state.frame
    end

    state.first_pick = nil
    if type(snapshot.first_pick) == "table" then
        local r = clamp(math.floor(tonumber(snapshot.first_pick.r) or 0), 1, size)
        local c = clamp(math.floor(tonumber(snapshot.first_pick.c) or 0), 1, size)
        if not state.matched[r][c] then
            state.first_pick = { r = r, c = c }
            state.revealed[r][c] = true
        end
    end

    state.pending_hide = nil
    stop_pending_hide_timer()
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

local function load_game_state(snapshot)
    if type(snapshot) == "table" then
        return restore_snapshot(snapshot)
    end
    return false
end

local function load_best_record()
    local ok, data = pcall(get_best_score)
    if not ok or type(data) ~= "table" then
        return nil
    end

    local difficulty = tonumber(data.difficulty)
    local min_steps = tonumber(data.min_steps or data.steps)
    local min_time_sec = tonumber(data.min_time_sec or data.time_sec)
    if difficulty == nil or min_steps == nil or min_time_sec == nil then
        return nil
    end

    return {
        difficulty = clamp(math.floor(difficulty), MIN_DIFFICULTY, MAX_DIFFICULTY),
        min_steps = math.max(0, math.floor(min_steps)),
        min_time_sec = math.max(0, math.floor(min_time_sec))
    }
end

local function should_replace_best(old, new)
    if old == nil then
        return true
    end
    if new.difficulty ~= old.difficulty then
        return new.difficulty > old.difficulty
    end
    if new.min_steps ~= old.min_steps then
        return new.min_steps < old.min_steps
    end
    return new.min_time_sec < old.min_time_sec
end

local function request_best_save()
    if type(request_save_best_score) == "function" then
        pcall(request_save_best_score)
    end
end

local function commit_best_if_needed()
    if state.best_committed then
        return
    end
    local record = {
        difficulty = state.difficulty,
        min_steps = state.steps,
        min_time_sec = elapsed_seconds()
    }
    if should_replace_best(state.best, record) then
        state.best = record
        request_best_save()
    end
    state.best_committed = true
end

local function mark_won()
    if state.won then
        return
    end
    state.won = true
    state.end_frame = state.frame
    state.confirm_mode = nil
    state.pending_hide = nil
    stop_pending_hide_timer()
    state.first_pick = nil
    commit_best_if_needed()
    state.dirty = true
end

local function reset_game(new_difficulty)
    if new_difficulty ~= nil then
        state.difficulty = clamp(new_difficulty, MIN_DIFFICULTY, MAX_DIFFICULTY)
    end
    state.size = difficulty_to_size(state.difficulty)
    state.board = generate_board(state.size)
    state.revealed = new_matrix(state.size, false)
    state.matched = new_matrix(state.size, false)
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
    state.first_pick = nil
    state.pending_hide = nil
    stop_pending_hide_timer()
    state.last_area = nil
    state.dirty = true
end

local function init_runtime_state(saved_state)
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
    if state.launch_mode == "continue" and type(saved_state) == "table" then
        if not load_game_state(saved_state) then
            reset_game(DEFAULT_DIFFICULTY)
        end
    else
        reset_game(DEFAULT_DIFFICULTY)
    end
end

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

local function board_geometry()
    local w, h = terminal_size()
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H

    local status_w = text_width(tr("game.memory_flip.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.memory_flip.steps") .. " 9999")
    local win_line_w = text_width(
        tr("game.memory_flip.win_banner") .. restart_quit_controls_text()
    )
    local content_w = math.max(LABEL_W + grid_w, status_w, win_line_w)
    local content_h = 1 + grid_h
    local frame_w = content_w + 2
    local frame_h = content_h + 2

    local x = math.floor((w - frame_w) / 2)
    local y = math.floor((h - frame_h) / 2)
    if x < 1 then x = 1 end
    if y < 6 then y = 6 end

    return x, y, frame_w, frame_h
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then
        return
    end
    local line = string.rep(" ", w)
    for row = 0, h - 1 do
        draw_text(x, y + row, line, "white", bg or "black")
    end
end

local function draw_outer_frame(x, y, frame_w, frame_h)
    draw_text(x, y, "╔" .. string.rep("═", frame_w - 2) .. "╗", "white", "black")
    for i = 1, frame_h - 2 do
        draw_text(x, y + i, "║", "white", "black")
        draw_text(x + frame_w - 1, y + i, "║", "white", "black")
    end
    draw_text(x, y + frame_h - 1, "╚" .. string.rep("═", frame_w - 2) .. "╝", "white", "black")
end

local function draw_card(x, y, pair_id, visible, selected)
    local bg = "rgb(90,90,90)"  -- 未翻开时的灰色背景
    local fg = "white"
    local face = ".."            -- 未翻开时的背面图案
    local frame_x = x - 1
    local body = " " .. face .. " "

    if visible then
        bg = pair_bg_color(pair_id)
        fg = pair_text_color(pair_id)
        local symbol = pair_symbol(pair_id)
        face = symbol .. symbol
        body = " " .. face .. " "
    end

    if selected then
        draw_text(frame_x, y, "┌────┐", "green", "black")
        draw_text(frame_x, y + 1, "│", "green", "black")
        draw_text(x, y + 1, body, fg, bg)
        draw_text(frame_x + 5, y + 1, "│", "green", "black")
        draw_text(frame_x, y + 2, "└────┘", "green", "black")
    else
        draw_text(frame_x, y, "      ", "white", "black")
        draw_text(frame_x, y + 1, "      ", "white", "black")
        draw_text(x, y + 1, body, fg, bg)
        draw_text(frame_x, y + 2, "      ", "white", "black")
    end
end

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
    
    for c = 1, state.size do
        local cx = grid_x + (c - 1) * CELL_STEP_X + 1
        draw_text(cx, inner_y, string.format("%2d", c), "dark_gray", "black")
    end

    for r = 1, state.size do
        local row_base = inner_y + 1 + (r - 1) * CELL_STEP_Y
        draw_text(grid_block_x, row_base + 1, string.format("%2d", r), "dark_gray", "black")

        for c = 1, state.size do
            local cx = grid_x + (c - 1) * CELL_STEP_X
            local selected = (r == state.cursor_r and c == state.cursor_c)
            local visible = state.matched[r][c] or state.revealed[r][c]
            draw_card(cx, row_base, state.board[r][c], visible, selected)
        end
    end

    local sr = state.cursor_r
    local sc = state.cursor_c
    if sr >= 1 and sr <= state.size and sc >= 1 and sc <= state.size then
        local sel_y = inner_y + 1 + (sr - 1) * CELL_STEP_Y
        local sel_x = grid_x + (sc - 1) * CELL_STEP_X
        local visible = state.matched[sr][sc] or state.revealed[sr][sc]
        draw_card(sel_x, sel_y, state.board[sr][sc], visible, true)
    end
end

local function best_line()
    if state.best == nil then
        return tr("game.memory_flip.best_none")
    end

    return string.format(
        "%s %d  %s %d  %s %s",
        tr("game.memory_flip.best_difficulty"),
        state.best.difficulty,
        tr("game.memory_flip.best_steps"),
        state.best.min_steps,
        tr("game.memory_flip.best_time"),
        format_duration(state.best.min_time_sec)
    )
end

local function draw_status(x, y, frame_w)
    local elapsed = elapsed_seconds()
    local time_text = tr("game.memory_flip.time") .. " " .. format_duration(elapsed)
    local steps_text = tr("game.memory_flip.steps") .. " " .. tostring(state.steps)
    local term_w = terminal_size()
    local right_x = x + frame_w - text_width(steps_text)
    if right_x < 1 then right_x = 1 end

    draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

    draw_text(x, y - 3, best_line(), "dark_gray", "black")
    draw_text(x, y - 2, time_text, "light_cyan", "black")
    draw_text(right_x, y - 2, steps_text, "light_cyan", "black")

    if state.input_mode == "difficulty" then
        if state.input_buffer == "" then
            draw_text(
                x,
                y - 1,
                tr("game.memory_flip.input_size_hint"),
                "dark_gray",
                "black"
            )
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.input_mode == "jump" then
        if state.input_buffer == "" then
            draw_text(
                x,
                y - 1,
                tr("game.memory_flip.input_jump_hint"),
                "dark_gray",
                "black"
            )
        else
            draw_text(x, y - 1, state.input_buffer, "white", "black")
        end
    elseif state.won then
        local line = tr("game.memory_flip.win_banner") .. restart_quit_controls_text()
        draw_text(x, y - 1, line, "yellow", "black")
    elseif state.confirm_mode == "restart" then
        draw_text(x, y - 1, replace_prompt_keys(tr("game.memory_flip.confirm_restart")), "yellow", "black")
    elseif state.confirm_mode == "exit" then
        draw_text(x, y - 1, replace_prompt_keys(tr("game.memory_flip.confirm_exit")), "yellow", "black")
    elseif state.toast_text ~= nil and state.frame <= state.toast_until then
        draw_text(x, y - 1, state.toast_text, "green", "black")
    end
end

local function draw_controls(x, y, frame_h)
    local term_w = terminal_size()
    local text = controls_text()
    local max_w = math.max(10, term_w - 2)
    local lines = wrap_words(text, max_w)
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 1, 3 do
        draw_text(1, y + frame_h + i, string.rep(" ", term_w), "white", "black")
    end

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    
    for i = 1, #lines do
        local line = lines[i]
        local line_x = math.floor((term_w - text_width(line)) / 2)
        if line_x < 1 then line_x = 1 end
        draw_text(line_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
    end
end

local function clear_last_area()
    if state.last_area == nil then
        return
    end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

local function render_frame()
    local x, y, frame_w, frame_h = board_geometry()
    local area = { x = x, y = y - 3, w = frame_w, h = frame_h + 7 }

    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y or
        state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    draw_status(x, y, frame_w)
    draw_board(x, y, frame_w, frame_h)
    draw_controls(x, y, frame_h)
end

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

local function minimum_required_size()
    local grid_w = (state.size - 1) * CELL_STEP_X + CELL_W
    local grid_h = (state.size - 1) * CELL_STEP_Y + CELL_H
    local frame_w = LABEL_W + grid_w + 2
    local frame_h = 1 + grid_h + 2

    local controls_w = min_width_for_lines(
        controls_text(),
        3,
        24
    )
    local status_w = text_width(tr("game.memory_flip.time") .. " 00:00:00")
        + 2
        + text_width(tr("game.memory_flip.steps") .. " 9999")
    local hint_w = math.max(
        text_width(tr("game.memory_flip.input_size_hint")),
        text_width(tr("game.memory_flip.input_jump_hint"))
    )
    local win_w = text_width(
        tr("game.memory_flip.win_banner") .. restart_quit_controls_text()
    )

    local min_w = math.max(frame_w, controls_w, status_w, hint_w, win_w) + 2
    local min_h = frame_h + 9
    return min_w, min_h
end

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

local function start_input_mode(mode)
    state.input_mode = mode
    state.input_buffer = ""
    state.dirty = true
end

local function parse_difficulty_input()
    local value = tonumber(state.input_buffer)
    if value == nil then
        return nil
    end
    value = math.floor(value)
    if value < MIN_DIFFICULTY or value > MAX_DIFFICULTY then
        return nil
    end
    return value
end

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

local function handle_input_mode_key(key)
    if key == "quit_action" or key == "confirm_no" then
        state.input_mode = nil
        state.input_buffer = ""
        state.dirty = true
        return "changed"
    end

    if key == "confirm" then
        if state.input_mode == "difficulty" then
            local difficulty = parse_difficulty_input()
            state.input_mode = nil
            state.input_buffer = ""
            if difficulty ~= nil then
                clear()
                state.last_area = nil
                reset_game(difficulty)
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

    if key == "remove_last" or key == "backspace" or key == "del" then
        if #state.input_buffer > 0 then
            state.input_buffer = string.sub(state.input_buffer, 1, #state.input_buffer - 1)
            state.dirty = true
            return "changed"
        end
        return "none"
    end

    if state.input_mode == "difficulty" then
        if key:match("^[1-3]$") and #state.input_buffer < 1 then
            state.input_buffer = state.input_buffer .. key
            state.dirty = true
            return "changed"
        end
        return "none"
    end

    if state.input_mode == "jump" then
        if key:match("^%d$") or key == "flip" then
            local token = key
            if key == "flip" then
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

local function handle_confirm_key(key)
    if key == "confirm_yes" then
        if state.confirm_mode == "restart" then
            reset_game(state.difficulty)
            return "changed"
        end
        if state.confirm_mode == "exit" then
            return "exit"
        end
    end

    if key == "quit_action" or key == "confirm_no" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

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

local function hide_pending_pair_if_needed()
    if state.pending_hide == nil then
        return
    end
    if not pending_hide_completed() then
        return
    end

    stop_pending_hide_timer()
    local p = state.pending_hide
    if not state.matched[p.r1][p.c1] then
        state.revealed[p.r1][p.c1] = false
    end
    if not state.matched[p.r2][p.c2] then
        state.revealed[p.r2][p.c2] = false
    end
    state.pending_hide = nil
    state.dirty = true
end

local function try_flip_current()
    local r = state.cursor_r
    local c = state.cursor_c

    if state.matched[r][c] then
        return
    end
    if state.revealed[r][c] then
        return
    end

    state.revealed[r][c] = true
    if state.first_pick == nil then
        state.first_pick = { r = r, c = c }
        state.dirty = true
        return
    end

    local fr = state.first_pick.r
    local fc = state.first_pick.c
    if fr == r and fc == c then
        return  -- 不能重复翻同一张牌
    end

    state.steps = state.steps + 1
    if state.board[fr][fc] == state.board[r][c] then
        state.matched[fr][fc] = true
        state.matched[r][c] = true
        state.first_pick = nil
        if all_matched() then
            mark_won()
        else
            state.dirty = true
        end
    else
        state.pending_hide = {
            r1 = fr,
            c1 = fc,
            r2 = r,
            c2 = c,
            until_frame = state.frame + math.floor((HIDE_DELAY_MS / 1000) * FPS)
        }
        start_pending_hide_timer()
        state.first_pick = nil
        state.dirty = true
    end
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    if should_debounce(key) then
        return "none"
    end

    if state.input_mode ~= nil then
        return handle_input_mode_key(key)
    end

    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    if state.won then
        if key == "restart" then
            reset_game(state.difficulty)
            return "changed"
        end
        if key == "quit_action" or key == "confirm_no" then
            return "exit"
        end
        return "none"
    end

    if key == "restart" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end

    if key == "quit_action" or key == "confirm_no" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end

    if key == "save" then
        save_game_state(true)
        return "changed"
    end

    if state.pending_hide ~= nil then
        return "none"
    end

    if key == "difficulty_input" then
        start_input_mode("difficulty")
        return "changed"
    end

    if key == "quick_jump" then
        start_input_mode("jump")
        return "changed"
    end

    if key == "move_up" then
        state.cursor_r = clamp(state.cursor_r - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "move_down" then
        state.cursor_r = clamp(state.cursor_r + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "move_left" then
        state.cursor_c = clamp(state.cursor_c - 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "move_right" then
        state.cursor_c = clamp(state.cursor_c + 1, 1, state.size)
        state.dirty = true
        return "changed"
    end

    if key == "flip" then
        try_flip_current()
        return "changed"
    end

    return "none"
end

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

local function step_runtime(key)
    if ensure_terminal_size_ok() then
        hide_pending_pair_if_needed()

        local action = "none"
        if key ~= nil and key ~= "" then
            action = handle_input(key)
        end
        if action == "exit" then
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
        end

        sync_terminal_resize()
        auto_save_if_needed()
        refresh_dirty_flags()
    else
        if key == "quit_action" or key == "confirm_no" then
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
        end
    end

    state.frame = state.frame + 1
end

local function runtime_init_game(saved_state)
    init_runtime_state(saved_state)
    return state
end

local function runtime_handle_event(state_arg, event)
    state = state_arg or state
    event = event or {}

    if event ~= nil and event.type == "resize" then
        state.last_term_w = event.width or state.last_term_w
        state.last_term_h = event.height or state.last_term_h
        clear()
        state.last_area = nil
        state.dirty = true
        return state
    end

    local key = normalize_event_key(event)
    step_runtime(key)
    return state
end

local function runtime_render(state_arg)
    state = state_arg or state
    if not ensure_terminal_size_ok() then
        return
    end
    render_frame()
end

local function runtime_save_best_score(state_arg)
    state = state_arg or state
    if state.best == nil then
        return { best_string = "game.memory_flip.best_none_block" }
    end

    return {
        best_string = "game.memory_flip.best_block",
        difficulty = state.best.difficulty,
        min_steps = state.best.min_steps,
        min_time_sec = state.best.min_time_sec,
        steps = state.best.min_steps,
        time = format_duration(state.best.min_time_sec)
    }
end

local function runtime_save_game(state_arg)
    state = state_arg or state
    return make_snapshot()
end

local function runtime_exit_game(state_arg)
    state = state_arg or state
    if not state.won then
        save_game_state(false)
    end
    stop_pending_hide_timer()
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

_G.MEMORY_FLIP_RUNTIME = Runtime
return Runtime
