local M = {}

M.BORDER_COLOR = "white"
M.TITLE_COLOR = "white"
M.SELECTED_BG_COLOR = "#78a8da"
M.SELECTED_FG_COLOR = "black"
M.NORMAL_COLOR = "white"
M.KEY_COLOR = "dark_gray"
M.MARK_COLOR = "yellow"
M.SORT_COLOR = "yellow"
M.ORDER_COLOR = "green"
M.PAGE_COLOR = "dark_gray"
M.HEADER_COLOR = "yellow"
M.INFO_LABEL_COLOR = "yellow"
M.INFO_TEXT_COLOR = "white"
M.INPUT_BG_COLOR = "yellow"
M.INPUT_FG_COLOR = "black"
M.LEFT_RATIO = 0.33
M.BOTTOM_RESERVED_ROWS = 3
M.MIN_PANEL_WIDTH = 24

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

local function action_key(action)
  local value = get_key(action)
  if type(value) == "table" and type(value.key_display) == "table" then
    return value.key_display.key_user
  end
  return nil
end

M.DEFAULT_TEXT = {
  list_title = "Game List",
  info_title = "Game Info",
  sort_name = "Name",
  sort_mod_official = "Official & Mod",
  sort_author = "Author",
  order_ascending = "Asc",
  order_descending = "Desc",
  source_mod = "MOD",
  info_mod = "Mod: ",
  info_author = "Author: ",
  info_version = "Version: ",
  none_game = "No games found",
  none_info = "No game information found",
  select = "Select",
  flip = "Flip",
  scroll = "Scroll",
  jump = "Jump",
  order = "Order",
  sort = "Sort",
  start = "Start",
  confirm = "Confirm",
  cancel = "Cancel",
  back = "Back",
  back_cancel = "Back / Cancel",
  game_name = "Game",
  package = "Mod: ",
  author = "Author: ",
  version = "Version: ",
  best_score = "Best",
  description = "Description",
  detail = "Detail",
  prev_option_key = key_label(action_key("prev_option")),
  next_option_key = key_label(action_key("next_option")),
  prev_page_key = key_label(action_key("prev_page")),
  next_page_key = key_label(action_key("next_page")),
  scroll_up_key = first_key_label(action_key("scroll_up")),
  scroll_down_key = first_key_label(action_key("scroll_down")),
  jump_key = key_label(action_key("jump")),
  order_key = key_label(action_key("order")),
  sort_key = key_label(action_key("sort")),
  confirm_key = key_label(action_key("confirm")),
  return_key = key_label(action_key("return"))
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
