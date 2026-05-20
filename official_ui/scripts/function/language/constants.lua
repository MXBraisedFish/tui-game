local M = {}

M.TOP_RESERVED_ROWS = 3
M.BOTTOM_RESERVED_ROWS = 3
M.CELL_PADDING = 4
M.CELL_HEIGHT = 3
M.MIN_CELL_WIDTH = 12
M.SELECTED_COLOR = CYAN
M.NORMAL_COLOR = "white"
M.USE_COLOR = "green"
M.KEY_COLOR = DARK_GRAY
M.PAGE_COLOR = DARK_GRAY
M.TITLE_COLOR = "white"
M.INPUT_BG_COLOR = "yellow"
M.INPUT_FG_COLOR = "black"

function M.key_label(keys)
  if type(keys) == "string" then
    return "[" .. keys .. "]"
  elseif type(keys) == "table" then
    local formatted = {}
    for i, key in ipairs(keys) do
      formatted[i] = "[" .. key .. "]"
    end
    return table.concat(formatted, "/")
  end
  return "[]"
end

local function safe_key_label(action_name)
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return M.key_label(info.key_display.key_user)
  end
  return "[" .. tostring(action_name or "?") .. "]"
end

M.DEFAULT_TEXT = {
  title = "Language",
  select = "Select",
  confirm = "Confirm",
  jump = "Jump",
  flip = "Flip",
  page = "Page",
  back = "Back",
  cancel = "Cancel",
  up_key = safe_key_label("up_option"),
  down_key = safe_key_label("down_option"),
  left_key = safe_key_label("left_option"),
  right_key = safe_key_label("right_option"),
  confirm_key = safe_key_label("confirm"),
  jump_key = safe_key_label("jump"),
  prev_page_key = safe_key_label("prev_page"),
  next_page_key = safe_key_label("next_page"),
  return_key = safe_key_label("return")
}

return M
