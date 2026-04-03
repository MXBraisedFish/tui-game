local WIDTH = 30
local HEIGHT = 16

local function tr(key)
  return translate(key)
end

local function centered_x(text)
  return resolve_x(ANCHOR_CENTER, select(1, measure_text(text)), 0)
end

local function clone_entities(list)
  local out = {}
  for i = 1, #list do
    local item = {}
    for k, v in pairs(list[i]) do
      item[k] = v
    end
    out[i] = item
  end
  return out
end

local function load_best_record(state)
  local best = load_data("best_record")
  if type(best) == "table" then
    state.best_score = math.max(0, math.floor(tonumber(best.score) or 0))
    state.best_stage = math.max(1, math.floor(tonumber(best.stage) or 1))
  end
end

local function save_best_record(state)
  save_data("best_record", {
    score = state.best_score,
    stage = state.best_stage,
  })
  request_refresh_best_score()
end

local function save_progress(state)
  save_data("state", {
    player_x = state.player_x,
    hp = state.hp,
    score = state.score,
    stage = state.stage,
    fire_mode = state.fire_mode,
    nuke_stock = state.nuke_stock,
    elapsed_ms = state.elapsed_ms,
    bullets = clone_entities(state.bullets),
    enemies = clone_entities(state.enemies),
    enemy_tick = state.enemy_tick,
    fire_tick = state.fire_tick,
    message = state.message,
    finished = state.finished,
  })
end

local function fresh_state()
  local state = {
    player_x = math.floor(WIDTH / 2),
    hp = 10,
    score = 0,
    stage = 1,
    fire_mode = "auto",
    nuke_stock = 1,
    elapsed_ms = 0,
    bullets = {},
    enemies = {},
    enemy_tick = 0,
    fire_tick = 0,
    message = "game.shooter.runtime_ready",
    finished = false,
    best_score = 0,
    best_stage = 1,
  }
  load_best_record(state)
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
  state.player_x = math.max(1, math.min(WIDTH, math.floor(tonumber(state.player_x) or math.floor(WIDTH / 2))))
  state.hp = math.max(0, math.floor(tonumber(state.hp) or 10))
  state.score = math.max(0, math.floor(tonumber(state.score) or 0))
  state.stage = math.max(1, math.floor(tonumber(state.stage) or 1))
  state.fire_mode = state.fire_mode == "manual" and "manual" or "auto"
  state.nuke_stock = math.max(0, math.floor(tonumber(state.nuke_stock) or 0))
  state.elapsed_ms = math.max(0, math.floor(tonumber(state.elapsed_ms) or 0))
  state.bullets = type(state.bullets) == "table" and state.bullets or {}
  state.enemies = type(state.enemies) == "table" and state.enemies or {}
  state.enemy_tick = math.max(0, math.floor(tonumber(state.enemy_tick) or 0))
  state.fire_tick = math.max(0, math.floor(tonumber(state.fire_tick) or 0))
  state.message = state.message or "game.shooter.runtime_ready"
  state.finished = state.finished == true
  state.best_score = 0
  state.best_stage = 1
  load_best_record(state)
  return state
end

function init_game()
  math.randomseed(os.time())
  return restore_state()
end

local function update_best_record(state)
  local improved = false
  if state.score > state.best_score then
    state.best_score = state.score
    improved = true
  end
  if state.stage > state.best_stage then
    state.best_stage = state.stage
    improved = true
  end
  if improved then
    save_best_record(state)
  end
end

