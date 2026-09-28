-- B8.5 visual review scenes for the terminal renderer.

local MODE_REFERENCE = "reference"
local MODE_LOCAL = "local"
local MODE_FULL = "full"
local MODE_PULSE = "pulse"
local MODE_ORDER = { MODE_REFERENCE, MODE_LOCAL, MODE_FULL, MODE_PULSE }

local width = 80
local height = 24
local mode = MODE_REFERENCE
local paused = false
local elapsed = 0
local frame_index = 0

local function fit_size(new_width, new_height)
  width = math.max { values = { 1, new_width } }
  height = math.max { values = { 1, new_height } }
end

local function draw_line(y, text, fg, bg, bold)
  if y < 0 or y >= height then
    return
  end
  draw.text {
    x = 1,
    y = y,
    text = text,
    fg = fg,
    bg = bg,
    bold = bold or false,
    max_width = math.max { values = { 1, width - 2 } },
    max_height = 1,
  }
end

local function mode_title()
  if mode == MODE_REFERENCE then
    return "STATIC REFERENCE"
  elseif mode == MODE_LOCAL then
    return "LOCAL CELL UPDATE"
  elseif mode == MODE_FULL then
    return "SMOOTH FULL-SCREEN REFRESH"
  end
  return "OPTIONAL LOW-FREQUENCY PULSE — 2 Hz"
end

local function draw_header(fg)
  draw_line(0, "B8.5  |  " .. mode_title(), fg, nil, true)
  draw_line(1, "Keys: 1 static  2 local  3 full motion  4 pulse  N next  Space pause  Esc back", fg)
end

local function draw_reference()
  draw.fill_rect {
    x = 0,
    y = 0,
    width = width,
    height = height,
    char = " ",
    bg = color.BLACK,
  }
  draw_header(color.BRIGHT_CYAN)
  draw_line(4, "Static screen: watch for unintended flicker or stale cells.", color.WHITE)
  draw_line(6, "ASCII: !\"#$%&'()*+,-./ 0123456789  ABC xyz", color.BRIGHT_GREEN)
  draw_line(8, "CJK: 中文繁體  日本語かな  한글  界面稳定", color.BRIGHT_YELLOW)
  draw_line(10, "Rich style: bright foreground, bold labels and a fixed dark background.", color.BRIGHT_MAGENTA)
  draw_line(height - 2, "Select another scene with 1–4 or N. Escape returns to the game list.", color.GRAY)
end

local function draw_local_update()
  draw.fill_rect {
    x = 0,
    y = 0,
    width = width,
    height = height,
    char = " ",
    bg = color.BLACK,
  }
  draw_header(color.BRIGHT_CYAN)
  draw_line(4, "Only the marker below moves; the rest of the frame stays unchanged.", color.WHITE)
  draw_line(7, "Local change:", color.GRAY)

  local travel = math.max { values = { 1, width - 8 } }
  local marker_y = math.min { values = { height - 4, 9 } }
  marker_y = math.max { values = { 0, marker_y } }
  local marker_x = 3 + (frame_index // 4) % travel
  draw.text {
    x = marker_x,
    y = marker_y,
    text = "#",
    fg = color.BRIGHT_YELLOW,
    bold = true,
  }
  draw_line(height - 2, "Observe whether the small update causes visible flashes elsewhere.", color.GRAY)
end

local function draw_full_refresh()
  draw.fill_rect {
    x = 0,
    y = 0,
    width = width,
    height = height,
    char = " ",
    bg = color.BLACK,
  }

  local stripe_width = 6
  for x = 0, width - 1, stripe_width do
    local sweep = (x * 3 + frame_index * 2) % 160
    local bg = color.rgb {
      r = 20 + sweep,
      g = 28 + (sweep * 3) % 180,
      b = 50 + (sweep * 5) % 200,
    }
    draw.fill_rect {
      x = x,
      y = 2,
      width = math.min { values = { stripe_width, width - x } },
      height = math.max { values = { 1, height - 4 } },
      char = " ",
      bg = bg,
    }
  end

  draw_header(color.WHITE)
  draw_line(4, "A smooth color field moves across most visible cells on every rendered frame.", color.WHITE)
  draw_line(height - 2, "This is the main continuous-redraw scene; change the player FPS limit to compare.", color.WHITE)
end

local function draw_pulse()
  local light_phase = math.floor(elapsed / 0.25) % 2 == 0
  local bg = light_phase and color.WHITE or color.BLACK
  local fg = light_phase and color.BLACK or color.WHITE
  draw.fill_rect {
    x = 0,
    y = 0,
    width = width,
    height = height,
    char = " ",
    bg = bg,
  }
  draw_header(fg)
  draw_line(7, "2 Hz high-contrast pulse (opt-in scene)", fg, bg, true)
  draw_line(9, "Press 1, 2 or 3 to return to the render scenes.", fg, bg)
  draw_line(height - 2, "N returns to the reference scene after the other modes.", fg, bg)
end

local function set_mode(new_mode)
  mode = new_mode
end

local function next_mode()
  for index = 1, #MODE_ORDER do
    if MODE_ORDER[index] == mode then
      set_mode(MODE_ORDER[index % #MODE_ORDER + 1])
      return
    end
  end
  set_mode(MODE_REFERENCE)
end

function Init(ctx)
  fit_size(ctx.base.width, ctx.base.height)
end

function HandleEvent(event)
  if event.type == "resize" then
    fit_size(event.data.width, event.data.height)
  elseif event.type == "action" and event.data.state == "pressed" then
    local action = event.data.action
    if action == "scene_reference" then
      set_mode(MODE_REFERENCE)
    elseif action == "scene_local" then
      set_mode(MODE_LOCAL)
    elseif action == "scene_full" then
      set_mode(MODE_FULL)
    elseif action == "scene_pulse" then
      set_mode(MODE_PULSE)
    elseif action == "scene_next" then
      next_mode()
    elseif action == "pause" then
      paused = not paused
    elseif action == "leave" then
      game.exit_game {}
    end
  end
end

function Update(dt)
  if not paused then
    elapsed = elapsed + dt
  end
end

function UpdateFrame(_dt, _alpha)
  if not paused then
    frame_index = frame_index + 1
  end
end

function Render()
  if mode == MODE_REFERENCE then
    draw_reference()
  elseif mode == MODE_LOCAL then
    draw_local_update()
  elseif mode == MODE_FULL then
    draw_full_refresh()
  else
    draw_pulse()
  end
end

function SaveGame()
  return {}
end

function SaveBest()
  return { best_string = "" }
end
