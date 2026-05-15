local C = load_function("mod_list/constants.lua")
local L = load_function("mod_list/layout.lua")
local State = load_function("mod_list/state.lua")

local M = {}

local function draw_panel(x, y, width, height, title)
  canvas_border_rect(x, y, width, height, C.BORDER_CHARS, C.BORDER_COLOR, nil)
  if title ~= nil and title ~= "" then
    canvas_draw_text(x + 2, y, " " .. title .. " ", C.TITLE_COLOR, nil, BOLD, nil)
  end
end

local function draw_debug_mark(x, y, bg)
  canvas_draw_text(x, y, "[", C.NORMAL_COLOR, bg, nil, nil, nil)
  canvas_draw_text(x + 1, y, "D", C.DEBUG_COLOR, bg, BOLD, nil)
  canvas_draw_text(x + 2, y, "]", C.NORMAL_COLOR, bg, nil, nil, nil)
  return x + 3
end

local function status_text(root_state, enabled, brief)
  if enabled then
    return brief
      and L.language(root_state, "MOD_LIST_TOGGLE_MOD_ON_BRIEF", C.DEFAULT_TEXT.mod_on_brief)
      or L.language(root_state, "MOD_LIST_TOGGLE_MOD_ON", C.DEFAULT_TEXT.mod_on), C.ON_COLOR
  end
  return brief
    and L.language(root_state, "MOD_LIST_TOGGLE_MOD_OFF_BRIEF", C.DEFAULT_TEXT.mod_off_brief)
    or L.language(root_state, "MOD_LIST_TOGGLE_MOD_OFF", C.DEFAULT_TEXT.mod_off), C.OFF_COLOR
end

local function bool_text(root_state, value, on_key, off_key, on_fallback, off_fallback)
  if value then
    return L.language(root_state, on_key, on_fallback)
  end
  return L.language(root_state, off_key, off_fallback)
end

local function draw_colored_header(layout, root_state)
  local title = L.language(root_state, "MOD_LIST_LIST_TITLE", C.DEFAULT_TEXT.list_title)
  local order_value = tostring(root_state.order or "asc")
  local sort_value = tostring(root_state.sort or "name")
  local order_text = order_value == "desc"
    and L.language(root_state, "MOD_LIST_INFO_ORDER_DESCENDING", C.DEFAULT_TEXT.order_descending)
    or L.language(root_state, "MOD_LIST_INFO_ORDER_ASCENDING", C.DEFAULT_TEXT.order_ascending)
  local sort_text
  if sort_value == "author" then
    sort_text = L.language(root_state, "MOD_LIST_INFO_SORT_AUTHOR", C.DEFAULT_TEXT.sort_author)
  elseif sort_value == "safe_mode" then
    sort_text = L.language(root_state, "MOD_LIST_INFO_SORT_SAFE_MODE", C.DEFAULT_TEXT.sort_safe_mode)
  elseif sort_value == "toggle" then
    sort_text = L.language(root_state, "MOD_LIST_INFO_SORT_TOGGLE", C.DEFAULT_TEXT.sort_toggle)
  else
    sort_text = L.language(root_state, "MOD_LIST_INFO_SORT_NAME", C.DEFAULT_TEXT.sort_name)
  end

  local x = layout.left_x + 2
  local y = layout.left_y
  canvas_draw_text(x, y, " " .. title .. " *", C.TITLE_COLOR, nil, BOLD, nil)
  x = x + L.text_width(" " .. title .. " *")
  canvas_draw_text(x, y, "[", C.TITLE_COLOR, nil, BOLD, nil)
  x = x + 1
  canvas_draw_text(x, y, order_text, C.ORDER_COLOR, nil, BOLD, nil)
  x = x + L.text_width(order_text)
  canvas_draw_text(x, y, "] ", C.TITLE_COLOR, nil, BOLD, nil)
  x = x + 2
  canvas_draw_text(x, y, sort_text, C.SORT_COLOR, nil, BOLD, nil)
end

local function draw_icon(icon, x, y, bg)
  if type(icon) ~= "table" then
    return
  end
  for row = 1, C.ICON_HEIGHT do
    local line = tostring(icon[row] or "")
    if line ~= "" then
      canvas_draw_rich_text(x, y + row - 1, line, C.NORMAL_COLOR, bg, nil, nil, C.ICON_WIDTH)
    end
  end