local function spawn_enemy(state)
  state.enemies[#state.enemies + 1] = { x = math.random(WIDTH), y = 1 }
end

local function fire_bullet(state)
  state.bullets[#state.bullets + 1] = { x = state.player_x, y = HEIGHT - 1 }
end

local function use_nuke(state)
  if state.nuke_stock <= 0 then
    state.message = "game.shooter.msg_nuke_empty"
    return
  end
  state.nuke_stock = state.nuke_stock - 1
  state.score = state.score + #state.enemies * 5
  state.enemies = {}
  update_best_record(state)
end

function handle_event(state, event)
  if event.type == "tick" then
    if state.finished then
      return state
    end
    local dt = event.dt_ms or 16
    state.elapsed_ms = state.elapsed_ms + dt
    state.enemy_tick = state.enemy_tick + dt
    state.fire_tick = state.fire_tick + dt
    if state.fire_mode == "auto" and state.fire_tick >= 220 then
      state.fire_tick = 0
      fire_bullet(state)
    end
    if state.enemy_tick >= math.max(180, 700 - state.stage * 30) then
      state.enemy_tick = 0
      spawn_enemy(state)
    end
    for i = #state.bullets, 1, -1 do
      state.bullets[i].y = state.bullets[i].y - 1
      if state.bullets[i].y < 1 then
        table.remove(state.bullets, i)
      end
    end
    for i = #state.enemies, 1, -1 do
      state.enemies[i].y = state.enemies[i].y + 1
      if state.enemies[i].y >= HEIGHT then
        state.hp = state.hp - 1
        table.remove(state.enemies, i)
      end
    end
    for bi = #state.bullets, 1, -1 do
      local bullet = state.bullets[bi]
      local hit = false
      for ei = #state.enemies, 1, -1 do
        local enemy = state.enemies[ei]
        if enemy.x == bullet.x and enemy.y == bullet.y then
          table.remove(state.enemies, ei)
          table.remove(state.bullets, bi)
          state.score = state.score + 2
          state.stage = math.max(1, math.floor(state.score / 20) + 1)
          update_best_record(state)
          hit = true
          break
        end
      end
      if hit then
        break
      end
    end
    if state.hp <= 0 then
      state.finished = true
      state.message = "game.shooter.lose_banner"
      save_progress(state)
    end
    return state
  end
  if event.type == "resize" then
    state.message = "game.shooter.runtime_resized"
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
    state.message = "game.shooter.save_success"
  elseif event.name == "move_left" and not state.finished then
    state.player_x = math.max(1, state.player_x - 1)
  elseif event.name == "move_right" and not state.finished then
    state.player_x = math.min(WIDTH, state.player_x + 1)
  elseif event.name == "fire" and not state.finished and state.fire_mode == "manual" then
    fire_bullet(state)
  elseif event.name == "toggle_fire_mode" and not state.finished then
    state.fire_mode = state.fire_mode == "auto" and "manual" or "auto"
    state.message = state.fire_mode == "auto" and "game.shooter.msg_fire_mode_auto" or "game.shooter.msg_fire_mode_manual"
  elseif event.name == "use_nuke" and not state.finished then
    use_nuke(state)
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
  local score_line = tr("game.shooter.score") .. ": " .. tostring(state.score)
  local stage_line = tr("game.shooter.stage") .. ": " .. tostring(state.stage)
  local hp_line = tr("game.shooter.hp") .. ": " .. tostring(state.hp)
  local time_line = tr("game.shooter.time") .. ": " .. format_duration(math.floor(state.elapsed_ms / 1000))
  local mode_key = state.fire_mode == "auto" and "game.shooter.fire_mode_auto" or "game.shooter.fire_mode_manual"
  local mode_line = tr("game.shooter.fire_mode") .. ": " .. tr(mode_key)
  canvas_draw_text(centered_x(score_line), 1, score_line, "yellow", nil)
  canvas_draw_text(centered_x(stage_line), 2, stage_line, "white", nil)
  canvas_draw_text(centered_x(hp_line), 3, hp_line, "light_red", nil)
  canvas_draw_text(centered_x(time_line), 4, time_line, "dark_gray", nil)
  canvas_draw_text(centered_x(mode_line), 5, mode_line, "light_cyan", nil)

  local field_x = resolve_x(ANCHOR_CENTER, WIDTH + 2, 0)
  local field_y = 7
  for x = 0, WIDTH + 1 do
    canvas_draw_text(field_x + x, field_y, "-", "white", nil)
    canvas_draw_text(field_x + x, field_y + HEIGHT + 1, "-", "white", nil)
  end
  for y = 1, HEIGHT do
    canvas_draw_text(field_x, field_y + y, "|", "white", nil)
    canvas_draw_text(field_x + WIDTH + 1, field_y + y, "|", "white", nil)
  end
  for i = 1, #state.enemies do
    canvas_draw_text(field_x + state.enemies[i].x, field_y + state.enemies[i].y, "V", "light_red", nil)
  end
  for i = 1, #state.bullets do
    canvas_draw_text(field_x + state.bullets[i].x, field_y + state.bullets[i].y, "|", "green", nil)
  end
  canvas_draw_text(field_x + state.player_x, field_y + HEIGHT, "A", "yellow", nil)

  local message = tr(state.message)
  canvas_draw_text(centered_x(message), math.max(field_y + HEIGHT + 3, height - 3), message, state.finished and "red" or "white", nil)
  canvas_draw_text(centered_x(tr("game.shooter.controls")), height - 1, tr("game.shooter.controls"), "dark_gray", nil)
end

function best_score(state)
  if state.best_score <= 0 then
    return nil
  end
  return {
    best_string = "game.shooter.best_block",
    score = state.best_score,
    stage = state.best_stage,
  }
end
