local M = {}

M.BORDER_COLOR = "white"
M.TITLE_COLOR = "white"
M.SELECTED_BG_COLOR = DARK_GRAY
M.SELECTED_FG_COLOR = "white"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = DARK_GRAY
M.MARK_COLOR = "yellow"
M.SORT_COLOR = "yellow"
M.ORDER_COLOR = "green"
M.PAGE_COLOR = DARK_GRAY
M.HEADER_COLOR = "yellow"
M.INFO_LABEL_COLOR = "yellow"
M.INFO_TEXT_COLOR = "white"
M.INPUT_BG_COLOR = "yellow"
M.INPUT_FG_COLOR = "black"
M.ON_COLOR = "green"
M.OFF_COLOR = "red"
M.DANGER_COLOR = "red"
M.DISABLED_COLOR = DARK_GRAY
M.DEBUG_COLOR = "red"
M.SAFE_OFF_BG_COLOR = "red"
M.LEFT_RATIO = 0.33
M.BOTTOM_RESERVED_ROWS = 1
M.MIN_PANEL_WIDTH = 24
M.FULL_ITEM_HEIGHT = 5
M.BRIEF_ITEM_HEIGHT = 1
M.ICON_WIDTH = 8
M.ICON_HEIGHT = 4

local function key_label(keys)
  if type(keys) == "string" then
    return "[" .. keys .. "]"
  elseif type(keys) == "table" then
    local formatted = {}
    for index, key in ipairs(keys) do
      formatted[index] = "[" .. tostring(key) .. "]"
    end
    return table.concat(formatted, "/")
  end
  return "[]"
end

local function first_key_label(keys)
  if type(keys) == "table" then
    return key_label(keys[1])
  end
  return key_label(keys)
end

local function first_key_text(keys)
  if type(keys) == "table" then
    return tostring(keys[1] or "")
  elseif type(keys) == "string" then
    return keys
  end
  return ""
end

local function action_key(action)
  local value = get_key(action)
  if type(value) == "table" and type(value.key_display) == "table" then
    return value.key_display.key_user
  end
  return nil
end

M.DEFAULT_TEXT = {
  list_title = "Mods",
  info_title = "Mod Info",
  sort_name = "Name",
  sort_author = "Author",
  sort_safe_mode = "Safe Mode",
  sort_toggle = "Enabled",
  order_ascending = "Asc",
  order_descending = "Desc",
  author = "Author: ",
  version = "Version: ",
  base = "Basic Info:",
  safe = "Security Info:",
  safe_switch = "Enabled: ",
  status = "Status: ",
  safe_debug = "Debug: ",
  safe_write = "Write Request: ",
  safe_safe_mode = "Safe Mode: ",
  introduction = "Introduction:",
  none_mod = "No mods found",
  none_info = "No mod information found",
  mod_on = "Enabled",
  mod_off = "Disabled",
  mod_on_brief = "On",
  mod_off_brief = "Off",
  write_on = "Required",
  write_off = "Not Required",
  debug_on = "On",
  debug_off = "Off",
  safe_mode_on = "On",
  safe_mode_off_temporary = "Off (Session)",
  safe_mode_off_permanent = "Off (Permanent)",
  select = "Select",
  flip = "Flip",
  scroll = "Scroll",
  jump = "Jump",
  order = "Order",
  sort = "Sort",
  toggle = "Toggle",
  confirm = "Confirm",
  cancel = "Cancel",
  toggle_confirm = "Toggle / Confirm",
  back = "Back",
  debug = "Debug",
  list = "List",
  safe_mode = "Safe Mode",
  prev_option_key = key_label(action_key("prev_option")),
  next_option_key = key_label(action_key("next_option")),
  prev_page_key = key_label(action_key("prev_page")),
  next_page_key = key_label(action_key("next_page")),
  scroll_up_key = first_key_label(action_key("scroll_up")),
  scroll_down_key = first_key_label(action_key("scroll_down")),
  scroll_up_key_text = first_key_text(action_key("scroll_up")),
  scroll_down_key_text = first_key_text(action_key("scroll_down")),
  jump_key = key_label(action_key("jump")),
  order_key = key_label(action_key("order")),
  sort_key = key_label(action_key("sort")),
  confirm_key = key_label(action_key("confirm")),
  return_key = key_label(action_key("return")),
  debug_key = key_label(action_key("debug")),
  list_key = key_label(action_key("list")),
  safe_mode_key = key_label(action_key("safe_mode"))
}

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

return M
