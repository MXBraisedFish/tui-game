local C = load_function("/constants.lua")
local U = load_function("/utils.lua")

local M = {}

function M.deep_copy(board)
  local out = {}
  for r = 1, C.SIZE do
    out[r] = {}
    for col = 1, C.SIZE do
      out[r][col] = board[r][col]
    end
  end
  return out
end

function M.empty()
  local board = {}
  for r = 1, C.SIZE do
    board[r] = {}
    for col = 1, C.SIZE do
      board[r][col] = 0
    end
  end
  return board
end

local function random_tile_value()
  if random(9) == 0 then
    return 4
  end
  return 2
end

function M.empty_cells(board)
  local cells = {}
  for r = 1, C.SIZE do
    for col = 1, C.SIZE do
      if board[r][col] == 0 then
        cells[#cells + 1] = { r = r, c = col }
      end
    end
  end
  return cells
end

function M.spawn_tile(board)
  local empty = M.empty_cells(board)
  if #empty == 0 then
    return false
  end
  local pick = empty[U.random_index(#empty)]
  if pick == nil then
    return false
  end
  board[pick.r][pick.c] = random_tile_value()
  return true
end

local function merge_line(values)
  local compact = {}
  for i = 1, #values do
    if values[i] ~= 0 then
      compact[#compact + 1] = values[i]
    end
  end

  local out = {}
  local gained = 0
  local i = 1
  while i <= #compact do
    if i < #compact and compact[i] == compact[i + 1] then
      local merged = compact[i] * 2
      if merged > C.MAX_TILE then
        merged = C.MAX_TILE
      end
      out[#out + 1] = merged
      gained = gained + merged
      i = i + 2
    else
      out[#out + 1] = compact[i]
      i = i + 1
    end
  end

  while #out < C.SIZE do
    out[#out + 1] = 0
  end
  return out, gained
end

local function get_row(board, r)
  local line = {}
  for col = 1, C.SIZE do
    line[col] = board[r][col]
  end
  return line
end

local function set_row(board, r, line)
  for col = 1, C.SIZE do
    board[r][col] = line[col]
  end
end

local function get_col(board, col)
  local line = {}
  for r = 1, C.SIZE do
    line[r] = board[r][col]
  end
  return line
end

local function set_col(board, col, line)
  for r = 1, C.SIZE do
    board[r][col] = line[r]
  end
end

local function reverse_line(line)
  local out = {}
  for i = 1, C.SIZE do
    out[i] = line[C.SIZE - i + 1]
  end
  return out
end

local function lines_equal(a, b)
  for i = 1, C.SIZE do
    if a[i] ~= b[i] then
      return false
    end
  end
  return true
end

function M.apply_move(state, dir)
  local moved = false
  local gained = 0

  if dir == "left" or dir == "right" then
    for r = 1, C.SIZE do
      local old = get_row(state.board, r)
      local line = old
      local gained_line = 0
      if dir == "right" then
        line = reverse_line(line)
      end
      line, gained_line = merge_line(line)
      if dir == "right" then
        line = reverse_line(line)
      end
      set_row(state.board, r, line)
      if not lines_equal(old, line) then
        moved = true
      end
      gained = gained + gained_line
    end
  else
    for col = 1, C.SIZE do
      local old = get_col(state.board, col)
      local line = old
      local gained_line = 0
      if dir == "down" then
        line = reverse_line(line)
      end
      line, gained_line = merge_line(line)
      if dir == "down" then
        line = reverse_line(line)
      end
      set_col(state.board, col, line)
      if not lines_equal(old, line) then
        moved = true
      end
      gained = gained + gained_line
    end
  end

  if moved then
    state.score = state.score + gained
  end
  return moved
end

function M.can_move_any(board)
  if #M.empty_cells(board) > 0 then
    return true
  end
  for r = 1, C.SIZE do
    for col = 1, C.SIZE do
      local v = board[r][col]
      if r < C.SIZE and board[r + 1][col] == v then
        return true
      end
      if col < C.SIZE and board[r][col + 1] == v then
        return true
      end
    end
  end
  return false
end

function M.has_target_tile(board)
  for r = 1, C.SIZE do
    for col = 1, C.SIZE do
      if board[r][col] >= C.TARGET_TILE then
        return true
      end
    end
  end
  return false
end

function M.normalize_board(value)
  if type(value) ~= "table" then
    return nil
  end
  local board = M.empty()
  for r = 1, C.SIZE do
    if type(value[r]) ~= "table" then
      return nil
    end
    for col = 1, C.SIZE do
      board[r][col] = math.max(0, math.floor(tonumber(value[r][col]) or 0))
    end
  end
  return board
end

return M
