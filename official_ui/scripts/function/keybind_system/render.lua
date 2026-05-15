local C = load_function("keybind_system/constants.lua")
local L = load_function("keybind_system/layout.lua")
local State = load_function("keybind_system/state.lua")

local M = {}

local function language(root_state, key, fallback)
  return L.language(root_state, key, fallback)
end

local function draw_panel(x, y, width, height, title, extra_title)
  canvas_border_rect(x, y, width, height, C.BORDER_CHARS, C.BORDER_COLOR, nil)
  local header = " " .. tostring(title or "") .. " "
  canvas_draw_text(math.max(0, x + 2), math.max(0, y), header, C.TITLE_COLOR, nil, BOLD, nil)
  if extra_title ~= nil and tostring(extra_title) ~= "" then
    local header_x = x + 2 + L.text_width(header)
    canvas_draw_text(math.max(0, header_x), math.max(0, y), tostring(extra_title), C.SORT_COLOR, nil, BOLD, nil)
  end
end

local function draw_list_title(layout, root_state)
  local title = language(root_state, "SETTING_KEYBIND_SYSTEM_LIST_TITLE", C.DEFAULT_TEXT.list_title)
  local order_text = tostring(root_state.order or "asc") == "desc"
    and language(root_state, "SETTING_KEYBIND_SYSTEM_ORDER_DESCENDING", C.DEFAULT_TEXT.order_descending)
    or language(root_state, "SETTING_KEYBIND_SYSTEM_ORDER_ASCENDING", C.DEFAULT_TEXT.order_ascending)
  local sort_text = tostring(root_state.sort or "name") == "conflict"
    and language(root_state, "SETTING_KEYBIND_SYSTEM_SORT_CONFLICT", C.DEFAULT_TEXT.sort_conflict)
    or language(root_state, "SETTING_KEYBIND_SYSTEM_SORT_NAME", C.DEFAULT_TEXT.sort_name)

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

local function draw_page_list(layout, root_state)
  local list = root_state.page_list or {}
  local selected = tostring(root_state.select or "")
  local first, last = State.visible_page_range(root_state.page or 1)
  local y = layout.left_y + 1
  local inner_x = layout.left_x + 1
  local inner_width = math.max(1, layout.left_width - 2)

  for index = first, last do
    local item = list[index] or {}
    local is_selected = tostring(item.id or "") == selected
    local row_y = y + index - first
    local fg = is_selected and C.SELECTED_FG_COLOR or C.NORMAL_COLOR
    local bg = is_selected and C.SELECTED_BG_COLOR or nil
    if is_selected then
      canvas_fill_rect(inner_x, row_y, inner_width, 1, " ", nil, C.SELECTED_BG_COLOR)
    end
    canvas_draw_text(inner_x, row_y, tostring(item.name or ""), fg, bg, BOLD, nil, inner_width)
    if item.has_empty or item.has_conflict then
      canvas_fill_rect(layout.left_x + layout.left_width - 2, row_y, 1, 1, " ", nil, C.EMPTY_BG_COLOR)
    end
  end
end

local function draw_page_line(layout, root_state)
  local y = layout.left_y + layout.content_height - 2
  local current_page = tostring(root_state.page or 1)
  if root_state.jump then
    current_page = tostring(root_state.user_page or 0)
    if current_page == "0" then current_page = "_" end
  end
  local total_pages = tostring(root_state.pages or 1)
  local page_text = current_page .. "/" .. total_pages
  local page_x = layout.left_x + math.floor((layout.left_width - L.text_width(page_text)) / 2)

  if root_state.jump then
    canvas_draw_text(math.max(0, page_x), y, current_page, C.INPUT_FG_COLOR, C.INPUT_BG_COLOR, BOLD, nil)
    canvas_draw_text(math.max(0, page_x + L.text_width(current_page)), y, "/" .. total_pages, C.KEY_COLOR, nil, BOLD, nil)
  else
    canvas_draw_text(math.max(0, page_x), y, page_text, C.KEY_COLOR, nil, BOLD, nil)
  end

  if (root_state.page or 1) > 1 then
    canvas_draw_text(layout.left_x + 2, y, "◀ " .. C.DEFAULT_TEXT.prev_page_key, C.KEY_COLOR, nil, BOLD, nil)
  end
  if (root_state.page or 1) < (root_state.pages or 1) then
    local right = C.DEFAULT_TEXT.next_page_key .. " ▶"
    canvas_draw_text(layout.left_x + layout.left_width - L.text_width(right) - 2, y, right, C.KEY_COLOR, nil, BOLD, nil)
  end
end

local function key_array(value)
  if type(value) == "table" then
    return value
  end
  if value == nil or tostring(value) == "" then
    return {}
  end
  return { tostring(value) }
end

local function key_label(value)
  local text = tostring(value or "")
  if text == "" then return "[]" end
  return "[" .. text .. "]"
end

