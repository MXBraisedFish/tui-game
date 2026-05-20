local M = {}
M.SELECTED_COLOR = CYAN
M.NORMAL_COLOR = WHITE
M.KEY_COLOR = DARK_GRAY
M.TITLE_COLOR = WHITE
M.HINT_COLOR = DARK_GRAY
M.VALUE_COLOR = GREEN
M.OFF_COLOR = RED
M.DISABLED_COLOR = DARK_GRAY
M.MOD_COLOR = YELLOW
M.SPLIT_COLOR = WHITE

local function append_key_labels(value, formatted)
  if type(value) == "table" then
    for _, item in ipairs(value) do
      append_key_labels(item, formatted)
    end
    return
  end
  local key = tostring(value or "")
  if key ~= "" then
    formatted[#formatted + 1] = "[" .. key .. "]"
  end
end

function key_label(keys)
  local formatted = {}
  append_key_labels(keys, formatted)
  if #formatted == 0 then
    return "[]"
  end
  return table.concat(formatted, "/")
end

local function safe_key_value(action_name)
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return info.key_display.key_user
  end
  return tostring(action_name or "?")
end

local function safe_key_label(action_name)
  return key_label(safe_key_value(action_name))
end

M.DEFAULT_TEXT = {
  title = "Display Settings",
  mod_on = "Show",
  mod_off = "Hide",
  saver_on = "Enable",
  saver_off = "Disable",
  second = "s",
  minute = "min",
  never = "Never",
  ordered = "Ordered",
  random = "Random",
  mode_off = "Disable",
  info_on = "Show",
  info_off = "Disable",
  theme_system = "System",
  option_mod = "Mod badge: ",
  option_theme = "Theme color: ",
  option_afk_time = "Idle threshold: ",
  option_afk_saver = "Enter screen saver on idle: ",
  option_info = "Host status display: ",
  option_saver_sort = "Screen saver display order: ",
  option_boss_sort = "Boss screen display order: ",
  option_saver_list = "Screen Saver List",
  option_boss_list = "Boss Screen List",
  list_on = "Enable",
  list_off = "Disable",
  list_mod = "MOD",
  select = "Select",
  toggle = "Toggle",
  confirm = "Confirm",
  back = "Back",
  order = "Order",
  position = "Toggle Position",
  scroll = "Scroll",
  select_key = key_label({safe_key_value("prev_option"), safe_key_value("next_option")}),
  scroll_key = key_label({safe_key_value("scroll_up"), safe_key_value("scroll_down")}),
  confirm_key = safe_key_label("confirm"),
  back_key = safe_key_label("return"),
  order_key = safe_key_label("order"),
  position_key = safe_key_label("position"),
}

return M
