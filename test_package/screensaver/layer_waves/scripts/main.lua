-- Layer waves package callbacks and terminal-cell drawing.

local WAVE_GLYPHS = { "~", "-", "~", "=" }

local width = 80
local height = 24
local phase = 0
local back_panel = nil
local front_panel = nil

-- Return the dimensions of the overlapping background and foreground wave panels.
local function panel_sizes()
  local back = {
    width = math.max({ 8, width * 3 // 4 }),
    height = math.max({ 4, height // 2 }),
  }
  local front = {
    width = math.max({ 6, width // 2 }),
    height = math.max({ 3, height // 3 }),
  }
  return back, front
end

-- Fill and outline one slice, then draw its animated wave positions.
local function draw_waves(panel, panel_width, panel_height, speed, fg)
  draw.fill_rect(0, 0, panel_width, panel_height, { char = " ", slice_layer = panel })
  draw.stroke_rect(0, 0, panel_width, panel_height, { fg = fg, border_char = char.ROUNDED_LINE, slice_layer = panel })
  for row = 1, panel_height - 2 do
    local wave = math.sin(phase * speed + row * 0.6)
    local x = math.floor((wave + 1) * (panel_width - 4) / 2) + 1
    local glyph = WAVE_GLYPHS[(row % #WAVE_GLYPHS) + 1]
    draw.text(x, row, glyph .. glyph, { fg = fg, slice_layer = panel })
  end
end

-- Initialize package state from the supplied base dimensions and startup data.
function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  local back, front = panel_sizes()
  back_panel = slice.create(back.width, back.height, { bg = color.BLUE, layer = 5 })
  front_panel = slice.create(front.width, front.height, { bg = color.MAGENTA, layer = 10 })
end

-- Apply the resize, action, or completion events handled by this package.
function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    local back, front = panel_sizes()
    slice.set(back_panel, { width = back.width, height = back.height })
    slice.set(front_panel, { width = front.width, height = front.height })
  end
end

-- Advance package simulation using the fixed-update delta in seconds.
function Update(dt)
  phase = phase + dt
end

-- Keep this optional callback empty; this package needs no work in this phase.
function UpdateFrame(dt, alpha)
end

-- Draw the current package state in terminal-cell coordinates.
function Render()
  draw.fill_rect(0, 0, width, height, { char = " ", bg = color.BLACK })

  local back = slice.get_info(back_panel)
  local front = slice.get_info(front_panel)
  local back_width, back_height = back.width, back.height
  local front_width, front_height = front.width, front.height
  local drift_x = math.floor(math.cos(phase * 0.5) * 4)
  local drift_y = math.floor(math.sin(phase * 0.7) * 2)

  slice.draw(back_panel, (width - back_width) // 2 + drift_x, (height - back_height) // 2)
  slice.draw(front_panel, (width - front_width) // 2 - drift_x, (height - front_height) // 2 + drift_y)

  draw_waves(back_panel, back_width, back_height, 1.5, color.BRIGHT_CYAN)
  draw_waves(front_panel, front_width, front_height, 2.5, color.WHITE)
  draw.text(1, height - 1, slice.count() .. " slices", { fg = color.GRAY })
end