local function draw_key_table(layout, root_state)
  local actions = root_state.action_list or {}
  local selected = tostring(root_state.action_select or "")
  local content_x = layout.right_x + 1
  local content_y = layout.right_y + 1
  local content_width = math.max(1, layout.right_width - 2)
  local action_width = math.max(8, math.floor(content_width * 0.40))
  local key_width = math.max(4, math.floor((content_width - action_width) / 4))

  local headers = {
    language(root_state, "SETTING_KEYBIND_SYSTEM_TABLE_ACTION", C.DEFAULT_TEXT.action),
    "[1]" .. language(root_state, "SETTING_KEYBIND_SYSTEM_TABLE_KEY1", C.DEFAULT_TEXT.key1),
    "[2]" .. language(root_state, "SETTING_KEYBIND_SYSTEM_TABLE_KEY2", C.DEFAULT_TEXT.key2),
    "[3]" .. language(root_state, "SETTING_KEYBIND_SYSTEM_TABLE_KEY3", C.DEFAULT_TEXT.key3),
    "[4]" .. language(root_state, "SETTING_KEYBIND_SYSTEM_TABLE_KEY4", C.DEFAULT_TEXT.key4)
  }

  canvas_draw_text(content_x + 1, content_y, headers[1], C.HEADER_COLOR, nil, BOLD, nil, action_width)
  for slot = 1, 4 do
    canvas_draw_text(content_x + action_width + (slot - 1) * key_width, content_y, headers[slot + 1], C.HEADER_COLOR, nil, BOLD, nil, key_width)
  end
  canvas_draw_text(content_x, content_y + 1, string.rep("─", content_width), C.SEPARATOR_COLOR, nil, nil, nil)

  local scroll = math.max(0, tonumber(root_state.action_scroll or 0) or 0)
  local max_rows = math.max(1, layout.table_body_height - 2)
  for offset = 1, max_rows do
    local index = scroll + offset
    local action = actions[index]
    if action == nil then break end
    local row_y = content_y + 1 + offset
    local is_selected = tostring(action.id or "") == selected
    local row_bg = nil
    if is_selected and root_state.focus == "keys" then
      row_bg = root_state.mode == "delete" and C.DELETE_BG_COLOR or C.SELECTED_BG_COLOR
      canvas_fill_rect(content_x, row_y, content_width, 1, " ", nil, row_bg)
    end
    if action.empty then
      canvas_fill_rect(content_x, row_y, 1, 1, " ", nil, C.EMPTY_BG_COLOR)
      canvas_fill_rect(content_x + content_width - 1, row_y, 1, 1, " ", nil, C.EMPTY_BG_COLOR)
    end
    local fg = row_bg ~= nil and C.SELECTED_FG_COLOR or C.NORMAL_COLOR
    canvas_draw_text(content_x + 1, row_y, tostring(action.name or action.id or ""), fg, row_bg, BOLD, ALIGN_LEFT, {
      warp_width = WINDOW,
      warp_height = 1,
      text_overflow = "..."
    })

    local display = action.key_display or {}
    local keys = key_array(display.key_user)
    for slot = 1, 4 do
      local key_text = key_label(keys[slot])
      canvas_draw_text(content_x + action_width + (slot - 1) * key_width, row_y, key_text, fg, row_bg, BOLD, nil, key_width)
    end
  end
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

local function slot_has_key(root_state, slot)
  local actions = root_state.action_list or {}
  for _, action in ipairs(actions) do
    if tostring(action.id or "") == tostring(root_state.action_select or "") then
      local display = action.key_display or {}
      local keys = key_array(display.key_user)
      local key = keys[slot]
      return key ~= nil and tostring(key) ~= ""
    end
  end
  return false
end

