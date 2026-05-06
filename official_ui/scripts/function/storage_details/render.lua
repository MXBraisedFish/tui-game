local C = load_function("storage_details/constants.lua")
local F = load_function("storage_details/format.lua")
local L = load_function("storage_details/layout.lua")

local M = {}

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

local function calculate_table_frame(root_state)
  local terminal_width, terminal_height = L.terminal_size()
  local name_title = L.language(root_state, "MEMORY_INFO_DIR", C.DEFAULT_TEXT.dir)
  local size_title = L.language(root_state, "MEMORY_INFO_SIZE", C.DEFAULT_TEXT.size)
  local path_title = L.language(root_state, "MEMORY_INFO_PATH", C.DEFAULT_TEXT.path)
  local max_name_width = L.text_width(name_title)
  local max_size_width = L.text_width(size_title)
  local max_path_width = L.text_width(path_title)

  for _, row in ipairs(directory_rows(root_state)) do
    max_name_width = math.max(max_name_width, L.text_width(row.name))
    max_size_width = math.max(max_size_width, L.text_width(row.size))
    max_path_width = math.max(max_path_width, L.text_width(row.path))
  end

  local name_width = max_name_width + 6
  local size_width = max_size_width + 6
  local width = math.min(terminal_width - 4, name_width + size_width + max_path_width)
  local content_height = 8
  return {
    terminal_width = terminal_width,
    terminal_height = terminal_height,
    x = L.center_x(width, 0),
    y = resolve_y(ANCHOR_MIDDLE, content_height, 0),
    width = width,
    name_width = name_width,
    size_width = size_width
  }
end

local function draw_title(root_state)
  local title = L.language(root_state, "MEMORY_SHOW", C.DEFAULT_TEXT.title)
  canvas_draw_text(L.center_x(L.text_width(title), 0), 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function draw_table_header(root_state, frame, y)
  local name_title = L.language(root_state, "MEMORY_INFO_DIR", C.DEFAULT_TEXT.dir)
  local size_title = L.language(root_state, "MEMORY_INFO_SIZE", C.DEFAULT_TEXT.size)
  local path_title = L.language(root_state, "MEMORY_INFO_PATH", C.DEFAULT_TEXT.path)
  local size_x = frame.x + frame.name_width
  local path_x = size_x + frame.size_width
  canvas_draw_text(frame.x, y, name_title, C.HEADER_COLOR, nil, BOLD, nil)
  canvas_draw_text(size_x, y, size_title, C.HEADER_COLOR, nil, BOLD, nil)
  canvas_draw_text(path_x, y, path_title, C.HEADER_COLOR, nil, BOLD, nil)
end

local function draw_table(root_state, frame)
  local size_x = frame.x + frame.name_width
  local path_x = size_x + frame.size_width
  draw_table_header(root_state, frame, frame.y)

  for index, row in ipairs(directory_rows(root_state)) do
    local y = frame.y + index + 1
    canvas_draw_text(frame.x, y, row.name, C.NORMAL_COLOR, nil, nil, nil)
    canvas_draw_text(size_x, y, row.size, C.NORMAL_COLOR, nil, nil, nil)
    canvas_draw_text(path_x, y, row.path, C.PATH_COLOR, nil, nil, nil)
  end
end

local function draw_tip(root_state, frame)
  local tip = L.language(root_state, "MEMORY_TIP", C.DEFAULT_TEXT.tip)
  canvas_draw_text(L.center_x(L.text_width(tip), 0), frame.terminal_height - 2, tip, C.TIP_COLOR, nil, nil, nil)
end

local function draw_action_line(root_state, frame)
  local back_text = L.language(root_state, "STORAGE_DETAILS_BACK", C.DEFAULT_TEXT.back)
  local action = C.DEFAULT_TEXT.back_key .. " " .. back_text
  canvas_draw_text(L.center_x(L.text_width(action), 0), frame.terminal_height - 1, action, C.KEY_COLOR, nil, nil, nil)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  local frame = calculate_table_frame(root_state)
  draw_title(root_state)
  draw_table(root_state, frame)
  draw_tip(root_state, frame)
  draw_action_line(root_state, frame)
end

return M
