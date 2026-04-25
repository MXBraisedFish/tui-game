local C = load_function("/constants.lua")
local U = load_function("/utils.lua")
local S = load_function("/storage.lua")

local M = {}

local function format_cell_value(v)
  if v == 0 then
    return "."
  end
  local text = tostring(v)
  if #text > 4 then
    if v >= 1000000000 then
      text = tostring(math.floor(v / 1000000000)) .. "g"
    elseif v >= 1000000 then
      text = tostring(math.floor(v / 1000000)) .. "m"
    elseif v >= 1000 then
      text = tostring(math.floor(v / 1000)) .. "k"
    end
  end
  if #text > 4 then
    text = string.sub(text, 1, 4)
  end
  return text
end

local function tile_bg_color(v)
  if v == 0 then return "rgb(90,90,90)" end
  if v == 2 then return "rgb(255,255,255)" end
  if v == 4 then return "rgb(255,229,229)" end
  if v == 8 then return "rgb(255,204,204)" end
  if v == 16 then return "rgb(255,178,178)" end
  if v == 32 then return "rgb(255,153,153)" end
  if v == 64 then return "rgb(255,127,127)" end
  if v == 128 then return "rgb(255,102,102)" end
  if v == 256 then return "rgb(255,76,76)" end
  if v == 512 then return "rgb(255,50,50)" end
  if v == 1024 then return "rgb(255,25,25)" end
  if v == 2048 then return "rgb(255,0,0)" end
  if v == 4096 then return "rgb(212,0,0)" end
  if v == 8192 then return "rgb(170,0,0)" end
  if v == 16384 then return "rgb(127,0,0)" end
  if v == 32768 then return "rgb(85,0,0)" end
  if v == 65536 then return "rgb(42,0,0)" end
  return "rgb(0,0,0)"
end

local function text_color_for_value(v)
  if v == 0 then
    return "black"
  end
  if v <= 2048 then
    return "black"
  end
  return "white"
end

function M.board_geometry()
  local w, h = get_terminal_size()
  local grid_w = C.SIZE * C.CELL_W
  local grid_h = C.SIZE * C.CELL_H
  local frame_w = grid_w + 2
  local frame_h = grid_h + 2
  local x = math.floor((w - frame_w) / 2)
  local y = math.floor((h - frame_h) / 2)
  if x < 1 then x = 1 end
  if y < 5 then y = 5 end
  return x, y, frame_w, frame_h
end

local function draw_outer_frame(x, y, frame_w, frame_h)
  U.draw_text(x, y, C.BORDER_TL .. string.rep(C.BORDER_H, frame_w - 2) .. C.BORDER_TR, "white", "black")
  for i = 1, frame_h - 2 do
    U.draw_text(x, y + i, C.BORDER_V, "white", "black")
    U.draw_text(x + frame_w - 1, y + i, C.BORDER_V, "white", "black")
  end
  U.draw_text(x, y + frame_h - 1, C.BORDER_BL .. string.rep(C.BORDER_H, frame_w - 2) .. C.BORDER_BR, "white", "black")
end

