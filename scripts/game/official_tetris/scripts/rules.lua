-- The game sizes, timing values, colors, and piece shapes.
local config = loader.require("config.lua")
-- The game rules for speed, colors, stages, and time display.
local rules = {}

-- Return the level number within the current 256-level cycle.
function rules.cycle_level(level)
  return level % 256
end

-- Return the translation key for the current challenge stage.
function rules.stage_key(level)
  -- The challenge level within the current 256-level cycle.
  local lv = rules.cycle_level(level)
  if lv == 255 then return "dawn" end
  if lv == 235 then return "green_corridor" end
  if lv == 148 then return "darkness" end
  if lv == 146 then return "dusk" end
  if lv >= 155 then return "crash" end
  if lv >= 29 then return "challenge" end
  if level >= 256 then return "rebirth" end
  return "classic"
end

-- Return the time in seconds between automatic downward steps.
function rules.drop_interval(mode, level)
  if mode == "normal" then return 1 end
  -- The challenge level within the current 256-level cycle.
  local lv = rules.cycle_level(level)
  -- The number of 60 Hz frames between downward steps.
  local frames
  if lv >= 29 then frames = 1
  elseif lv >= 19 then frames = 2
  elseif lv >= 16 then frames = 3
  elseif lv >= 13 then frames = 4
  elseif lv >= 10 then frames = 5
  else frames = config.drop_frames[lv] end
  return frames / 60
end

-- Return the number of cleared lines needed for the next challenge level.
function rules.lines_needed(level)
  return rules.cycle_level(level) == 235 and 810 or 10
end

-- Choose the three piece colors for the current challenge level.
function rules.colors(level)
  -- The challenge level within the current 256-level cycle.
  local lv = rules.cycle_level(level)
  if lv == 146 then return { "#05134f", "#022805", "#05134f" } end
  if lv == 148 then return { "#424242", "#1b1b1b", "#101010" } end
  if lv == 235 then return { "#122f0e", "#122f0e", "#122f0e" } end

  -- The three colors assigned to the piece groups.
  local colors = {}
  if lv < 138 then
    -- The palette indexes for the current fixed color set.
    local row = config.fixed_color_rows[lv % 10]
    for group = 1, 3 do colors[group] = config.palette[row[group]] end
  else
    for group = 1, 3 do
      colors[group] = config.palette[random.randint { min = 0, max = 54 }]
    end
  end
  return colors
end

-- Choose three different colors from the full palette for normal mode.
function rules.normal_colors()
  -- The unused palette colors and the colors already added to that list.
  local available, seen = {}, {}
  for index = 0, #config.palette do
    -- The color stored at the current palette index.
    local value = config.palette[index]
    if not seen[value] then
      available[#available + 1] = value
      seen[value] = true
    end
  end

  -- The three colors assigned to the piece groups.
  local colors = {}
  for group = 1, 3 do
    -- The randomly chosen position in the remaining color list.
    local index = random.randint { min = 1, max = #available }
    colors[group] = table.remove(available, { pos = index })
  end
  return colors
end

-- Return the color assigned to the given piece type.
function rules.color_for(colors, kind)
  return colors[config.piece_groups[kind]] or color.WHITE
end

-- Turn elapsed seconds into hours, minutes, and seconds.
function rules.format_time(seconds)
  -- The elapsed time rounded down to whole seconds.
  local total = math.floor(seconds)
  return string.format("%02d:%02d:%02d", math.floor(total / 3600),
    math.floor(total % 3600 / 60), total % 60)
end

return rules
