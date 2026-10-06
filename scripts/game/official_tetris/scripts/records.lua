-- The game rules for speed, colors, stages, and time display.
local rules = loader.require("rules.lua")
-- The tools for reading and saving the best results.
local records = {}

-- Read a nonnegative whole number or return zero for an invalid value.
local function nonnegative_integer(value)
  if type(value) ~= "number" and type(value) ~= "string" then return 0 end
  -- The numeric value read from the saved field.
  local n = tonumber(value)
  if not n or n ~= n or n < 0 or n > 9007199254740991 then return 0 end
  return math.floor(n)
end

-- Read saved best results or use zero when no valid result is available.
function records.load(data)
  -- The best scores and challenge level loaded for this session.
  local result = { normal_score = 0, challenge_score = 0, challenge_level = 0 }
  if type(data) ~= "table" then return result end

  if type(data.records) == "table" then
    for key in pairs(result) do result[key] = nonnegative_integer(data.records[key]) end
    return result
  end

  -- The display values stored by an older save format.
  local values = type(data.value) == "table" and data.value or {}
  result.normal_score = nonnegative_integer(data.normal_high_score or values.normal_score)
  result.challenge_score = nonnegative_integer(data.challenge_high_score or values.challenge_score)
  if data.best_level ~= nil then
    result.challenge_level = nonnegative_integer(data.best_level)
  else
    -- The old display text containing the number of completed level cycles.
    local cycle_text = type(values.stage_cycle) == "string" and values.stage_cycle or ""
    -- The parts found in the old level-cycle text.
    local captures = string.match(cycle_text, "x(%d+)")
    -- The number of completed 256-level cycles.
    local cycles = nonnegative_integer(captures and captures[1])
    result.challenge_level = cycles * 256 + nonnegative_integer(values.level)
  end
  return result
end

-- Return the best score for the selected mode.
function records.score(data, mode)
  return mode == "challenge" and data.challenge_score or data.normal_score
end

-- Update the best score and challenge level when the new result is higher.
function records.commit(data, mode, score, highest_level)
  -- The saved score field for the selected mode.
  local key = mode == "challenge" and "challenge_score" or "normal_score"
  if score > data[key] then data[key] = score end
  if mode == "challenge" and highest_level > data.challenge_level then
    data.challenge_level = highest_level
  end
end

-- Prepare the best results and their translated display text for saving.
function records.serialize(data)
  -- The translation key for the best challenge stage.
  local stage = rules.stage_key(data.challenge_level)
  -- The number of completed 256-level cycles.
  local cycles = math.floor(data.challenge_level / 256)
  return {
    version = 1,
    records = {
      normal_score = data.normal_score,
      challenge_score = data.challenge_score,
      challenge_level = data.challenge_level
    },
    best_string = {
      type = "i18n",
      key = "best_string.string",
      callback = "f%<fg:bright_green>Normal Mode</fg>\nScore: {value:normal_score}\n<fg:bright_red>Challenge Mode</fg>\nScore: {value:challenge_score}\nLevel: {value:level}\nStage: {value:stage}{value:stage_cycle}"
    },
    value = {
      normal_score = tostring(data.normal_score),
      challenge_score = tostring(data.challenge_score),
      level = tostring(rules.cycle_level(data.challenge_level)),
      stage_cycle = cycles > 0 and string.format(" (x%d)", cycles) or "",
      stage = { type = "i18n", key = "best_string.stage." .. stage, callback = stage }
    }
  }
end

return records
