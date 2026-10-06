-- The handler for game input and other incoming events.
local input = {}

-- Apply a game event to the current session.
function input.handle(session, event)
  -- The details carried by the incoming event.
  local data = event.data or {}
  if event.type == "resize" then
    session.viewport_width = data.width
    return
  end
  if event.type == "focus" and data.gained == false then
    session:clear_inputs()
    session:pause()
    return
  end
  if event.type == "overlay_started" then
    session:clear_inputs()
    return
  end

  if event.type == "i18n" and data.request_id == session.i18n_request_id then
    if not data.ok then debug.assert(false, { message = data.message }) end
    session.i18n_ready = true
    return
  end

  if event.type ~= "action" then return end
  -- The action name and its current key state.
  local action, state = data.action, data.state
  if action == "exit" and state == "pressed" and session.phase == "stop" then
    game.exit_game()
    return
  end
  if not session.i18n_ready then return end

  if state == "pressed" then
    if action == "start-resume" then
      if session.phase == "stop" and not session.mode_menu then session:start()
      elseif session.phase == "pause" then session:resume() end
      return
    elseif action == "restart" and session.phase == "pause" and session.mode == "normal" then
      session:start()
      return
    elseif action == "pause" and session.phase == "running" then
      session:pause()
      return
    end

    if session.phase == "stop" then
      if action == "switch_mode" then
        session.mode_menu = not session.mode_menu
      elseif session.mode_menu and action == "normal_mode" then
        session.mode, session.mode_menu = "normal", false
      elseif session.mode_menu and action == "challenge_mode" then
        session.mode, session.mode_menu = "challenge", false
      end
      return
    end
  end

  if session.phase ~= "running" then return end
  if action == "move_left" or action == "move_right" then
    session:horizontal(action, state)
  elseif action == "soft_drop" then
    if state == "pressed" or state == "held" then session:set_soft_drop(true)
    elseif state == "released" then session:set_soft_drop(false) end
  elseif state == "pressed" then
    if action == "rotate_clockwise" then session:rotate(true)
    elseif action == "rotate_counterclockwise" then session:rotate(false)
    elseif action == "hard_drop" then session:hard_drop()
    elseif action == "landing_preview" and session.mode == "normal" then
      session.landing_preview = not session.landing_preview
    end
  end
end

return input
