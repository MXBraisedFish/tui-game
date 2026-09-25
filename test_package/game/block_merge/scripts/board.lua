-- Pure board logic for Block Merge. Loaded through loader.require("board").

local board = {}

board.SIZE = 4

local function index_of(row, column)
  return (row - 1) * board.SIZE + column
end

function board.new()
  local cells = {}
  for index = 1, board.SIZE * board.SIZE do
    cells[index] = 0
  end
  return cells
end

-- Returns the cell indices of one line, ordered from the edge the tiles slide towards.
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
    table.insert { table = indices, value = index_of(row, column) }
  end
  return indices
end

-- Packs the non-empty values towards the front and merges equal neighbours once.
local function merge_line(values)
  local packed = {}
  -- The host's ipairs yields one { index, value } record per element.
  for item in ipairs(values) do
    if item.value ~= 0 then
      table.insert { table = packed, value = item.value }
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
    table.insert { table = merged, value = value }
  end
  while #merged < board.SIZE do
    table.insert { table = merged, value = 0 }
  end
  return merged, gained
end

-- Slides every line in one direction. Returns whether any tile moved and the points gained.
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

function board.empty_cells(cells)
  local empty = {}
  for index = 1, #cells do
    if cells[index] == 0 then
      table.insert { table = empty, value = index }
    end
  end
  return empty
end

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

function board.max_tile(cells)
  return math.floor(math.max { values = cells })
end

return board
