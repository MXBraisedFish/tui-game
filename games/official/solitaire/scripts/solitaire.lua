local DEFAULT_MODE_ORDER = { "freecell", "klondike", "spider" }
local MODE_KEY = {
  freecell = "game.solitaire.mode.freecell",
  klondike = "game.solitaire.mode.klondike",
  spider = "game.solitaire.mode.spider",
}

local DEFAULT_TARGETS = { freecell = 18, klondike = 22, spider = 26 }
local MODE_ORDER = DEFAULT_MODE_ORDER
local TARGETS = DEFAULT_TARGETS

load_helper("helpers/clone.lua")

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function load_mode_config()
  local ok, data = pcall(read_json, "data/modes.json")
  if not ok or type(data) ~= "table" then
    return
  end

  if type(data.order) == "table" and #data.order > 0 then
    local order = {}
    for _, value in ipairs(data.order) do
      if type(value) == "string" and value ~= "" then
        order[#order + 1] = value
      end
    end
    if #order > 0 then
      MODE_ORDER = order
    end
  end

  if type(data.targets) == "table" then
    local targets = {}
    for mode, fallback in pairs(DEFAULT_TARGETS) do
      local value = data.targets[mode]
      targets[mode] = type(value) == "number" and math.max(1, math.floor(value)) or fallback
    end
    TARGETS = targets
  end
end

load_mode_config()

local function mode_name(mode)
  return tr(MODE_KEY[mode] or "game.solitaire.mode.freecell")
end

local function fresh_columns()
  return {
    { "K", "Q", "J" },
    { "10", "9", "8" },
    { "7", "6", "5" },
    { "4", "3", "2", "A" },
  }
end

local function load_best_record(state)
  local best = load_data("best_record")
  state.best = { freecell = 0, klondike = 0, spider = 0 }
  if type(best) == "table" then
    for _, mode in ipairs(MODE_ORDER) do
      state.best[mode] = math.max(0, math.floor(tonumber(best[mode]) or 0))
    end
  end
end

local function save_best_record(state)
  save_data("best_record", state.best)
  request_refresh_best_score()
end

local function fresh_state(mode)
  local state = {
    mode = mode or "freecell",
    cursor = 1,
    columns = fresh_columns(),
    foundations = 0,
    elapsed_ms = 0,
    won = false,
    message = "game.solitaire.runtime_ready",
    undo_stack = {},
    best = { freecell = 0, klondike = 0, spider = 0 },
  }
  load_best_record(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" or type(state.columns) ~= "table" then
    return fresh_state("freecell")
  end
  state.mode = state.mode or "freecell"
  state.cursor = math.max(1, math.min(4, math.floor(tonumber(state.cursor) or 1)))
  state.foundations = math.max(0, math.floor(tonumber(state.foundations) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.won = state.won == true
  state.message = state.message or "game.solitaire.runtime_ready"
  state.undo_stack = type(state.undo_stack) == "table" and state.undo_stack or {}
  load_best_record(state)
  return state
end

function init_game()
  return restore_state()
end

local function save_progress(state)
  save_data("state", {
    mode = state.mode,
    cursor = state.cursor,
    columns = state.columns,
    foundations = state.foundations,
    elapsed_ms = state.elapsed_ms,
    won = state.won,
    message = state.message,
    undo_stack = state.undo_stack,
  })
end

local function maybe_update_best(state)
  local elapsed = math.floor(state.elapsed_ms / 1000)
  local current = state.best[state.mode]
  if current <= 0 or elapsed < current then
    state.best[state.mode] = elapsed
    save_best_record(state)
  end
end

local function push_undo(state)
  state.undo_stack[#state.undo_stack + 1] = {
    columns = clone_columns(state.columns),
    foundations = state.foundations,
  }
  if #state.undo_stack > 20 then
    table.remove(state.undo_stack, 1)
  end
end

local function move_card(state)
  if state.won then
    return state
  end
  local column = state.columns[state.cursor]
  if #column == 0 then
    state.message = "game.solitaire.select_empty"
    return state
  end
  push_undo(state)
  table.remove(column)
  state.foundations = state.foundations + 1
  if state.foundations >= TARGETS[state.mode] then
    state.won = true
    state.message = "game.solitaire.win_banner"
    maybe_update_best(state)
  else
    state.message = "game.solitaire.move_invalid"
  end
  save_progress(state)
  return state
end

local function switch_mode(state)
  local index = 1
  for i, mode in ipairs(MODE_ORDER) do
    if mode == state.mode then
      index = i
      break
    end
  end
  index = index % #MODE_ORDER + 1
  return fresh_state(MODE_ORDER[index])
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.won then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.solitaire.runtime_resized"
    return state
  end
  if event.type == "quit" then
    request_exit()
    return state
  end
  if event.type ~= "action" then
    return state
  end

  if event.name == "quit_action" then
    request_exit()
  elseif event.name == "restart" then
    return fresh_state(state.mode)
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.solitaire.runtime_saved"
  elseif event.name == "switch_mode" then
    return switch_mode(state)
  elseif event.name == "prev_column" then
    state.cursor = math.max(1, state.cursor - 1)
  elseif event.name == "next_column" then
    state.cursor = math.min(#state.columns, state.cursor + 1)
  elseif event.name == "move_card" then
    return move_card(state)
  elseif event.name == "undo" then
    local snapshot = table.remove(state.undo_stack)
    if snapshot then
      state.columns = snapshot.columns
      state.foundations = snapshot.foundations
      state.message = "game.solitaire.undo_done"
      save_progress(state)
    else
      state.message = "game.solitaire.undo_empty"
    end
  end
  return state
end

local function format_duration(total_seconds)
  local h = math.floor(total_seconds / 3600)
  local m = math.floor((total_seconds % 3600) / 60)
  local s = total_seconds % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function render(state)
  canvas_clear()
  local _, term_h = get_terminal_size()
  local title = tr("game.solitaire.name")
  canvas_draw_text(centered_x(title), 2, title, "cyan", nil)
  canvas_draw_text(4, 4, tr("game.solitaire.mode") .. ": " .. mode_name(state.mode), "white", nil)
  canvas_draw_text(4, 5, tr("game.solitaire.foundations") .. ": " .. tostring(state.foundations), "white", nil)
  canvas_draw_text(4, 6, tr("game.solitaire.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000)), "white", nil)
  local base_x = math.max(4, resolve_x(ANCHOR_CENTER, 36, 0))
  for i = 1, #state.columns do
    local x = base_x + (i - 1) * 9
    canvas_draw_text(x, 9, "[" .. tostring(i) .. "]", i == state.cursor and "yellow" or "dark_gray", nil)
    local cards = #state.columns[i] > 0 and table.concat(state.columns[i], " ") or "--"
    canvas_draw_text(x, 10, cards, i == state.cursor and "white" or "dark_gray", nil)
  end
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.min(term_h - 3, 14), message, state.won and "green" or "white", nil)
  local controls_key = "game.solitaire.controls." .. state.mode
  canvas_draw_text(centered_x(tr(controls_key)), term_h - 1, tr(controls_key), "dark_gray", nil)
end

function best_score(state)
  local function best_value(mode)
    local value = state.best[mode]
    if not value or value <= 0 then
      return "--:--:--"
    end
    return format_duration(value)
  end
  return {
    best_string = "game.solitaire.best_block",
    freecell = best_value("freecell"),
    klondike = best_value("klondike"),
    spider = best_value("spider"),
  }
end
