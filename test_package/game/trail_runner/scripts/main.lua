-- Trail Runner: steer a runner that leaves a growing trail and collect gems.
-- Exercises i18n (package language files), rich text with key placeholders,
-- utf8, measurement, the direct random API and continue/best data.

local STEP_SECONDS = 0.12
local GEM_SCORE = 10
local FALLBACK_TEXT = {
  title = "Trail Runner",
  score = "Score",
  best = "Best",
  length = "Length",
  paused = "Paused",
  crashed = "The runner crossed the trail. Steer to start again.",
  help = "Steer with WASD or arrows, Space pauses, Esc leaves.",
}
local HEADINGS = {
  move_up = { x = 0, y = -1 },
  move_down = { x = 0, y = 1 },
  move_left = { x = -1, y = 0 },
  move_right = { x = 1, y = 0 },
}

local width = 80
local height = 24
local labels_ready = false
local trail = {}
local length = 4
local heading = HEADINGS.move_right
local gem = nil
local score = 0
local best_score = 0
local step_left = STEP_SECONDS
local paused = false
local crashed = false
local blink_time = 0

local function label(key)
  if labels_ready then
    return i18n.get_value { namespace = "ui", key = key }
  end
  return FALLBACK_TEXT[key]
end

-- The playfield is the inside of the border drawn by Render.
local function field()
  return { left = 1, top = 3, right = width - 2, bottom = height - 5 }
end

local function occupies_trail(x, y)
  for index = 1, #trail do
    local point = trail[index]
    if point.x == x and point.y == y then
      return true
    end
  end
  return false
end

local function place_gem()
  local bounds = field()
  for _ = 1, 64 do
    local x = random.randint { min = bounds.left, max = bounds.right }
    local y = random.randint { min = bounds.top, max = bounds.bottom }
    if not occupies_trail(x, y) then
      gem = { x = x, y = y }
      return
    end
  end
  gem = nil
end

local function reset_run()
  local bounds = field()
  local start_y = (bounds.top + bounds.bottom) // 2
  trail = { { x = bounds.left + 4, y = start_y } }
  length = 4
  heading = HEADINGS.move_right
  score = 0
  crashed = false
  paused = false
  step_left = STEP_SECONDS
  place_gem()
end

local function advance()
  local head = trail[#trail]
  local next_x = head.x + heading.x
  local next_y = head.y + heading.y
  local bounds = field()
  if next_x < bounds.left or next_x > bounds.right or next_y < bounds.top or next_y > bounds.bottom or occupies_trail(next_x, next_y) then
    crashed = true
    game.save_best()
    return
  end
  table.insert { table = trail, value = { x = next_x, y = next_y } }
  if gem ~= nil and gem.x == next_x and gem.y == next_y then
    score = score + GEM_SCORE
    length = length + 2
    if score > best_score then
      best_score = score
    end
    place_gem()
  end
  while #trail > length do
    table.remove { table = trail, position = 1 }
  end
end

local function steer(action)
  local next_heading = HEADINGS[action]
  if crashed then
    reset_run()
    heading = next_heading
    return
  end
  -- A runner cannot reverse straight into its own trail.
  if next_heading.x ~= -heading.x or next_heading.y ~= -heading.y then
    heading = next_heading
  end
end

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  i18n.create {}
  if ctx.best_data ~= nil then
    best_score = ctx.best_data.score or 0
  end
  reset_run()
  local saved = ctx.continue_data
  if ctx.start_mode == "continue" and saved ~= nil then
    trail = saved.trail
    length = saved.length
    heading = HEADINGS[saved.heading] or HEADINGS.move_right
    score = saved.score
    paused = true
    place_gem()
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
    reset_run()
  elseif event.type == "i18n" then
    labels_ready = event.data.ok
  elseif event.type == "action" and event.data.state == "pressed" then
    local action = event.data.action
    if action == "leave" then
      game.exit_game()
    elseif action == "pause" then
      paused = not paused
    elseif HEADINGS[action] ~= nil then
      steer(action)
    end
  end
end

function Update(dt)
  if paused or crashed then
    return
  end
  step_left = step_left - dt
  if step_left <= 0 then
    step_left = step_left + STEP_SECONDS
    advance()
  end
end

function UpdateFrame(dt, alpha)
  blink_time = (blink_time + dt) % 1
end

local function heading_name()
  -- The host's pairs yields one { index, value } record per key.
  for item in pairs(HEADINGS) do
    if item.value == heading then
      return item.index
    end
  end
  return "move_right"
end

function Render()
  draw.fill_rect { x = 0, y = 0, width = width, height = height, char = " ", bg = color.BLACK }
  draw.stroke_rect { x = 0, y = 2, width = width, height = height - 5, fg = color.GREEN, border_char = char.ROUNDED_LINE }

  local title = label("title")
  draw.text { x = 2, y = 0, text = "f%<fg:bright_green>" .. title .. "</fg>", bold = true }
  local summary = string.format {
    format_string = "%s %d   %s %d   %s %d",
    values = { label("score"), score, label("best"), best_score, label("length"), #trail },
  }
  draw.text { x = width - 2 - measurement.get_text_width { text = summary }, y = 0, text = summary, fg = color.BRIGHT_YELLOW }

  for index = 1, #trail do
    local point = trail[index]
    local glyph = "o"
    local fg = color.GREEN
    if index == #trail then
      glyph = "@"
      fg = color.BRIGHT_GREEN
    elseif index <= #trail - 6 then
      fg = color.GRAY
    end
    draw.text { x = point.x, y = point.y, text = glyph, fg = fg }
  end
  if gem ~= nil and blink_time < 0.7 then
    draw.text { x = gem.x, y = gem.y, text = "*", fg = color.BRIGHT_CYAN, bold = true }
  end

  local message = label("help")
  local message_color = color.GRAY
  if crashed then
    message = label("crashed")
    message_color = color.BRIGHT_RED
  elseif paused then
    message = "f%" .. label("paused") .. " ({key:pause})"
    message_color = color.BRIGHT_YELLOW
  end
  draw.text { x = 2, y = height - 2, text = message, fg = message_color, max_width = width - 4, max_height = 1 }
  local language_line = string.format {
    format_string = "Language %s, title length %d",
    values = { i18n.get_language_code(), utf8.len(title) },
  }
  draw.text { x = 2, y = height - 1, text = language_line, fg = color.GRAY, max_width = width - 4, max_height = 1 }
end

function SaveGame()
  return {
    trail = trail,
    length = length,
    heading = heading_name(),
    score = score,
  }
end

function SaveBest()
  if score > best_score then
    best_score = score
  end
  return {
    best_string = "f%<fg:bright_yellow>Best trail: " .. best_score .. "</fg>",
    score = best_score,
  }
end
