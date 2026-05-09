local C = load_function("game_list/constants.lua")
local L = load_function("game_list/layout.lua")
local State = load_function("game_list/state.lua")

local M = {}

local function draw_panel(x, y, width, height, title)
  canvas_border_rect(x, y, width, height, C.BORDER_CHARS, C.BORDER_COLOR, nil)
  if title ~= nil and title ~= "" then
    canvas_draw_text(x + 2, y, " " .. title .. " ", C.TITLE_COLOR, nil, BOLD, nil)
  end
end

local function draw_colored_header(layout, root_state)
  local title = L.language(root_state, "GAME_LIST_LIST_TITLE", C.DEFAULT_TEXT.list_title)
  local order_value = tostring(root_state.order or "asc")
  local sort_value = tostring(root_state.sort or "official_mod")
  local order_text = order_value == "desc"
    and L.language(root_state, "GAME_LIST_INFO_ORDER_DESCENDING", C.DEFAULT_TEXT.order_descending)
    or L.language(root_state, "GAME_LIST_INFO_ORDER_ASCENDING", C.DEFAULT_TEXT.order_ascending)
  local sort_text
  if sort_value == "name" then
    sort_text = L.language(root_state, "GAME_LIST_INFO_SORT_NAME", C.DEFAULT_TEXT.sort_name)
  elseif sort_value == "author" then
    sort_text = L.language(root_state, "GAME_LIST_INFO_SORT_AUTHOR", C.DEFAULT_TEXT.sort_author)
  else
    sort_text = L.language(root_state, "GAME_LIST_INFO_SORT_MOD_OFFICIAL", C.DEFAULT_TEXT.sort_mod_official)
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

local function draw_game_list(layout, root_state)
  local selected_uid = tostring(root_state.select or "")
  local page = math.max(1, math.min(root_state.pages or 1, root_state.page or 1))
  local start_index, end_index = State.visible_range(page)
  local y = layout.left_y + 1
  local label_mod = L.language(root_state, "GAME_LIST_SOURCE_MOD", C.DEFAULT_TEXT.source_mod)
  local inner_x = layout.left_x + 1
  local inner_width = math.max(1, layout.left_width - 2)

  if type(root_state.game_list) ~= "table" or #root_state.game_list == 0 then
    local text = L.language(root_state, "GAME_LIST_NONE_GAME", C.DEFAULT_TEXT.none_game)
    local x = layout.left_x + math.max(1, math.floor((layout.left_width - L.text_width(text)) / 2))
    local empty_y = layout.left_y + math.floor(layout.content_height / 2)
    canvas_draw_text(x, empty_y, text, C.KEY_COLOR, nil, BOLD, nil)
    return
  end

  for index = start_index, end_index do
    local game = root_state.game_list[index] or {}
    local is_selected = tostring(game.uid or "") == selected_uid
    local row_y = y + (index - start_index)
    local fg = is_selected and C.SELECTED_FG_COLOR or C.NORMAL_COLOR
    local mark_color = is_selected and C.SELECTED_FG_COLOR or C.MARK_COLOR
    if is_selected then
      canvas_fill_rect(inner_x, row_y, inner_width, 1, " ", nil, C.SELECTED_BG_COLOR)
    end

    local name = tostring(game.name or game.game_name or "")
    local mark = tostring(game.source or "") == "mod" and (" " .. label_mod) or ""
    local max_name_width = inner_width - L.text_width(mark) - 1
    if max_name_width < 1 then
      max_name_width = inner_width
    end
    local bg = is_selected and C.SELECTED_BG_COLOR or nil
    canvas_draw_text(inner_x + 1, row_y, name, fg, bg, BOLD, nil, max_name_width)
    if mark ~= "" then
      local mark_x = layout.left_x + layout.left_width - L.text_width(mark) - 2
      canvas_draw_text(mark_x, row_y, mark, mark_color, bg, BOLD, nil)
    end
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

