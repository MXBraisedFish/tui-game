local M = {}

M.MENU_WIDTH = 34
M.MENU_HEIGHT = 6
M.CONTENT_HEIGHT = 9
M.SELECTED_COLOR = CYAN
M.NORMAL_COLOR = "white"
M.KEY_COLOR = DARK_GRAY
M.VERSION_COLOR = DARK_GRAY
M.TITLE_COLOR = "white"

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
  local info = get_key(action_name)
  if type(info) == "table" and type(info.key_display) == "table" then
    return key_label(info.key_display.key_user)
  end
  return "[" .. tostring(action_name or "?") .. "]"
end

M.DEFAULT_TEXT = {
  title = "Settings",
  language = "Language",
  keybind = "Key Bindings",
  mods = "Mod List",
  memory = "Memory Management",
  security = "Security Settings",
  display = "Display Settings",
  enter = safe_key_label("confirm"),
  option1 = safe_key_label("option1"),
  option2 = safe_key_label("option2"),
  option3 = safe_key_label("option3"),
  option4 = safe_key_label("option4"),
  option5 = safe_key_label("option5"),
  option6 = safe_key_label("option6"),
  back_key = safe_key_label("return"),
  select_key = key_label({safe_key_value("prev_option"), safe_key_value("next_option")}),
  confirm_key = safe_key_label("confirm"),
  select = "Select",
  confirm = "Confirm",
  back = "Back"
}

return M
