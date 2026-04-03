local EMPTY = 0
local MARK_X = 1
local MARK_O = 2

local function tr(key)
  return translate(key)
end

local function new_board()
  return {
    { EMPTY, EMPTY, EMPTY },
    { EMPTY, EMPTY, EMPTY },
    { EMPTY, EMPTY, EMPTY },
  }
end

local function state_new()
  return {
    board = new_board(),
    cursor_row = 1,
    cursor_col = 1,
    player_mark = MARK_X,
    ai_mark = MARK_O,
    turn = MARK_X,
    winner = EMPTY,
    finished = false,
    draw = false,
    win_cells = {},
    message = "game.tic_tac_toe.ready",
  }
end

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

local function win_key(row, col)
  return tostring(row) .. "," .. tostring(col)
end

local function board_full(board)
  for row = 1, 3 do
    for col = 1, 3 do
      if board[row][col] == EMPTY then
        return false
      end
    end
  end
  return true
end

local function evaluate_board(board)
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

local function copy_board(board)
  return {
    { board[1][1], board[1][2], board[1][3] },
    { board[2][1], board[2][2], board[2][3] },
    { board[3][1], board[3][2], board[3][3] },
  }
end

local function apply_result(state, winner, line)
  if winner == EMPTY then
    return state
  end

  state.finished = true
  state.winner = winner
  state.draw = winner == -1
  state.win_cells = {}
  if line then
    for i = 1, #line do
      local point = line[i]
      state.win_cells[win_key(point[1], point[2])] = true
    end
  end

  if winner == state.player_mark then
    state.message = "game.tic_tac_toe.win_banner"
  elseif winner == state.ai_mark then
    state.message = "game.tic_tac_toe.lose_banner"
  else
    state.message = "game.tic_tac_toe.draw_banner"
  end
  return state
end

local function minimax(board, turn, ai_mark, player_mark, depth)
  local winner = evaluate_board(board)
  local result = winner
  if type(winner) == "table" then
    result = winner[1]
  end

  if result == ai_mark then
    return 10 - depth
  end
  if result == player_mark then
    return depth - 10
  end
  if result == -1 then
    return 0
  end

  if turn == ai_mark then
    local best = -999
    for row = 1, 3 do
      for col = 1, 3 do
        if board[row][col] == EMPTY then
          board[row][col] = ai_mark
          local score = minimax(board, player_mark, ai_mark, player_mark, depth + 1)
          board[row][col] = EMPTY
          if score > best then
            best = score
          end
        end
      end
    end
    return best
  end

  local best = 999
  for row = 1, 3 do
    for col = 1, 3 do
      if board[row][col] == EMPTY then
        board[row][col] = player_mark
        local score = minimax(board, ai_mark, ai_mark, player_mark, depth + 1)
        board[row][col] = EMPTY
        if score < best then
          best = score
        end
      end
    end
  end
  return best
end

local function ai_play(state)
  if state.finished or state.turn ~= state.ai_mark then
    return state
  end

  local temp = copy_board(state.board)
  local best_score = -999
  local best_row, best_col = nil, nil
  for row = 1, 3 do
    for col = 1, 3 do
      if temp[row][col] == EMPTY then
        temp[row][col] = state.ai_mark
        local score = minimax(temp, state.player_mark, state.ai_mark, state.player_mark, 0)
        temp[row][col] = EMPTY
        if score > best_score then
          best_score = score
          best_row, best_col = row, col
        end
      end
    end
  end

  if best_row == nil then
    return state
  end

  state.board[best_row][best_col] = state.ai_mark
  state.turn = state.player_mark
  local winner, line = evaluate_board(state.board)
  return apply_result(state, winner, line)
end

local function reset_round(state)
  state.board = new_board()
  state.cursor_row = 1
  state.cursor_col = 1
  state.winner = EMPTY
  state.finished = false
  state.draw = false
  state.win_cells = {}
  state.turn = MARK_X
  state.message = "game.tic_tac_toe.ready"
  if state.ai_mark == MARK_X then
    state = ai_play(state)
  end
  return state
end

function init_game()
  return reset_round(state_new())
end

local function clamp(value, min_value, max_value)
  if value < min_value then
    return min_value
  end
  if value > max_value then
    return max_value
  end
  return value
