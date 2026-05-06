local C = load_function("language/constants.lua")
local L = load_function("language/layout.lua")
local State = load_function("language/state.lua")

local M = {}

local function draw_title(root_state)
  local title = L.language(root_state, "LANGUAGE_TITLE", C.DEFAULT_TEXT.title)
  local x = L.center_x(L.text_width(title), 0)
  canvas_draw_text(x, 1, title, C.TITLE_COLOR, nil, BOLD, nil)
end

local function draw_languages(root_state)
  local grid = State.grid()
  local start_index = ((root_state.page or 1) - 1) * grid.per_page + 1
  local end_index = math.min(#grid.order, start_index + grid.per_page - 1)
  local border_chars = {
    top = "═",
    top_right = "╗",
    right = "║",
    bottom_right = "╝",
    bottom = "═",
    bottom_left = "╚",
    left = "║",
    top_left = "╔"
  }

  for index = start_index, end_index do
    local relative = index - start_index
    local row = math.floor(relative / grid.columns)
    local column = relative % grid.columns
    local code = grid.order[index]
    local name = L.language_name(root_state, code)
    local x = grid.origin_x + column * grid.cell_width
    local y = grid.origin_y + row * C.CELL_HEIGHT + 1
    local is_selected = code == root_state.select
    local is_used = code == root_state.use
    local color = C.NORMAL_COLOR
    if is_used then
      color = C.USE_COLOR
    end
    if is_selected then
      canvas_border_rect(x, y - 1, grid.cell_width, C.CELL_HEIGHT, border_chars, C.SELECTED_COLOR, nil)
    end
    local text_x = x + math.max(1, math.floor((grid.cell_width - L.text_width(name)) / 2))
    canvas_draw_text(text_x, y, name, color, nil, BOLD, nil)
  end
end

local function draw_page_line(root_state)
  local grid = State.grid()
  local y = grid.terminal_height - 2
  local current_page = tostring(root_state.page or 1)
  if root_state.jump then
    current_page = tostring(root_state.user_page or 0)
    if current_page == "0" then
      current_page = "_"
    end
  end
  local total_pages = tostring(root_state.pages or 1)
  local page_text = current_page .. "/" .. total_pages
  local page_x = L.center_x(L.text_width(page_text), 0)
  if root_state.jump then
    canvas_draw_text(page_x, y, current_page, C.INPUT_FG_COLOR, C.INPUT_BG_COLOR, BOLD, nil)
    canvas_draw_text(page_x + L.text_width(current_page), y, "/" .. total_pages, C.PAGE_COLOR, nil, BOLD, nil)
  else
    canvas_draw_text(page_x, y, page_text, C.PAGE_COLOR, nil, BOLD, nil)
  end

  if (root_state.page or 1) > 1 then
    local left = "◀ " .. C.DEFAULT_TEXT.prev_page_key
    canvas_draw_text(2, y, left, C.KEY_COLOR, nil, BOLD, nil)
  end
  if (root_state.page or 1) < (root_state.pages or 1) then
    local right = C.DEFAULT_TEXT.next_page_key .. " ▶"
    canvas_draw_text(grid.terminal_width - L.text_width(right) - 2, y, right, C.KEY_COLOR, nil, BOLD, nil)
  end
end

local function draw_action_line(root_state)
  local grid = State.grid()
  local y = grid.terminal_height - 1
  local action
  if root_state.jump then
    action = "[1]-[9] " .. L.language(root_state, "LANGUAGE_PAGE", C.DEFAULT_TEXT.page)
      .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "LANGUAGE_CONFIRM", C.DEFAULT_TEXT.confirm)
      .. "  " .. C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "LANGUAGE_CANCEL", C.DEFAULT_TEXT.cancel)
  else
    local move_keys = C.DEFAULT_TEXT.up_key .. "/" .. C.DEFAULT_TEXT.down_key .. "/"
      .. C.DEFAULT_TEXT.left_key .. "/" .. C.DEFAULT_TEXT.right_key
    action = move_keys .. " " .. L.language(root_state, "LANGUAGE_SELECT", C.DEFAULT_TEXT.select)
      .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "LANGUAGE_CONFIRM", C.DEFAULT_TEXT.confirm)
    if root_state.pages and root_state.pages > 1 then
      action = action
        .. "  " .. C.DEFAULT_TEXT.jump_key .. " " .. L.language(root_state, "LANGUAGE_JUMP", C.DEFAULT_TEXT.jump)
        .. "  " .. C.DEFAULT_TEXT.prev_page_key .. "/" .. C.DEFAULT_TEXT.next_page_key .. " " .. L.language(root_state, "LANGUAGE_FLIP", C.DEFAULT_TEXT.flip)
    end
    action = action .. "  " .. C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "LANGUAGE_BACK", C.DEFAULT_TEXT.back)
  end
  canvas_draw_text(L.center_x(L.text_width(action), 0), y, action, C.KEY_COLOR, nil, nil, nil)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  State.set_root_state(root_state)
  local grid = State.grid()
  root_state.pages = grid.pages
  root_state.page = math.max(1, math.min(grid.pages, root_state.page or 1))
  for index, code in ipairs(grid.order) do
    if code == root_state.select then
      root_state.page = math.max(1, math.ceil(index / grid.per_page))
      break
    end
  end

  draw_title(root_state)
  draw_languages(root_state)
  draw_page_line(root_state)
  draw_action_line(root_state)
end

return M
