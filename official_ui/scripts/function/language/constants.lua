local M = {}

M.TOP_RESERVED_ROWS = 3
M.BOTTOM_RESERVED_ROWS = 3
M.CELL_PADDING = 4
M.CELL_HEIGHT = 3
M.MIN_CELL_WIDTH = 12
M.SELECTED_COLOR = "light_cyan"
M.NORMAL_COLOR = "white"
M.USE_COLOR = "green"
M.KEY_COLOR = "dark_gray"
M.PAGE_COLOR = "dark_gray"
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

M.DEFAULT_TEXT = {
  title = "Language",
  select = "Select",
  confirm = "Confirm",
  jump = "Jump",
  flip = "Flip",
  page = "Page",
  back = "Back",
  cancel = "Cancel",
  up_key = M.key_label(get_key("up_option").key_display.key_user),
  down_key = M.key_label(get_key("down_option").key_display.key_user),
  left_key = M.key_label(get_key("left_option").key_display.key_user),
  right_key = M.key_label(get_key("right_option").key_display.key_user),
  confirm_key = M.key_label(get_key("confirm").key_display.key_user),
  jump_key = M.key_label(get_key("jump").key_display.key_user),
  prev_page_key = M.key_label(get_key("prev_page").key_display.key_user),
  next_page_key = M.key_label(get_key("next_page").key_display.key_user),
  return_key = M.key_label(get_key("return").key_display.key_user)
}

return M