local function action_segments(root_state)
  local segments = {}

  if root_state.jump then
    table.insert(segments, "[1]-[9] " .. language(root_state, "SETTING_KEYBIND_SYSTEM_SELECT", C.DEFAULT_TEXT.select))
    table.insert(segments, C.DEFAULT_TEXT.confirm_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_CONFIRM", C.DEFAULT_TEXT.confirm))
    table.insert(segments, C.DEFAULT_TEXT.return_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_BACK", C.DEFAULT_TEXT.back))
  elseif State.listening_slot > 0 then
    local is_modify = slot_has_key(root_state, State.listening_slot)
    local any_text = language(root_state, "SETTING_KEYBIND_SYSTEM_KEY_ANY", C.DEFAULT_TEXT.key_any)
    local act_text = is_modify
      and language(root_state, "SETTING_KEYBIND_SYSTEM_MODIFY", C.DEFAULT_TEXT.modify)
      or language(root_state, "SETTING_KEYBIND_SYSTEM_ADD", C.DEFAULT_TEXT.add)
    local shift_text = is_modify
      and language(root_state, "SETTING_KEYBIND_SYSTEM_MODIFY_SHIFT", C.DEFAULT_TEXT.modify_shift)
      or language(root_state, "SETTING_KEYBIND_SYSTEM_ADD_SHIFT", C.DEFAULT_TEXT.add_shift)
    table.insert(segments, any_text .. " " .. act_text)
    table.insert(segments, "[Shift] " .. shift_text)
  elseif root_state.focus == "keys" then
    local mode_text = root_state.mode == "delete"
      and language(root_state, "SETTING_KEYBIND_SYSTEM_TIP_DELETE", C.DEFAULT_TEXT.delete_tip)
      or language(root_state, "SETTING_KEYBIND_SYSTEM_TIP_ADD_MODIFY", C.DEFAULT_TEXT.add_modify_tip)
    table.insert(segments, C.DEFAULT_TEXT.prev_option_key .. "/" .. C.DEFAULT_TEXT.next_option_key .. " "
      .. language(root_state, "SETTING_KEYBIND_SYSTEM_SELECT", C.DEFAULT_TEXT.select))
    local actions = root_state.action_list or {}
    local has_scroll = #actions > 0
    if has_scroll then
      table.insert(segments, C.DEFAULT_TEXT.scroll_up_key .. "/" .. C.DEFAULT_TEXT.scroll_down_key .. " "
        .. language(root_state, "SETTING_KEYBIND_SYSTEM_SCROLL", "Scroll"))
    end
    table.insert(segments, C.DEFAULT_TEXT.key1_key .. "/" .. C.DEFAULT_TEXT.key2_key .. "/" .. C.DEFAULT_TEXT.key3_key .. "/" .. C.DEFAULT_TEXT.key4_key .. " "
      .. mode_text)
    table.insert(segments, C.DEFAULT_TEXT.mode_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_KEY_MODE", C.DEFAULT_TEXT.key_mode))
    table.insert(segments, C.DEFAULT_TEXT.reset_only_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_RESET_ONLY", C.DEFAULT_TEXT.reset_only))
    table.insert(segments, C.DEFAULT_TEXT.page_reset_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_RESET_PAGE", C.DEFAULT_TEXT.page_reset))
    table.insert(segments, C.DEFAULT_TEXT.return_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_LIST", C.DEFAULT_TEXT.list))
  else
    table.insert(segments, C.DEFAULT_TEXT.prev_option_key .. "/" .. C.DEFAULT_TEXT.next_option_key .. " "
      .. language(root_state, "SETTING_KEYBIND_SYSTEM_SELECT", C.DEFAULT_TEXT.select))
    table.insert(segments, C.DEFAULT_TEXT.confirm_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_CONFIRM", C.DEFAULT_TEXT.confirm))
    table.insert(segments, C.DEFAULT_TEXT.order_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_ORDER", "Order"))
    table.insert(segments, C.DEFAULT_TEXT.sort_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_SORT", "Sort"))
    if root_state.pages and root_state.pages > 1 then
      table.insert(segments, C.DEFAULT_TEXT.jump_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_JUMP", "Jump"))
      table.insert(segments, C.DEFAULT_TEXT.prev_page_key .. "/" .. C.DEFAULT_TEXT.next_page_key .. " "
        .. language(root_state, "SETTING_KEYBIND_SYSTEM_NEXT_PAGE", "Page"))
    end
    table.insert(segments, C.DEFAULT_TEXT.return_key .. " " .. language(root_state, "SETTING_KEYBIND_SYSTEM_BACK", C.DEFAULT_TEXT.back))
  end

  return segments
end

local function draw_action_line(layout, root_state)
  local segments = action_segments(root_state)
  local width = math.max(1, layout.terminal_width - 2)
  local lines = wrap_segments(segments, "  ", width)
  local base_y = math.max(0, layout.terminal_height - #lines)
  for i, line in ipairs(lines) do
    local x = math.max(0, math.floor((layout.terminal_width - math.min(L.text_width(line), width)) / 2))
    canvas_draw_text(x, base_y + i - 1, line, C.KEY_COLOR, nil, nil, ALIGN_LEFT, width)
  end
end

function M.render(root_state)
  canvas_clear()
  root_state = root_state or {}
  State.set_root_state(root_state)
  root_state.pages = State.pages()
  root_state.page = tonumber(root_state.page or 1) or 1
  if root_state.page < 1 then root_state.page = 1 end
  if root_state.page > root_state.pages then root_state.page = root_state.pages end
  local hint_lines = #wrap_segments(action_segments(root_state), "  ", math.max(1, (L.terminal_size()) - 2))
  local layout = L.layout(hint_lines)

  draw_panel(layout.left_x, layout.left_y, layout.left_width, layout.content_height, "")
  draw_panel(layout.right_x, layout.right_y, layout.right_width, layout.right_height, language(root_state, "SETTING_KEYBIND_SYSTEM_KEY_TITLE", C.DEFAULT_TEXT.key_title))
  draw_list_title(layout, root_state)
  draw_page_list(layout, root_state)
  draw_page_line(layout, root_state)
  draw_key_table(layout, root_state)
  draw_action_line(layout, root_state)
end

M.State = State
return M
