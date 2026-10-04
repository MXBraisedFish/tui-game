-- The game sizes, timing values, colors, and piece shapes.
local config = loader.require("config.lua")
-- The game rules for speed, colors, stages, and time display.
local rules = loader.require("rules.lua")
-- The tools for checking and changing board cells.
local board = loader.require("board.lua")
-- The tools for reading and saving the best results.
local records = loader.require("records.lua")
-- The methods used to create and run a game session.
local Session = {}
-- The methods shared by all game sessions.
Session.__index = Session

-- A small allowance for rounding when checking elapsed time.
local epsilon = 1e-9

-- Create a zero count for each name in the given order.
local function counters(order)
  -- The counts stored under each requested name.
  local result = {}
  for _, key in ipairs(order) do result[key] = 0 end
  return result
end

-- Choose a random piece type.
local function random_piece()
  return config.piece_order[random.randint { min = 1, max = #config.piece_order }]
end

-- Create a stopped game session with its initial state and saved results.
function Session.new(ctx)
  return setmetatable({
    -- The game canvas width used to lay out the control hints.
    viewport_width = ctx and ctx.base and ctx.base.width or 100,
    -- The game phase, selected mode, mode-menu visibility, and landing-preview setting.
    phase = "stop", mode = "normal", mode_menu = false, landing_preview = false,
    -- The translation status, board, current piece cells, and piece position.
    i18n_ready = false, board = board.new(), cells = {}, x = 5, y = 1,
    -- The score, levels, lines toward the next level or color change, and elapsed play time.
    score = 0, level = 0, highest_level = 0, lines_for_level = 0, normal_color_lines = 0, elapsed_time = 0,
    -- The counts of cleared lines and placed pieces in this round.
    line_counts = counters(config.line_order), piece_counts = counters(config.piece_order),
    -- The saved best results and the current colors of the three piece groups.
    best = records.load(ctx and ctx.best_data), colors = rules.colors(0),
    -- The falling delay, elapsed step time, and faster-falling setting.
    drop_interval = 1, drop_timer = 0, soft_drop = false,
    -- The movement direction, repeat timer, and held left and right keys.
    move_direction = 0, move_timer = 0, move_held = { [-1] = false, [1] = false },
    -- The time in seconds until the next repeated sideways move.
    move_delay = config.move_initial_delay,
    -- The countdown time, end-animation time and row count, and whether the result was saved.
    ready_timer = 0, end_timer = 0, end_rows = 0, result_committed = false
  }, Session)
end

-- Update the falling speed while keeping the current step progress.
function Session:refresh_speed()
  -- The current time in seconds between downward steps.
  local interval = rules.drop_interval(self.mode, self.level)
  if self.soft_drop then
    interval = interval / 5
    if interval < 1 / 60 then interval = 1 / 60 end
  end
  self.drop_timer = self.drop_timer / self.drop_interval * interval
  self.drop_interval = interval
end

-- Clear held movement keys and stop soft dropping.
function Session:clear_inputs()
  self.move_direction, self.move_timer = 0, 0
  self.move_delay = config.move_initial_delay
  self.move_held[-1], self.move_held[1] = false, false
  self.soft_drop = false
  self:refresh_speed()
end

-- Reset the round and prepare its first piece.
function Session:start()
  self.board = board.new()
  self.score, self.level, self.highest_level, self.lines_for_level = 0, 0, 0, 0
  self.normal_color_lines = 0
  self.line_counts = counters(config.line_order)
  self.piece_counts = counters(config.piece_order)
  self.drop_timer, self.elapsed_time, self.ready_timer = 0, 0, 0
  self.end_timer, self.end_rows = 0, 0
  -- The round start timestamp, saved-result flag, and mode-menu visibility.
  self.started_at, self.result_committed, self.mode_menu = nil, false, false
  -- The game phase to return to after a pause.
  self.paused_phase = nil
  if self.mode == "challenge" then self.landing_preview = false end
  self:clear_inputs()
  self.colors = self.mode == "normal" and rules.normal_colors() or rules.colors(0)
  self.phase = "ready"
  -- The type of piece that will appear next.
  self.next_kind = random_piece()
  self:spawn()
end

-- Pause a normal-mode round that is ready or running.
function Session:pause()
  if self.mode ~= "normal" or (self.phase ~= "running" and self.phase ~= "ready") then return end
  self.paused_phase = self.phase
  self:clear_inputs()
  self.phase = "pause"
end

-- Return a paused normal-mode round to its previous phase.
function Session:resume()
  if self.phase ~= "pause" or self.mode ~= "normal" then return end
  self.phase = self.paused_phase or "running"
  self.paused_phase = nil
end

-- Bring in the next piece and end the round if it cannot fit.
function Session:spawn()
  -- The type of piece currently falling.
  self.current_kind = self.next_kind
  self.cells = config.pieces[self.current_kind]
  self.next_kind = random_piece()
  -- The lowest row offset in the new piece shape.
  local max_row = 0
  for _, pos in ipairs(self.cells) do
    if pos[1] > max_row then max_row = pos[1] end
  end
  self.x, self.y, self.drop_timer = 5, -max_row, 0
  if not board.can_place(self.board, self.cells, self.x, self.y) then self:finish() end
end

-- End the round and save its best results once.
function Session:finish()
  if self.result_committed then return end
  self:clear_inputs()
  self.phase, self.end_timer, self.end_rows = "end", 0, 0
  self.drop_timer = 0
  -- The highest challenge level reached in this round.
  local highest = self.highest_level
  if self.level > highest then highest = self.level end
  records.commit(self.best, self.mode, self.score, highest)
  self.result_committed = true
  game.save_best()
end

-- Change normal-mode colors or advance challenge levels after cleared lines.
function Session:advance_level(cleared)
  if self.mode == "normal" then
    self.normal_color_lines = self.normal_color_lines + cleared
    while self.normal_color_lines >= 10 do
      self.normal_color_lines = self.normal_color_lines - 10
      self.colors = rules.normal_colors()
    end
    return
  end
  if self.mode ~= "challenge" then return end
  self.lines_for_level = self.lines_for_level + cleared
  while self.lines_for_level >= rules.lines_needed(self.level) do
    self.lines_for_level = self.lines_for_level - rules.lines_needed(self.level)
    self.level = self.level + 1
    self.highest_level = self.level
    self:refresh_speed()
    self.colors = rules.colors(self.level)
  end
end

-- Lock the current piece, score cleared lines, and bring in the next piece.
function Session:settle()
  -- Whether the locked piece extends above the board.
  local above_top = board.lock(self.board, self.cells, self.current_kind, self.x, self.y)
  self.piece_counts[self.current_kind] = self.piece_counts[self.current_kind] + 1
  if above_top then self:finish(); return end

  -- The number of full rows removed after this piece lands.
  local cleared = board.clear_lines(self.board)
  self.line_counts.lines = self.line_counts.lines + cleared
  if cleared > 0 then
    -- The count name for this number of cleared rows.
    local key = config.line_order[cleared]
    self.line_counts[key] = self.line_counts[key] + 1
    self:advance_level(cleared)
    self.score = self.score + config.clear_scores[cleared] * (rules.cycle_level(self.level) + 1)
  end
  self:spawn()
end

-- Move the piece sideways if the destination is clear.
function Session:move(direction)
  if board.can_place(self.board, self.cells, self.x + direction, self.y) then
    self.x = self.x + direction
  end
end

-- Track left and right key changes and move on a new press.
function Session:horizontal(action, state)
  -- The horizontal movement direction, with left as minus one and right as one.
  local direction = action == "move_left" and -1 or 1
  if state == "pressed" then
    self.move_held[direction] = true
    self.move_direction, self.move_timer = direction, 0
    self.move_delay = config.move_initial_delay
    self:move(direction)
  elseif state == "released" then
    self.move_held[direction] = false
    if self.move_direction == direction then
      self.move_direction = self.move_held[-direction] and -direction or 0
      self.move_timer = 0
      self.move_delay = config.move_initial_delay
    end
  end
end

-- Rotate the current piece if the new shape fits.
function Session:rotate(clockwise)
  if self.current_kind == "o" then return end
  -- The point around which the piece rotates.
  local center = self.current_kind == "i" and 1.5 or 1
  -- The cell positions of the rotated piece.
  local rotated = {}
  for i, pos in ipairs(self.cells) do
    -- The horizontal and vertical distances from the rotation center.
    local dx, dy = pos[2] - center, pos[1] - center
    if clockwise then
      rotated[i] = { center + dx, center - dy }
    else
      rotated[i] = { center - dx, center + dy }
    end
  end
  if board.can_place(self.board, rotated, self.x, self.y) then self.cells = rotated end
end

-- Return the row where the current piece would land.
function Session:landing_y()
  return board.landing_y(self.board, self.cells, self.x, self.y)
end

-- Drop the current piece to its landing row and lock it.
function Session:hard_drop()
  self.y = self:landing_y()
  self:settle()
end

-- Turn faster falling on or off.
function Session:set_soft_drop(enabled)
  if self.soft_drop ~= enabled then
    self.soft_drop = enabled
    self:refresh_speed()
  end
end

-- Advance the countdown, movement, falling, and end animation.
function Session:update(dt)
  if self.phase == "ready" then
    self.ready_timer = self.ready_timer + dt
    if self.ready_timer + epsilon >= config.ready_duration then
      self.phase, self.started_at = "running", date.now()
      self.drop_timer = 0
    end
    return
  end

  if self.phase == "end" then
    self.end_timer = self.end_timer + dt
    self.end_rows = math.floor((self.end_timer + epsilon) / config.end_row_interval)
    if self.end_rows > config.height then self.end_rows = config.height end
    if self.end_timer + epsilon >= config.height * config.end_row_interval + config.end_hold_time then
      self.board, self.cells = board.new(), {}
      self.current_kind, self.next_kind = nil, nil
      self.phase, self.end_rows = "stop", 0
    end
    return
  end

  if self.phase ~= "running" then return end
  self.elapsed_time = self.elapsed_time + dt
  if self.move_direction ~= 0 then
    self.move_timer = self.move_timer + dt
    while self.move_timer + epsilon >= self.move_delay do
      self.move_timer = self.move_timer - self.move_delay
      if self.move_timer < 0 then self.move_timer = 0 end
      self:move(self.move_direction)
      self.move_delay = config.move_interval
    end
  end

  self.drop_timer = self.drop_timer + dt
  -- The number of downward steps due in this update.
  local steps = math.floor((self.drop_timer + epsilon) / self.drop_interval)
  self.drop_timer = self.drop_timer - steps * self.drop_interval
  if self.drop_timer < 0 then self.drop_timer = 0 end
  for _ = 1, steps do
    if board.can_place(self.board, self.cells, self.x, self.y + 1) then
      self.y = self.y + 1
      if self.soft_drop then self.score = self.score + 1 end
    else
      self:settle()
      break
    end
  end
end

return Session
