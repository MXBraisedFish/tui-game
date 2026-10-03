-- Block merge package callbacks and terminal-cell drawing.

local board = {}

board.SIZE = 4

-- Convert one-based row and column coordinates to a flat board index.
local function index_of(row, column)
  return (row - 1) * board.SIZE + column
end

-- Create a four-by-four board containing only empty tiles.
function board.new()
  local cells = {}
  for index = 1, board.SIZE * board.SIZE do
    cells[index] = 0
  end
  return cells
end

-- Return board indices in traversal order for one directional move.
local function line_indices(direction, line)
  local indices = {}
  for step = 1, board.SIZE do
    local row, column
    if direction == "left" then
      row, column = line, step
    elseif direction == "right" then
      row, column = line, board.SIZE + 1 - step
    elseif direction == "up" then
      row, column = step, line
    else
      row, column = board.SIZE + 1 - step, line
    end
    table.insert(indices, index_of(row, column))
  end
  return indices
end

-- Pack nonempty tiles, merge equal neighbors once, and return the line and gained score.
local function merge_line(values)
  local packed = {}
  for _, value in ipairs(values) do
    if value ~= 0 then
      table.insert(packed, value)
    end
  end

  local merged = {}
  local gained = 0
  local position = 1
  while position <= #packed do
    local value = packed[position]
    if position < #packed and packed[position + 1] == value then
      value = value * 2
      gained = gained + value
      position = position + 2
    else
      position = position + 1
    end
    table.insert(merged, value)
  end
  while #merged < board.SIZE do
    table.insert(merged, 0)
  end
  return merged, gained
end

-- Apply a directional move in place and return whether tiles moved and the gained score.
function board.slide(cells, direction)
  local moved = false
  local gained = 0
  for line = 1, board.SIZE do
    local indices = line_indices(direction, line)
    local values = {}
    for step = 1, #indices do
      values[step] = cells[indices[step]]
    end
    local merged, line_gain = merge_line(values)
    gained = gained + line_gain
    for step = 1, #indices do
      local index = indices[step]
      if cells[index] ~= merged[step] then
        cells[index] = merged[step]
        moved = true
      end
    end
  end
  return moved, gained
end

-- Return the indices of all empty tiles.
function board.empty_cells(cells)
  local empty = {}
  for index = 1, #cells do
    if cells[index] == 0 then
      table.insert(empty, index)
    end
  end
  return empty
end

-- Report whether the board has an empty tile or equal adjacent tiles.
function board.can_move(cells)
  if #board.empty_cells(cells) > 0 then
    return true
  end
  for row = 1, board.SIZE do
    for column = 1, board.SIZE do
      local value = cells[index_of(row, column)]
      if column < board.SIZE and cells[index_of(row, column + 1)] == value then
        return true
      end
      if row < board.SIZE and cells[index_of(row + 1, column)] == value then
        return true
      end
    end
  end
  return false
end

-- Return the largest tile value as an integer.
function board.max_tile(cells)
  return math.floor(math.max(cells))
end

return board
