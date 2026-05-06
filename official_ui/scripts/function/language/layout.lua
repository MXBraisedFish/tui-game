local C = load_function("language/constants.lua")

local M = {}

function M.text_width(text)
  local width = get_text_width(text or "")
  if width == nil then
    return #(text or "")
  end
  return width
end

function M.terminal_size()
  local width, height = get_terminal_size()
  return width or 98, height or 26
end

function M.center_x(width, offset)
  return resolve_x(ANCHOR_CENTER, width, offset or 0)
end

function M.language(root_state, key, fallback)
  if type(root_state) == "table" and type(root_state.language) == "table" then
    local value = root_state.language[key]
    if value ~= nil and tostring(value) ~= "" then
      return tostring(value)
    end
  end
  return fallback
end

function M.ordered_languages(root_state)
  local order = {}
  if type(root_state) == "table" and type(root_state.language_order) == "table" then
    for _, code in ipairs(root_state.language_order) do
      order[#order + 1] = tostring(code)
    end
  end
  if #order == 0 and type(root_state) == "table" and type(root_state.languages) == "table" then
    for code, _ in pairs(root_state.languages) do
      order[#order + 1] = tostring(code)
    end
    table.sort(order)
  end
  return order
end

function M.language_name(root_state, code)
  if type(root_state) == "table" and type(root_state.languages) == "table" then
    local name = root_state.languages[code]
    if name ~= nil then
      return tostring(name)
    end
  end
  return tostring(code or "")
end

function M.grid(root_state)
  local terminal_width, terminal_height = M.terminal_size()
  local order = M.ordered_languages(root_state)
  local max_name_width = C.MIN_CELL_WIDTH
  for _, code in ipairs(order) do
    local width = M.text_width(M.language_name(root_state, code))
    if width > max_name_width then
      max_name_width = width
    end
  end

  local cell_width = max_name_width + C.CELL_PADDING
  local available_width = math.max(1, terminal_width)
  local available_height = math.max(1, terminal_height - C.TOP_RESERVED_ROWS - C.BOTTOM_RESERVED_ROWS)
  local columns = math.max(1, math.floor(available_width / cell_width))
  local rows = math.max(1, math.floor(available_height / C.CELL_HEIGHT))
  local per_page = math.max(1, columns * rows)
  local pages = math.max(1, math.ceil(#order / per_page))

  return {
    terminal_width = terminal_width,
    terminal_height = terminal_height,
    order = order,
    cell_width = cell_width,
    rows = rows,
    columns = columns,
    per_page = per_page,
    pages = pages,
    grid_width = columns * cell_width,
    grid_height = rows * C.CELL_HEIGHT,
    origin_x = M.center_x(columns * cell_width, 0),
    origin_y = C.TOP_RESERVED_ROWS
  }
end

return M
