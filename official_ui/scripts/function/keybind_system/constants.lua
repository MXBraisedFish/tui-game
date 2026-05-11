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
    local first = value[1]
    if first ~= nil then
      return display_key_value(first)
    end
    for _, v in pairs(value) do
      return display_key_value(v)
    end
    return ""
  end
  return tostring(value or "")
end

local function key_label(value)
  local key = display_key_value(value)
  if type(key) ~= "string" or key == "" then
    return "[]"
  end
  return "[" .. key .. "]"
end

local function safe_key_label(action_name)
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return key_label(info.key_display.key_user)
  end
  return "[" .. tostring(action_name or "?") .. "]"
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
  page_reset = "Reset page keys",
  key_any = "[Any]",
  add_shift = "Hold 2s for Shift",
  modify_shift = "Hold 2s for Shift",
  prev_option_key = safe_key_label("prev_option"),
  next_option_key = safe_key_label("next_option"),
  prev_page_key = safe_key_label("prev_page"),
  next_page_key = safe_key_label("next_page"),
  scroll_up_key = safe_key_label("scroll_up"),
  scroll_down_key = safe_key_label("scroll_down"),
  jump_key = safe_key_label("jump"),
  order_key = safe_key_label("order"),
  sort_key = safe_key_label("sort"),
  confirm_key = safe_key_label("confirm"),
  return_key = safe_key_label("return"),
  list_key = safe_key_label("list"),
  mode_key = safe_key_label("key_mode"),
  reset_only_key = safe_key_label("reset_only"),
  key1_key = safe_key_label("key1"),
  key2_key = safe_key_label("key2"),
  key3_key = safe_key_label("key3"),
  key4_key = safe_key_label("key4"),
  page_reset_key = safe_key_label("page_reset")
}

return M
