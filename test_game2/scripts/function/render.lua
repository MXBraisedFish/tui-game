local M = {}

local function key_label(action)
  local key_info = get_key(action)
  if key_info == nil or key_info.key_display == nil then
    return "[]"
  end
  local key_user = key_info.key_display.key_user
  if type(key_user) == "table" then
    local parts = {}
    for index = 1, #key_user do
      parts[#parts + 1] = "[" .. tostring(key_user[index]) .. "]"
    end
    return table.concat(parts, "/")
  end
  return "[" .. tostring(key_user) .. "]"
end

local function draw_centered(y, text, color, style)
  local x = resolve_x(ANCHOR_CENTER, get_text_width(text), 0)
  canvas_draw_text(x, y, text, color, nil, style, nil)
end

local function draw_help(y)
  local text = table.concat({
    key_label("move_up") .. "/" .. key_label("move_down") .. "/" .. key_label("move_left") .. "/" .. key_label("move_right") .. " " .. translate("test_game2.help.move"),
    key_label("collect") .. " " .. translate("test_game2.help.collect"),
    key_label("reset") .. " " .. translate("test_game2.help.reset"),
    key_label("save") .. " " .. translate("test_game2.help.save"),
    key_label("debug") .. " " .. translate("test_game2.help.debug"),
    key_label("quit") .. " " .. translate("test_game2.help.quit")
  }, "  ")
  canvas_draw_text(1, y, text, "grey", nil, nil, ALIGN_CENTER, math.max(1, get_terminal_size() - 2))
end

local function draw_stats(state)
  local runtime_seconds = math.floor((state.running_ms or 0) / 100) / 10
  local stats = string.format(
    "%s: %d  %s: %d  %s: %d  %s: %s  %s: %.1fs  %s: %d",
    translate("test_game2.label.score"), state.score or 0,
    translate("test_game2.label.best"), state.best_score or 0,
    translate("test_game2.label.moves"), state.moves or 0,
    translate("test_game2.label.event"), tostring(state.last_event or "-"),
    translate("test_game2.label.time"), runtime_seconds,
    translate("test_game2.label.saved"), state.saved_count or 0
  )
  canvas_draw_text(1, 3, stats, "white", nil, nil, ALIGN_CENTER, math.max(1, get_terminal_size() - 2))
end

function M.render(state)
  canvas_clear()
  state = state or {}
  local width, height = get_terminal_size()

  draw_centered(1, translate("test_game2.title"), "cyan", BOLD)
  draw_stats(state)
  draw_centered(5, translate("test_game2.subtitle"), "grey", nil)

  local play_y = math.max(7, math.floor(height / 2))
  canvas_fill_rect(0, play_y - 2, width, 5, " ", nil, "dark_gray")

  if state.star ~= nil and state.star_visible ~= false then
    canvas_draw_text(state.star.x or 0, state.star.y or play_y, "*", "yellow", nil, BOLD, nil)
  end

  if state.player ~= nil then
    canvas_draw_text(state.player.x or 0, state.player.y or play_y, "@", "orange", nil, BOLD, nil)
  end

  if state.message ~= nil then
    draw_centered(math.max(8, height - 4), tostring(state.message), "green", nil)
  end
  draw_help(math.max(9, height - 2))
end

return M