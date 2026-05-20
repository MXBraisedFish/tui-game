local M = {}

local function normalize_select(value)
  if type(value) ~= "number" then return 1 end
  value = math.floor(value)
  if value < 1 then return 9 end
  if value > 9 then return 1 end
  return value
end

local function current_list(root_state, panel)
  if panel == "saver" then
    return root_state.saver_list or {}
  elseif panel == "boss" then
    return root_state.boss_list or {}
  end
  return {}
end

local function is_back_action(event)
  return event.name == "return" or event.name == "back"
end

local function visible_list_height()
  local _, terminal_height = get_terminal_size()
  terminal_height = tonumber(terminal_height or 26) or 26
  return math.max(1, terminal_height - 3)
end

local function clamp_list_scroll(state, root_state)
  local items = current_list(root_state, state.panel)
  local max_scroll = math.max(0, #items - visible_list_height())
  state.list_scroll = math.max(0, math.min(max_scroll, math.floor(state.list_scroll or 0)))
end

local function normalize_list_select(state, root_state)
  local items = current_list(root_state, state.panel)
  if #items == 0 then
    state.list_select = ""
    return
  end
  for _, item in ipairs(items) do
    if item.uid == state.list_select then
      return
    end
  end
  state.list_select = items[1].uid or ""
end

local function selected_index(state, root_state)
  local items = current_list(root_state, state.panel)
  for index, item in ipairs(items) do
    if item.uid == state.list_select then
      return index
    end
  end
  return #items > 0 and 1 or 0
end

local function move_selection(state, root_state, delta)
  local items = current_list(root_state, state.panel)
  if #items == 0 then return end
  local index = selected_index(state, root_state)
  index = math.max(1, math.min(#items, index + delta))
  state.list_select = items[index].uid or ""
end

function M.initial_state(root_state)
  root_state = root_state or {}
  local state = {
    select = normalize_select(root_state.select),
    confirm = false,
    back = false,
    panel = root_state.panel or "none",
    list_select = root_state.list_select or "",
    list_scroll = math.max(0, math.floor(root_state.list_scroll or 0)),
    move_mode = root_state.move_mode == true,
    move_delta = 0,
    position_mode = root_state.position_mode == true,
    position_input = math.max(0, math.floor(root_state.position_input or 0)),
    position_target = 0,
  }
  normalize_list_select(state, root_state)
  return state
end

local function exit_panel(state)
  state.panel = "none"
  state.move_mode = false
  state.position_mode = false
  state.position_input = 0
end

function M.handle_event(state, root_state, event)
  state.confirm = false
  state.back = false
  state.move_delta = 0
  state.position_target = 0

  if type(event) ~= "table" then return state end
  if event.type == "key" and event.status ~= nil and event.status ~= "press" then
    return state
  end
  if event.type ~= "action" then return state end

  local in_panel = state.panel ~= "none"

  if in_panel and is_back_action(event) then
    if state.position_mode then
      state.position_mode = false
      state.position_input = 0
    end
    exit_panel(state)
    return state
  end

  if in_panel and state.position_mode then
    if event.name and event.name:match("^option[1-9]$") then
      local digit = tonumber(event.name:sub(7)) or 0
      state.position_input = math.min(999, state.position_input * 10 + digit)
    elseif event.name == "confirm" then
      state.position_target = state.position_input
      state.position_input = 0
    elseif event.name == "position" then
      state.position_mode = false
      state.position_input = 0
    end
    return state
  end

  if in_panel then
    if event.name == "scroll_up" then
      state.list_scroll = math.max(0, state.list_scroll - 1)
    elseif event.name == "scroll_down" then
      state.list_scroll = state.list_scroll + 1
      clamp_list_scroll(state, root_state)
    elseif event.name == "prev_option" then
      if state.move_mode then state.move_delta = -1 else move_selection(state, root_state, -1) end
    elseif event.name == "next_option" then
      if state.move_mode then state.move_delta = 1 else move_selection(state, root_state, 1) end
    elseif event.name == "confirm" then
      state.confirm = true
    elseif event.name == "order" then
      state.move_mode = not state.move_mode
      state.position_mode = false
    elseif event.name == "position" then
      state.position_mode = not state.position_mode
      state.move_mode = false
      state.position_input = 0
    end
    return state
  end

  if event.name == "prev_option" then
    state.select = normalize_select(state.select - 1)
  elseif event.name == "next_option" then
    state.select = normalize_select(state.select + 1)
  elseif event.name == "confirm" then
    if state.select == 8 or state.select == 9 then
      state.panel = state.select == 8 and "saver" or "boss"
      normalize_list_select(state, root_state)
    else
      state.confirm = true
    end
  elseif is_back_action(event) then
    state.back = true
  elseif event.name and event.name:match("^option[1-9]$") then
    state.select = tonumber(event.name:sub(7)) or state.select
  end
  return state
end

return M
