local BOARD_W = 10
local BOARD_H = 20
local PIECES = {
  I = {
    { { 0, 1 }, { 1, 1 }, { 2, 1 }, { 3, 1 } },
    { { 2, 0 }, { 2, 1 }, { 2, 2 }, { 2, 3 } },
  },
  O = {
    { { 1, 0 }, { 2, 0 }, { 1, 1 }, { 2, 1 } },
    { { 1, 0 }, { 2, 0 }, { 1, 1 }, { 2, 1 } },
  },
  T = {
    { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 2, 1 } },
    { { 1, 0 }, { 1, 1 }, { 2, 1 }, { 1, 2 } },
    { { 0, 1 }, { 1, 1 }, { 2, 1 }, { 1, 2 } },
    { { 1, 0 }, { 0, 1 }, { 1, 1 }, { 1, 2 } },
  },
  L = {
    { { 0, 0 }, { 0, 1 }, { 1, 1 }, { 2, 1 } },
    { { 1, 0 }, { 2, 0 }, { 1, 1 }, { 1, 2 } },
    { { 0, 1 }, { 1, 1 }, { 2, 1 }, { 2, 2 } },
    { { 1, 0 }, { 1, 1 }, { 0, 2 }, { 1, 2 } },
  },
  S = {
    { { 1, 0 }, { 2, 0 }, { 0, 1 }, { 1, 1 } },
    { { 1, 0 }, { 1, 1 }, { 2, 1 }, { 2, 2 } },
  },
}
local ORDER = { "I", "O", "T", "L", "S" }
local COLORS = { I = "cyan", O = "yellow", T = "magenta", L = "light_red", S = "green" }

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function new_board()
  local board = {}
  for r = 1, BOARD_H do
    board[r] = {}
    for c = 1, BOARD_W do
      board[r][c] = ""
    end
  end
  return board
end

local function clone_board(board)
  local out = new_board()
  for r = 1, BOARD_H do
    for c = 1, BOARD_W do
      out[r][c] = board[r][c]
    end
  end
  return out
end

