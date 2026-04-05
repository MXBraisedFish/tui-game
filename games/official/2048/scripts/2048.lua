local SIZE = 4
local TARGET_TILE = 131072
local MAX_TILE = 2147483647
local FPS = 60
local CELL_W = 8
local CELL_H = 4

local BORDER_TL = utf8.char(9484)
local BORDER_TR = utf8.char(9488)
local BORDER_BL = utf8.char(9492)
local BORDER_BR = utf8.char(9496)
local BORDER_H = utf8.char(9472)
local BORDER_V = utf8.char(9474)

local state = nil

local function tr(key)
  return translate(key)
end

local function draw_text(x, y, text, fg, bg)
  canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text, fg, bg)
end

local function fill_rect(x, y, w, h, bg)
  if w <= 0 or h <= 0 then
    return
  end
  canvas_fill_rect(math.max(0, x - 1), math.max(0, y - 1), w, h, " ", nil, bg or "black")
end

local function random(n)
  if n <= 0 then
    return 0
  end
  return math.random(0, n - 1)
end

local function normalize_key_name(event)
  if type(event) ~= "table" then
    return ""
  end
  if event.type == "quit" then
    return "esc"
  end
  if event.type == "key" and type(event.name) == "string" then
    return string.lower(event.name)
  end
  if event.type ~= "action" then
    return ""
  end

  local map = {
    move_left = "left",
    move_right = "right",
    move_up = "up",
    move_down = "down",
    save = "s",
    restart = "r",
    quit_action = "q",
    confirm_yes = "enter",
    confirm_no = "esc",
  }
  return map[event.name] or ""
end

local function deep_copy_board(board)
  local out = {}
  for r = 1, SIZE do
    out[r] = {}
    for c = 1, SIZE do
      out[r][c] = board[r][c]
    end
  end
  return out
end

local function init_empty_board()
  local board = {}
  for r = 1, SIZE do
    board[r] = {}
    for c = 1, SIZE do
      board[r][c] = 0
    end
  end
  return board
end

local function random_tile_value()
  if random(10) == 0 then
    return 4
  end
  return 2
end

