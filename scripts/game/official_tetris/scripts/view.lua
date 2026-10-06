-- The game sizes, timing values, colors, and piece shapes.
local config = loader.require("config.lua")
-- The game rules for speed, colors, stages, and time display.
local rules = loader.require("rules.lua")
-- The tools for reading and saving the best results.
local records = loader.require("records.lua")
-- The drawing functions for the game screen.
local view = {}

-- The English stage names used when a translation is missing.
local stage_names = {
  classic = "Classic", challenge = "Challenge", dusk = "Dusk", darkness = "Darkness",
  crash = "Crash Point", green_corridor = "Green Corridor", dawn = "Dawn", rebirth = "Rebirth"
}

-- Read a UI translation or return the supplied fallback text.
local function tr(key, fallback)
  return i18n.get_value("ui", key, { callback = fallback })
end

-- Draw the visible cells of a piece at the given board position.
local function draw_cells(cells, origin_x, origin_y, stroke_x, stroke_y, glyph, fg)
  for _, pos in ipairs(cells) do
    -- The column and row where the current content is drawn.
    local x, y = origin_x + pos[2], origin_y + pos[1]
    if y >= 1 and y <= config.height then
      draw.text(stroke_x + (x - 1) * 2 + 1, stroke_y + y, glyph, { fg = fg })
    end
  end
end

-- Draw a text block in the middle of the board.
local function centered_board_text(text, stroke_x, stroke_y)
  -- The available width measured in terminal character cells.
  local width = config.width * 2
  -- The displayed width and height of the text block.
  local w, h = measurement.get_text_size(text, { horizontal_align = align.CENTER, max_width = width })
  -- The left edge of the text or panel being drawn.
  local x = stroke_x + 1 + math.floor((width - w) / 2)
  -- The top edge of the text or panel being drawn.
  local y = stroke_y + 1 + math.floor((config.height - h) / 2)
  draw.text(x, y, text, { horizontal_align = align.CENTER, max_width = width })
end

-- Draw placed pieces, the current piece, and the end animation.
local function draw_board(session, stroke_x, stroke_y)
  for y = 1, config.height do
    for x = 1, config.width do
      -- The board cell currently being drawn.
      local cell = session.board[y][x]
      if cell.state ~= 0 then
        draw.text(stroke_x + (x - 1) * 2 + 1, stroke_y + y, "██",
          { fg = rules.color_for(session.colors, cell.type) })
      end
    end
  end
  if session.phase == "running" or session.phase == "pause" then
    if session.mode == "normal" and session.landing_preview then
      draw_cells(session.cells, session.x, session:landing_y(), stroke_x, stroke_y, "[]", color.GRAY)
    end
    draw_cells(session.cells, session.x, session.y, stroke_x, stroke_y, "██",
      rules.color_for(session.colors, session.current_kind))
  elseif session.phase == "end" then
    for y = config.height - session.end_rows + 1, config.height do
      draw.text(stroke_x + 1, stroke_y + y, string.rep("█", config.width * 2), { fg = color.WHITE })
    end
  end
end

-- Draw the cleared-line and piece counts beside the board.
local function draw_statistics(session, stroke_x)
  -- The top row of the statistics panel.
  local left_y = align.resolve_y(12, align.CENTER)
  -- The translated count labels and the width of their longest label.
  local labels, max_width = {}, 0
  for i, key in ipairs(config.line_order) do
    labels[i] = tr("left." .. key, string.upper(key))
    -- The width of the current translated count label in character cells.
    local width = measurement.get_text_width(labels[i])
    if width > max_width then max_width = width end
  end
  for i, key in ipairs(config.line_order) do
    draw.text(stroke_x - 7 - max_width, left_y + i - 1, labels[i])
    draw.text(stroke_x - 6, left_y + i - 1, string.format("%05d", session.line_counts[key]))
  end
  for i, key in ipairs(config.piece_order) do
    draw.text(stroke_x - 8, left_y + 5 + i, string.upper(key))
    draw.text(stroke_x - 6, left_y + 5 + i, string.format("%05d", session.piece_counts[key]))
  end
