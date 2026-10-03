-- Pulse grid package callbacks and terminal-cell drawing.

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

-- Construct the color sequence used by the pulsing grid.
local function build_shades()
  local stops = table.deepcopy(PALETTE)
  table.sort(stops, function(left, right)
    return left.r + left.g + left.b < right.r + right.g + right.b
  end)
  shades = {}
  for shade = 0, SHADES - 1 do
    local scaled = shade / (SHADES - 1) * (#stops - 1)
    local lower = math.floor(scaled) + 1
    local upper = math.min({ lower + 1, #stops })
    local mix = scaled - (lower - 1)
    local from = stops[lower]
    local to = stops[upper]
    shades[shade + 1] = color.rgb(
      math.round(from.r + (to.r - from.r) * mix),
      math.round(from.g + (to.g - from.g) * mix),
      math.round(from.b + (to.b - from.b) * mix)
    )
  end
end

-- Rebuild grid-cell placement for the current screen dimensions.
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

-- Initialize package state from the supplied base dimensions and startup data.
function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  build_shades()
  layout_cells()
end

-- Apply the resize, action, or completion events handled by this package.
function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    layout_cells()
  end
end

-- Advance package simulation using the fixed-update delta in seconds.
function Update(dt)
  previous_phase = phase
  phase = phase + dt * PULSE_SPEED
end

-- Apply per-frame state using elapsed seconds and the fixed-step interpolation fraction.
function UpdateFrame(dt, alpha)
  shown_phase = previous_phase + (phase - previous_phase) * alpha
end

-- Draw the current package state in terminal-cell coordinates.
function Render()
  draw.fill_rect(0, 0, width, height, { char = " ", bg = color.hex(8, 10, 20) })
  for index = 1, #cells do
    local cell = cells[index]
    local intensity = (math.sin(cell.distance - shown_phase) + 1) / 2
    local shade = (intensity * (SHADES - 1)) // 1 + 1
    draw.fill_rect(cell.x, cell.y, CELL_WIDTH - 1, CELL_HEIGHT - 1, { char = " ", bg = shades[shade] })
  end
  local caption = table.concat({ "pulse", "grid", tostring(#cells) .. " cells" }, " / ")
  draw.text(1, height - 1, caption, { fg = color.GRAY })
end
