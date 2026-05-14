local C = load_function("mod_security/constants.lua")
local L = load_function("mod_security/layout.lua")

local M = {}

local function remaining_seconds(started_at, delay_ms)
  local elapsed = math.max(0, now() - (started_at or now()))
  local remaining = math.max(0, delay_ms - elapsed)
  return math.ceil(remaining / 1000)
end

local function draw_title(root_state)
  local title = L.language(root_state, "MOD_SECURITY_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function add_option(lines, key, text, remain, second)
  if remain > 0 then
    lines[#lines + 1] = {
      text = key .. " " .. text .. " " .. tostring(remain) .. second,
      color = C.DISABLED_COLOR
    }
  else
    lines[#lines + 1] = { text = key .. " " .. text, color = C.CONFIRM_COLOR }
  end
end

local function build_lines(root_state, started_at)
  local lines = {}
  local warn = L.language(root_state, "MOD_SECURITY_WARN", C.DEFAULT_TEXT.warn)
  for _, line in ipairs(L.wrap(warn, 72)) do
    lines[#lines + 1] = { text = line, color = C.WARN_COLOR }
  end

  local package_name = tostring(root_state.package_name or root_state.mod_uid or "")
  local mod_label = L.language(root_state, "MOD_SECURITY_MOD", C.DEFAULT_TEXT.mod)
  lines[#lines + 1] = { text = "", color = C.MOD_COLOR }
  lines[#lines + 1] = { text = mod_label .. package_name, color = C.MOD_COLOR }

  local cancel = L.language(root_state, "MOD_SECURITY_CANCEL", C.DEFAULT_TEXT.cancel)
  local close_temporary = L.language(root_state, "MOD_SECURITY_CLOSE_TEMPORARY", C.DEFAULT_TEXT.close_temporary)
  local close_permanent = L.language(root_state, "MOD_SECURITY_CLOSE_PERMANENT", C.DEFAULT_TEXT.close_permanent)
  local second = L.language(root_state, "MOD_SECURITY_SECOND", C.DEFAULT_TEXT.second)
  lines[#lines + 1] = { text = "", color = C.MOD_COLOR }
  lines[#lines + 1] = { text = C.DEFAULT_TEXT.cancel_key .. " " .. cancel, color = C.CANCEL_COLOR }
  add_option(
    lines,
    C.DEFAULT_TEXT.temporary_key,
    close_temporary,
    remaining_seconds(started_at, C.TEMPORARY_DELAY_MS),
    second
  )
  add_option(
    lines,
    C.DEFAULT_TEXT.permanent_key,
    close_permanent,
    remaining_seconds(started_at, C.PERMANENT_DELAY_MS),
    second
  )

  return lines
end

function M.render(root_state, started_at)
  canvas_clear()
  root_state = root_state or {}
  draw_title(root_state)

  local lines = build_lines(root_state, started_at)
  local max_width = 0
  for _, line in ipairs(lines) do
    max_width = math.max(max_width, L.text_width(line.text))
  end

  local terminal_width, _ = L.terminal_size()
  local draw_width = math.min(max_width, math.max(1, terminal_width - 2))
  local x = math.max(0, L.center_x(draw_width, 0))
  local y = resolve_y(ANCHOR_MIDDLE, #lines, 0)
  for index, line in ipairs(lines) do
    canvas_draw_text(x, y + index - 1, line.text, line.color, nil, BOLD, ALIGN_LEFT)
  end
end

return M
