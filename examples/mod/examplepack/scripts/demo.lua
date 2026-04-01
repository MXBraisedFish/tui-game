GAME_META = {
  name = "example_mod.game_name",
  description = "example_mod.game_description",
  detail = "example_mod.game_detail",
  best_none = "example_mod.best_none",
  save = true,
  min_width = 60,
  min_height = 24,
}

local FIELD_WIDTH = 17
local FIELD_HEIGHT = 9

local function new_state()
  return {
    player_x = 2,
    player_y = 2,
    goal_x = FIELD_WIDTH - 1,
    goal_y = FIELD_HEIGHT - 1,
    steps = 0,
    message = "example_mod.msg_reach_goal",
    finished = false,
  }
end

local function load_state()
  if get_launch_mode() == "continue" then
    local state = load_data("state")
    if type(state) == "table" then
      state.message = "example_mod.msg_loaded"
      return state
    end
  end
  return new_state()
end

local function save_state(state)
  save_data("state", {
    player_x = state.player_x,
    player_y = state.player_y,
    goal_x = state.goal_x,
    goal_y = state.goal_y,
    steps = state.steps,
    finished = state.finished,
  })
end

local function load_best_steps()
  local best = load_data("best_steps")
  if type(best) == "number" and best > 0 then
    return math.floor(best)
  end
  return nil
end

local function update_best_score(steps)
  local current_best = load_best_steps()
  if not current_best or steps < current_best then
    save_data("best_steps", steps)
    return true
  end
  return false
end

local function draw_field(state)
  local origin_x, origin_y = example_util.field_origin(FIELD_WIDTH + 2, FIELD_HEIGHT + 2)
  for y = 0, FIELD_HEIGHT + 1 do
    local row = {}
    for x = 0, FIELD_WIDTH + 1 do
      if y == 0 or y == FIELD_HEIGHT + 1 then
        row[#row + 1] = "#"
      elseif x == 0 or x == FIELD_WIDTH + 1 then
        row[#row + 1] = "#"
      elseif x == state.player_x and y == state.player_y then
        row[#row + 1] = "@"
      elseif x == state.goal_x and y == state.goal_y then
        row[#row + 1] = "X"
      else
        row[#row + 1] = "."
      end
    end
    draw_text(origin_x, origin_y + y, table.concat(row), "white", nil)
  end
end

local function draw_ui(state)
  clear()
  example_util.draw_anchor(ANCHOR_CENTER, ANCHOR_TOP, translate("example_mod.game_name"), "cyan", nil, 0, 1)
  example_util.draw_anchor(ANCHOR_CENTER, ANCHOR_TOP, translate("example_mod.game_description"), "gray", nil, 0, 2)

  draw_field(state)

  local best_steps = load_best_steps()
  local best_text = best_steps and tostring(best_steps) or translate("example_mod.best_none")
  example_util.draw_anchor(
    ANCHOR_RIGHT,
    ANCHOR_TOP,
    translate("example_mod.label_steps") .. ": " .. tostring(state.steps),
    "yellow",
    nil,
    -2,
    1
  )
  example_util.draw_anchor(
    ANCHOR_RIGHT,
    ANCHOR_TOP,
    translate("example_mod.label_best") .. ": " .. best_text,
    "green",
    nil,
    -2,
    2
  )
  example_util.draw_anchor(
    ANCHOR_CENTER,
    ANCHOR_BOTTOM,
    translate(state.message),
    state.finished and "green" or "white",
    nil,
    0,
    -3
  )
  example_util.draw_anchor(
    ANCHOR_CENTER,
    ANCHOR_BOTTOM,
    translate("example_mod.demo_controls"),
    "dark_gray",
    nil,
    0,
    -1
  )
end

function init_game()
  register_action("move_left", { "left", "a" }, "Move Left")
  register_action("move_right", { "right", "d" }, "Move Right")
  register_action("move_up", { "up", "w" }, "Move Up")
  register_action("move_down", { "down", "s" }, "Move Down")
  register_action("save_exit", { "enter", "space" }, "Save And Exit")
  register_action("restart", { "r" }, "Restart")
  register_action("back", { "esc", "q" }, "Back")
end

function game_loop()
  local state = load_state()
  mod_log("info", "example block game started in mode: " .. get_launch_mode())

  while true do
    if consume_resize_event() then
      state.message = "example_mod.msg_resized"
    end
    draw_ui(state)
    local action = get_action_blocking()

    if action == "move_left" and not state.finished then
      state.player_x = example_util.clamp(state.player_x - 1, 1, FIELD_WIDTH)
      state.steps = state.steps + 1
      state.message = "example_mod.msg_moving"
    elseif action == "move_right" and not state.finished then
      state.player_x = example_util.clamp(state.player_x + 1, 1, FIELD_WIDTH)
      state.steps = state.steps + 1
      state.message = "example_mod.msg_moving"
    elseif action == "move_up" and not state.finished then
      state.player_y = example_util.clamp(state.player_y - 1, 1, FIELD_HEIGHT)
      state.steps = state.steps + 1
      state.message = "example_mod.msg_moving"
    elseif action == "move_down" and not state.finished then
      state.player_y = example_util.clamp(state.player_y + 1, 1, FIELD_HEIGHT)
      state.steps = state.steps + 1
      state.message = "example_mod.msg_moving"
    elseif action == "restart" then
      state = new_state()
      state.message = "example_mod.msg_restart"
      mod_log("info", "example block game restarted")
    elseif action == "save_exit" then
      save_state(state)
      mod_log("info", "example block game saved and exited")
      break
    elseif action == "back" then
      mod_log("info", "example block game exited without saving")
      break
    end

    if not state.finished and state.player_x == state.goal_x and state.player_y == state.goal_y then
      state.finished = true
      if update_best_score(state.steps) then
        state.message = "example_mod.msg_new_record"
      else
        state.message = "example_mod.msg_finished"
      end
      save_state(state)
      mod_log("info", "goal reached in " .. tostring(state.steps) .. " steps")
    end
  end
end

function best_score()
  local best_steps = load_best_steps()
  if not best_steps then
    return nil
  end
  return {
    best_string = "example_mod.best_record",
    steps = best_steps,
  }
end
