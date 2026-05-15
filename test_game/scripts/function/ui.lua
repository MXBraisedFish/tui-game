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

local function draw_centered(y, text, color)
  local terminal_width = get_terminal_size()
  local text_width = get_text_width(text)
  local x = resolve_x(ANCHOR_CENTER, text_width, 0)
  canvas_draw_text(x, y, text, color)
end

local function draw_help_line(y)
  local help_text = table.concat({
    key_label("move_up") .. "/" .. key_label("move_down") .. "/" .. key_label("move_left") .. "/" .. key_label("move_right") .. " " .. translate("test_game.help.move"),
    key_label("confirm") .. " " .. translate("test_game.help.confirm"),
    key_label("quit") .. " " .. translate("test_game.help.quit")
  }, "  ")
  draw_centered(y, help_text, "grey")
end

function M.render(state)
  canvas_clear()

  local terminal_width, terminal_height = get_terminal_size()
  local title = translate("test_game.title")
  local subtitle = translate("test_game.subtitle")
  local info = string.format(
    "%s: %d  %s: %s  %s: %.1fs",
    translate("test_game.label.moves"),
    state.moves or 0,
    translate("test_game.label.event"),
    tostring(state.last_event or "-"),
    translate("test_game.label.time"),
    (state.running_ms or 0) / 1000
  )

  draw_centered(1, title, "cyan")
  draw_centered(3, subtitle, "white")
  draw_centered(5, info, "grey")

  local player_x = math.max(0, math.min(state.player_x or math.floor(terminal_width / 2), terminal_width - 1))
  local player_y = math.max(0, math.min(state.player_y or math.floor(terminal_height / 2), terminal_height - 1))
  canvas_draw_text(player_x, player_y, "@", "#ffa500", nil, BOLD)

  if state.message ~= nil then
    draw_centered(math.max(7, terminal_height - 4), tostring(state.message), "green")
  end

  draw_help_line(math.max(8, terminal_height - 2))
end

return M