end

local function draw_safe_bar(x, y, height)
  canvas_fill_rect(x, y, 1, height, " ", nil, C.SAFE_OFF_BG_COLOR)
end

local function draw_full_item(layout, root_state, item, row_y, is_selected)
  local inner_x = layout.left_x + 1
  local inner_width = math.max(1, layout.left_width - 2)
  local bg = is_selected and C.SELECTED_BG_COLOR or nil
  local fg = is_selected and C.SELECTED_FG_COLOR or C.NORMAL_COLOR

  if is_selected then
    canvas_fill_rect(inner_x, row_y, inner_width, 4, " ", nil, C.SELECTED_BG_COLOR)
  end

  draw_icon(item.icon, inner_x, row_y, bg)
  local text_x = inner_x + C.ICON_WIDTH + 1
  local max_width = math.max(1, inner_width - C.ICON_WIDTH - 4)
  local name_x = text_x
  if item.debug == true then
    name_x = draw_debug_mark(text_x, row_y, bg) + 1
  end
  canvas_draw_text(name_x, row_y, tostring(item.package_name or item.name or ""), fg, bg, BOLD, nil, math.max(1, max_width - (name_x - text_x)))
  canvas_draw_text(text_x, row_y + 1, L.language(root_state, "MOD_LIST_INFO_AUTHOR", C.DEFAULT_TEXT.author), C.NORMAL_COLOR, bg, nil, nil, nil)
  canvas_draw_rich_text(text_x + L.text_width(L.language(root_state, "MOD_LIST_INFO_AUTHOR", C.DEFAULT_TEXT.author)), row_y + 1, tostring(item.author or ""), fg, bg, nil, nil, max_width)
  canvas_draw_text(text_x, row_y + 2, L.language(root_state, "MOD_LIST_INFO_VERSION", C.DEFAULT_TEXT.version), C.NORMAL_COLOR, bg, nil, nil, nil)
  canvas_draw_rich_text(text_x + L.text_width(L.language(root_state, "MOD_LIST_INFO_VERSION", C.DEFAULT_TEXT.version)), row_y + 2, tostring(item.version or ""), fg, bg, nil, nil, max_width)
  local status, status_color = status_text(root_state, item.enabled == true, false)
  local prefix = L.language(root_state, "MOD_LIST_STATUS", C.DEFAULT_TEXT.status)
  canvas_draw_text(text_x, row_y + 3, prefix, C.NORMAL_COLOR, bg, nil, nil, nil)
  canvas_draw_text(text_x + L.text_width(prefix), row_y + 3, status, status_color, bg, BOLD, ALIGN_LEFT, max_width)

  if item.safe_mode == false then
    draw_safe_bar(layout.left_x + layout.left_width - 2, row_y, 4)
  end
end

local function draw_brief_item(layout, root_state, item, row_y, is_selected)
  local inner_x = layout.left_x + 1
  local inner_width = math.max(1, layout.left_width - 2)
  local bg = is_selected and C.SELECTED_BG_COLOR or nil
  local fg = is_selected and C.SELECTED_FG_COLOR or C.NORMAL_COLOR

  if is_selected then
    canvas_fill_rect(inner_x, row_y, inner_width, 1, " ", nil, C.SELECTED_BG_COLOR)
  end

  local x = inner_x + 1
  if item.debug == true then
    x = draw_debug_mark(x, row_y, bg) + 1
  end

  local status, status_color = status_text(root_state, item.enabled == true, true)
  local status_text_value = "[" .. status .. "]"
  local status_width = L.text_width(status_text_value)
  local safe_bar_space = 2
  local max_name_width = math.max(1, inner_width - (x - inner_x) - status_width - safe_bar_space - 2)
  canvas_draw_text(x, row_y, tostring(item.package_name or item.name or ""), fg, bg, BOLD, nil, max_name_width)
  local status_x = layout.left_x + layout.left_width - status_width - safe_bar_space - 2
  canvas_draw_text(status_x, row_y, "[", C.NORMAL_COLOR, bg, nil, nil, nil)
  canvas_draw_text(status_x + 1, row_y, status, status_color, bg, BOLD, nil)
  canvas_draw_text(status_x + 1 + L.text_width(status), row_y, "]", C.NORMAL_COLOR, bg, nil, nil, nil)

  if item.safe_mode == false then
    draw_safe_bar(layout.left_x + layout.left_width - 2, row_y, 1)
  end
