local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    request_exit()
end

local FPS = 60
local FRAME_MS = 16

local EMPTY = 0
local MARK_X = 1
local MARK_O = 2

local BOARD_W = 8
local BOARD_H = 5

local WIN_LINES = {
    { { 1, 1 }, { 1, 2 }, { 1, 3 } }, 
    { { 2, 1 }, { 2, 2 }, { 2, 3 } }, 
    { { 3, 1 }, { 3, 2 }, { 3, 3 } }, 
    { { 1, 1 }, { 2, 1 }, { 3, 1 } }, 
    { { 1, 2 }, { 2, 2 }, { 3, 2 } }, 
    { { 1, 3 }, { 2, 3 }, { 3, 3 } }, 
    { { 1, 1 }, { 2, 2 }, { 3, 3 } }, 
    { { 1, 3 }, { 2, 2 }, { 3, 1 } }, 
}

local state = {
    
    board = {}, 

    
    cursor_row = 1,
    cursor_col = 1,

    
    player_mark = MARK_X,
    ai_mark = MARK_O,

    
    turn = MARK_X,     
    winner = EMPTY,    
    game_over = false, 
    win_cells = {},    

    
    confirm_mode = nil, 
    toast_text = nil,   
    toast_until = 0,    

    
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    result_committed = false,

    
    dirty = true,
    last_term_w = 0,
    last_term_h = 0,

    
    last_key = "",
    last_key_frame = -100,

    
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
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
        move_up = "up",
        move_down = "down",
        move_left = "left",
        move_right = "right",
        confirm = "enter",
        confirm_alt = "space",
        switch_mark = "x",
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
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
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

local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end
    local lines = {}
    local current = ""
    local had = false
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

local function centered_x(text, min_x, max_x)
    local span = max_x - min_x + 1
    local x = min_x + math.floor((span - text_width(text)) / 2)
    if x < min_x then x = min_x end
    return x
end

local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

local function mark_symbol(mark)
    if mark == MARK_X then
        return "><"
    end
    if mark == MARK_O then
        return "()"
    end
    return "  "
end

local function mark_name(mark)
    if mark == MARK_X then
        return tr("game.tic_tac_toe.mark_x")
    end
    return tr("game.tic_tac_toe.mark_o")
end

local function set_empty_board()
    state.board = {
        { EMPTY, EMPTY, EMPTY },
        { EMPTY, EMPTY, EMPTY },
        { EMPTY, EMPTY, EMPTY },
    }
    state.cursor_row = 1
    state.cursor_col = 1
    state.winner = EMPTY
    state.game_over = false
    state.turn = MARK_X
    state.win_cells = {}
    state.confirm_mode = nil
    state.end_frame = nil
    state.result_committed = false
    state.start_frame = state.frame
    state.toast_text = nil
    state.toast_until = 0
    state.dirty = true
end

local function win_key(r, c)
    return tostring(r) .. "," .. tostring(c)
end

local function board_full(board)
    for r = 1, 3 do
        for c = 1, 3 do
            if board[r][c] == EMPTY then
                return false
            end
        end
    end
    return true
end

local function check_winner(board)
    for i = 1, #WIN_LINES do
        local line = WIN_LINES[i]
        local a = board[line[1][1]][line[1][2]]
        local b = board[line[2][1]][line[2][2]]
        local c = board[line[3][1]][line[3][2]]
        if a ~= EMPTY and a == b and b == c then
            return a, line
        end
    end
    if board_full(board) then
        return -1, nil
    end
    return EMPTY, nil
end

local function copy_board(src)
    local out = {
        { src[1][1], src[1][2], src[1][3] },
        { src[2][1], src[2][2], src[2][3] },
        { src[3][1], src[3][2], src[3][3] },
    }
    return out
end

local function minimax(board, turn, depth)
    local winner = check_winner(board)
    local result = winner
    if type(winner) == "table" then
        result = winner[1]
    end

    if result == state.ai_mark then
        return 10 - depth
    end
    if result == state.player_mark then
        return depth - 10
    end
    if result == -1 then
        return 0
    end

    if turn == state.ai_mark then
        local best = -999
        for r = 1, 3 do
            for c = 1, 3 do
                if board[r][c] == EMPTY then
                    board[r][c] = state.ai_mark
                    local score = minimax(board, state.player_mark, depth + 1)
                    board[r][c] = EMPTY
                    if score > best then
                        best = score
                    end
                end
            end
        end
        return best
    end

    local best = 999
    for r = 1, 3 do
        for c = 1, 3 do
            if board[r][c] == EMPTY then
                board[r][c] = state.player_mark
                local score = minimax(board, state.ai_mark, depth + 1)
                board[r][c] = EMPTY
                if score < best then
                    best = score
                end
            end
        end
    end
    return best
end

local function commit_result_once()
    if state.result_committed then
        return
    end
    state.result_committed = true
    local score = 0
    if state.winner == state.player_mark then
        score = 1
    end
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "tic_tac_toe", score, elapsed_seconds())
    end
