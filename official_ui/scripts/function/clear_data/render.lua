local C = load_function("clear_data/constants.lua")
local L = load_function("clear_data/layout.lua")

local M = {}

local function remaining_seconds(started_at)
  local elapsed = math.max(0, now() - (started_at or now()))
  local remaining = math.max(0, C.CONFIRM_DELAY_MS - elapsed)
  return math.ceil(remaining / 1000)
end

local function draw_title(root_state)
  local title = L.language(root_state, "CLEAR_DATA_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function build_lines(root_state, started_at)
  local lines = {}
  local warn = L.language(root_state, "CLEAR_DATA_WARN", C.DEFAULT_TEXT.warn)
  for _, line in ipairs(L.wrap(warn, 72)) do
    lines[#lines + 1] = { text = line, color = C.WARN_COLOR }
  end

  local dir = root_state.dir or {}
  local data_path = L.language(root_state, "CLEAR_DATA_PATH", C.DEFAULT_TEXT.path)
    .. tostring(dir.data_dir or "")
  lines[#lines + 1] = { text = "", color = C.TITLE_COLOR }
  lines[#lines + 1] = { text = data_path, color = C.TITLE_COLOR }

  local remain = remaining_seconds(started_at)
  local confirm = L.language(root_state, "CLEAR_DATA_CONFIRM", C.DEFAULT_TEXT.confirm)
  local cancel = L.language(root_state, "CLEAR_DATA_CANCEL", C.DEFAULT_TEXT.cancel)
  lines[#lines + 1] = { text = "", color = C.TITLE_COLOR }
  lines[#lines + 1] = { text = C.DEFAULT_TEXT.cancel_key .. " " .. cancel, color = C.CANCEL_COLOR }
  if remain > 0 then
    lines[#lines + 1] = {
      text = C.DEFAULT_TEXT.confirm_key .. " " .. confirm .. " " .. tostring(remain) .. L.language(root_state, "CLEAR_DATA_SECOND", C.DEFAULT_TEXT.second),
      color = C.DISABLED_COLOR
    }
  else
    lines[#lines + 1] = { text = C.DEFAULT_TEXT.confirm_key .. " " .. confirm, color = C.CONFIRM_COLOR }
  end
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
