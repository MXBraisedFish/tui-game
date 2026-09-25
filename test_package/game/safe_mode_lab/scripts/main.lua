-- Safe Mode Lab: exercises debug logging and the high-privilege file API.
-- In Safe Mode every write is ignored by the host; with Safe Mode disabled the
-- write, read and list requests are dispatched and answered through file events.

local PROBE_PATH = "state/probe.log"
local MAX_LOG_LINES = 10

local width = 80
local height = 24
local probes = 0
local best_probes = 0
local log_lines = {}

local function push_log(line)
  table.insert { table = log_lines, value = line }
  while #log_lines > MAX_LOG_LINES do
    table.remove { table = log_lines, position = 1 }
  end
end

local function first_line(text)
  local lines = string.split { text = text, sep = "\n" }
  return lines[1] or ""
end

local function handle_file_event(data)
  if not data.ok then
    push_log("File " .. data.kind .. " failed: " .. data.error.code)
    debug.warn("file " .. data.kind .. " failed: " .. data.error.message)
    return
  end
  if data.kind == "write_text" then
    push_log("Wrote " .. data.path .. " (tip: " .. tostring(data.tip) .. ")")
    debug.info("probe written to " .. data.path)
  elseif data.kind == "read_text" then
    push_log("Read back: " .. first_line(data.text))
  elseif data.kind == "list_dir" then
    local names = {}
    -- The host's ipairs yields one { index, value } record per element.
    for item in ipairs(data.entries) do
      local entry = item.value
      table.insert { table = names, value = entry.path .. " [" .. entry.file_type .. "]" }
    end
    push_log("state/ contains: " .. table.concat { table = names, sep = ", " })
  end
end

local function self_check()
  local decoded = debug.pcall {
    func = function()
      return serialization.json_decode("{\"probes\": 1}")
    end,
  }
  debug.assert { value = decoded.ok and decoded.values[1].probes == 1, message = "json round trip failed" }
  local broken = debug.pcall {
    func = function()
      return serialization.json_decode("{broken")
    end,
  }
  debug.assert { value = not broken.ok, message = "invalid JSON was accepted" }
end

function Init(ctx)
  width = ctx.base.width
  height = ctx.base.height
  if ctx.continue_data ~= nil then
    probes = ctx.continue_data.probes or 0
  end
  if ctx.best_data ~= nil then
    best_probes = ctx.best_data.probes or 0
  end
  self_check()
  debug.info("Safe Mode Lab initialized")
  push_log("P writes " .. PROBE_PATH .. ", R reads it back, L lists state/.")
  if file.exists(PROBE_PATH) then
    push_log("A probe file from an earlier run exists.")
  end
end

function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
  elseif event.type == "action" and event.data.state == "pressed" then
    local action = event.data.action
    if action == "write_probe" then
      probes = probes + 1
      debug.print { message = "write probe #" .. probes, title = "safe_mode_lab", level = debug.INFO, time = true }
      file.write {
        path = PROBE_PATH,
        text = "probe=" .. probes .. "\n",
        encoding = file.UTF_8,
        end_of_line = file.LF,
        event_tip = "probe_written",
      }
      push_log("Requested write #" .. probes .. " (ignored while Safe Mode is on).")
    elseif action == "read_probe" then
      if file.exists(PROBE_PATH) then
        file.read { path = PROBE_PATH, encoding = file.UTF_8, event_tip = "probe_read" }
        push_log("Requested a read of " .. PROBE_PATH .. ".")
      else
        push_log("Nothing to read yet; write a probe first.")
      end
    elseif action == "list_probe" then
      file.list_dir { path = "state", event_tip = "state_listed" }
      push_log("Requested a listing of state/.")
    elseif action == "leave" then
      game.exit_game()
    end
  elseif event.type == "file" then
    handle_file_event(event.data)
  end
end

function Update(dt)
end

function UpdateFrame(dt, alpha)
end

function Render()
  draw.fill_rect { x = 0, y = 0, width = width, height = height, char = " ", bg = color.BLACK }
  draw.stroke_rect { x = 0, y = 0, width = width, height = height, fg = color.BRIGHT_RED, border_char = char.DOUBLE_LINE }
  draw.text { x = 2, y = 1, text = "Safe Mode Lab", fg = color.BRIGHT_RED, bold = true }
  draw.text { x = 2, y = 3, text = "Probes requested: " .. probes .. "   Best: " .. best_probes, fg = color.BRIGHT_YELLOW }
  for line_number = 1, #log_lines do
    draw.text { x = 2, y = 4 + line_number, text = log_lines[line_number], fg = color.BRIGHT_GRAY, max_width = width - 4, max_height = 1 }
  end
  draw.text { x = 2, y = height - 2, text = "P write  R read  L list  Esc leave", fg = color.GRAY }
end

function SaveGame()
  return { probes = probes }
end

function SaveBest()
  if probes > best_probes then
    best_probes = probes
  end
  return {
    best_string = "f%<fg:bright_red>Completed probes: " .. best_probes .. "</fg>",
    probes = best_probes,
  }
end
