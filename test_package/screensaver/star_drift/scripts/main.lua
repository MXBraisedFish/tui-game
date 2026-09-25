-- Star Drift: three parallax layers of stars drift from right to left.
-- Exercises seeded random generators (create/set_range/generate/get_info),
-- string helpers and fractional movement between fixed updates.

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

-- Keeps the star count small so Init and resize stay far below the callback budgets.
local function total_stars()
  return math.min { values = { MAX_STARS, width * height // 40 } }
end

local function scatter()
  stars = {}
  random.set_range { id = column_rng, min = 0, max = width - 1 }
  random.set_range { id = row_rng, min = 0, max = height - 2 }
  local total = total_stars()
  for layer_index = 1, #LAYERS do
    local count = math.max { values = { 1, math.floor(total * LAYERS[layer_index].share) } }
    for _ = 1, count do
      stars[#stars + 1] = {
        layer = layer_index,
        x = random.generate(column_rng),
        y = random.generate(row_rng),
      }
    end
  end
end

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  -- Columns are floats because stars move by fractional steps; rows stay on whole lines.
  column_rng = random.create { type = random.FLOAT, seed = SEED }
  row_rng = random.create { type = random.INT, seed = SEED + 1 }
  scatter()
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    scatter()
  end
end

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

function UpdateFrame(dt, alpha)
end

function Render()
  draw.fill_rect { x = 0, y = 0, width = width, height = height, char = " ", bg = color.BLACK }
  for index = 1, #stars do
    local star = stars[index]
    local layer = LAYERS[star.layer]
    draw.text { x = star.x // 1, y = star.y, text = layer.glyph, fg = layer.fg }
  end
  local info = random.get_info(row_rng)
  local footer = string.upper("star drift") .. "  seed " .. info.seed .. "  draws " .. info.step
  draw.text { x = 1, y = height - 1, text = footer, fg = color.GRAY, max_width = width - 2, max_height = 1 }
end
