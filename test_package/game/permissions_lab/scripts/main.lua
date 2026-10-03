-- Permissions lab package callbacks and terminal-cell drawing.

local PROBE_PATH = "state/probe.log"
local MAX_LOG_LINES = 10

local width = 80
local height = 24
local probes = 0
local best_probes = 0
local log_lines = {}

-- Append a visible diagnostic and discard the oldest entries beyond the display limit.
local function push_log(line)
  table.insert(log_lines, line)
  while #log_lines > MAX_LOG_LINES do
    table.remove(log_lines, 1)
  end
end

-- Return the first line of text, or an empty string when no line is present.
local function first_line(text)
  local lines = string.split(text, "\n")
  return lines[1] or ""
end

-- Display file-request outcomes and report failures through the debug library.
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
    for _, entry in ipairs(data.entries) do
      table.insert(names, entry.path .. " [" .. entry.file_type .. "]")
    end
    push_log("state/ contains: " .. table.concat(names, ", "))
  end
end

-- Assert successful JSON decoding and rejection of malformed JSON.
local function self_check()
  local decoded, decoded_value1 = debug.pcall(function()
      return serialization.json_decode("{\"probes\": 1}")
    end)
  debug.assert(decoded and decoded_value1.probes == 1, { message = "json round trip failed" })
  local broken = debug.pcall(function()
      return serialization.json_decode("{broken")
    end)
  debug.assert(not broken, { message = "invalid JSON was accepted" })
end

-- Initialize package state from the supplied base dimensions and startup data.
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
  debug.info("Permissions Lab initialized")
  push_log("P writes inside package assets, R reads it back, L lists state/.")
  if file.exists(PROBE_PATH) then
    push_log("A probe file from an earlier run exists.")
  end
end

-- Apply the resize, action, or completion events handled by this package.
function HandleEvent(event)
  if event.type == "resize" then
    width = event.data.width
    height = event.data.height
  elseif event.type == "action" and event.data.state == "pressed" then
    local action = event.data.action
    if action == "write_probe" then
      probes = probes + 1
      debug.print("write probe #" .. probes, { title = "permissions_lab", level = debug.INFO, time = true })
      file.write(PROBE_PATH, "probe=" .. probes .. "\n", {
        encoding = file.UTF_8,
        end_of_line = file.LF,
        event_tip = "probe_written",
      })
      push_log("Requested write #" .. probes .. " inside package assets.")
    elseif action == "read_probe" then
      if file.exists(PROBE_PATH) then
        file.read(PROBE_PATH, { encoding = file.UTF_8, event_tip = "probe_read" })
        push_log("Requested a read of " .. PROBE_PATH .. ".")
      else
        push_log("Nothing to read yet; write a probe first.")
      end
    elseif action == "list_probe" then
      file.list_dir("state", { event_tip = "state_listed" })
      push_log("Requested a listing of state/.")
    elseif action == "leave" then
      game.exit_game()
    end
  elseif event.type == "file" then
    handle_file_event(event.data)
  end
end

-- Keep this optional callback empty; this package needs no work in this phase.
function Update(dt)
end

-- Keep this optional callback empty; this package needs no work in this phase.
function UpdateFrame(dt, alpha)
end

-- Draw the current package state in terminal-cell coordinates.
function Render()
  draw.fill_rect(0, 0, width, height, { char = " ", bg = color.BLACK })
  draw.stroke_rect(0, 0, width, height, { fg = color.BRIGHT_RED, border_char = char.DOUBLE_LINE })
  draw.text(2, 1, "Permissions Lab", { fg = color.BRIGHT_RED, bold = true })
  draw.text(2, 3, "Probes requested: " .. probes .. "   Best: " .. best_probes, { fg = color.BRIGHT_YELLOW })
  for line_number = 1, #log_lines do
    draw.text(2, 4 + line_number, log_lines[line_number], { fg = color.BRIGHT_GRAY, max_width = width - 4, max_height = 1 })
  end
  draw.text(2, height - 2, "P write  R read  L list  Esc leave", { fg = color.GRAY })
end

-- Return the structured state needed to continue this game.
function SaveGame()
  return { probes = probes }
end

-- Return the best-result data and its display text for host score persistence.
function SaveBest()
  if probes > best_probes then
    best_probes = probes
  end
  return {
    best_string = "f%<fg:bright_red>Completed probes: " .. best_probes .. "</fg>",
    probes = best_probes,
  }
end
