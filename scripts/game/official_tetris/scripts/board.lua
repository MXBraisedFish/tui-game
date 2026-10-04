-- The game sizes, timing values, colors, and piece shapes.
local config = loader.require("config.lua")
-- The tools for checking and changing board cells.
local board = {}

-- Create a row with no occupied cells.
local function empty_row()
  -- The cells in the new empty row.
  local row = {}
  for x = 1, config.width do row[x] = { state = 0 } end
  return row
end

-- Create an empty game board.
function board.new()
  -- The rows in the new empty board.
  local data = {}
  for y = 1, config.height do data[y] = empty_row() end
  return data
end

-- Check whether a piece fits at the given board position.
function board.can_place(data, cells, target_x, target_y)
  for _, pos in ipairs(cells) do
    -- The board column and row being checked for this piece cell.
    local x, y = target_x + pos[2], target_y + pos[1]
    if x < 1 or x > config.width or y > config.height then return false end
    if y >= 1 and data[y][x].state ~= 0 then return false end
  end
  return true
end

-- Find the lowest row where a piece can land.
function board.landing_y(data, cells, x, y)
  while board.can_place(data, cells, x, y + 1) do y = y + 1 end
  return y
end

-- Place a piece on the board and report whether it extends above the top.
function board.lock(data, cells, kind, x, y)
  -- Whether any cell of the piece is above the board.
  local above_top = false
  for _, pos in ipairs(cells) do
    -- The board column and row where this piece cell will be placed.
    local cell_x, cell_y = x + pos[2], y + pos[1]
    if cell_y < 1 then
      above_top = true
    else
      data[cell_y][cell_x] = { type = kind, state = 1 }
    end
  end
  return above_top
end

-- Remove full rows and return the number of rows removed.
function board.clear_lines(data)
  -- The row being checked and the number of full rows removed.
  local y, cleared = config.height, 0
  while y >= 1 do
    -- Whether every cell in the current row is occupied.
    local full = true
    for x = 1, config.width do
      if data[y][x].state == 0 then full = false; break end
    end
    if full then
      cleared = cleared + 1
      for row = y, 2, -1 do data[row] = data[row - 1] end
      data[1] = empty_row()
    else
      y = y - 1
    end
  end
  return cleared
end

return board
