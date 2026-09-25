-- Block Merge: a 4x4 sliding merge puzzle.
-- Exercises loader, random generators, table, string.format, align, measurement,
-- serialization and the save callbacks.

local board = loader.require("board")

local CELL_WIDTH = 7
local CELL_HEIGHT = 3
local HIGHLIGHT_SECONDS = 0.35
local DIRECTIONS = {
  move_up = "up",
  move_down = "down",
  move_left = "left",
  move_right = "right",
}
local TILE_COLORS = {
  [2] = color.WHITE,
  [4] = color.BRIGHT_YELLOW,
  [8] = color.YELLOW,
  [16] = color.BRIGHT_RED,
  [32] = color.RED,
  [64] = color.BRIGHT_MAGENTA,
  [128] = color.MAGENTA,
  [256] = color.BRIGHT_CYAN,
  [512] = color.CYAN,
  [1024] = color.BRIGHT_GREEN,
  [2048] = color.GREEN,
}

local width = 40
local height = 22
local cells = nil
local score = 0
local best_score = 0
local moves = 0
local seed = 0
local position_rng = nil
local value_rng = nil
local game_over = false
local status = ""
local highlight_index = nil
local highlight_left = 0

local function create_generators(new_seed, position_step, value_step)
  random.clear()
  seed = new_seed
  position_rng = random.create {
    type = random.INT,
    min = 1,
    max = board.SIZE * board.SIZE,
    seed = seed,
    step = position_step,
  }
  value_rng = random.create {
    type = random.FLOAT,
    min = 0,
    max = 1,
    seed = seed + 1,
    step = value_step,
  }
end

local function spawn_tile()
  local empty = board.empty_cells(cells)
  if #empty == 0 then
    return
  end
  random.set_range { id = position_rng, min = 1, max = #empty }
  local index = empty[random.generate(position_rng)]
  if random.generate(value_rng) < 0.9 then
    cells[index] = 2
  else
    cells[index] = 4
  end
  highlight_index = index
  highlight_left = HIGHLIGHT_SECONDS
end

local function new_game(new_seed)
  cells = board.new()
  score = 0
  moves = 0
  game_over = false
  create_generators(new_seed, 0, 0)
  spawn_tile()
  spawn_tile()
  status = "Slide with WASD or the arrow keys. R restarts, Esc leaves."
end

local function restore(saved)
  cells = serialization.json_decode(saved.board)
  score = saved.score or 0
  moves = saved.moves or 0
  game_over = not board.can_move(cells)
  create_generators(saved.seed, saved.position_step or 0, saved.value_step or 0)
  status = "Welcome back. Continue sliding."
end

local function apply_move(direction)
  if game_over then
    status = "No moves left. Press R to start a new board."
    return
  end
  local moved, gained = board.slide(cells, direction)
  if not moved then
    status = "Nothing can slide " .. direction .. "."
    return
  end
  moves = moves + 1
  score = score + gained
  if score > best_score then
    best_score = score
  end
  spawn_tile()
  if board.can_move(cells) then
    status = string.format { format_string = "Slid %s and gained %d points.", values = { direction, gained } }
  else
    game_over = true
    status = "No moves left. Press R to start a new board."
    game.save_best()
  end
end

local function tile_color(value)
  if value == 0 then
    return color.GRAY
  end
  return TILE_COLORS[value] or color.BRIGHT_BLUE
end

local function draw_board(origin_x, origin_y)
  for row = 1, board.SIZE do
    for column = 1, board.SIZE do
      local index = (row - 1) * board.SIZE + column
      local value = cells[index]
      local x = origin_x + 1 + (column - 1) * CELL_WIDTH
      local y = origin_y + 1 + (row - 1) * CELL_HEIGHT
      draw.fill_rect { x = x, y = y, width = CELL_WIDTH - 1, height = CELL_HEIGHT - 1, char = " ", bg = tile_color(value) }
      if value > 0 then
        local label = tostring(value)
        draw.text {
          x = x + (CELL_WIDTH - 1 - utf8.len(label)) // 2,
          y = y,
          text = label,
          fg = color.BLACK,
          bg = tile_color(value),
          bold = true,
          reverse = index == highlight_index and highlight_left > 0,
        }
      end
    end
  end
end

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  if ctx.best_data ~= nil then
    best_score = ctx.best_data.score or 0
  end
  if ctx.start_mode == "continue" and ctx.continue_data ~= nil then
    restore(ctx.continue_data)
  else
    new_game(random.randint { min = 1, max = 2147483646 })
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
  elseif event.type == "action" and event.data.state == "pressed" then
    local action = event.data.action
    if action == "leave" then
      game.exit_game()
    elseif action == "restart" then
      game.save_best()
      new_game(random.randint { min = 1, max = 2147483646 })
    elseif DIRECTIONS[action] ~= nil then
      apply_move(DIRECTIONS[action])
    end
  end
end

function Update(dt)
  if highlight_left > 0 then
    highlight_left = math.max { values = { 0, highlight_left - dt } }
  end
end

function UpdateFrame(dt, alpha)
end

function Render()
  draw.fill_rect { x = 0, y = 0, width = width, height = height, char = " ", bg = color.BLACK }

  local title = "f%<fg:bright_magenta>Block</fg> Merge"
  local title_width = measurement.get_text_width { text = title }
  draw.text { x = align.resolve_x { width = title_width, horizontal_align = align.HORIZONTAL_CENTER }, y = 1, text = title, bold = true }
  draw.text {
    x = 2,
    y = 3,
    text = string.format { format_string = "Score %d   Best %d   Moves %d", values = { score, best_score, moves } },
    fg = color.BRIGHT_YELLOW,
  }

  local board_width = board.SIZE * CELL_WIDTH + 1
  local board_height = board.SIZE * CELL_HEIGHT + 1
  local origin = align.resolve_rect {
    width = board_width,
    height = board_height,
    horizontal_align = align.HORIZONTAL_CENTER,
    vertical_align = align.TOP,
    offset_y = 5,
  }
  draw.stroke_rect { x = origin.x, y = origin.y, width = board_width, height = board_height, fg = color.BRIGHT_GRAY, border_char = char.ROUNDED_LINE }
  draw_board(origin.x, origin.y)

  local status_color = color.GRAY
  if game_over then
    status_color = color.BRIGHT_RED
  end
  draw.text { x = 2, y = height - 2, text = status, fg = status_color, max_width = width - 4, max_height = 1 }
end

function SaveGame()
  return {
    board = serialization.json_encode(cells),
    score = score,
    moves = moves,
    seed = seed,
    position_step = random.get_step(position_rng),
    value_step = random.get_step(value_rng),
  }
end

function SaveBest()
  if score > best_score then
    best_score = score
  end
  return {
    best_string = "f%<fg:bright_magenta>Best score: " .. best_score .. "</fg>",
    score = best_score,
    max_tile = board.max_tile(cells),
  }
end
