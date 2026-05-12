local Render = load_function("keybind_system/render.lua")
local State = Render.State

local function normalize_state(lua_state)
  lua_state = State.normalize(lua_state or {})
  lua_state.confirm = false
  lua_state.back = false
  lua_state.pending_update = nil
  if not (lua_state.waiting_slot and lua_state.waiting_slot > 0) then
    shift_press = nil
    State.listening_slot = 0
  end
  return lua_state
end

local function selected_slot(action_name)
  local slot = tonumber(tostring(action_name or ""):match("^key(%d)$")) or 1
  if slot < 1 then return 1 end
  if slot > 4 then return 4 end
  return slot
end

local SHIFT_NAMES = { shift = true, left_shift = true, right_shift = true }
local SHIFT_TIMEOUT_MS = 2000
local shift_press = nil

local function key_from_event(event)
  if type(event) ~= "table" then
    return ""
  end
  if event.type == "key" then
    return tostring(event.name or "")
  end
  if event.type == "action" then
    local key_info = get_key(tostring(event.name or ""))
    if type(key_info) == "table" then
      local key_user = key_info.key_user
      if type(key_user) == "table" then
        return tostring(key_user[1] or "")
      end
      return tostring(key_user or "")
    end
  end
  return ""
end

local function set_update(lua_state, op, slot, key)
  lua_state.pending_update = {
    op = op,
    page = tostring(lua_state.select or ""),
    action = tostring(lua_state.action_select or ""),
    slot = slot or lua_state.key_slot or 1,
    key = key or ""
  }
end

local function handle_jump(lua_state, event)
  if event.type == "key" then
    if event.status ~= "press" then
      return lua_state
    end
    local digit = tonumber(event.name)
    if digit ~= nil then
      lua_state.user_page = (tonumber(lua_state.user_page) or 0) * 10 + digit
    end
    return lua_state
  end

  if event.type == "action" then
    if event.name == "confirm" then
      local pages = math.max(1, tonumber(lua_state.pages) or 1)
      local page = tonumber(lua_state.user_page) or lua_state.page or 1
      if page >= 1 and page <= pages then
        lua_state.page = page
      end
      lua_state.jump = false
      lua_state.user_page = 0
    elseif event.name == "return" then
      lua_state.jump = false
      lua_state.user_page = 0
    end
  end
  return lua_state
end

local function handle_waiting(lua_state, event)
  if shift_press then
    if event.type == "tick" then
      shift_press.elapsed = shift_press.elapsed + (event.dt_ms or 0)
      if shift_press.elapsed >= SHIFT_TIMEOUT_MS then
        lua_state.key_slot = lua_state.waiting_slot
        lua_state.waiting_slot = 0
        State.listening_slot = 0
        set_update(lua_state, "bind", lua_state.key_slot, shift_press.key)
        shift_press = nil
      end
      return lua_state
    end

    if event.type == "key" and event.status == "release" then
      local released_key = key_from_event(event)
      if released_key == shift_press.key then
        if shift_press.elapsed >= SHIFT_TIMEOUT_MS then
          lua_state.key_slot = lua_state.waiting_slot
          lua_state.waiting_slot = 0
          State.listening_slot = 0
          set_update(lua_state, "bind", lua_state.key_slot, shift_press.key)
        else
          lua_state.waiting_slot = 0
          State.listening_slot = 0
        end
        shift_press = nil
      end
      return lua_state
    end

    if event.status == "press" then
      local second_key = key_from_event(event)
      if second_key ~= "" and not SHIFT_NAMES[second_key] then
        lua_state.key_slot = lua_state.waiting_slot
        lua_state.waiting_slot = 0
        State.listening_slot = 0
        set_update(lua_state, "bind", lua_state.key_slot, second_key)
        shift_press = nil
      end
    end
    return lua_state
  end

  if event.status == "press" then
    local key = key_from_event(event)
    if key ~= "" then
      if SHIFT_NAMES[key] then
        shift_press = { key = key, elapsed = 0 }
      else
        lua_state.key_slot = lua_state.waiting_slot
        lua_state.waiting_slot = 0
        State.listening_slot = 0
        set_update(lua_state, "bind", lua_state.key_slot, key)
      end
    end
  end
  return lua_state