local function draw_tile(tile_x, tile_y, value)
  local bg = tile_bg_color(value)
  local fg = text_color_for_value(value)
  for row = 0, C.CELL_H - 1 do
    U.draw_text(tile_x, tile_y + row, string.rep(" ", C.CELL_W), fg, bg)
  end
  local text = format_cell_value(value)
  local text_x = tile_x + math.floor((C.CELL_W - #text) / 2)
  local text_y = tile_y + math.floor(C.CELL_H / 2)
  U.draw_text(text_x, text_y, text, fg, bg)
end

local function draw_status(state, x, y, frame_w)
  local elapsed = S.elapsed_seconds(state)
  local left = U.tr("game.2048.time") .. " " .. U.format_duration(elapsed)
  local right = U.tr("game.2048.score") .. " " .. tostring(state.score)
  local term_w = select(1, get_terminal_size())
  local right_x = x + frame_w - U.text_width(right)
  if right_x < 1 then
    right_x = 1
  end

  U.draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
  U.draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
  U.draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

  local best_line = U.tr("game.2048.best_score")
      .. " "
      .. tostring(math.max(0, state.best_score))
      .. "  "
      .. U.tr("game.2048.best_time")
      .. " "
      .. U.format_duration(math.max(0, state.best_time_sec))
  local best_x = math.floor((term_w - U.text_width(best_line)) / 2)
  if best_x < 1 then best_x = 1 end
  U.draw_text(best_x, y - 3, best_line, "dark_gray", "black")
  U.draw_text(x, y - 2, left, "light_cyan", "black")
  U.draw_text(right_x, y - 2, right, "light_cyan", "black")

  local notice = nil
  local notice_fg = "white"
  if state.won then
    notice = U.tr("game.2048.win_banner") .. U.key_label("restart") .. " " .. U.tr("game.2048.action.restart") .. "  " .. U.key_label("quit_action") .. " " .. U.tr("game.2048.action.quit")
    notice_fg = "yellow"
  elseif state.confirm_mode == "game_over" then
    notice = U.tr("game.2048.game_over")
    notice_fg = "red"
  elseif state.confirm_mode == "restart" then
    notice = U.replace_prompt_keys(U.tr("game.2048.confirm_restart"))
    notice_fg = "yellow"
  elseif state.confirm_mode == "exit" then
    notice = U.replace_prompt_keys(U.tr("game.2048.confirm_exit"))
    notice_fg = "yellow"
  elseif state.toast_text ~= nil and state.frame <= state.toast_until then
    notice = state.toast_text
    notice_fg = "green"
  end

  if notice ~= nil then
    local notice_x = math.floor((term_w - U.text_width(notice)) / 2)
    if notice_x < 1 then notice_x = 1 end
    U.draw_text(notice_x, y - 1, notice, notice_fg, "black")
  end
end

local function draw_controls(x, y, frame_h)
  local term_w = select(1, get_terminal_size())
  local controls = table.concat({
    U.key_label("move_up") .. "/" .. U.key_label("move_down") .. "/" .. U.key_label("move_left") .. "/" .. U.key_label("move_right") .. " " .. U.tr("game.2048.action.move_up"),
    U.key_label("restart") .. " " .. U.tr("game.2048.action.restart"),
    U.key_label("save") .. " " .. U.tr("game.2048.action.save"),
    U.key_label("quit_action") .. " " .. U.tr("game.2048.action.quit")
  }, "  ")
  local max_w = math.max(10, term_w - 2)
  local lines = U.wrap_words(controls, max_w)
  if #lines > 3 then
    lines = { lines[1], lines[2], lines[3] }
  end

  U.draw_text(1, y + frame_h + 1, string.rep(" ", term_w), "white", "black")
  U.draw_text(1, y + frame_h + 2, string.rep(" ", term_w), "white", "black")
  U.draw_text(1, y + frame_h + 3, string.rep(" ", term_w), "white", "black")

  local offset = 0
  if #lines < 3 then
    offset = math.floor((3 - #lines) / 2)
  end
  for i = 1, #lines do
    local line = lines[i]
    local controls_x = math.floor((term_w - U.text_width(line)) / 2)
    if controls_x < 1 then
      controls_x = 1
    end
    U.draw_text(controls_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
  end
end

function M.render(state)
  local x, y, frame_w, frame_h = M.board_geometry()
  U.fill_rect(x, y - 3, frame_w, frame_h + 7, "black")
  draw_status(state, x, y, frame_w)
  draw_outer_frame(x, y, frame_w, frame_h)

  local inner_x = x + 1
  local inner_y = y + 1
  for r = 1, C.SIZE do
    for col = 1, C.SIZE do
      local tx = inner_x + (col - 1) * C.CELL_W
      local ty = inner_y + (r - 1) * C.CELL_H
      draw_tile(tx, ty, state.board[r][col])
    end
  end

  draw_controls(x, y, frame_h)
end

return M