end

-- Draw scores, elapsed time, the next piece, and the mode details.
local function draw_right_panel(session, stroke_x)
  -- The left edge of the text or panel being drawn.
  local x = stroke_x + config.width * 2 + 3
  -- The top edge of the text or panel being drawn.
  local y = align.resolve_y(18, align.CENTER)
  draw.text(x, y, tr("right.high_score", "High Score"))
  draw.text(x, y + 1, tostring(records.score(session.best, session.mode)))
  draw.text(x, y + 3, tr("right.current_score", "Current Score"))
  draw.text(x, y + 4, tostring(session.score))
  draw.text(x, y + 6, tr("right.time", "Time"))
  draw.text(x, y + 7, rules.format_time(session.elapsed_time), { fg = color.BRIGHT_CYAN })
  draw.text(x, y + 9, tr("right.next", "Next"))
  if session.next_kind then
    for _, pos in ipairs(config.pieces[session.next_kind]) do
      draw.text(x + pos[2] * 2, y + 10 + pos[1], "██",
        { fg = rules.color_for(session.colors, session.next_kind) })
    end
  end
  if session.mode == "challenge" then
    -- The translation key for the current challenge stage.
    local stage = rules.stage_key(session.level)
    draw.text(x, y + 15, "LV " .. rules.cycle_level(session.level))
    draw.text(x, y + 16, i18n.get_value("stage", "stage." .. stage, { callback = stage_names[stage] }))
  else
    draw.text(x, y + 15, tr("right.normal1", "Have fun~"))
    draw.text(x, y + 16, tr("right.normal2", "Keep going!"))
  end
end

-- Draw the control hints for the current game phase.
local function draw_controls(session)
  -- The translation key for the current control hints.
  local key
  if session.phase == "running" then key = session.mode .. ".action"
  elseif session.phase == "stop" then key = "stop.action"
  elseif session.phase == "pause" then key = "pause.action"
  elseif session.phase == "end" then key = "end.action" end
  if not key then return end
  -- The text currently being drawn.
  local text = tr(key, "")
  -- The width limit and alignment used for both measuring and drawing.
  local options = { max_width = session.viewport_width, horizontal_align = align.CENTER }
  -- The displayed width and height of the text block.
  local w, h = measurement.get_text_size(text, options)
  -- The column and row where the current content is drawn.
  local x, y = align.resolve_rect(w, h, align.CENTER, align.BOTTOM)
  draw.text(x, y, text, options)
end

-- Draw the full game screen once translations are ready.
function view.render(session)
  if not session.i18n_ready then return end
  -- The width and height of the board frame in terminal character cells.
  local width, height = config.width * 2 + 2, config.height + 2
  -- The top-left corner of the board frame.
  local stroke_x, stroke_y = align.resolve_rect(width, height, align.CENTER, align.CENTER)
  draw.stroke_rect(stroke_x, stroke_y, width, height,
    { border_char = char.DOUBLE_LINE, fg = color.BRIGHT_BLUE })
  draw_board(session, stroke_x, stroke_y)
  draw_statistics(session, stroke_x)
  draw_right_panel(session, stroke_x)
  draw_controls(session)

  if session.phase == "stop" then
    -- The text currently being drawn.
    local text
    if session.mode_menu then
      text = tr("switch_mode.action", "f%{key:normal_mode} Normal Mode\n{key:challenge_mode} Challenge Mode")
    else
      text = tr(session.mode .. "_mode", session.mode == "normal" and "Normal Mode" or "Challenge Mode")
        .. "\n\n{key:start-resume}\n" .. tr("start", "Start")
    end
    centered_board_text(text, stroke_x, stroke_y)
  elseif session.phase == "ready" then
    -- The text currently being drawn.
    local text = session.ready_timer < 1 and tr("start.ready", "Ready") or tr("start.go", "Go!")
    centered_board_text(text, stroke_x, stroke_y)
  end
end

return view