end

local function draw_mod_list(layout, root_state)
  local selected_uid = tostring(root_state.select or "")
  local page = math.max(1, math.min(root_state.pages or 1, root_state.page or 1))
  local start_index, end_index = State.visible_range(page)
  local list_mode = tostring(root_state.list_mode or "full")
  local y = layout.left_y + 1

  if type(root_state.mod_list) ~= "table" or #root_state.mod_list == 0 then
    local text = L.language(root_state, "MOD_LIST_NONE_MOD", C.DEFAULT_TEXT.none_mod)
    local x = layout.left_x + math.max(1, math.floor((layout.left_width - L.text_width(text)) / 2))
    local empty_y = layout.left_y + math.floor(layout.content_height / 2)
    canvas_draw_text(x, empty_y, text, C.KEY_COLOR, nil, BOLD, nil)
    return
  end

  for index = start_index, end_index do
    local item = root_state.mod_list[index] or {}
    local is_selected = tostring(item.uid or "") == selected_uid
    local row_y = y + (index - start_index) * layout.item_height
    if list_mode == "brief" then
      draw_brief_item(layout, root_state, item, row_y, is_selected)
    else
      draw_full_item(layout, root_state, item, row_y, is_selected)
    end
  end
end

local function draw_page_line(layout, root_state)
  local y = layout.left_y + layout.content_height - 2
  local current_page = tostring(root_state.page or 1)
  if root_state.jump then
    current_page = tostring(root_state.user_page or 0)
    if current_page == "0" then
      current_page = "_"
    end
  end
  local total_pages = tostring(root_state.pages or 1)
  local page_text = current_page .. "/" .. total_pages
  local page_x = layout.left_x + math.floor((layout.left_width - L.text_width(page_text)) / 2)

  if root_state.jump then
    canvas_draw_text(page_x, y, current_page, C.INPUT_FG_COLOR, C.INPUT_BG_COLOR, BOLD, nil)
    canvas_draw_text(page_x + L.text_width(current_page), y, "/" .. total_pages, C.PAGE_COLOR, nil, BOLD, nil)
  else
    canvas_draw_text(page_x, y, page_text, C.PAGE_COLOR, nil, BOLD, nil)
  end

  if (root_state.page or 1) > 1 then
    canvas_draw_text(layout.left_x + 2, y, "◀ " .. C.DEFAULT_TEXT.prev_page_key, C.KEY_COLOR, nil, BOLD, nil)
  end
  if (root_state.page or 1) < (root_state.pages or 1) then
    local right = C.DEFAULT_TEXT.next_page_key .. " ▶"
    canvas_draw_text(layout.left_x + layout.left_width - L.text_width(right) - 2, y, right, C.KEY_COLOR, nil, BOLD, nil)
  end
end

