local C = load_function("display/constants.lua")
local L = load_function("display/layout.lua")

local M = {}

local function center_in_area(area_x, area_width, content_width)
  return math.max(0, area_x + math.floor((math.max(0, area_width) - math.max(0, content_width)) / 2))
end

local function safe_area_width(width)
  return math.max(1, math.floor(width or 1))
end

function mode_text(root_state, mode)
  if mode == "random" then return L.language(root_state, "DISPLAY_TOGGLE_SORT_RANDOM", C.DEFAULT_TEXT.random) end
  if mode == "off" then return L.language(root_state, "DISPLAY_TOGGLE_SORT_OFF", C.DEFAULT_TEXT.mode_off) end
  return L.language(root_state, "DISPLAY_TOGGLE_SORT_ORDER", C.DEFAULT_TEXT.ordered)
end

local function setting_value(root_state, index)
  local settings = root_state.settings or {}
  if index == 1 then
    return settings.mod_badge and L.language(root_state, "DISPLAY_TOGGLE_MOD_ON", C.DEFAULT_TEXT.mod_on) or L.language(root_state, "DISPLAY_TOGGLE_MOD_OFF", C.DEFAULT_TEXT.mod_off)
  elseif index == 2 then
    return L.language(root_state, "DISPLAY_TOGGLE_THEME_SYSTEM", C.DEFAULT_TEXT.theme_system)
  elseif index == 3 then
    local value = tonumber(settings.idle_threshold or 60) or 60
    if value <= 0 then return L.language(root_state, "DISPLAY_TOGGLE_AFK_TIME_NEVER", C.DEFAULT_TEXT.never) end
    if value >= 60 and value % 60 == 0 then
      return tostring(math.floor(value / 60)) .. L.language(root_state, "DISPLAY_TOGGLE_AFK_TIME_MINUTE", C.DEFAULT_TEXT.minute)
    end
    return tostring(value) .. L.language(root_state, "DISPLAY_TOGGLE_AFK_TIME_SECOND", C.DEFAULT_TEXT.second)
  elseif index == 4 then
    return settings.idle_enter_saver and L.language(root_state, "DISPLAY_TOGGLE_AFK_SAVER_ON", C.DEFAULT_TEXT.saver_on) or L.language(root_state, "DISPLAY_TOGGLE_AFK_SAVER_OFF", C.DEFAULT_TEXT.saver_off)
  elseif index == 5 then
    return settings.host_status and L.language(root_state, "DISPLAY_OPTION_INFO_ON", C.DEFAULT_TEXT.info_on) or L.language(root_state, "DISPLAY_OPTION_INFO_OFF", C.DEFAULT_TEXT.info_off)
  elseif index == 6 then
    return mode_text(root_state, settings.saver_mode)
  elseif index == 7 then
    return mode_text(root_state, settings.boss_mode)
  end
  return ""
end

local function value_color(root_state, index)
  local settings = root_state.settings or {}
  if index == 1 then
    return settings.mod_badge ~= false and C.VALUE_COLOR or C.OFF_COLOR
  elseif index == 3 then
    local value = tonumber(settings.idle_threshold or 60) or 60
    return value > 0 and C.VALUE_COLOR or C.OFF_COLOR
  elseif index == 4 then
    return settings.idle_enter_saver ~= false and C.VALUE_COLOR or C.OFF_COLOR
  elseif index == 5 then
    return settings.host_status ~= false and C.VALUE_COLOR or C.OFF_COLOR
  elseif index == 6 then
    return (settings.saver_mode or "ordered") ~= "off" and C.VALUE_COLOR or C.OFF_COLOR
  elseif index == 7 then
    return (settings.boss_mode or "ordered") ~= "off" and C.VALUE_COLOR or C.OFF_COLOR
  end
  return C.VALUE_COLOR
end

