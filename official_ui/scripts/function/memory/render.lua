local C = load_function("memory/constants.lua")
local F = load_function("memory/format.lua")
local L = load_function("memory/layout.lua")

local M = {}

local function selected_index(root_state, count)
  if type(root_state) == "table" and type(root_state.select) == "number" then
    return math.max(1, math.min(count or 1, math.floor(root_state.select)))
  end
  return 1
end

local function option_items(root_state)
  return {
    {
      key = C.DEFAULT_TEXT.option1,
      label = L.language(root_state, "MEMORY_CACHE", C.DEFAULT_TEXT.cache)
    },
    {
      key = C.DEFAULT_TEXT.option2,
      label = L.language(root_state, "MEMORY_DATA", C.DEFAULT_TEXT.data)
    },
    {
      key = C.DEFAULT_TEXT.option3,
      label = L.language(root_state, "MEMORY_SHOW", C.DEFAULT_TEXT.show)
    }
  }
end

local function directory_rows(root_state)
  local dir = root_state.dir or {}
  local size = root_state.size or {}
  return {
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_ROOT", C.DEFAULT_TEXT.root),
      size = F.size(size.root_size),
      path = tostring(dir.root_dir or "")
    },
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_DATA", C.DEFAULT_TEXT.data_dir),
      size = F.size(size.data_size),
      path = tostring(dir.data_dir or "")
    },
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_CACHE", C.DEFAULT_TEXT.cache_dir),
      size = F.size(size.cache_size),
      path = tostring(dir.cache_dir or "")
    },
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_PROFILES", C.DEFAULT_TEXT.profiles_dir),
      size = F.size(size.profiles_size),
      path = tostring(dir.profiles_dir or "")
    },
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_LOG", C.DEFAULT_TEXT.log_dir),
      size = F.size(size.log_size),
      path = tostring(dir.log_dir or "")
    },
    {
      name = L.language(root_state, "MEMORY_INFO_NAME_MOD", C.DEFAULT_TEXT.mod_dir),
      size = F.size(size.mod_size),
      path = tostring(dir.mod_dir or "")
    }
  }
end

local function calculate_frame(root_state)
  local frame = L.content_frame()
  local max_name_width = 0
  local max_size_width = 0
  local max_path_width = 0

  local name_title = L.language(root_state, "MEMORY_INFO_DIR", C.DEFAULT_TEXT.dir)
  local size_title = L.language(root_state, "MEMORY_INFO_SIZE", C.DEFAULT_TEXT.size)
  local path_title = L.language(root_state, "MEMORY_INFO_PATH", C.DEFAULT_TEXT.path)
  max_name_width = L.text_width(name_title)
  max_size_width = L.text_width(size_title)
  max_path_width = math.max(max_path_width, L.text_width(path_title))

  for _, row in ipairs(directory_rows(root_state)) do
    max_name_width = math.max(max_name_width, L.text_width(row.name))
    max_size_width = math.max(max_size_width, L.text_width(row.size))
    max_path_width = math.max(max_path_width, L.text_width(row.path))
  end

  frame.name_width = max_name_width + 6
  frame.size_width = max_size_width + 6
  frame.width = math.min(frame.terminal_width - 4, frame.name_width + frame.size_width + max_path_width)
  frame.x = L.center_x(frame.width, 0)
  return frame
end

local function draw_title(root_state)
  local title = L.language(root_state, "MEMORY_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function draw_options(root_state, frame)
  local items = option_items(root_state)
  local selected = selected_index(root_state, #items)
  local option_width = 0

  for _, item in ipairs(items) do
    local key_width = math.max(L.text_width(item.key), L.text_width(C.DEFAULT_TEXT.confirm_key))
    local line_width = L.text_width("▶ ") + key_width + L.text_width(" " .. item.label)
    option_width = math.max(option_width, line_width)
  end

  local option_x = L.center_x(option_width, 0)
  local y = frame.y
  for index, item in ipairs(items) do
    local is_selected = index == selected
    local prefix = is_selected and "▶ " or "  "
    local key_text = is_selected and C.DEFAULT_TEXT.confirm_key or item.key
    local color = is_selected and C.SELECTED_COLOR or C.NORMAL_COLOR
    local cursor_x = option_x
    canvas_draw_text(cursor_x, y + index - 1, prefix, color, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(prefix)
    canvas_draw_text(cursor_x, y + index - 1, key_text, C.KEY_COLOR, nil, BOLD, nil)
    cursor_x = cursor_x + L.text_width(key_text)
    canvas_draw_text(cursor_x, y + index - 1, " " .. item.label, color, nil, BOLD, nil)
  end
end

local function draw_action_line(root_state, frame)
  local select_text = L.language(root_state, "MEMORY_SELECT", C.DEFAULT_TEXT.select)
  local confirm_text = L.language(root_state, "MEMORY_CONFIRM", C.DEFAULT_TEXT.confirm)
  local back_text = L.language(root_state, "MEMORY_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.select_key .. " " .. select_text
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. confirm_text
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. back_text
  canvas_draw_text(L.center_x(L.text_width(action), 0), frame.terminal_height - 1, action, C.KEY_COLOR, nil, nil, nil)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  local frame = calculate_frame(root_state)
  frame.y = resolve_y(ANCHOR_MIDDLE, #option_items(root_state), 0)
  draw_title(root_state, frame)
  draw_options(root_state, frame)
  draw_action_line(root_state, frame)
end

return M
