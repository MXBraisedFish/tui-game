local M = {}

M.LEFT_RATIO = 0.33
M.MIN_PANEL_WIDTH = 24
M.BORDER_COLOR = "white"
M.TITLE_COLOR = "white"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.SELECTED_BG_COLOR = "#78a8da"
M.SELECTED_FG_COLOR = "black"
M.DELETE_BG_COLOR = "light_red"
M.EMPTY_BG_COLOR = "red"
M.HEADER_COLOR = "white"
M.SEPARATOR_COLOR = "white"
M.SORT_COLOR = "yellow"
M.ORDER_COLOR = "green"
M.CASE_COLOR = "yellow"
M.INPUT_BG_COLOR = "yellow"
M.INPUT_FG_COLOR = "black"
M.MAX_KEYS = 4
M.BORDER_CHARS = {
  top = "═",
  top_right = "╗",
  right = "║",
  bottom_right = "╝",
  bottom = "═",
  bottom_left = "╚",
  left = "║",
  top_left = "╔"
}

local function display_key_value(value)
  if type(value) == "table" then
    return value[1] or ""
  end
  return tostring(value or "")
end

local function key_label(value)
  local key = display_key_value(value)
  if key == "" then
    return "[]"
  end
  return "[" .. key .. "]"
end

M.DEFAULT_TEXT = {
  list_title = "Game List",
  key_title = "Key Info",
  action = "Action",
  key1 = "Key 1",
  key2 = "Key 2",
  key3 = "Key 3",
  key4 = "Key 4",
  sort_name = "Game name",
  sort_conflict = "Conflict keys",
  order_ascending = "Ascending↓",
  order_descending = "Descending↑",
  case_sensitive = "Keys are case-sensitive",
  select = "Select",
  confirm = "Confirm",
  back = "Back",
  list = "List",
  add = "Add",
  modify = "Modify",
  delete = "Delete",
  add_modify_tip = "Add/Modify key",
  delete_tip = "Delete key",
  key_mode = "Key mode",
  reset_only = "Reset action keys",
  prev_option_key = key_label(get_key("prev_option").key_display.key_user),
  next_option_key = key_label(get_key("next_option").key_display.key_user),
  prev_page_key = key_label(get_key("prev_page").key_display.key_user),
  next_page_key = key_label(get_key("next_page").key_display.key_user),
  scroll_up_key = key_label(get_key("scroll_up").key_display.key_user),
  scroll_down_key = key_label(get_key("scroll_down").key_display.key_user),
  jump_key = key_label(get_key("jump").key_display.key_user),
  order_key = key_label(get_key("order").key_display.key_user),
  sort_key = key_label(get_key("sort").key_display.key_user),
  confirm_key = key_label(get_key("confirm").key_display.key_user),
  return_key = key_label(get_key("return").key_display.key_user),
  list_key = key_label(get_key("list").key_display.key_user),
  mode_key = key_label(get_key("key_mode").key_display.key_user),
  delete_key = key_label(get_key("delete").key_display.key_user),
  reset_only_key = key_label(get_key("reset_only").key_display.key_user),
  key1_key = key_label(get_key("key1").key_display.key_user),
  key2_key = key_label(get_key("key2").key_display.key_user),
  key3_key = key_label(get_key("key3").key_display.key_user),
  key4_key = key_label(get_key("key4").key_display.key_user)
}

return M
