-- Star drift package callbacks and terminal-cell drawing.

local SEED = 20260924
local MAX_STARS = 90
local LAYERS = {
  { glyph = ".", speed = 4, fg = color.GRAY, share = 0.6 },
  { glyph = "+", speed = 9, fg = color.BRIGHT_GRAY, share = 0.3 },
  { glyph = "*", speed = 18, fg = color.WHITE, share = 0.1 },
}

local width = 80
local height = 24
local stars = {}
local column_rng = nil
local row_rng = nil

-- Return the star count selected for the current screen size.
local function total_stars()
  return math.min({ MAX_STARS, width * height // 40 })
end

-- Create the star positions and speeds for the current screen dimensions.
local function scatter()
  stars = {}
  random.set(column_rng, { min = 0, max = width - 1 })
  random.set(row_rng, { min = 0, max = height - 2 })
  local total = total_stars()
  for layer_index = 1, #LAYERS do
    local count = math.max({ 1, math.floor(total * LAYERS[layer_index].share) })
    for _ = 1, count do
      stars[#stars + 1] = {
        layer = layer_index,
        x = random.generate(column_rng),
        y = random.generate(row_rng),
      }
    end
  end
end

-- Initialize package state from the supplied base dimensions and startup data.
function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height

  column_rng = random.create({ type = random.FLOAT, seed = SEED })
  row_rng = random.create({ type = random.INT, seed = SEED + 1 })
  scatter()
end

-- Apply the resize, action, or completion events handled by this package.
function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    scatter()
  end
end

-- Advance package simulation using the fixed-update delta in seconds.
function Update(dt)
  for index = 1, #stars do
    local star = stars[index]
    star.x = star.x - LAYERS[star.layer].speed * dt
    if star.x < 0 then
      star.x = star.x + width
      star.y = random.generate(row_rng)
    end
  end
end

-- Keep this optional callback empty; this package needs no work in this phase.
function UpdateFrame(dt, alpha)
end

-- Draw the current package state in terminal-cell coordinates.
function Render()
  draw.fill_rect(0, 0, width, height, { char = " ", bg = color.BLACK })
  for index = 1, #stars do
    local star = stars[index]
    local layer = LAYERS[star.layer]
    draw.text(star.x // 1, star.y, layer.glyph, { fg = layer.fg })
  end
  local info = random.get_info(row_rng)
  local footer = string.upper("star drift") .. "  seed " .. info.seed .. "  draws " .. info.step
  draw.text(1, height - 1, footer, { fg = color.GRAY, max_width = width - 2, max_height = 1 })
end