local function add_separator(lines, width)
  lines[#lines + 1] = { text = string.rep("─", math.max(1, width)), color = C.BORDER_COLOR }
end

local function add_rich(lines, text)
  lines[#lines + 1] = { text = tostring(text or ""), rich = true, color = C.INFO_TEXT_COLOR }
end

local function add_rich_value(lines, prefix, value)
  lines[#lines + 1] = {
    text = tostring(prefix or ""),
    color = C.INFO_TEXT_COLOR,
    text2 = tostring(value or ""),
    rich2 = true,
    color2 = C.INFO_TEXT_COLOR
  }
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

local function rich_wrapped_lines(text, wrap_width, color)
  local width = math.max(1, math.floor(wrap_width or 1))
  local value = tostring(text or "")
  local lines = {}
  local active = {}
  local current = ""
  local current_width = 0
  local index = 1

  local function push_line()
    lines[#lines + 1] = { text = current .. clear_active_rich_suffix(active), rich = true, color = color or C.INFO_TEXT_COLOR }
    current = active_rich_prefix(active)
    current_width = 0
  end

  while index <= #value do
    local byte = value:sub(index, index)
    if byte == "{" then
      local close_index = value:find("}", index + 1, true)
      if close_index then
        local token = value:sub(index, close_index)
        update_active_rich_prefix(token, active)
        current = current .. token
        index = close_index + 1
      else
        local next_index = utf8.offset(value, 2, index) or (#value + 1)
        local character = value:sub(index, next_index - 1)
        local character_width = math.max(1, get_text_width(character))
        if current_width > 0 and current_width + character_width > width then
          push_line()
        end
        current = current .. character
        current_width = current_width + character_width
        index = next_index
      end
    elseif byte == "\\" and value:sub(index + 1, index + 1) == "n" then
      push_line()
      index = index + 2
    else
      local next_index = utf8.offset(value, 2, index) or (#value + 1)
      local character = value:sub(index, next_index - 1)
      if character == "\n" then
        push_line()
      else
        local character_width = math.max(1, get_text_width(character))
        if current_width > 0 and current_width + character_width > width then
          push_line()
        end
        current = current .. character
        current_width = current_width + character_width
      end
      index = next_index
    end
  end

  if current ~= "" or #lines == 0 then
    lines[#lines + 1] = { text = current .. clear_active_rich_suffix(active), rich = true, color = color or C.INFO_TEXT_COLOR }
  end
  return lines
end

local function wrapped_row_lines(row, wrap_width)
  local width = math.max(1, math.floor(wrap_width or 1))
  if row.rich then
    return rich_wrapped_lines(row.text or "", width, row.color)
  end
  local lines = {}
  for _, line in ipairs(wrapped_lines(row.text or "", width)) do
    lines[#lines + 1] = { text = line, color = row.color }
  end
  if #lines == 0 then
    lines[#lines + 1] = { text = "", color = row.color }
  end
  return lines
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

local function fixed_info_lines(root_state, wrap_width)
  local info = root_state.game_info or {}
  local lines = {}
  local function add(text, color)
    lines[#lines + 1] = { text = text, color = color or C.INFO_TEXT_COLOR }
  end
  local function add_text_block(text)
    if text == nil or tostring(text) == "" then
      return
    end
    for _, line in ipairs(wrapped_lines(tostring(text), wrap_width)) do
      add(line, C.INFO_TEXT_COLOR)
    end
  end

  add(tostring(info.game_name or info.name or ""), C.TITLE_COLOR)
  add_separator(lines, wrap_width)
  add_rich_value(lines, L.language(root_state, "GAME_LIST_INFO_MOD", C.DEFAULT_TEXT.info_mod), info.mod_name)
  add_rich_value(lines, L.language(root_state, "GAME_LIST_INFO_AUTHOR", C.DEFAULT_TEXT.info_author), info.author)
  add_rich_value(lines, L.language(root_state, "GAME_LIST_INFO_VERSION", C.DEFAULT_TEXT.info_version), info.version)
  if info.best_score ~= nil and tostring(info.best_score) ~= "" then
    add_separator(lines, wrap_width)
    add_rich(lines, info.best_score)
  end
  if info.description ~= nil and tostring(info.description) ~= "" then
    add_separator(lines, wrap_width)
    add_rich(lines, info.description)
  end
  if info.detail ~= nil and tostring(info.detail) ~= "" then
    add_separator(lines, wrap_width)
  end

  return lines
end

local function detail_info_lines(root_state, wrap_width)
  local info = root_state.game_info or {}
  if info.detail == nil or tostring(info.detail) == "" then
    return {}
  end

  local output = {}
  output[#output + 1] = { text = tostring(info.detail), rich = true, color = C.INFO_TEXT_COLOR }
  return output
end

local function has_game_info(root_state)
  local info = root_state.game_info or {}
  if type(root_state.game_list) == "table" and #root_state.game_list == 0 then
    return false
  end
  return (info.uid ~= nil and tostring(info.uid) ~= "")
    or (info.game_name ~= nil and tostring(info.game_name) ~= "")
    or (info.name ~= nil and tostring(info.name) ~= "")
end

local function detail_scroll_metrics(root_state)
  State.set_root_state(root_state or {})
  local layout = State.layout()
  local fixed_rows = fixed_info_lines(root_state or {}, layout.info_width)
  local detail_rows = detail_info_lines(root_state or {}, layout.info_width)
  local content_width = math.max(1, layout.right_width - 2)
  local max_rows = layout.info_height
  local row_index = 0

  for index = 1, #fixed_rows do
    if row_index >= max_rows then
      break
    end
    local row = fixed_rows[index]
    local height = row_height(row, content_width)
    if row_index + height > max_rows then
      break
    end
    row_index = row_index + height
  end

  local detail_capacity = math.max(0, max_rows - row_index)
  local detail_line_count = 0
  for _, row in ipairs(detail_rows) do
    detail_line_count = detail_line_count + #wrapped_row_lines(row, layout.info_width)
  end
  if detail_line_count > detail_capacity then
    local adjusted_width = math.max(1, layout.info_width - 2)
    detail_line_count = 0
    for _, row in ipairs(detail_rows) do
      detail_line_count = detail_line_count + #wrapped_row_lines(row, adjusted_width)
    end
  end
  return detail_rows, detail_capacity, math.max(0, detail_line_count - detail_capacity)
end

local function draw_info(layout, root_state)
  local title = L.language(root_state, "GAME_LIST_INFO_TITLE", C.DEFAULT_TEXT.info_title)
  draw_panel(layout.right_x, layout.right_y, layout.right_width, layout.right_height, title)

  if not has_game_info(root_state) then
    local text = L.language(root_state, "GAME_LIST_NONE_INFO", C.DEFAULT_TEXT.none_info)
    local x = layout.right_x + math.max(1, math.floor((layout.right_width - L.text_width(text)) / 2))
    local y = layout.right_y + math.floor(layout.right_height / 2)
    canvas_draw_text(x, y, text, C.KEY_COLOR, nil, BOLD, nil)
    return
  end

  local scroll = math.max(0, math.floor(root_state.info_scroll or 0))
  local fixed_rows = fixed_info_lines(root_state, layout.info_width)
  local detail_rows, _, max_scroll = detail_scroll_metrics(root_state)
  local content_x = layout.right_x + 1
  local content_width = math.max(1, layout.right_width - 2)
  local y = layout.right_y + 1
  local max_rows = layout.info_height
  local row_index = 0

  for index = 1, #fixed_rows do
    if row_index >= max_rows then
      break
    end
    local row = fixed_rows[index]
    local height = row_height(row, content_width)
    if row_index + height > max_rows then
      break
    end
    if row.rich then
      canvas_draw_rich_text(content_x, y + row_index, row.text or "", row.color, nil, ALIGN_LEFT, content_width)
    else
      canvas_draw_text(content_x, y + row_index, row.text or "", row.color, nil, nil, ALIGN_LEFT, content_width)
    end
    if row.text2 ~= nil then
      local x2 = content_x + L.text_width(row.text or "")
      local remaining_width = math.max(1, content_width - L.text_width(row.text or ""))
      if row.rich2 then
        canvas_draw_rich_text(x2, y + row_index, tostring(row.text2), row.color2 or C.INFO_TEXT_COLOR, nil, ALIGN_LEFT, remaining_width)
      else
        canvas_draw_text(x2, y + row_index, tostring(row.text2), row.color2 or C.INFO_TEXT_COLOR, nil, BOLD, nil, remaining_width)
      end
    end
    row_index = row_index + height
  end

  local detail_capacity = math.max(0, max_rows - row_index)
  local needs_scroll = max_scroll > 0
  if needs_scroll then
    content_width = math.max(1, content_width - 2)
    local adjusted_lines = 0
    for _, row in ipairs(detail_rows) do
      adjusted_lines = adjusted_lines + #wrapped_row_lines(row, content_width)
    end
    max_scroll = math.max(0, adjusted_lines - detail_capacity)
  end
  if detail_capacity > 0 then
    scroll = math.min(scroll, max_scroll)
  else
    scroll = 0
  end

  if detail_capacity > 0 then
    local detail_lines = {}
    for _, row in ipairs(detail_rows) do
      for _, line in ipairs(wrapped_row_lines(row, content_width)) do
        detail_lines[#detail_lines + 1] = line
      end
    end
    for index = 1 + scroll, #detail_lines do
      local draw_index = index - scroll - 1
      if draw_index >= detail_capacity then
        break
      end
      local line = detail_lines[index]
      if line.rich then
        canvas_draw_rich_text(content_x, y + row_index + draw_index, line.text or "", line.color, nil, nil, nil)
      else
        canvas_draw_text(content_x, y + row_index + draw_index, line.text or "", line.color, nil, nil, nil, nil)
      end
    end
  end

  draw_scroll_indicator(
    layout.right_x + layout.right_width - 2,
    y + row_index,
    y + row_index + detail_capacity - 1,
    detail_capacity + max_scroll,
    detail_capacity,
    scroll,
    C.DEFAULT_TEXT.scroll_up_key_text,
    C.DEFAULT_TEXT.scroll_down_key_text
  )
end

function M.max_info_scroll(root_state)
  local _, _, max_scroll = detail_scroll_metrics(root_state or {})
  return max_scroll
end

local function draw_action_line(layout, root_state)
  local y = layout.terminal_height - 1
  local action
  if root_state.jump then
    action = "[1]-[9] " .. L.language(root_state, "GAME_LIST_SELECT", C.DEFAULT_TEXT.select)
      .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "GAME_LIST_CONFIRM", C.DEFAULT_TEXT.confirm)
      .. "  " .. C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "GAME_LIST_CANCEL", C.DEFAULT_TEXT.cancel)
  else
    action = C.DEFAULT_TEXT.prev_option_key .. "/" .. C.DEFAULT_TEXT.next_option_key .. " "
      .. L.language(root_state, "GAME_LIST_SELECT", C.DEFAULT_TEXT.select)
      .. "  " .. C.DEFAULT_TEXT.confirm_key .. " " .. L.language(root_state, "GAME_LIST_START", C.DEFAULT_TEXT.start)
      .. "  " .. C.DEFAULT_TEXT.scroll_up_key .. "/" .. C.DEFAULT_TEXT.scroll_down_key .. " "
      .. L.language(root_state, "GAME_LIST_SCROLL", C.DEFAULT_TEXT.scroll)
      .. "  " .. C.DEFAULT_TEXT.order_key .. " " .. L.language(root_state, "GAME_LIST_ORDER", C.DEFAULT_TEXT.order)
      .. "  " .. C.DEFAULT_TEXT.sort_key .. " " .. L.language(root_state, "GAME_LIST_SORT", C.DEFAULT_TEXT.sort)
    if root_state.pages and root_state.pages > 1 then
      action = action
        .. "  " .. C.DEFAULT_TEXT.jump_key .. " " .. L.language(root_state, "GAME_LIST_JUMP", C.DEFAULT_TEXT.jump)
        .. "  " .. C.DEFAULT_TEXT.prev_page_key .. "/" .. C.DEFAULT_TEXT.next_page_key .. " "
        .. L.language(root_state, "GAME_LIST_FLIP", C.DEFAULT_TEXT.flip)
    end
    action = action .. "  " .. C.DEFAULT_TEXT.return_key .. " " .. L.language(root_state, "GAME_LIST_BACK", C.DEFAULT_TEXT.back)
  end
  local wrap_width = math.max(1, layout.terminal_width - 2)
  local action_width = math.min(get_text_width(action), wrap_width)
  local x = math.max(0, math.floor((layout.terminal_width - action_width) / 2))
  canvas_draw_text(x, y, action, C.KEY_COLOR, nil, nil, ALIGN_LEFT, wrap_width)
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  State.set_root_state(root_state)
  local layout = State.layout()
  root_state.pages = State.pages()
  root_state.page = math.max(1, math.min(root_state.pages, root_state.page or 1))

  draw_panel(layout.left_x, layout.left_y, layout.left_width, layout.content_height, "")
  draw_colored_header(layout, root_state)
  draw_game_list(layout, root_state)
  draw_page_line(layout, root_state)
  draw_info(layout, root_state)
  draw_action_line(layout, root_state)
end

return M