local function chars(text)
  local value = tostring(text or "")
  local ok, result = pcall(function()
    local output = {}
    for _, code in utf8.codes(value) do
      output[#output + 1] = utf8.char(code)
    end
    return output
  end)
  if ok then
    return result
  end

  local output = {}
  for index = 1, #value do
    output[#output + 1] = value:sub(index, index)
  end
  return output
end

local function split_newlines(text)
  local value = tostring(text or "")
  local lines = {}
  value = value:gsub("\\n", "\n")
  for line in (value .. "\n"):gmatch("(.-)\n") do
    lines[#lines + 1] = line
  end
  return lines
end

local function wrapped_lines(text, width)
  local max_width = math.max(1, math.floor(width or 1))
  local output = {}

  for _, raw_line in ipairs(split_newlines(text)) do
    local current = ""
    if raw_line == "" then
      output[#output + 1] = ""
    else
      for _, ch in ipairs(chars(raw_line)) do
        local candidate = current .. ch
        if current ~= "" and get_text_width(candidate) > max_width then
          output[#output + 1] = current
          current = ch
        else
          current = candidate
        end
      end
      output[#output + 1] = current
    end
  end

  return output
end

local function crop_center_text(text, width)
  local max_width = math.max(1, math.floor(width or 1))
  local value = tostring(text or "")
  if L.text_width(value) <= max_width then
    return value
  end

  local total_width = L.text_width(value)
  local skip_width = math.max(0, math.floor((total_width - max_width) / 2))
  local current_width = 0
  local output = ""
  local started = false

  for _, ch in ipairs(chars(value)) do
    local char_width = math.max(1, L.text_width(ch))
    if not started then
      if current_width + char_width <= skip_width then
        current_width = current_width + char_width
      else
        started = true
      end
    end

    if started then
      if L.text_width(output .. ch) > max_width then
        break
      end
      output = output .. ch
    end
  end

  return output
end

local function update_active_rich_prefix(token, active)
  local command_text = token:sub(2, -2)
  for command in command_text:gmatch("[^|]+") do
    if command:match("^tc:clear") then
      active.tc = nil
    elseif command:match("^bg:clear") then
      active.bg = nil
    elseif command:match("^ts:clear") then
      active.ts = nil
    elseif command:match("^tc:") then
      active.tc = "{" .. command .. "}"
    elseif command:match("^bg:") then
      active.bg = "{" .. command .. "}"
    elseif command:match("^ts:") then
      active.ts = "{" .. command .. "}"
    end
  end
end

local function active_rich_prefix(active)
  return (active.tc or "") .. (active.bg or "") .. (active.ts or "")
end

local function clear_active_rich_suffix(active)
  local suffix = ""
  if active.tc ~= nil then
    suffix = suffix .. "{tc:clear}"
  end
  if active.bg ~= nil then
    suffix = suffix .. "{bg:clear}"
  end
  if active.ts ~= nil then
    suffix = suffix .. "{ts:clear}"
  end
  return suffix
end

local function crop_center_rich_text(text, width)
  local max_width = math.max(1, math.floor(width or 1))
  local value = tostring(text or "")
  if get_rich_text_width(value) <= max_width then
    return value
  end

  local total_width = get_rich_text_width(value)
  local skip_width = math.max(0, math.floor((total_width - max_width) / 2))
  local active = {}
  local current_width = 0
  local output_width = 0
  local output = ""
  local started = false
  local index = 1

  while index <= #value do
    local byte = value:sub(index, index)
    if byte == "{" then
      local close_index = value:find("}", index + 1, true)
      if close_index then
        local token = value:sub(index, close_index)
        update_active_rich_prefix(token, active)
        if started then
          output = output .. token
        end
        index = close_index + 1
      else
        local next_index = utf8.offset(value, 2, index) or (#value + 1)
        local character = value:sub(index, next_index - 1)
        local character_width = math.max(1, get_text_width(character))
        if not started then
          if current_width + character_width <= skip_width then
            current_width = current_width + character_width
          else
            started = true
            output = active_rich_prefix(active)
          end
        end
        if started then
          if output_width + character_width > max_width then
            break
          end
          output = output .. character
          output_width = output_width + character_width
        end
        index = next_index
      end
    elseif byte == "\\" then
      local next_byte = value:sub(index + 1, index + 1)
      local escaped = next_byte == "{" or next_byte == "}" or next_byte == "\\"
      local token = escaped and value:sub(index, index + 1) or byte
      local visible = escaped and next_byte or byte
      local character_width = math.max(1, get_text_width(visible))
      if not started then
        if current_width + character_width <= skip_width then
          current_width = current_width + character_width
        else
          started = true
          output = active_rich_prefix(active)
        end
      end
      if started then
        if output_width + character_width > max_width then
          break
        end
        output = output .. token
        output_width = output_width + character_width
      end
      index = index + (escaped and 2 or 1)
    else
      local next_index = utf8.offset(value, 2, index) or (#value + 1)
      local character = value:sub(index, next_index - 1)
      local character_width = math.max(1, get_text_width(character))
      if not started then
        if current_width + character_width <= skip_width then
          current_width = current_width + character_width
        else
          started = true
          output = active_rich_prefix(active)
        end
      end
      if started then
        if output_width + character_width > max_width then
          break
        end
        output = output .. character
        output_width = output_width + character_width
      end
      index = next_index
    end
  end

  if output == "" then
    return ""
  end
  return output .. clear_active_rich_suffix(active)
end

local function add_line(lines, text, color, fg2, text2, rich, rich2)
  lines[#lines + 1] = { text = text or "", color = color or C.INFO_TEXT_COLOR, text2 = text2, color2 = fg2, rich = rich, rich2 = rich2 }
end

local function add_blank(lines)
  add_line(lines, "", C.INFO_TEXT_COLOR)
end

local function add_wrapped(lines, text, width, color)
  if text == nil or tostring(text) == "" then
    return
  end
  for _, line in ipairs(wrapped_lines(text, width)) do
    add_line(lines, line, color or C.INFO_TEXT_COLOR)
  end
end

local function add_rich_line(lines, text, color)
  add_line(lines, tostring(text or ""), color or C.INFO_TEXT_COLOR, nil, nil, true, false)
end

local function add_rich_value(lines, prefix, value)
  add_line(lines, tostring(prefix or ""), C.INFO_TEXT_COLOR, C.INFO_TEXT_COLOR, tostring(value or ""), false, true)
end

local function row_height(row, wrap_width)
  local height = row.rich
    and get_rich_text_height(row.text or "", wrap_width)
    or get_text_height(row.text or "", wrap_width)
  height = math.max(1, height)
  if row.text2 ~= nil then
    local remaining_width = math.max(1, wrap_width - L.text_width(row.text or ""))
    local value_height = row.rich2
      and get_rich_text_height(tostring(row.text2), remaining_width)
      or get_text_height(tostring(row.text2), remaining_width)
    height = math.max(height, value_height)
  end
  return height
end

local function add_status_line(lines, prefix, value, value_color)
  lines[#lines + 1] = { text = prefix, color = C.INFO_TEXT_COLOR, text2 = value, color2 = value_color }
end

local function draw_scroll_indicator(x, top_y, bottom_y, total_rows, visible_rows, scroll, up_key, down_key)
  if total_rows <= visible_rows or visible_rows <= 0 then
    return
  end

  local max_scroll = math.max(1, total_rows - visible_rows)
  if scroll > 0 then
    if top_y <= bottom_y then
      canvas_draw_text(x, top_y, "↑", C.KEY_COLOR, nil, BOLD, nil)
    end
    if up_key ~= nil and tostring(up_key) ~= "" and top_y + 1 <= bottom_y then
      canvas_draw_text(x, top_y + 1, tostring(up_key), C.KEY_COLOR, nil, BOLD, nil)
    end
  end
  if scroll < max_scroll then
    if down_key ~= nil and tostring(down_key) ~= "" and bottom_y - 1 >= top_y then
      canvas_draw_text(x, bottom_y - 1, tostring(down_key), C.KEY_COLOR, nil, BOLD, nil)
    end
    if bottom_y >= top_y then
      canvas_draw_text(x, bottom_y, "↓", C.KEY_COLOR, nil, BOLD, nil)
    end
  end

  local track_top = top_y + 3
  local track_bottom = bottom_y - 3
  if track_top > track_bottom then
    return
  end
  local track_height = math.max(1, track_bottom - track_top + 1)
  local thumb_height = math.max(1, math.floor(track_height * visible_rows / total_rows))
  local thumb_top = track_top + math.floor((track_height - thumb_height) * math.min(scroll, max_scroll) / max_scroll)
  for y = thumb_top, math.min(track_bottom, thumb_top + thumb_height - 1) do
    canvas_draw_text(x, y, "█", C.KEY_COLOR, nil, BOLD, nil)
  end
end

local function banner_lines(info, width)
  local lines = {}
  if type(info.banner) ~= "table" then
    return lines
  end
  for _, raw_line in ipairs(info.banner) do
    local line = tostring(raw_line or "")
    local line_width = get_rich_text_width(line)
    if line_width <= width then
      local pad = math.max(0, math.floor((width - line_width) / 2))
      lines[#lines + 1] = { text = string.rep(" ", pad) .. line, color = C.INFO_TEXT_COLOR, rich = true }
    else
      line = crop_center_rich_text(line, width)
      lines[#lines + 1] = { text = line, color = C.INFO_TEXT_COLOR, rich = true }
    end
  end
  local pad_to = 13
  local add_to_top = true
  while #lines > 0 and #lines < pad_to do
    local blank = { text = "", color = C.INFO_TEXT_COLOR, rich = false }
    if add_to_top then
      table.insert(lines, 1, blank)
    else
      lines[#lines + 1] = blank
    end
    add_to_top = not add_to_top
  end
  return lines
end

local function info_lines(root_state, width)
  local info = root_state.mod_info or {}
  local lines = banner_lines(info, width)
  if #lines > 0 then
    add_blank(lines)
  end

  add_line(lines, L.language(root_state, "MOD_LIST_INFO_BASE", C.DEFAULT_TEXT.base), C.INFO_LABEL_COLOR)
  add_line(lines, tostring(info.package_name or info.name or ""), C.INFO_TEXT_COLOR)
  add_rich_value(lines, L.language(root_state, "MOD_LIST_INFO_AUTHOR", C.DEFAULT_TEXT.author), info.author)
  add_rich_value(lines, L.language(root_state, "MOD_LIST_INFO_VERSION", C.DEFAULT_TEXT.version), info.version)
  add_blank(lines)

  add_line(lines, L.language(root_state, "MOD_LIST_INFO_SAFE", C.DEFAULT_TEXT.safe), C.INFO_LABEL_COLOR)
  local enabled, enabled_color = status_text(root_state, info.enabled == true, false)
  add_status_line(lines, L.language(root_state, "MOD_LIST_INFO_SAFE_SWITCH", C.DEFAULT_TEXT.safe_switch), enabled, enabled_color)
  local debug_prefix = L.language(root_state, "MOD_LIST_INFO_SAFE_DEBUG", C.DEFAULT_TEXT.safe_debug)
  local debug_text = bool_text(root_state, info.debug == true, "MOD_LIST_TOGGLE_DEBUG_ON", "MOD_LIST_TOGGLE_DEBUG_OFF", C.DEFAULT_TEXT.debug_on, C.DEFAULT_TEXT.debug_off)
  add_status_line(lines, debug_prefix, debug_text, info.debug == true and C.DANGER_COLOR or C.DISABLED_COLOR)
  local write_text = bool_text(root_state, info.write == true, "MOD_LIST_TOGGLE_WRITE_ON", "MOD_LIST_TOGGLE_WRITE_OFF", C.DEFAULT_TEXT.write_on, C.DEFAULT_TEXT.write_off)
  add_status_line(lines, L.language(root_state, "MOD_LIST_INFO_SAFE_WRITE", C.DEFAULT_TEXT.safe_write), write_text, info.write == true and C.DANGER_COLOR or C.DISABLED_COLOR)
  local safe_text
  if info.safe_mode == true then
    safe_text = L.language(root_state, "MOD_LIST_TOGGLE_SAFE_MODE_ON", C.DEFAULT_TEXT.safe_mode_on)
  elseif info.safe_mode_permanent == true then
    safe_text = L.language(root_state, "MOD_LIST_TOGGLE_SAFE_MODE_OFF_PERMANENT", C.DEFAULT_TEXT.safe_mode_off_permanent)
  else
    safe_text = L.language(root_state, "MOD_LIST_TOGGLE_SAFE_MODE_OFF_TEMPORARY", C.DEFAULT_TEXT.safe_mode_off_temporary)
  end
  add_status_line(lines, L.language(root_state, "MOD_LIST_INFO_SAFE_SAFE_MODE", C.DEFAULT_TEXT.safe_safe_mode), safe_text, info.safe_mode == true and C.ON_COLOR or C.OFF_COLOR)
  add_blank(lines)

  add_line(lines, L.language(root_state, "MOD_LIST_INFO_INTRODUCTION", C.DEFAULT_TEXT.introduction), C.INFO_LABEL_COLOR)
  add_rich_line(lines, tostring(info.introduction or ""), C.INFO_TEXT_COLOR)

  return lines
end

local function has_mod_info(root_state)
  local info = root_state.mod_info or {}
  if type(root_state.mod_list) == "table" and #root_state.mod_list == 0 then
    return false
  end
  return (info.uid ~= nil and tostring(info.uid) ~= "")
    or (info.package_name ~= nil and tostring(info.package_name) ~= "")
    or (info.name ~= nil and tostring(info.name) ~= "")
end

local function info_scroll_metrics(root_state)
  State.set_root_state(root_state or {})
  local layout = State.layout()
  local rows = info_lines(root_state or {}, layout.info_width)
  local total_height = 0
  for _, row in ipairs(rows) do
    total_height = total_height + row_height(row, layout.info_width)
  end
  local needs_scroll = total_height > layout.info_height
  local content_width = needs_scroll and math.max(1, layout.info_width - 2) or layout.info_width
  if needs_scroll then
    rows = info_lines(root_state or {}, content_width)
    total_height = 0
    for _, row in ipairs(rows) do
      total_height = total_height + row_height(row, content_width)
    end
  end
  return rows, layout.info_height, math.max(0, total_height - layout.info_height)
end

local function draw_info(layout, root_state)
  local title = L.language(root_state, "MOD_LIST_INFO_TITLE", C.DEFAULT_TEXT.info_title)
  draw_panel(layout.right_x, layout.right_y, layout.right_width, layout.right_height, title)

  if not has_mod_info(root_state) then
    local text = L.language(root_state, "MOD_LIST_NONE_INFO", C.DEFAULT_TEXT.none_info)
    local x = layout.right_x + math.max(1, math.floor((layout.right_width - L.text_width(text)) / 2))
    local y = layout.right_y + math.floor(layout.right_height / 2)
    canvas_draw_text(x, y, text, C.KEY_COLOR, nil, BOLD, nil)
    return
  end

  local content_x = layout.right_x + 1
  local y = layout.right_y + 1
  local rows, max_rows, max_scroll = info_scroll_metrics(root_state)
  local content_width = max_scroll > 0 and math.max(1, layout.info_width - 2) or layout.info_width
  local scroll = math.max(0, math.floor(root_state.info_scroll or 0))
  scroll = math.min(scroll, max_scroll)

  local drawn = 0
  local skipped_height = 0
  for index = 1, #rows do
    local row = rows[index]
    local row_draw_height = row_height(row, content_width)
    if skipped_height + row_draw_height <= scroll then
      skipped_height = skipped_height + row_draw_height
    elseif drawn >= max_rows then
      break
    else
    if drawn + row_draw_height > max_rows then
      break
    end
    if row.rich then
      canvas_draw_rich_text(content_x, y + drawn, row.text or "", row.color, nil, ALIGN_LEFT, content_width)
    else
      canvas_draw_text(content_x, y + drawn, row.text or "", row.color, nil, nil, ALIGN_LEFT, content_width)
    end
    if row.text2 ~= nil then
      local x2 = content_x + L.text_width(row.text or "")
      local remaining_width = math.max(1, content_width - L.text_width(row.text or ""))
      if row.rich2 then
        canvas_draw_rich_text(x2, y + drawn, tostring(row.text2), row.color2 or C.INFO_TEXT_COLOR, nil, ALIGN_LEFT, remaining_width)
      else
        canvas_draw_text(x2, y + drawn, tostring(row.text2), row.color2 or C.INFO_TEXT_COLOR, nil, BOLD, ALIGN_LEFT, remaining_width)
      end
    end
    drawn = drawn + row_draw_height
    end
  end

  draw_scroll_indicator(
    layout.right_x + layout.right_width - 2,
    layout.right_y + 1,
    layout.right_y + layout.right_height - 2,
    max_rows + max_scroll,
    max_rows,
    scroll,
    C.DEFAULT_TEXT.scroll_up_key_text,
    C.DEFAULT_TEXT.scroll_down_key_text
  )
end

function M.max_info_scroll(root_state)
  local _, _, max_scroll = info_scroll_metrics(root_state or {})
  return max_scroll
end

local function wrap_segments(segments, separator, max_width)
  local lines = {}
  local current = nil
  for _, seg in ipairs(segments) do
    if current == nil then
      current = seg
    else
      local candidate = current .. separator .. seg
      if L.text_width(candidate) <= max_width then
        current = candidate
      else
        table.insert(lines, current)
        current = seg
      end
    end
  end
  if current ~= nil then
    table.insert(lines, current)
  end
  if #lines == 0 then
    table.insert(lines, "")
  end
  return lines
end

local function action_segments(root_state)
  local segments = {}
  if root_state.jump then
    table.insert(segments, "[1]-[9] " .. L.language(root_state, "MOD_LIST_SELECT", C.DEFAULT_TEXT.select))
    table.insert(segments, C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "MOD_LIST_CONFIRM", C.DEFAULT_TEXT.confirm))
    table.insert(segments, C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "MOD_LIST_CANCEL", C.DEFAULT_TEXT.cancel))
  else
    table.insert(segments, C.DEFAULT_TEXT.prev_option_key .. "/" .. C.DEFAULT_TEXT.next_option_key .. " "
      .. L.language(root_state, "MOD_LIST_SELECT", C.DEFAULT_TEXT.select))
    table.insert(segments, C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "MOD_LIST_TOGGLE_CONFIRM", C.DEFAULT_TEXT.toggle_confirm))
    table.insert(segments, C.DEFAULT_TEXT.debug_key .. " " .. L.language(root_state, "MOD_LIST_DEBUG", C.DEFAULT_TEXT.debug))
    table.insert(segments, C.DEFAULT_TEXT.safe_mode_key .. " " .. L.language(root_state, "MOD_LIST_SAFE_MODE", C.DEFAULT_TEXT.safe_mode))
    table.insert(segments, C.DEFAULT_TEXT.list_key .. " " .. L.language(root_state, "MOD_LIST_LIST", C.DEFAULT_TEXT.list))
    table.insert(segments, C.DEFAULT_TEXT.scroll_up_key .. "/" .. C.DEFAULT_TEXT.scroll_down_key .. " "
      .. L.language(root_state, "MOD_LIST_SCROLL", C.DEFAULT_TEXT.scroll))
    table.insert(segments, C.DEFAULT_TEXT.order_key .. " " .. L.language(root_state, "MOD_LIST_ORDER", C.DEFAULT_TEXT.order))
    table.insert(segments, C.DEFAULT_TEXT.sort_key .. " " .. L.language(root_state, "MOD_LIST_SORT", C.DEFAULT_TEXT.sort))
    if root_state.pages and root_state.pages > 1 then
      table.insert(segments, C.DEFAULT_TEXT.jump_key .. " " .. L.language(root_state, "MOD_LIST_JUMP", C.DEFAULT_TEXT.jump))
      table.insert(segments, C.DEFAULT_TEXT.prev_page_key .. "/" .. C.DEFAULT_TEXT.next_page_key .. " "
        .. L.language(root_state, "MOD_LIST_FLIP", C.DEFAULT_TEXT.flip))
    end
    table.insert(segments, C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "MOD_LIST_BACK", C.DEFAULT_TEXT.back))
  end
  return segments
end

local function draw_action_line(layout, root_state)
  local segments = action_segments(root_state)
  local wrap_width = math.max(1, layout.terminal_width - 2)
  local lines = wrap_segments(segments, "  ", wrap_width)
  local base_y = math.max(0, layout.terminal_height - #lines)
  for i, line in ipairs(lines) do
    local x = math.max(0, math.floor((layout.terminal_width - math.min(L.text_width(line), wrap_width)) / 2))
    canvas_draw_text(x, base_y + i - 1, line, C.KEY_COLOR, nil, nil, ALIGN_LEFT, wrap_width)
  end
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  State.set_root_state(root_state)
  root_state.pages = State.pages()
  root_state.page = math.max(1, math.min(root_state.pages, root_state.page or 1))
  local hint_lines = #wrap_segments(action_segments(root_state), "  ", math.max(1, (L.terminal_size()) - 2))
  local layout = L.layout(root_state.list_mode or "full", hint_lines)

  draw_panel(layout.left_x, layout.left_y, layout.left_width, layout.content_height, "")
  draw_colored_header(layout, root_state)
  draw_mod_list(layout, root_state)
  draw_page_line(layout, root_state)
  draw_info(layout, root_state)
  draw_action_line(layout, root_state)
end

return M