local function option_rows(root_state)
  return {
    { label = L.language(root_state, "DISPLAY_OPTION_MOD", C.DEFAULT_TEXT.option_mod), value = setting_value(root_state, 1) },
    { label = L.language(root_state, "DISPLAY_OPTION_THEME", C.DEFAULT_TEXT.option_theme), value = setting_value(root_state, 2) },
    { label = L.language(root_state, "DISPLAY_OPTION_AFK_TIME", C.DEFAULT_TEXT.option_afk_time), value = setting_value(root_state, 3) },
    { label = L.language(root_state, "DISPLAY_OPTION_AFK_SAVER", C.DEFAULT_TEXT.option_afk_saver), value = setting_value(root_state, 4) },
    { label = L.language(root_state, "DISPLAY_OPTION_INFO", C.DEFAULT_TEXT.option_info), value = setting_value(root_state, 5) },
    { label = L.language(root_state, "DISPLAY_OPTION_SAVER_SORT", C.DEFAULT_TEXT.option_saver_sort), value = setting_value(root_state, 6) },
    { label = L.language(root_state, "DISPLAY_OPTION_BOSS_SORT", C.DEFAULT_TEXT.option_boss_sort), value = setting_value(root_state, 7) },
    { label = L.language(root_state, "DISPLAY_OPTION_SAVER_LIST", C.DEFAULT_TEXT.option_saver_list), value = "" },
    { label = L.language(root_state, "DISPLAY_OPTION_BOSS_LIST", C.DEFAULT_TEXT.option_boss_list), value = "" },
  }
end

local function setting_key_text(index, selected)
  if selected then return C.DEFAULT_TEXT.confirm_key end
  return "[" .. tostring(index) .. "]"
end

local function max_value_width(rows)
  local max_w = 0
  for _, row in ipairs(rows) do
    if row.value ~= "" then
      max_w = math.max(max_w, L.text_width(row.value))
    end
  end
  return max_w
end

local function setting_line_width(row, index, key_w, val_max_w)
  local label = tostring(row.label or ""):gsub("%s+$", "")
  local width = key_w + L.text_width(" " .. label)
  if row.value ~= "" then
    width = width + L.text_width("[ ") + val_max_w + L.text_width(" ]")
  end
  return width
end

local function block_layout(root_state, readonly)
  local rows = option_rows(root_state)
  local key_w = math.max(L.text_width("[1]"), L.text_width(C.DEFAULT_TEXT.confirm_key))
  local val_max_w = max_value_width(rows)
  local width = 1
  for index, row in ipairs(rows) do
    width = math.max(width, setting_line_width(row, index, key_w, val_max_w))
  end
  return rows, key_w, val_max_w, width
end

local function draw_text_segment(x, y, text, fg, bg, style, max_width)
  local value = tostring(text or "")
  if max_width <= 0 or value == "" then return x end
  canvas_draw_text(x, y, value, fg, bg, style, ALIGN_LEFT, {wrap_width = max_width, wrap_height = 1, text_overflow = "..."})
  return x + math.min(L.text_width(value), max_width)
end

