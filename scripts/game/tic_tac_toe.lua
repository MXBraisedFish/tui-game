-- 井字棋游戏元数据
GAME_META = {
    name = "井字棋",
    description = "Place X and O marks and connect three in a row."
}

-- 游戏常量
local FPS = 60
local FRAME_MS = 16

-- 棋盘格子状态常量
local EMPTY = 0
local MARK_X = 1
local MARK_O = 2

-- 棋盘尺寸（用于布局）
local BOARD_W = 8
local BOARD_H = 5

-- 胜利条件：所有可能的3连一线组合
local WIN_LINES = {
    { { 1, 1 }, { 1, 2 }, { 1, 3 } }, -- 第一行
    { { 2, 1 }, { 2, 2 }, { 2, 3 } }, -- 第二行
    { { 3, 1 }, { 3, 2 }, { 3, 3 } }, -- 第三行
    { { 1, 1 }, { 2, 1 }, { 3, 1 } }, -- 第一列
    { { 1, 2 }, { 2, 2 }, { 3, 2 } }, -- 第二列
    { { 1, 3 }, { 2, 3 }, { 3, 3 } }, -- 第三列
    { { 1, 1 }, { 2, 2 }, { 3, 3 } }, -- 主对角线
    { { 1, 3 }, { 2, 2 }, { 3, 1 } }, -- 副对角线
}

-- 游戏状态表
local state = {
    -- 棋盘数据
    board = {}, -- 3x3 棋盘，存储每个格子的状态（EMPTY/MARK_X/MARK_O）

    -- 光标位置
    cursor_row = 1,
    cursor_col = 1,

    -- 玩家和AI的标记
    player_mark = MARK_X,
    ai_mark = MARK_O,

    -- 游戏状态
    turn = MARK_X,     -- 当前回合该谁下
    winner = EMPTY,    -- 获胜者（EMPTY表示无）
    game_over = false, -- 游戏是否结束
    win_cells = {},    -- 获胜格子的映射表

    -- UI状态
    confirm_mode = nil, -- 确认模式：nil, "restart", "exit"
    toast_text = nil,   -- 提示消息
    toast_until = 0,    -- 提示消息截止帧

    -- 时间相关
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    result_committed = false,

    -- 渲染脏标记
    dirty = true,
    last_term_w = 0,
    last_term_h = 0,

    -- 输入防抖
    last_key = "",
    last_key_frame = -100,

    -- 尺寸警告
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
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

-- 规范化按键
local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    if type(key) == "table" and type(key.code) == "string" then return string.lower(key.code) end
    return tostring(key):lower()
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

-- 计算文本居中位置
local function centered_x(text, min_x, max_x)
    local span = max_x - min_x + 1
    local x = min_x + math.floor((span - text_width(text)) / 2)
    if x < min_x then x = min_x end
    return x
end

-- 计算已过秒数
local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.start_frame) / FPS))
end

-- 获取标记的显示符号
local function mark_symbol(mark)
    if mark == MARK_X then
        return "><"
    end
    if mark == MARK_O then
        return "()"
    end
    return "  "
end

-- 获取标记的名称
local function mark_name(mark)
    if mark == MARK_X then
        return tr("game.tic_tac_toe.mark_x")
    end
    return tr("game.tic_tac_toe.mark_o")
end

-- 设置空棋盘
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

-- 生成格子的键名（用于映射表）
local function win_key(r, c)
    return tostring(r) .. "," .. tostring(c)
end

-- 检查棋盘是否已满
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

-- 检查获胜者
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

-- 复制棋盘
local function copy_board(src)
    local out = {
        { src[1][1], src[1][2], src[1][3] },
        { src[2][1], src[2][2], src[2][3] },
        { src[3][1], src[3][2], src[3][3] },
    }
    return out
end

-- 极小极大算法
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

-- 提交游戏结果
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

-- 更新结果状态
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

-- 评估游戏结果
local function evaluate_result()
    local winner, line = check_winner(state.board)
    return update_result_state(winner, line)
end

-- AI走棋
local function ai_move()
    if state.game_over or state.turn ~= state.ai_mark then
        return
    end

    -- 寻找强制走法（能直接赢或阻止玩家赢）
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

    -- 列出所有空位
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

    -- 先检查是否能直接赢
    local pick = find_forced_move(state.ai_mark)
    if pick == nil then
        -- 再检查是否需要阻止玩家赢
        pick = find_forced_move(state.player_mark)
    end

    if pick == nil then
        local all_moves = list_empty_moves()
        if #all_moves == 0 then
            return
        end
        -- 随机模式概率
        local random_mode = (random(100) < 50)

        if random_mode then
            pick = all_moves[random(#all_moves) + 1]
        else
            -- 使用极小极大算法选择最佳走法
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

    -- 执行AI走棋
    state.board[pick.r][pick.c] = state.ai_mark
    state.turn = state.player_mark
    evaluate_result()
    state.dirty = true
end

-- 玩家在当前光标位置下棋
local function place_current_cell()
    if state.game_over or state.turn ~= state.player_mark then
        return
    end
    if state.board[state.cursor_row][state.cursor_col] ~= EMPTY then
        return
    end

    -- 玩家下棋
    state.board[state.cursor_row][state.cursor_col] = state.player_mark
    state.turn = state.ai_mark
    if not evaluate_result() then
        ai_move()
    end
    state.dirty = true
end

-- 开始新一局
local function begin_round()
    set_empty_board()
    if state.ai_mark == MARK_X then
        ai_move()
    end
end

-- 移动光标
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

-- 交换玩家和AI的标记
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

-- 绘制尺寸警告
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

-- 获取控制说明文本
local function controls_text()
    return tr("game.tic_tac_toe.controls")
end

-- 计算最小所需终端尺寸
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

-- 确保终端尺寸足够
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

-- 获取指定格子的符号
local function row_cell_symbol(r, c)
    return mark_symbol(state.board[r][c])
end

-- 获取指定格子的前景色
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

-- 绘制棋盘
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

-- 获取当前消息
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

-- 绘制控制说明
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

-- 主渲染函数
local function render()
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

-- 同步终端尺寸变化
local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        state.dirty = true
    end
end

-- 处理确认模式下的按键
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

    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

-- 防抖处理
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

-- 处理游戏进行中的按键
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

-- 处理游戏结束后的按键
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

-- 游戏初始化
local function init_game()
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

-- 主游戏循环
local function game_loop()
    while true do
        local key = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            if key ~= "" and not should_debounce(key) then
                if state.confirm_mode ~= nil then
                    local action = handle_confirm_key(key)
                    if action == "exit" then
                        return
                    end
                elseif state.game_over then
                    local action = handle_result_key(key)
                    if action == "exit" then
                        return
                    end
                else
                    handle_active_key(key)
                end
            end

            sync_terminal_resize()
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