end

local function handle_keys_focus(lua_state, event)
  if event.type ~= "action" then
    return lua_state
  end

  if event.status ~= "press" then
    return lua_state
  end

  if event.name == "prev_option" then
    State.move_action_select(lua_state, -1)
  elseif event.name == "next_option" then
    State.move_action_select(lua_state, 1)
  elseif event.name == "scroll_up" then
    State.scroll_actions(lua_state, -1)
  elseif event.name == "scroll_down" then
    State.scroll_actions(lua_state, 1)
  elseif event.name == "list" or event.name == "return" then
    lua_state.focus = "list"
    lua_state.action_select = ""
    lua_state.action_scroll = 0
  elseif event.name == "key_mode" then
    lua_state.mode = lua_state.mode == "delete" and "add" or "delete"
  elseif event.name == "delete" then
    lua_state.mode = "delete"
  elseif event.name == "reset_only" then
    set_update(lua_state, "reset", lua_state.key_slot or 1, "")
  elseif event.name == "page_reset" then
    set_update(lua_state, "page_reset", 0, "")
  elseif event.name == "key1" or event.name == "key2" or event.name == "key3" or event.name == "key4" then
    lua_state.key_slot = selected_slot(event.name)
    if lua_state.mode == "delete" then
      set_update(lua_state, "delete", lua_state.key_slot, "")
    else
      lua_state.waiting_slot = lua_state.key_slot
      State.listening_slot = lua_state.key_slot
    end
  end

  return lua_state
end

local function handle_list_focus(lua_state, event)
  if event.type ~= "action" then
    return lua_state
  end

  if event.status ~= "press" then
    return lua_state
  end

  if event.name == "prev_option" then
    State.move_page_select(lua_state, -1)
  elseif event.name == "next_option" then
    State.move_page_select(lua_state, 1)
  elseif event.name == "prev_page" then
    lua_state.page = math.max(1, (lua_state.page or 1) - 1)
    State.select_first_visible_page(lua_state)
  elseif event.name == "next_page" then
    lua_state.page = math.min(lua_state.pages or 1, (lua_state.page or 1) + 1)
    State.select_first_visible_page(lua_state)
  elseif event.name == "jump" and (lua_state.pages or 1) > 1 then
    lua_state.jump = true
    lua_state.user_page = 0
  elseif event.name == "order" then
    lua_state.order = lua_state.order == "desc" and "asc" or "desc"
  elseif event.name == "sort" then
    lua_state.sort = lua_state.sort == "conflict" and "name" or "conflict"
  elseif event.name == "confirm" then
    lua_state.focus = "keys"
    lua_state.confirm = true
    lua_state.action_select = ""
    lua_state.action_scroll = 0
    lua_state.key_slot = 1
    lua_state.mode = "add"
  elseif event.name == "return" then
    if not State.has_empty_actions() then
      lua_state.back = true
    end
  end

  return lua_state
end

function handle_event(lua_state, event)
  lua_state = normalize_state(lua_state)
  if type(event) ~= "table" then
    return lua_state
  end

  if lua_state.waiting_slot and lua_state.waiting_slot > 0 then
    return State.normalize(handle_waiting(lua_state, event))
  end

  if lua_state.jump then
    return State.normalize(handle_jump(lua_state, event))
  end

  if lua_state.focus == "keys" then
    return State.normalize(handle_keys_focus(lua_state, event))
  end

  return State.normalize(handle_list_focus(lua_state, event))
end

function render(root_state)
  Render.render(root_state or {})
end
