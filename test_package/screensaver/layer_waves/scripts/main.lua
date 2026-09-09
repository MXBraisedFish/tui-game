local panel = nil
local phase = 0
local width = 80
local height = 24

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  panel = slice.create{
    width = math.max{ values = { 1, math.floor(width * 0.75) } },
    height = math.max{ values = { 1, math.floor(height * 0.5) } },
    layer = 10,
  }
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    slice.set_size{
      id = panel,
      width = math.max{ values = { 1, math.floor(width * 0.75) } },
      height = math.max{ values = { 1, math.floor(height * 0.5) } },
    }
  end
end

function Update(dt)
  phase = phase + dt * 2
end

function UpdateFrame(dt, alpha)
end

function Render()
  local panel_width = slice.get_width(panel)
  local panel_height = slice.get_height(panel)
  slice.draw{
    id = panel,
    x = math.floor((width - panel_width) / 2),
    y = math.floor((height - panel_height) / 2)
  }
  draw.fill_rect{ x = 0, y = 0, width = width, height = height, char = " ", bg = "black" }
  draw.fill_rect{ x = 0, y = 0, width = panel_width, height = panel_height, char = " ", bg = "blue", slice_layer = panel }
  draw.stroke_rect{ x = 0, y = 0, width = panel_width, height = panel_height, fg = "bright_cyan", slice_layer = panel }
  for y = 2, panel_height - 2, 2 do
    local x = math.floor((math.sin(phase + y * 0.4) + 1) * (panel_width - 4) / 2) + 1
    draw.text{ x = x, y = y, text = "~", fg = "white", slice_layer = panel }
  end
end