local function draw_settings(root_state, x, y, width, rows, key_w, readonly)
  local selected = math.max(1, math.min(9, math.floor(root_state.select or 1)))
  for index, row in ipairs(rows) do
    local row_y = y + index - 1
    local label = tostring(row.label or ""):gsub("%s+$", "")
    local is_selected = (index == selected and not readonly and (root_state.panel or "none") == "none")
    local key_text = setting_key_text(index, is_selected)
    local color = is_selected and C.SELECTED_COLOR or C.NORMAL_COLOR
    local cursor_x = math.max(0, x)
    local remaining = safe_area_width(width)

    if is_selected then
      canvas_draw_text(math.max(0, x - 2), row_y, "▶", color, nil, BOLD, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
    end

    cursor_x = draw_text_segment(cursor_x, row_y, key_text, C.KEY_COLOR, nil, BOLD, remaining)
    cursor_x = cursor_x + L.text_width(" ")
    remaining = math.max(0, x + width - cursor_x)
    cursor_x = draw_text_segment(cursor_x, row_y, label, color, nil, BOLD, remaining)

    if row.value ~= "" and cursor_x < x + width then
      remaining = math.max(0, x + width - cursor_x)
      cursor_x = draw_text_segment(cursor_x, row_y, "[ ", C.NORMAL_COLOR, nil, BOLD, remaining)
      remaining = math.max(0, x + width - cursor_x)
      cursor_x = draw_text_segment(cursor_x, row_y, row.value, value_color(root_state, index), nil, BOLD, remaining)
      remaining = math.max(0, x + width - cursor_x)
      draw_text_segment(cursor_x, row_y, " ]", C.NORMAL_COLOR, nil, BOLD, remaining)
    end
  end
end

local function list_items(root_state)
  if root_state.panel == "saver" then return root_state.saver_list or {} end
  if root_state.panel == "boss" then return root_state.boss_list or {} end
  return {}
end

local function selected_index(root_state, items)
  for index, item in ipairs(items) do
    if item.uid == root_state.list_select then return index end
  end
  return #items > 0 and 1 or 0
end

local function clamp_scroll(scroll, selected, item_count, visible_height)
  scroll = math.max(0, math.floor(scroll or 0))
  local max_scroll = math.max(0, item_count - visible_height)
  if selected > 0 and selected <= scroll then scroll = math.max(0, selected - 1) end
  if selected > 0 and selected > scroll + visible_height then scroll = selected - visible_height end
  return math.max(0, math.min(max_scroll, scroll))
end

local function draw_scroll_hint(x, y, height, can_up, can_down)
  if height < 4 then return end
  local key_up = C.DEFAULT_TEXT.scroll_key:match("%[([^%]]+)%]") or "W"
  if can_up then
    canvas_draw_text(x, y, "↑", C.HINT_COLOR, nil, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
    canvas_draw_text(x, y + 1, key_up, C.HINT_COLOR, nil, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
  end
  if can_down then
    canvas_draw_text(x, y + height - 2, "S", C.HINT_COLOR, nil, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
    canvas_draw_text(x, y + height - 1, "↓", C.HINT_COLOR, nil, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
  end
end

local function draw_side_status(root_state, x, y, width, height, items, visible_height)
  if root_state.move_mode then
    canvas_draw_text(x, y + visible_height, C.DEFAULT_TEXT.order_key .. " " .. L.language(root_state, "DISPLAY_ORDER", C.DEFAULT_TEXT.order), YELLOW, nil, BOLD, ALIGN_LEFT, {wrap_width = width, wrap_height = 1, text_overflow = "..."})
  elseif root_state.position_mode then
    local input = tostring(root_state.position_input or 0)
    if input == "0" then input = "_" end
    canvas_draw_text(x, y + visible_height, C.DEFAULT_TEXT.position_key .. " " .. input, YELLOW, nil, BOLD, ALIGN_LEFT, {wrap_width = width, wrap_height = 1, text_overflow = "..."})
  end
end

local function draw_side_list(root_state, split_x, y, width, height)
  if (root_state.panel or "none") == "none" then return end

  height = math.max(1, math.floor(height or 1))
  width = math.max(4, math.floor(width or 4))
  local items = list_items(root_state)
  local split_height = math.max(1, y + height)
  canvas_draw_text(math.max(0, split_x), 0, string.rep("║", split_height), C.SPLIT_COLOR, nil, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = split_height})

  local list_x = math.max(0, split_x + 1)
  local selected = selected_index(root_state, items)
  local status_lines = (root_state.move_mode or root_state.position_mode) and 1 or 0
  local visible_height = math.max(1, height - status_lines)
  local needs_scroll = #items > visible_height
  local hint_width = needs_scroll and 2 or 0
  local row_width = math.max(1, width - hint_width)
  local text_width = math.max(1, row_width - 2)
  local scroll = clamp_scroll(root_state.list_scroll, selected, #items, visible_height)

  for row = 1, visible_height do
    local item = items[scroll + row]
    if item then
      local row_y = y + row - 1
      local is_selected = item.uid == root_state.list_select
      local bg = is_selected and "#78a8da" or nil
      local fg = is_selected and BLACK or WHITE
      if is_selected then
        canvas_fill_rect(list_x, row_y, row_width, 1, " ", nil, bg)
      end
      local cursor_x = list_x
      canvas_draw_text(cursor_x, row_y, string.format("%4d", scroll + row), DARK_GRAY, bg, BOLD, ALIGN_LEFT, {wrap_width = 4, wrap_height = 1, text_overflow = ""})
      cursor_x = cursor_x + 5
      local name_limit = math.max(1, text_width - 5)
      if (root_state.settings or {}).mod_badge ~= false and item.is_mod then
        local mod_text = L.language(root_state, "DISPLAY_OPTION_LIST_MOD", C.DEFAULT_TEXT.list_mod)
        local mod_width = L.text_width(mod_text)
        name_limit = math.max(1, name_limit - mod_width - 2)
        canvas_draw_text(list_x + row_width - mod_width - 2, row_y, mod_text, C.MOD_COLOR, bg, BOLD, ALIGN_LEFT, {wrap_width = mod_width, wrap_height = 1, text_overflow = ""})
      end
      canvas_draw_text(cursor_x, row_y, item.name or "", fg, bg, BOLD, ALIGN_LEFT, {wrap_width = name_limit, wrap_height = 1, text_overflow = "..."})
      canvas_draw_text(list_x + row_width - 1, row_y, " ", nil, item.enabled and GREEN or RED, nil, ALIGN_LEFT, {wrap_width = 1, wrap_height = 1})
    end
  end

  if needs_scroll then
    draw_scroll_hint(list_x + row_width, y, visible_height, scroll > 0, scroll + visible_height < #items)
  end
  draw_side_status(root_state, list_x, y, row_width, height, items, visible_height)
end

local function build_hint(root_state, has_panel)
  if has_panel then
    return C.DEFAULT_TEXT.select_key .. " " .. L.language(root_state, "DISPLAY_SELECT", C.DEFAULT_TEXT.select)
      .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "DISPLAY_TOGGLE", C.DEFAULT_TEXT.toggle)
      .. "  " .. C.DEFAULT_TEXT.scroll_key .. " " .. L.language(root_state, "DISPLAY_SCROLL", C.DEFAULT_TEXT.scroll)
      .. "  " .. C.DEFAULT_TEXT.order_key .. " " .. L.language(root_state, "DISPLAY_ORDER", C.DEFAULT_TEXT.order)
      .. "  " .. C.DEFAULT_TEXT.position_key .. " " .. L.language(root_state, "DISPLAY_POSITION", C.DEFAULT_TEXT.position)
      .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. L.language(root_state, "DISPLAY_BACK", C.DEFAULT_TEXT.back)
  end
  return C.DEFAULT_TEXT.select_key .. " " .. L.language(root_state, "DISPLAY_SELECT", C.DEFAULT_TEXT.select)
    .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "DISPLAY_TOGGLE_CONFIRM", C.DEFAULT_TEXT.toggle)
    .. "  " .. C.DEFAULT_TEXT.back_key .. " " .. L.language(root_state, "DISPLAY_BACK", C.DEFAULT_TEXT.back)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  local terminal_width, terminal_height = L.terminal_size()
  local has_panel = (root_state.panel or "none") ~= "none"
  local left_width = has_panel and math.max(30, math.floor(terminal_width * 0.7)) or terminal_width
  local left_x = 0

  local title = L.language(root_state, "DISPLAY_TITLE", C.DEFAULT_TEXT.title)
  canvas_draw_text(center_in_area(left_x, left_width, L.text_width(title)), 1, title, C.TITLE_COLOR, nil, BOLD, ALIGN_LEFT, {wrap_width = left_width, wrap_height = 1, text_overflow = "..."})

  local rows, key_w, _, block_w = block_layout(root_state, has_panel)
  local rows_height = #rows
  local max_block_width = math.max(1, left_width - 2)
  local block_width = math.min(max_block_width, block_w)
  local block_x = center_in_area(left_x, left_width, block_width)
  local block_y = math.max(3, math.floor((terminal_height - rows_height) / 2))
  draw_settings(root_state, block_x, block_y, block_width, rows, key_w, has_panel)

  if has_panel then
    local split_x = left_width
    local right_width = math.max(4, terminal_width - split_x - 1)
    draw_side_list(root_state, split_x, 0, right_width, math.max(1, terminal_height - 1))
  end

  local hint = build_hint(root_state, has_panel)
  canvas_draw_text(center_in_area(0, terminal_width, L.text_width(hint)), math.max(0, terminal_height - 1), hint, C.HINT_COLOR, nil, BOLD, ALIGN_LEFT, {wrap_width = terminal_width, wrap_height = 1, text_overflow = "..."})
end

return M