end

local function move_cursor(state, d_row, d_col)
  state.cursor_row = clamp(state.cursor_row + d_row, 1, 3)
  state.cursor_col = clamp(state.cursor_col + d_col, 1, 3)
  return state
end

local function place_mark(state)
  if state.finished or state.turn ~= state.player_mark then
    return state
  end
  if state.board[state.cursor_row][state.cursor_col] ~= EMPTY then
    return state
  end

  state.board[state.cursor_row][state.cursor_col] = state.player_mark
  state.turn = state.ai_mark
  local winner, line = evaluate_board(state.board)
  state = apply_result(state, winner, line)
  if not state.finished then
    state = ai_play(state)
  end
  return state
end

local function switch_marks(state)
  if state.player_mark == MARK_X then
    state.player_mark = MARK_O
    state.ai_mark = MARK_X
    state.message = "game.tic_tac_toe.switch_to_o"
  else
    state.player_mark = MARK_X
    state.ai_mark = MARK_O
    state.message = "game.tic_tac_toe.switch_to_x"
  end
  return reset_round(state)
end

function handle_event(state, event)
  if event.type == "resize" then
    return state
  end

  if event.type == "quit" then
    request_exit()
    return state
  end

  if event.type ~= "action" then
    return state
  end

  if event.name == "quit_action" then
    request_exit()
    return state
  elseif event.name == "restart" then
    return reset_round(state)
  elseif event.name == "switch_mark" then
    return switch_marks(state)
  elseif event.name == "move_up" then
    return move_cursor(state, -1, 0)
  elseif event.name == "move_down" then
    return move_cursor(state, 1, 0)
  elseif event.name == "move_left" then
    return move_cursor(state, 0, -1)
  elseif event.name == "move_right" then
    return move_cursor(state, 0, 1)
  elseif event.name == "confirm" then
    return place_mark(state)
  end

  return state
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function draw_board(state, x, y)
  local separator = "--+--+--"
  for row = 1, 3 do
    local cell_y = y + (row - 1) * 2
    for col = 1, 3 do
      local cell_x = x + (col - 1) * 3
      local bg = (state.cursor_row == row and state.cursor_col == col) and "yellow" or nil
      local fg = "white"
      if state.win_cells[win_key(row, col)] then
        fg = "green"
      elseif state.board[row][col] == MARK_X then
        fg = "red"
      elseif state.board[row][col] == MARK_O then
        fg = "cyan"
      end
      canvas_draw_text(cell_x, cell_y, mark_symbol(state.board[row][col]), fg, bg)
      if col < 3 then
        canvas_draw_text(cell_x + 2, cell_y, "|", "white", nil)
      end
    end
    if row < 3 then
      canvas_draw_text(x, cell_y + 1, separator, "white", nil)
    end
  end
end

function render(state)
  canvas_clear()
  local term_w, term_h = get_terminal_size()
  local title = tr("game.tic_tac_toe.name")
  local desc = tr("game.tic_tac_toe.description")
  local status = tr("game.tic_tac_toe.you")
    .. ": "
    .. mark_name(state.player_mark)
    .. " "
    .. mark_symbol(state.player_mark)
    .. "  "
    .. tr("game.tic_tac_toe.ai")
    .. ": "
    .. mark_name(state.ai_mark)
    .. " "
    .. mark_symbol(state.ai_mark)
  local message = tr(state.message)
  local controls = state.finished and tr("game.tic_tac_toe.result_controls") or tr("game.tic_tac_toe.controls")

  canvas_draw_text(centered_x(title), 1, title, "cyan", nil)
  canvas_draw_text(centered_x(desc), 2, desc, "dark_gray", nil)
  canvas_draw_text(centered_x(status), 4, status, "white", nil)
  canvas_draw_text(centered_x(message), 5, message, state.finished and "green" or "white", nil)

  local board_x, board_y = resolve_rect(ANCHOR_CENTER, ANCHOR_MIDDLE, 8, 5, 0, 0)
  board_y = math.max(board_y, 7)
  draw_board(state, board_x, board_y)

  canvas_draw_text(centered_x(controls), math.max(0, term_h - 2), controls, "dark_gray", nil)
end

function best_score(_state)
  return nil
end