local function list_empty_cells(board)
  local cells = {}
  for r = 1, SIZE do
    for c = 1, SIZE do
      if board[r][c] == 0 then
        cells[#cells + 1] = { r = r, c = c }
      end
    end
  end
  return cells
end

local function spawn_tile(board)
  local empty = list_empty_cells(board)
  if #empty == 0 then
    return false
  end
  local pick = empty[random(#empty) + 1]
  if pick == nil then
    return false
  end
  board[pick.r][pick.c] = random_tile_value()
  return true
end

local function format_duration(sec)
  local h = math.floor(sec / 3600)
  local m = math.floor((sec % 3600) / 60)
  local s = sec % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

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

local function text_width(text)
  local ok, w = pcall(get_text_width, text)
  if ok and type(w) == "number" then
    return w
  end
  return #text
end

local function wrap_words(text, max_width)
  if max_width <= 1 then
    return { text }
  end
  local lines = {}
  local current = ""
  local had_token = false
  for token in string.gmatch(text, "%S+") do
    had_token = true
    if current == "" then
      current = token
    else
      local candidate = current .. " " .. token
      if text_width(candidate) <= max_width then
        current = candidate
      else
        lines[#lines + 1] = current
        current = token
      end
    end
  end
  if not had_token then
    return { "" }
  end
  if current ~= "" then
    lines[#lines + 1] = current
  end
  return lines
end

local function min_width_for_lines(text, max_lines, hard_min)
  local full = text_width(text)
  local width = hard_min
  while width <= full do
    if #wrap_words(text, width) <= max_lines then
      return width
    end
    width = width + 1
  end
  return full
end

local function merge_line(values)
  local compact = {}
  for i = 1, #values do
    if values[i] ~= 0 then
      compact[#compact + 1] = values[i]
    end
  end

  local out = {}
  local gained = 0
  local i = 1
  while i <= #compact do
    if i < #compact and compact[i] == compact[i + 1] then
      local merged = compact[i] * 2
      if merged > MAX_TILE then
        merged = MAX_TILE
      end
      out[#out + 1] = merged
      gained = gained + merged
      i = i + 2
    else
      out[#out + 1] = compact[i]
      i = i + 1
    end
  end

  while #out < SIZE do
    out[#out + 1] = 0
  end
  return out, gained
end

local function get_row(board, r)
  local line = {}
  for c = 1, SIZE do
    line[c] = board[r][c]
  end
  return line
end

local function set_row(board, r, line)
  for c = 1, SIZE do
    board[r][c] = line[c]
  end
end

local function get_col(board, c)
  local line = {}
  for r = 1, SIZE do
    line[r] = board[r][c]
  end
  return line
end

local function set_col(board, c, line)
  for r = 1, SIZE do
    board[r][c] = line[r]
  end
end

local function reverse_line(line)
  local out = {}
  for i = 1, SIZE do
    out[i] = line[SIZE - i + 1]
  end
  return out
end

local function lines_equal(a, b)
  for i = 1, SIZE do
    if a[i] ~= b[i] then
      return false
    end
  end
  return true
end

local function apply_move(dir)
  local moved = false
  local gained = 0

  if dir == "left" or dir == "right" then
    for r = 1, SIZE do
      local old = get_row(state.board, r)
      local line = old
      local gained_line = 0
      if dir == "right" then
        line = reverse_line(line)
      end
      line, gained_line = merge_line(line)
      if dir == "right" then
        line = reverse_line(line)
      end
      set_row(state.board, r, line)
      if not lines_equal(old, line) then
        moved = true
      end
      gained = gained + gained_line
    end
  else
    for c = 1, SIZE do
      local old = get_col(state.board, c)
      local line = old
      local gained_line = 0
      if dir == "down" then
        line = reverse_line(line)
      end
      line, gained_line = merge_line(line)
      if dir == "down" then
        line = reverse_line(line)
      end
      set_col(state.board, c, line)
      if not lines_equal(old, line) then
        moved = true
      end
      gained = gained + gained_line
    end
  end

  if moved then
    state.score = state.score + gained
  end
  return moved
end

local function can_move_any()
  if #list_empty_cells(state.board) > 0 then
    return true
  end
  for r = 1, SIZE do
    for c = 1, SIZE do
      local v = state.board[r][c]
      if r < SIZE and state.board[r + 1][c] == v then
        return true
      end
      if c < SIZE and state.board[r][c + 1] == v then
        return true
      end
    end
  end
  return false
end

local function elapsed_seconds()
  local end_frame = state.end_frame
  if end_frame == nil then
    end_frame = state.frame
  end
  return math.floor((end_frame - state.start_frame) / FPS)
end

local function commit_stats()
  local score = tonumber(state.score) or 0
  local duration = elapsed_seconds()
  if score > state.best_score or (score == state.best_score and score > 0 and (state.best_time_sec == 0 or duration < state.best_time_sec)) then
    state.best_score = score
    state.best_time_sec = duration
    save_data("2048_best", { score = state.best_score, time_sec = state.best_time_sec })
    request_refresh_best_score()
  end
end

local function update_win_and_loss()
  local was_won = state.won
  state.won = false
  for r = 1, SIZE do
    for c = 1, SIZE do
      if state.board[r][c] >= TARGET_TILE then
        state.won = true
        state.win_message_until = state.frame + 3 * FPS
        if not was_won then
          state.end_frame = state.frame
          commit_stats()
        end
        return
      end
    end
  end
end

local function make_snapshot()
  return {
    board = deep_copy_board(state.board),
    score = state.score,
    elapsed_sec = math.floor((state.frame - state.start_frame) / FPS),
  }
end

local function load_best_record()
  local data = load_data("2048_best")
  if type(data) ~= "table" then
    state.best_score = 0
    state.best_time_sec = 0
    return
  end
  state.best_score = math.max(0, math.floor(tonumber(data.score) or 0))
  state.best_time_sec = math.max(0, math.floor(tonumber(data.time_sec) or 0))
end

local function restore_snapshot(snapshot)
  if type(snapshot) ~= "table" or type(snapshot.board) ~= "table" then
    return false
  end
  local board = init_empty_board()
  for r = 1, SIZE do
    if type(snapshot.board[r]) ~= "table" then
      return false
    end
    for c = 1, SIZE do
      board[r][c] = tonumber(snapshot.board[r][c]) or 0
    end
  end

  state.board = board
  state.score = tonumber(snapshot.score) or 0
  local elapsed = tonumber(snapshot.elapsed_sec) or 0
  state.start_frame = state.frame - math.floor(elapsed * FPS)
  state.last_auto_save_sec = elapsed
  state.game_over = false
  state.won = false
  state.confirm_mode = nil
  state.win_message_until = 0
  state.toast_text = nil
  state.toast_until = 0
  state.end_frame = nil
  state.dirty = true
  return true
end

local function save_game_state(show_toast)
  local ok = false
  local snapshot = make_snapshot()
  local ret = save_data("2048", snapshot)
  ok = ret ~= false

  if show_toast then
    local key = ok and "game.2048.save_success" or "game.2048.save_unavailable"
    state.toast_text = tr(key)
    state.toast_until = state.frame + 2 * FPS
    state.dirty = true
  end
end

local function load_game_state()
  local snapshot = load_data("2048")
  if snapshot ~= nil then
    return restore_snapshot(snapshot)
  end
  return false
end

local function reset_game()
  state.board = init_empty_board()
  state.score = 0
  state.game_over = false
  state.won = false
  state.confirm_mode = nil
  state.start_frame = state.frame
  state.last_auto_save_sec = 0
  state.toast_text = nil
  state.toast_until = 0
  state.win_message_until = 0
  state.end_frame = nil
  spawn_tile(state.board)
  spawn_tile(state.board)
  state.dirty = true
end

local function read_launch_mode()
  local mode = string.lower(tostring(get_launch_mode() or "new"))
  if mode == "continue" then
    return "continue"
  end
  return "new"
end

local function board_geometry()
  local w, h = get_terminal_size()
  local grid_w = SIZE * CELL_W
  local grid_h = SIZE * CELL_H
  local status_w = text_width(tr("game.2048.time") .. " 00:00:00")
      + 2
      + text_width(tr("game.2048.score") .. " 999999999")
  local best_w = text_width(
    tr("game.2048.best_title")
      .. "  "
      .. tr("game.2048.best_score")
      .. " "
      .. tostring(math.max(0, state.best_score))
      .. "  "
      .. tr("game.2048.best_time")
      .. " "
      .. format_duration(math.max(0, state.best_time_sec))
  )
  local frame_w = math.max(grid_w, status_w, best_w) + 2
  local frame_h = grid_h + 2
  local x = math.floor((w - frame_w) / 2)
  local y = math.floor((h - frame_h) / 2)
  if x < 1 then x = 1 end
  if y < 5 then y = 5 end
  return x, y, frame_w, frame_h
end

local function minimum_required_size()
  local frame_w = SIZE * CELL_W + 2
  local frame_h = SIZE * CELL_H + 2
  local controls_w = min_width_for_lines(tr("game.2048.controls"), 3, 24)
  local status_w = text_width(tr("game.2048.time") .. " 00:00:00")
      + 2
      + text_width(tr("game.2048.score") .. " 999999999")
  local best_w = text_width(
    tr("game.2048.best_title")
      .. "  "
      .. tr("game.2048.best_score")
      .. " 999999999  "
      .. tr("game.2048.best_time")
      .. " 00:00:00"
  )
  local win_line_w = text_width(tr("game.2048.win_banner") .. tr("game.2048.win_controls"))
  local tip_w = math.max(
    text_width(tr("game.2048.game_over")),
    text_width(tr("game.2048.confirm_restart")),
    text_width(tr("game.2048.confirm_exit")),
    win_line_w
  )
  local min_w = math.max(frame_w, controls_w, status_w, best_w, tip_w) + 2
  local min_h = frame_h + 8
  return min_w, min_h
end

local function draw_outer_frame(x, y, frame_w, frame_h)
  draw_text(x, y, BORDER_TL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_TR, "white", "black")
  for i = 1, frame_h - 2 do
    draw_text(x, y + i, BORDER_V, "white", "black")
    draw_text(x + frame_w - 1, y + i, BORDER_V, "white", "black")
  end
  draw_text(x, y + frame_h - 1, BORDER_BL .. string.rep(BORDER_H, frame_w - 2) .. BORDER_BR, "white", "black")
end

local function draw_tile(tile_x, tile_y, value)
  local bg = tile_bg_color(value)
  local fg = text_color_for_value(value)
  for row = 0, CELL_H - 1 do
    draw_text(tile_x, tile_y + row, string.rep(" ", CELL_W), fg, bg)
  end
  local text = format_cell_value(value)
  local text_x = tile_x + math.floor((CELL_W - #text) / 2)
  local text_y = tile_y + math.floor(CELL_H / 2)
  draw_text(text_x, text_y, text, fg, bg)
end

local function draw_status(x, y, frame_w)
  local elapsed = elapsed_seconds()
  local left = tr("game.2048.time") .. " " .. format_duration(elapsed)
  local right = tr("game.2048.score") .. " " .. tostring(state.score)
  local term_w = select(1, get_terminal_size())
  local right_x = x + frame_w - text_width(right)
  if right_x < 1 then
    right_x = 1
  end

  draw_text(1, y - 3, string.rep(" ", term_w), "white", "black")
  draw_text(1, y - 2, string.rep(" ", term_w), "white", "black")
  draw_text(1, y - 1, string.rep(" ", term_w), "white", "black")

  local best_line = tr("game.2048.best_title")
      .. "  "
      .. tr("game.2048.best_score")
      .. " "
      .. tostring(math.max(0, state.best_score))
      .. "  "
      .. tr("game.2048.best_time")
      .. " "
      .. format_duration(math.max(0, state.best_time_sec))
  draw_text(x, y - 3, best_line, "dark_gray", "black")
  draw_text(x, y - 2, left, "light_cyan", "black")
  draw_text(right_x, y - 2, right, "light_cyan", "black")

  if state.won then
    draw_text(x, y - 1, tr("game.2048.win_banner") .. tr("game.2048.win_controls"), "yellow", "black")
  elseif state.confirm_mode == "game_over" then
    draw_text(x, y - 1, tr("game.2048.game_over"), "red", "black")
  elseif state.confirm_mode == "restart" then
    draw_text(x, y - 1, tr("game.2048.confirm_restart"), "yellow", "black")
  elseif state.confirm_mode == "exit" then
    draw_text(x, y - 1, tr("game.2048.confirm_exit"), "yellow", "black")
  elseif state.toast_text ~= nil and state.frame <= state.toast_until then
    draw_text(x, y - 1, state.toast_text, "green", "black")
  end
end

local function draw_controls(x, y, frame_h)
  local term_w = select(1, get_terminal_size())
  local controls = tr("game.2048.controls")
  local max_w = math.max(10, term_w - 2)
  local lines = wrap_words(controls, max_w)
  if #lines > 3 then
    lines = { lines[1], lines[2], lines[3] }
  end

  draw_text(1, y + frame_h + 1, string.rep(" ", term_w), "white", "black")
  draw_text(1, y + frame_h + 2, string.rep(" ", term_w), "white", "black")
  draw_text(1, y + frame_h + 3, string.rep(" ", term_w), "white", "black")

  local offset = 0
  if #lines < 3 then
    offset = math.floor((3 - #lines) / 2)
  end
  for i = 1, #lines do
    local line = lines[i]
    local controls_x = math.floor((term_w - text_width(line)) / 2)
    if controls_x < 1 then
      controls_x = 1
    end
    draw_text(controls_x, y + frame_h + 1 + offset + i - 1, line, "white", "black")
  end
end

local function render_game()
  local x, y, frame_w, frame_h = board_geometry()
  fill_rect(x, y - 3, frame_w, frame_h + 7, "black")
  draw_status(x, y, frame_w)
  draw_outer_frame(x, y, frame_w, frame_h)

  local pad_x = math.floor((frame_w - 2 - SIZE * CELL_W) / 2)
  if pad_x < 0 then pad_x = 0 end
  local inner_x = x + 1 + pad_x
  local inner_y = y + 1
  for r = 1, SIZE do
    for c = 1, SIZE do
      local tx = inner_x + (c - 1) * CELL_W
      local ty = inner_y + (r - 1) * CELL_H
      draw_tile(tx, ty, state.board[r][c])
    end
  end

  draw_controls(x, y, frame_h)
end

local function handle_confirm_key(key)
  if key == "y" or key == "enter" then
    if state.confirm_mode == "game_over" or state.confirm_mode == "restart" then
      reset_game()
      return "changed"
    end
    if state.confirm_mode == "exit" then
      commit_stats()
      return "exit"
    end
  end

  if state.confirm_mode == "game_over" and (key == "q" or key == "esc") then
    commit_stats()
    return "exit"
  end

  if state.confirm_mode == "game_over" then
    return "none"
  end

  if key == "q" or key == "esc" then
    state.confirm_mode = nil
    state.dirty = true
    return "changed"
  end
  return "none"
end

local function reconcile_game_over_state()
  if state.confirm_mode == "game_over" and can_move_any() then
    state.game_over = false
    state.confirm_mode = nil
    state.end_frame = nil
    state.dirty = true
  end
end

local function apply_direction_key(key)
  if key == "up" or key == "down" or key == "left" or key == "right" then
    return key
  end
  return nil
end

local function is_move_key(key)
  return key == "up" or key == "down" or key == "left" or key == "right"
end

local function should_debounce(key)
  if not is_move_key(key) then
    return false
  end
  if key == state.last_key and (state.frame - state.last_key_frame) <= 2 then
    return true
  end
  state.last_key = key
  state.last_key_frame = state.frame
  return false
end

local function handle_input(key)
  if key == nil or key == "" then
    return "none"
  end
  if should_debounce(key) then
    return "none"
  end

  reconcile_game_over_state()

  if state.confirm_mode ~= nil then
    return handle_confirm_key(key)
  end

  if state.won then
    if key == "r" then
      reset_game()
      return "changed"
    end
    if key == "q" or key == "esc" then
      commit_stats()
      return "exit"
    end
    return "none"
  end

  if key == "r" then
    state.confirm_mode = "restart"
    state.dirty = true
    return "changed"
  end
  if key == "q" or key == "esc" then
    state.confirm_mode = "exit"
    state.dirty = true
    return "changed"
  end
  if key == "s" then
    save_game_state(true)
    return "changed"
  end

  if state.game_over then
    return "none"
  end

  local dir = apply_direction_key(key)
  if dir == nil then
    return "none"
  end

  local moved = apply_move(dir)
  if moved then
    spawn_tile(state.board)
    update_win_and_loss()
    state.dirty = true
    return "changed"
  end

  if not can_move_any() and not state.game_over then
    state.game_over = true
    state.confirm_mode = "game_over"
    state.end_frame = state.frame
    state.dirty = true
    commit_stats()
    return "changed"
  end

  return "none"
end

local function auto_save_if_needed()
  local elapsed = elapsed_seconds()
  if elapsed - state.last_auto_save_sec >= 60 then
    save_game_state(false)
    state.last_auto_save_sec = elapsed
  end
end

local function refresh_dirty_flags()
  local elapsed = math.floor((state.frame - state.start_frame) / FPS)
  if elapsed ~= state.last_elapsed_sec then
    state.last_elapsed_sec = elapsed
    state.dirty = true
  end
  local win_visible = state.frame <= state.win_message_until
  if win_visible ~= state.last_win_visible then
    state.last_win_visible = win_visible
    state.dirty = true
  end
  local toast_visible = state.toast_text ~= nil and state.frame <= state.toast_until
  if toast_visible ~= state.last_toast_visible then
    state.last_toast_visible = toast_visible
    state.dirty = true
  end
end

local function sync_terminal_resize()
  local w, h = get_terminal_size()
  if w ~= state.last_term_w or h ~= state.last_term_h then
    state.last_term_w = w
    state.last_term_h = h
    state.last_area = nil
    state.dirty = true
  end
end

function init_game()
  state = {
    board = init_empty_board(),
    score = 0,
    game_over = false,
    won = false,
    confirm_mode = nil,
    frame = 0,
    start_frame = 0,
    win_message_until = 0,
    last_auto_save_sec = 0,
    toast_text = nil,
    toast_until = 0,
    dirty = true,
    last_elapsed_sec = -1,
    last_win_visible = false,
    last_toast_visible = false,
    last_key = "",
    last_key_frame = -100,
    launch_mode = "new",
    last_area = nil,
    end_frame = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
    best_score = 0,
    best_time_sec = 0,
  }

  state.last_term_w, state.last_term_h = get_terminal_size()
  load_best_record()
  state.launch_mode = read_launch_mode()
  if state.launch_mode == "continue" then
    if not load_game_state() then
      reset_game()
    end
  else
    reset_game()
  end
  update_win_and_loss()
  state.dirty = true
  return state
end

function handle_event(in_state, event)
  state = in_state

  if event.type == "resize" then
    state.last_term_w = event.width or state.last_term_w
    state.last_term_h = event.height or state.last_term_h
    state.last_area = nil
    state.dirty = true
    return state
  end

  if event.type == "tick" then
    state.frame = state.frame + 1
    auto_save_if_needed()
    refresh_dirty_flags()
    sync_terminal_resize()
    return state
  end

  local key = normalize_key_name(event)
  local action = handle_input(key)
  if action == "exit" then
    request_exit()
  end
  return state
end

function render(in_state)
  state = in_state
  render_game()
end

function best_score(in_state)
  state = in_state
  if state.best_score <= 0 then
    return nil
  end
  return {
    best_string = "game.2048.best_block",
    score = state.best_score,
    time = format_duration(state.best_time_sec),
  }
end
