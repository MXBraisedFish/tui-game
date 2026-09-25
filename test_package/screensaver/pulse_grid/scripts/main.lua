-- Pulse Grid: rings of colour pulse outward from the centre of a cell grid.
-- Exercises color.rgb/hex, math trigonometry, table.deepcopy/sort/concat and
-- interpolation between fixed updates through UpdateFrame's alpha.

local CELL_WIDTH = 6
local CELL_HEIGHT = 3
local PULSE_SPEED = 2.2
local SHADES = 12
local PALETTE = {
  { r = 200, g = 240, b = 255 },
  { r = 20, g = 40, b = 90 },
  { r = 60, g = 190, b = 210 },
  { r = 30, g = 110, b = 170 },
}

local width = 80
local height = 24
local previous_phase = 0
local phase = 0
local shown_phase = 0
local shades = {}
local cells = {}

-- Sorts a copy of the palette from dark to bright and precomputes the shade strings.
local function build_shades()
  local stops = table.deepcopy(PALETTE)
  table.sort {
    table = stops,
    comparator = function(left, right)
      return left.r + left.g + left.b < right.r + right.g + right.b
    end,
  }
  shades = {}
  for shade = 0, SHADES - 1 do
    local scaled = shade / (SHADES - 1) * (#stops - 1)
    local lower = math.floor(scaled) + 1
    local upper = math.min { values = { lower + 1, #stops } }
    local mix = scaled - (lower - 1)
    local from = stops[lower]
    local to = stops[upper]
    shades[shade + 1] = color.rgb {
      r = math.round(from.r + (to.r - from.r) * mix),
      g = math.round(from.g + (to.g - from.g) * mix),
      b = math.round(from.b + (to.b - from.b) * mix),
    }
  end
end

-- Caches every cell position with its distance from the grid centre.
local function layout_cells()
  cells = {}
  local columns = width // CELL_WIDTH
  local rows = (height - 1) // CELL_HEIGHT
  local center_column = (columns - 1) / 2
  local center_row = (rows - 1) / 2
  for row = 0, rows - 1 do
    for column = 0, columns - 1 do
      local dx = (column - center_column) / 2
      local dy = row - center_row
      cells[#cells + 1] = {
        x = column * CELL_WIDTH,
        y = row * CELL_HEIGHT,
        distance = math.sqrt(dx * dx + dy * dy),
      }
    end
  end
end

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  build_shades()
  layout_cells()
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    layout_cells()
  end
end

function Update(dt)
  previous_phase = phase
  phase = phase + dt * PULSE_SPEED
end

function UpdateFrame(dt, alpha)
  shown_phase = previous_phase + (phase - previous_phase) * alpha
end

function Render()
  draw.fill_rect { x = 0, y = 0, width = width, height = height, char = " ", bg = color.hex { r = 8, g = 10, b = 20 } }
  for index = 1, #cells do
    local cell = cells[index]
    local intensity = (math.sin(cell.distance - shown_phase) + 1) / 2
    local shade = (intensity * (SHADES - 1)) // 1 + 1
    draw.fill_rect {
      x = cell.x,
      y = cell.y,
      width = CELL_WIDTH - 1,
      height = CELL_HEIGHT - 1,
      char = " ",
      bg = shades[shade],
    }
  end
  local caption = table.concat { table = { "pulse", "grid", tostring(#cells) .. " cells" }, sep = " / " }
  draw.text { x = 1, y = height - 1, text = caption, fg = color.GRAY }
end