local function random_piece()
  return ORDER[math.random(#ORDER)]
end

local function current_shape(piece)
  local shapes = PIECES[piece.kind]
  local rot = piece.rot
  if rot > #shapes then rot = 1 end
  return shapes[rot]
end

local function can_place(state, piece, dx, dy, rot)
  local temp = { kind = piece.kind, x = piece.x + (dx or 0), y = piece.y + (dy or 0), rot = rot or piece.rot }
  local shape = current_shape(temp)
  for i = 1, #shape do
    local x = temp.x + shape[i][1]
    local y = temp.y + shape[i][2]
    if x < 1 or x > BOARD_W or y > BOARD_H then
      return false
    end
    if y >= 1 and state.board[y][x] ~= "" then
      return false
    end
  end
  return true
end

local function spawn_piece(state)
  state.active = { kind = state.next_kind or random_piece(), x = 4, y = 0, rot = 1 }
  state.next_kind = random_piece()
  if not can_place(state, state.active, 0, 0, state.active.rot) then
    state.game_over = true
    state.message = "game.tetris.lose_banner"
  end
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_score = math.max(0, math.floor(tonumber(best.score) or 0))
  end
end

local function save_best_record(state)
  save_data("best_record", { score = state.best_score })
  request_refresh_best_score()
end

local function save_progress(state)
  save_data("state", {
    board = clone_board(state.board),
    active = state.active,
    next_kind = state.next_kind,
    score = state.score,
    lines = state.lines,
    level = state.level,
    elapsed_ms = state.elapsed_ms,
    drop_accum = state.drop_accum,
    game_over = state.game_over,
    message = state.message,
  })
end

local function fresh_state()
  local state = {
    board = new_board(),
    active = nil,
    next_kind = random_piece(),
    score = 0,
    lines = 0,
    level = 0,
    elapsed_ms = 0,
    drop_accum = 0,
    game_over = false,
    message = "game.tetris.runtime_ready",
    best_score = 0,
  }
  load_best_record(state)
  spawn_piece(state)
  return state
end

local function restore_state()
  local state = nil
  if get_launch_mode() == "continue" then
    state = load_data("state")
  end
  if type(state) ~= "table" then
    return fresh_state()
  end
  state.board = type(state.board) == "table" and state.board or new_board()
  state.active = type(state.active) == "table" and state.active or nil
  state.next_kind = state.next_kind or random_piece()
  state.score = math.max(0, math.floor(tonumber(state.score) or 0))
  state.lines = math.max(0, math.floor(tonumber(state.lines) or 0))
  state.level = math.max(0, math.floor(tonumber(state.level) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.drop_accum = math.max(0, math.floor(tonumber(state.drop_accum) or 0))
  state.game_over = state.game_over == true
  state.message = state.message or "game.tetris.runtime_ready"
  state.best_score = 0
  load_best_record(state)
  if state.active == nil and not state.game_over then
    spawn_piece(state)
  end
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function lock_piece(state)
  local shape = current_shape(state.active)
  for i = 1, #shape do
    local x = state.active.x + shape[i][1]
    local y = state.active.y + shape[i][2]
    if y >= 1 and y <= BOARD_H and x >= 1 and x <= BOARD_W then
      state.board[y][x] = state.active.kind
    end
  end
  local cleared = 0
  for r = BOARD_H, 1, -1 do
    local full = true
    for c = 1, BOARD_W do
      if state.board[r][c] == "" then
        full = false
        break
      end
    end
    if full then
      cleared = cleared + 1
      for rr = r, 2, -1 do
        state.board[rr] = state.board[rr - 1]
      end
      state.board[1] = {}
      for c = 1, BOARD_W do state.board[1][c] = "" end
      r = r + 1
    end
  end
  if cleared > 0 then
    state.lines = state.lines + cleared
    state.level = math.floor(state.lines / 10)
    state.score = state.score + cleared * cleared * 100
    if state.score > state.best_score then
      state.best_score = state.score
      save_best_record(state)
    end
  end
  spawn_piece(state)
  save_progress(state)
end

local function try_move(state, dx, dy)
  if not state.game_over and can_place(state, state.active, dx, dy, state.active.rot) then
    state.active.x = state.active.x + dx
    state.active.y = state.active.y + dy
    return true
  end
  return false
end

function handle_event(state, event)
  if event.type == "tick" then
    if not state.game_over then
      state.elapsed_ms = state.elapsed_ms + (event.dt_ms or 16)
      state.drop_accum = state.drop_accum + (event.dt_ms or 16)
      local interval = math.max(100, 700 - state.level * 50)
      while state.drop_accum >= interval do
        state.drop_accum = state.drop_accum - interval
        if not try_move(state, 0, 1) then
          lock_piece(state)
          break
        end
      end
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.tetris.runtime_resized"
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
    return fresh_state()
  elseif event.name == "save" then
    save_progress(state)
    state.message = "game.tetris.save_success"
  elseif not state.game_over and event.name == "move_left" then
    try_move(state, -1, 0)
  elseif not state.game_over and event.name == "move_right" then
    try_move(state, 1, 0)
  elseif not state.game_over and event.name == "soft_drop" then
    if not try_move(state, 0, 1) then
      lock_piece(state)
    end
  elseif not state.game_over and (event.name == "rotate_left" or event.name == "rotate_right") then
    local shapes = PIECES[state.active.kind]
    local target = state.active.rot + (event.name == "rotate_left" and -1 or 1)
    if target < 1 then target = #shapes end
    if target > #shapes then target = 1 end
    if can_place(state, state.active, 0, 0, target) then
      state.active.rot = target
    end
  elseif not state.game_over and event.name == "hard_drop" then
    while try_move(state, 0, 1) do end
    lock_piece(state)
  end
  return state
end

local function format_duration(sec)
  local h = math.floor(sec / 3600)
  local m = math.floor((sec % 3600) / 60)
  local s = sec % 60
  return string.format("%02d:%02d:%02d", h, m, s)
end

function render(state)
  canvas_clear()
  local _, height = get_terminal_size()
  local score_line = tr("game.tetris.current_score") .. " " .. tostring(state.score)
  local best_line = tr("game.tetris.best_score") .. " " .. tostring(state.best_score)
  local info_line = tr("game.tetris.time") .. " " .. format_duration(math.floor(state.elapsed_ms / 1000))
    .. "  LINES " .. tostring(state.lines) .. "  LV " .. tostring(state.level)
  canvas_draw_text(centered_x(score_line), 1, score_line, "yellow", nil)
  canvas_draw_text(centered_x(best_line), 2, best_line, "white", nil)
  canvas_draw_text(centered_x(info_line), 3, info_line, "light_cyan", nil)

  local board_x = resolve_x(ANCHOR_CENTER, BOARD_W * 2 + 2, 0)
  local board_y = 5
  for r = 0, BOARD_H + 1 do
    for c = 0, BOARD_W + 1 do
      local text = "  "
      local fg = "white"
      if r == 0 or r == BOARD_H + 1 or c == 0 or c == BOARD_W + 1 then
        text = "[]"
      elseif state.board[r] and state.board[r][c] and state.board[r][c] ~= "" then
        local kind = state.board[r][c]
        text = "[]"
        fg = COLORS[kind] or "white"
      end
      canvas_draw_text(board_x + c * 2, board_y + r, text, fg, nil)
    end
  end
  if not state.game_over and state.active then
    local shape = current_shape(state.active)
    for i = 1, #shape do
      local x = state.active.x + shape[i][1]
      local y = state.active.y + shape[i][2]
      if y >= 1 then
        canvas_draw_text(board_x + x * 2, board_y + y, "[]", COLORS[state.active.kind] or "white", nil)
      end
    end
  end

  local next_line = tr("game.tetris.next") .. ": " .. tostring(state.next_kind or "")
  canvas_draw_text(centered_x(next_line), board_y + BOARD_H + 3, next_line, "white", nil)
  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.max(board_y + BOARD_H + 5, height - 3), message, state.game_over and "red" or "white", nil)
  canvas_draw_text(centered_x(tr("game.tetris.controls")), height - 1, tr("game.tetris.controls"), "dark_gray", nil)
end

function best_score(state)
  if state.best_score <= 0 then
    return nil
  end
  return {
    best_string = "game.tetris.best_block",
    score = state.best_score,
  }
end