end

local function update_result_state(winner, line)
    if winner == EMPTY then
        return false
    end
    state.winner = winner
    state.game_over = true
    state.end_frame = state.frame
    state.win_cells = {}
    if line ~= nil then
        for i = 1, #line do
            local p = line[i]
            state.win_cells[win_key(p[1], p[2])] = true
        end
    end
    commit_result_once()
    state.dirty = true
    return true
end

local function evaluate_result()
    local winner, line = check_winner(state.board)
    return update_result_state(winner, line)
end

local function ai_move()
    if state.game_over or state.turn ~= state.ai_mark then
        return
    end

    
    local function find_forced_move(mark)
        local temp = copy_board(state.board)
        for r = 1, 3 do
            for c = 1, 3 do
                if temp[r][c] == EMPTY then
                    temp[r][c] = mark
                    local winner = check_winner(temp)
                    temp[r][c] = EMPTY
                    if winner == mark then
                        return { r = r, c = c }
                    end
                end
            end
        end
        return nil
    end

    
    local function list_empty_moves()
        local moves = {}
        for r = 1, 3 do
            for c = 1, 3 do
                if state.board[r][c] == EMPTY then
                    moves[#moves + 1] = { r = r, c = c }
                end
            end
        end
        return moves
    end

    
    local pick = find_forced_move(state.ai_mark)
    if pick == nil then
        
        pick = find_forced_move(state.player_mark)
    end

    if pick == nil then
        local all_moves = list_empty_moves()
        if #all_moves == 0 then
            return
        end
        
        local random_mode = (random(100) < 50)

        if random_mode then
            pick = all_moves[random(#all_moves) + 1]
        else
            
            local best_score = -999
            local best_moves = {}
            local temp = copy_board(state.board)
            for i = 1, #all_moves do
                local m = all_moves[i]
                temp[m.r][m.c] = state.ai_mark
                local score = minimax(temp, state.player_mark, 0)
                temp[m.r][m.c] = EMPTY
                if score > best_score then
                    best_score = score
                    best_moves = { { r = m.r, c = m.c } }
                elseif score == best_score then
                    best_moves[#best_moves + 1] = { r = m.r, c = m.c }
                end
            end
            if #best_moves > 0 then
                pick = best_moves[random(#best_moves) + 1]
            end
        end
    end

    if pick == nil then
        return
    end

    
    state.board[pick.r][pick.c] = state.ai_mark
    state.turn = state.player_mark
    evaluate_result()
    state.dirty = true
end

local function place_current_cell()
    if state.game_over or state.turn ~= state.player_mark then
        return
    end
    if state.board[state.cursor_row][state.cursor_col] ~= EMPTY then
        return
    end

    
    state.board[state.cursor_row][state.cursor_col] = state.player_mark
    state.turn = state.ai_mark
    if not evaluate_result() then
        ai_move()
    end
    state.dirty = true
end

local function begin_round()
    set_empty_board()
    if state.ai_mark == MARK_X then
        ai_move()
    end
end

local function move_cursor(dr, dc)
    local nr = state.cursor_row + dr
    local nc = state.cursor_col + dc
    if nr < 1 then nr = 1 end
    if nr > 3 then nr = 3 end
    if nc < 1 then nc = 1 end
    if nc > 3 then nc = 3 end
    if nr ~= state.cursor_row or nc ~= state.cursor_col then
        state.cursor_row = nr
        state.cursor_col = nc
        state.dirty = true
    end
end

local function switch_marks()
    set_empty_board()
    if state.player_mark == MARK_X then
        state.player_mark = MARK_O
        state.ai_mark = MARK_X
        state.toast_text = tr("game.tic_tac_toe.switch_to_o")
    else
        state.player_mark = MARK_X
        state.ai_mark = MARK_O
        state.toast_text = tr("game.tic_tac_toe.switch_to_x")
    end
    state.toast_until = state.frame + FPS * 2
    if state.ai_mark == MARK_X then
        ai_move()
    end
    state.dirty = true
end

local function draw_size_warning(term_w, term_h, min_w, min_h)
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
        local line = lines[i]
        draw_text(centered_x(line, 1, term_w), top + i - 1, line, "white", "black")
    end
end

local function controls_text()
    return tr("game.tic_tac_toe.controls")
end

local function minimum_required_size()
    local status = tr("game.tic_tac_toe.you")
        .. ":" .. mark_symbol(state.player_mark)
        .. "  "
        .. tr("game.tic_tac_toe.ai")
        .. ":" .. mark_symbol(state.ai_mark)

    local msg_w = math.max(
        text_width(tr("game.tic_tac_toe.confirm_restart")),
        text_width(tr("game.tic_tac_toe.confirm_exit")),
        text_width(tr("game.tic_tac_toe.win_banner") .. " " .. tr("game.tic_tac_toe.result_controls")),
        text_width(tr("game.tic_tac_toe.lose_banner") .. " " .. tr("game.tic_tac_toe.result_controls")),
        text_width(tr("game.tic_tac_toe.draw_banner") .. " " .. tr("game.tic_tac_toe.result_controls"))
    )
    local controls_w = min_width_for_lines(controls_text(), 3, 40)
    local min_w = math.max(BOARD_W, text_width(status), msg_w, controls_w) + 2
    local min_h = 12
    return min_w, min_h
end

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
        draw_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end
    state.size_warning_active = true
    return false
end

local function row_cell_symbol(r, c)
    return mark_symbol(state.board[r][c])
end

local function row_cell_fg(r, c)
    local v = state.board[r][c]
    if v == EMPTY then
        return "white"
    end
    if state.win_cells[win_key(r, c)] == true then
        return "green"
    end
    if v == MARK_X then
        return "red"
    end
    return "cyan"
end

local function draw_board(board_x, board_y)
    local row_offsets = { 0, 2, 4 }
    local row_ys = { board_y + row_offsets[1], board_y + row_offsets[2], board_y + row_offsets[3] }

    local separator = "──┼──┼──"
    draw_text(board_x, board_y + 1, separator, "white", "black")
    draw_text(board_x, board_y + 3, separator, "white", "black")

    for r = 1, 3 do
        local y = row_ys[r]
        draw_text(board_x, y, "        ", "white", "black")
        local x1 = board_x
        local x2 = board_x + 3
        local x3 = board_x + 6

        local bg1 = (state.cursor_row == r and state.cursor_col == 1) and "yellow" or "black"
        local bg2 = (state.cursor_row == r and state.cursor_col == 2) and "yellow" or "black"
        local bg3 = (state.cursor_row == r and state.cursor_col == 3) and "yellow" or "black"

        draw_text(x1, y, row_cell_symbol(r, 1), row_cell_fg(r, 1), bg1)
        draw_text(board_x + 2, y, "│", "white", "black")
        draw_text(x2, y, row_cell_symbol(r, 2), row_cell_fg(r, 2), bg2)
        draw_text(board_x + 5, y, "│", "white", "black")
        draw_text(x3, y, row_cell_symbol(r, 3), row_cell_fg(r, 3), bg3)
    end
end

local function current_message()
    if state.game_over then
        if state.winner == state.player_mark then
            return tr("game.tic_tac_toe.win_banner") .. " " .. tr("game.tic_tac_toe.result_controls"), "green"
        end
        if state.winner == state.ai_mark then
            return tr("game.tic_tac_toe.lose_banner") .. " " .. tr("game.tic_tac_toe.result_controls"), "red"
        end
        return tr("game.tic_tac_toe.draw_banner") .. " " .. tr("game.tic_tac_toe.result_controls"), "yellow"
    end
    if state.confirm_mode == "restart" then
        return tr("game.tic_tac_toe.confirm_restart"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.tic_tac_toe.confirm_exit"), "yellow"
    end
    if state.toast_text ~= nil and state.frame <= state.toast_until then
        return state.toast_text, "light_cyan"
    end
    return tr("game.tic_tac_toe.ready"), "dark_gray"
end

local function draw_controls(y, term_w)
    for i = 0, 2 do
        draw_text(1, y + i, string.rep(" ", term_w), "white", "black")
    end
    local lines = wrap_words(controls_text(), math.max(10, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end
    local offset = math.max(0, 3 - #lines)
    for i = 1, #lines do
        local line = lines[i]
        draw_text(centered_x(line, 1, term_w), y + offset + i - 1, line, "white", "black")
    end
end

local function render_frame()
    local term_w, term_h = terminal_size()
    clear()

    local board_x = math.floor((term_w - BOARD_W) / 2) + 1
    local board_y = math.floor((term_h - 10) / 2) + 1
    if board_y < 4 then board_y = 4 end

    local status = tr("game.tic_tac_toe.you")
        .. ":" .. mark_name(state.player_mark) .. " " .. mark_symbol(state.player_mark)
        .. "  "
        .. tr("game.tic_tac_toe.ai")
        .. ":" .. mark_name(state.ai_mark) .. " " .. mark_symbol(state.ai_mark)

    local msg, msg_color = current_message()
    draw_text(1, board_y - 3, string.rep(" ", term_w), "white", "black")
    draw_text(1, board_y - 2, string.rep(" ", term_w), "white", "black")
    draw_text(centered_x(status, 1, term_w), board_y - 3, status, "white", "black")
    draw_text(centered_x(msg, 1, term_w), board_y - 2, msg, msg_color, "black")

    draw_board(board_x, board_y)
    draw_controls(board_y + BOARD_H + 1, term_w)
end

local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        state.dirty = true
    end
end

local function handle_confirm_key(key)
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            begin_round()
            return "changed"
        end
        if state.confirm_mode == "exit" then
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

local function handle_active_key(key)
    if key == "up" or key == "k" then
        move_cursor(-1, 0)
        return
    end
    if key == "down" or key == "j" then
        move_cursor(1, 0)
        return
    end
    if key == "left" or key == "h" then
        move_cursor(0, -1)
        return
    end
    if key == "right" or key == "l" then
        move_cursor(0, 1)
        return
    end
    if key == "space" or key == "enter" then
        place_current_cell()
        return
    end
    if key == "x" then
        switch_marks()
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

local function handle_result_key(key)
    if key == "r" then
        begin_round()
        return "changed"
    end
    if key == "q" or key == "esc" then
        return "exit"
    end
    return "none"
end

local function bootstrap_game()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h
    state.player_mark = MARK_X
    state.ai_mark = MARK_O
    begin_round()
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
    if ensure_terminal_size_ok() then
        if key ~= "" and not should_debounce(key) then
            if state.confirm_mode ~= nil then
                local action = handle_confirm_key(key)
                if action == "exit" then
                    exit_game()
                    return state
                end
            elseif state.game_over then
                local action = handle_result_key(key)
                if action == "exit" then
                    exit_game()
                    return state
                end
            else
                handle_active_key(key)
            end
        end
        sync_terminal_resize()
        if type(event) == "table" and event.type == "tick" then
            state.frame = state.frame + 1
        end
    else
        if key == "q" or key == "esc" then
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
    render_frame()
end

function best_score(_state_arg)
    return nil
end
