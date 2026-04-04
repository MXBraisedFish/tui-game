local FPS                          = 60
local FRAME_MS                     = 16
local BOARD_W, BOARD_H             = 32, 18
local INNER_W, INNER_H             = 30, 16
local PLAYER_ROW                   = INNER_H
local PLAYER_MIN_C, PLAYER_MAX_C   = 2, 29
local ENEMY_COL_MIN, ENEMY_COL_MAX = 2, 29
local BASE_BOSS_SCORE              = 100
local BOSS_HP_BAR_W                = 20

local CH_DBL_TL                    = utf8.char(9556)
local CH_DBL_TR                    = utf8.char(9559)
local CH_DBL_BL                    = utf8.char(9562)
local CH_DBL_BR                    = utf8.char(9565)
local CH_DBL_H                     = utf8.char(9552)
local CH_DBL_V                     = utf8.char(9553)
local CH_BLOCK                     = utf8.char(9608)

local ENEMY_TYPES                  = {
    normal = {
        glyph = "V",
        color = "rgb(255,170,170)",
        score = 2,
        hp = 2,
        speed = 2,
        collide = 2,
        shooter = true,
        bullet = "v",
        base_dmg = 1,
        tracking = false,
        shot_active = 3.0,
        shot_idle = 6.0
    },
    fast = {
        glyph = "Y",
        color = "rgb(255,170,170)",
        score = 1,
        hp = 1,
        speed = 4,
        collide = 3,
        shooter = false
    },
    tank = {
        glyph = "W",
        color = "rgb(255,170,170)",
        score = 3,
        hp = 6,
        speed = 1,
        collide = 4,
        shooter = true,
        bullet = "v",
        base_dmg = 1,
        tracking = false,
        shot_active = 6.0,
        shot_idle = 8.0
    },
    heavy = {
        glyph = "U",
        color = "rgb(255,170,170)",
        score = 4,
        hp = 2,
        speed = 2,
        collide = 2,
        shooter = true,
        bullet = "u",
        base_dmg = 4,
        tracking = true,
        shot_active = 8.0,
        shot_idle = 12.0
    },
}

local ATTACK_BUFF                  = {
    ["@"] = { mode = "rapid", dur = 4 },
    ["%"] = { mode = "laser", dur = 10 },
    ["$"] = { mode = "double", dur = 10 },
    ["#"] = { mode = "single", dur = 10 },
    ["&"] = { mode = "missile", dur = 12 },
}

local FUNC_BUFF                    = {
    ["*"] = { mode = "shield", dur = 4 },
    ["~"] = { mode = "heal", dur = 0 },
    ["o"] = { mode = "coin", dur = 0 },
    ["c"] = { mode = "magnet", dur = 20 },
    ["+"] = { mode = "bullet_speed", dur = 20 },
    ["G"] = { mode = "nuke", dur = 0 },
}

local state                        = {

    frame = 0,
    dirty = true,
    launch_mode = "new",

    player_c = math.floor((PLAYER_MIN_C + PLAYER_MAX_C) / 2),
    player_last_dir = 1,
    hp = 10,
    score = 0,
    stage = 1,
    next_boss_score = BASE_BOSS_SCORE,

    run_start_frame = 0,
    end_frame = nil,
    phase = "playing",
    confirm_mode = nil,

    enemies = {},
    enemy_bullets = {},
    player_bullets = {},
    items = {},

    boss = {
        active = false,
        row = 1,
        center_c = 15,
        hp = 0,
        max_hp = 1,
        mode = "attack",
        mode_until = 0,
        next_move_at = 0,
        next_shot_at = 0,
        start_frame = 0,
        chase_cd_until = 0,
    },

    attack_symbol = nil,
    attack_until = 0,
    missile_shots = 0,
    missile_last = 0,
    last_player_fire = 0,

    buff_until = { ["*"] = 0, ["c"] = 0, ["+"] = 0 },
    buff_order = {},

    hurt_invuln_until = 0,

    nuke_stock = 0,

    fire_mode = "auto",

    enemy_spawn_block_until = 0,
    boom_until = 0,

    next_enemy_spawn_at = 0,
    next_item_spawn_at = 0,

    msg_text = "",
    msg_color = "dark_gray",
    msg_until = 0,
    msg_persistent = false,

    best_score = 0,
    best_stage = 1,
    result_committed = false,

    last_elapsed = -1,

    last_area = nil,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,
}

local function tr(key)
    if type(translate) ~= "function" then
        return key
    end

    local ok, value = pcall(translate, key)
    if not ok or value == nil or value == "" then
        return key
    end

    if type(value) == "string" and string.find(value, "[missing-i18n-key:", 1, true) ~= nil then
        return key
    end

    return value
end

local function text_width(t)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, t)
        if ok and type(w) == "number" then return w end
    end
    return #t
end

local function clamp(v, lo, hi)
    if v < lo then return lo end
    if v > hi then return hi end
    return v
end

local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

local function rand_range(lo, hi)
    if hi <= lo then return lo end
    return lo + rand_int(hi - lo + 1)
end

local function sec_to_frames(sec)
    return math.max(1, math.floor(sec * FPS + 0.5))
end

local function speed_to_interval(cps)
    if cps <= 0 then return sec_to_frames(999) end
    return math.max(1, math.floor(FPS / cps + 0.5))
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then
            w, h = tw, th
        end
    end
    return w, h
end

local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    if type(request_exit) == "function" then
        pcall(request_exit)
    end
end

local function normalize_key(key)
    if key == nil then return "" end
    if type(key) == "string" then return string.lower(key) end
    if type(key) == "table" then
        if key.type == "quit" then return "esc" end
        if key.type == "key" and type(key.name) == "string" then
            return string.lower(key.name)
        end
        if key.type == "action" and type(key.name) == "string" then
            local map = {
                move_left = "left",
                move_right = "right",
                fire = "space",
                toggle_fire_mode = "z",
                use_nuke = "x",
                restart = "r",
                save = "s",
                quit_action = "q",
                confirm_yes = "enter",
                confirm_no = "n",
            }
            return map[key.name] or ""
        end
    end
    return tostring(key):lower()
end

local function read_launch_mode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, mode = pcall(get_launch_mode)
    if not ok or type(mode) ~= "string" then return "new" end
    mode = string.lower(mode)
    return mode == "continue" and "continue" or "new"
end

local function elapsed_seconds()
    local ending = state.end_frame or state.frame
    return math.max(0, math.floor((ending - state.run_start_frame) / FPS))
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function show_message(text, color, dur_sec, persistent)
    state.msg_text = text or ""
    state.msg_color = color or "dark_gray"
    state.msg_persistent = persistent == true
    if dur_sec ~= nil and dur_sec > 0 then
        state.msg_until = state.frame + sec_to_frames(dur_sec)
    else
        state.msg_until = 0
    end
    state.dirty = true
end

local function clear_message()
    if state.msg_text ~= "" then
        state.msg_text = ""
        state.msg_color = "dark_gray"
        state.msg_until = 0
        state.msg_persistent = false
        state.dirty = true
    end
end

local function update_message_timer()
    if state.msg_persistent then return end
    if state.msg_until > 0 and state.frame >= state.msg_until then
        clear_message()
    end
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local line = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, line, "white", bg or "black")
    end
end

local function centered_x(text, x, w)
    local px = x + math.floor((w - text_width(text)) / 2)
    if px < x then px = x end
    return px
end

local function wrap_words(text, max_width)
    if max_width <= 1 then return { text } end
    local lines, cur, had = {}, "", false
    for token in string.gmatch(text, "%S+") do
        had = true
        if cur == "" then
            cur = token
        else
            local cand = cur .. " " .. token
            if text_width(cand) <= max_width then
                cur = cand
            else
                lines[#lines + 1] = cur
                cur = token
            end
        end
    end
    if not had then return { "" } end
    if cur ~= "" then lines[#lines + 1] = cur end
    return lines
end

local function min_width_for_lines(text, max_lines, hard_min)
    local full = text_width(text)
    local w = hard_min
    while w <= full do
        if #wrap_words(text, w) <= max_lines then return w end
        w = w + 1
    end
    return full
end

local function choose_weighted(entries)
    local total = 0
    for i = 1, #entries do
        total = total + entries[i].w
    end
    if total <= 0 then return entries[1] end
    local pick = rand_int(total) + 1
    local acc = 0
    for i = 1, #entries do
        acc = acc + entries[i].w
        if pick <= acc then return entries[i] end
    end
    return entries[#entries]
end

local function stage_level()
    return math.max(0, state.stage - 1)
end

local function scale_player_dmg(base)
    local s = stage_level()
    local mult = math.max(1, s / 2)
    return math.max(1, math.floor(base * mult))
end

local function scale_enemy_bullet_dmg(base)
    local s = stage_level()
    local mult = math.max(1, s / 2)
    return clamp(math.floor(base * mult), 1, 10)
end

local function scale_enemy_hp(base)
    local s = stage_level()
    if s <= 0 then return base end
    return base + math.floor(math.max(1, (s ^ 1.5) / 3))
end

local function scale_enemy_collide(base)
    local s = stage_level()
    if s <= 0 then return clamp(base, 1, 10) end
    return clamp(base + math.floor(math.max(1, s / 4)), 1, 10)
end

local function scale_boss_hp(base)
    local s = stage_level()
    return math.max(1, math.floor(base * math.max(1, s / 2)))
end

local function has_magnet()
    return state.buff_until["c"] > state.frame
end

local function has_shield()
    return state.buff_until["*"] > state.frame
end

local function has_bullet_speed()
    return state.buff_until["+"] > state.frame
end

local function player_invuln_active()
    local inv = state.hurt_invuln_until
    if state.buff_until["*"] > inv then
        inv = state.buff_until["*"]
    end
    return state.frame < inv
end

local function player_color()
    if player_invuln_active() then
        if (state.frame // 6) % 2 == 0 then
            return "yellow"
        end
        return "light_cyan"
    end
    return "yellow"
end

local function clear_world_entities()
    state.enemies = {}
    state.enemy_bullets = {}
    state.player_bullets = {}
    state.items = {}
end

local function remove_buff_order(symbol)
    local out = {}
    for i = 1, #state.buff_order do
        if state.buff_order[i] ~= symbol then
            out[#out + 1] = state.buff_order[i]
        end
    end
    state.buff_order = out
end

local function touch_buff_order(symbol)
    remove_buff_order(symbol)
    state.buff_order[#state.buff_order + 1] = symbol
end

local function clear_buffs()
    state.attack_symbol = nil
    state.attack_until = 0
    state.missile_shots = 0
    state.missile_last = 0
    state.buff_until["*"] = 0
    state.buff_until["c"] = 0
    state.buff_until["+"] = 0
    state.buff_order = {}
end

local function load_best_records()
    if type(load_data) ~= "function" then return end
    local ok, data = pcall(load_data, "shooter_best")
    if not ok or type(data) ~= "table" then return end
    local bs = tonumber(data.best_score)
    local bt = tonumber(data.best_stage)
    if bs ~= nil and bs >= 0 then state.best_score = math.floor(bs) end
    if bt ~= nil and bt >= 1 then state.best_stage = math.floor(bt) end
end

local function save_best_records()
    if type(save_data) ~= "function" then return end
    pcall(save_data, "shooter_best", { best_score = state.best_score, best_stage = state.best_stage })
    if type(request_refresh_best_score) == "function" then
        pcall(request_refresh_best_score)
    end
end

local function update_best_records()
    local changed = false
    if state.score > state.best_score then
        state.best_score = state.score
        changed = true
    end
    if state.stage > state.best_stage then
        state.best_stage = state.stage
        changed = true
    end
    if changed then
        save_best_records()
    end
end

local function commit_result_once()
    if state.result_committed then return end
    update_best_records()
    if type(update_game_stats) == "function" then
        pcall(update_game_stats, "shooter", state.score, elapsed_seconds())
    end
    state.result_committed = true
end

local function make_snapshot()
    local function cp(src)
        local out = {}
        for i = 1, #src do
            local t = {}
            for k, v in pairs(src[i]) do t[k] = v end
            out[#out + 1] = t
        end
        return out
    end

    return {
        frame = state.frame,
        score = state.score,
        stage = state.stage,
        hp = state.hp,
        next_boss_score = state.next_boss_score,
        elapsed_sec = elapsed_seconds(),
        player_c = state.player_c,
        player_last_dir = state.player_last_dir,

        attack_symbol = state.attack_symbol,
        attack_left = math.max(0, math.ceil((state.attack_until - state.frame) / FPS)),
        missile_shots = state.missile_shots,
        missile_cd_left = math.max(0, math.ceil((state.missile_last + sec_to_frames(2) - state.frame) / FPS)),

        shield_left = math.max(0, math.ceil((state.buff_until["*"] - state.frame) / FPS)),
        magnet_left = math.max(0, math.ceil((state.buff_until["c"] - state.frame) / FPS)),
        bullet_speed_left = math.max(0, math.ceil((state.buff_until["+"] - state.frame) / FPS)),
        hurt_invuln_left = math.max(0, math.ceil((state.hurt_invuln_until - state.frame) / FPS)),
        nuke_stock = state.nuke_stock,
        fire_mode = state.fire_mode,
        enemy_spawn_block_left = math.max(0, math.ceil((state.enemy_spawn_block_until - state.frame) / FPS)),
        boom_left = math.max(0, math.ceil((state.boom_until - state.frame) / FPS)),

        enemies = cp(state.enemies),
        enemy_bullets = cp(state.enemy_bullets),
        player_bullets = cp(state.player_bullets),
        items = cp(state.items),

        boss = {
            active = state.boss.active,
            row = state.boss.row,
            center_c = state.boss.center_c,
            hp = state.boss.hp,
            max_hp = state.boss.max_hp,
            mode = state.boss.mode,
            mode_left = math.max(0, math.ceil((state.boss.mode_until - state.frame) / FPS)),
            move_left = math.max(0, math.ceil((state.boss.next_move_at - state.frame) / FPS)),
            shot_left = math.max(0, math.ceil((state.boss.next_shot_at - state.frame) / FPS)),
            active_sec = math.max(0, math.floor((state.frame - state.boss.start_frame) / FPS)),
            chase_cd_left = math.max(0, math.ceil((state.boss.chase_cd_until - state.frame) / FPS)),
        },

        enemy_spawn_left = math.max(0, math.ceil((state.next_enemy_spawn_at - state.frame) / FPS)),
        item_spawn_left = math.max(0, math.ceil((state.next_item_spawn_at - state.frame) / FPS)),
    }
end

local function save_game_state(show_toast)
    local ok = false
    local snap = make_snapshot()

    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "shooter", snap)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "shooter", snap)
        ok = s and ret ~= false
    end

    if show_toast then
        if ok then
            show_message(tr("game.shooter.save_success"), "green", 3, false)
        else
            show_message(tr("game.shooter.save_failed"), "red", 3, false)
        end
    end
end

local function restore_snapshot(snap)
    if type(snap) ~= "table" then return false end

    state.score = math.max(0, math.floor(tonumber(snap.score) or 0))
    state.stage = math.max(1, math.floor(tonumber(snap.stage) or 1))
    state.hp = clamp(math.floor(tonumber(snap.hp) or 10), 1, 10)
    state.next_boss_score = math.max(BASE_BOSS_SCORE, math.floor(tonumber(snap.next_boss_score) or BASE_BOSS_SCORE))

    local elapsed = math.max(0, math.floor(tonumber(snap.elapsed_sec) or 0))
    state.run_start_frame = state.frame - elapsed * FPS

    state.player_c = clamp(math.floor(tonumber(snap.player_c) or state.player_c), PLAYER_MIN_C, PLAYER_MAX_C)
    state.player_last_dir = (tonumber(snap.player_last_dir) or 1) >= 0 and 1 or -1

    state.attack_symbol = snap.attack_symbol
    state.attack_until = state.frame + sec_to_frames(math.max(0, tonumber(snap.attack_left) or 0))
    state.missile_shots = math.max(0, math.floor(tonumber(snap.missile_shots) or 0))
    state.missile_last = state.frame + sec_to_frames(math.max(0, tonumber(snap.missile_cd_left) or 0)) - sec_to_frames(2)

    state.buff_until["*"] = state.frame + sec_to_frames(math.max(0, tonumber(snap.shield_left) or 0))
    state.buff_until["c"] = state.frame + sec_to_frames(math.max(0, tonumber(snap.magnet_left) or 0))
    state.buff_until["+"] = state.frame + sec_to_frames(math.max(0, tonumber(snap.bullet_speed_left) or 0))
    state.hurt_invuln_until = state.frame + sec_to_frames(math.max(0, tonumber(snap.hurt_invuln_left) or 0))
    state.nuke_stock = clamp(math.floor(tonumber(snap.nuke_stock) or 0), 0, 3)
    state.fire_mode = (type(snap.fire_mode) == "string" and string.lower(snap.fire_mode) == "manual") and "manual" or
        "auto"
    state.enemy_spawn_block_until = state.frame + sec_to_frames(math.max(0, tonumber(snap.enemy_spawn_block_left) or 0))
    state.boom_until = state.frame + sec_to_frames(math.max(0, tonumber(snap.boom_left) or 0))

    state.enemies = type(snap.enemies) == "table" and snap.enemies or {}
    state.enemy_bullets = type(snap.enemy_bullets) == "table" and snap.enemy_bullets or {}
    state.player_bullets = type(snap.player_bullets) == "table" and snap.player_bullets or {}
    state.items = type(snap.items) == "table" and snap.items or {}

    local saved_frame = math.floor(tonumber(snap.frame) or state.frame)
    local has_saved_frame = tonumber(snap.frame) ~= nil
    local function rebase_timer(next_at)
        if not has_saved_frame then return state.frame end
        local raw = math.floor(tonumber(next_at) or saved_frame)
        local left = raw - saved_frame
        if left < 0 then left = 0 end
        local cap = sec_to_frames(5)
        if left > cap then left = cap end
        return state.frame + left
    end

    for i = 1, #state.enemies do
        local e = state.enemies[i]
        if type(e) == "table" then
            local def = nil
            if type(e.kind) == "string" then def = ENEMY_TYPES[e.kind] end
            local base_speed = (def ~= nil and def.speed) or 2
            e.move_interval = math.max(1, math.floor(tonumber(e.move_interval) or speed_to_interval(base_speed / 4)))
            e.next_move_at = rebase_timer(e.next_move_at)
            e.shot_active = tonumber(e.shot_active) or ((def ~= nil and def.shot_active) or 3.0)
            e.shot_idle = tonumber(e.shot_idle) or ((def ~= nil and def.shot_idle) or 6.0)
            e.next_shot_at = rebase_timer(e.next_shot_at)
        end
    end

    for i = 1, #state.enemy_bullets do
        local b = state.enemy_bullets[i]
        if type(b) == "table" then
            b.move_interval = math.max(1, math.floor(tonumber(b.move_interval) or speed_to_interval(1)))
            b.next_move_at = rebase_timer(b.next_move_at)
        end
    end

    for i = 1, #state.player_bullets do
        local b = state.player_bullets[i]
        if type(b) == "table" then
            b.move_interval = math.max(1, math.floor(tonumber(b.move_interval) or speed_to_interval(2)))
            b.next_move_at = rebase_timer(b.next_move_at)
        end
    end

    for i = 1, #state.items do
        local it = state.items[i]
        if type(it) == "table" then
            it.next_move_at = rebase_timer(it.next_move_at)
        end
    end

    if type(snap.boss) == "table" then
        local b = snap.boss
        state.boss.active = b.active == true
        state.boss.row = clamp(math.floor(tonumber(b.row) or 1), 1, 3)
        state.boss.center_c = clamp(math.floor(tonumber(b.center_c) or 15), 3, 28)
        state.boss.hp = math.max(0, math.floor(tonumber(b.hp) or 0))
        state.boss.max_hp = math.max(1, math.floor(tonumber(b.max_hp) or 1))
        state.boss.mode = type(b.mode) == "string" and b.mode or "attack"
        state.boss.mode_until = state.frame + sec_to_frames(math.max(0, tonumber(b.mode_left) or 0))
        state.boss.next_move_at = state.frame + sec_to_frames(math.max(0, tonumber(b.move_left) or 0))
        state.boss.next_shot_at = state.frame + sec_to_frames(math.max(0, tonumber(b.shot_left) or 0))
        state.boss.start_frame = state.frame - sec_to_frames(math.max(0, tonumber(b.active_sec) or 0))
        state.boss.chase_cd_until = state.frame + sec_to_frames(math.max(0, tonumber(b.chase_cd_left) or 0))
    else
        state.boss.active = false
        state.boss.chase_cd_until = state.frame
    end

    state.next_enemy_spawn_at = state.frame + sec_to_frames(math.max(1, tonumber(snap.enemy_spawn_left) or 1))
    state.next_item_spawn_at = state.frame + sec_to_frames(math.max(1, tonumber(snap.item_spawn_left) or 1))
    if state.enemy_spawn_block_until > state.next_enemy_spawn_at then
        state.next_enemy_spawn_at = state.enemy_spawn_block_until
    end

    state.phase = "playing"
    state.confirm_mode = nil
    state.end_frame = nil
    state.result_committed = false
    state.last_elapsed = -1
    state.msg_text, state.msg_color, state.msg_until, state.msg_persistent = "", "dark_gray", 0, false

    state.dirty = true
    return true
end

local function load_game_state()
    local ok, snap = false, nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "shooter")
        ok = s and type(ret) == "table"
        snap = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "shooter")
        ok = s and type(ret) == "table"
        snap = ret
    end
    if not ok then return false end
    return restore_snapshot(snap)
end

local function reset_spawn_timers()
    state.next_enemy_spawn_at = state.frame + sec_to_frames(3)
    state.next_item_spawn_at = state.frame + sec_to_frames(rand_range(5, 10))
end

local function reset_run()
    state.score = 0
    state.stage = 1
    state.next_boss_score = BASE_BOSS_SCORE
    state.hp = 10
    state.player_c = math.floor((PLAYER_MIN_C + PLAYER_MAX_C) / 2)
    state.player_last_dir = 1

    clear_world_entities()
    clear_buffs()
    state.hurt_invuln_until = 0
    state.nuke_stock = 0
    state.fire_mode = "auto"
    state.enemy_spawn_block_until = state.frame
    state.boom_until = 0

    state.phase = "playing"
    state.confirm_mode = nil
    state.run_start_frame = state.frame
    state.end_frame = nil

    state.boss.active = false
    state.boss.row = 1
    state.boss.center_c = 15
    state.boss.hp = 0
    state.boss.max_hp = 1
    state.boss.mode = "attack"
    state.boss.mode_until = state.frame
    state.boss.next_move_at = state.frame
    state.boss.next_shot_at = state.frame
    state.boss.start_frame = state.frame
    state.boss.chase_cd_until = state.frame

    state.last_player_fire = state.frame
    reset_spawn_timers()

    state.msg_text, state.msg_color, state.msg_until, state.msg_persistent = "", "dark_gray", 0, false
    state.result_committed = false
    state.last_elapsed = -1
    state.dirty = true
end

local function next_boss_threshold(cur, next_stage)
    if next_stage <= 1 then return BASE_BOSS_SCORE end
    return cur + BASE_BOSS_SCORE + 50 * next_stage
end

local function set_lost_state()
    state.phase = "lost"
    state.end_frame = state.frame
    state.confirm_mode = nil
    commit_result_once()
    show_message(tr("game.shooter.lose_banner") .. " " .. tr("game.shooter.result_controls"), "red", 0, true)
end

local function apply_player_damage(dmg, ignore_invuln)
    if state.phase ~= "playing" then return end
    if not ignore_invuln and player_invuln_active() then return end

    state.hp = state.hp - dmg
    if not ignore_invuln then
        state.hurt_invuln_until = state.frame + sec_to_frames(2)
    end

    if state.hp <= 0 then
        state.hp = 0
        set_lost_state()
    else
        state.dirty = true
    end
end

local function remove_idx(list, idx)
    local n = #list
    if idx < 1 or idx > n then return end
    list[idx] = list[n]
    list[n] = nil
end

local function boss_cells()
    if not state.boss.active then return {} end
    local r, c = state.boss.row, state.boss.center_c
    return {
        { r = r,     c = c - 1 },
        { r = r,     c = c },
        { r = r,     c = c + 1 },
        { r = r + 1, c = c },
    }
end

local function boss_contains(r, c)
    local cells = boss_cells()
    for i = 1, #cells do
        if cells[i].r == r and cells[i].c == c then
            return true
        end
    end
    return false
end

local function spawn_enemy(kind, col)
    local def = ENEMY_TYPES[kind]
    if def == nil then return end

    local e = {
        kind = kind,
        glyph = def.glyph,
        color = def.color,
        score = def.score,
        hp = scale_enemy_hp(def.hp),
        collide = scale_enemy_collide(def.collide),
        shooter = def.shooter,
        bullet = def.bullet,
        base_dmg = def.base_dmg,
        tracking = def.tracking,
        shot_active = def.shot_active or 1.5,
        shot_idle = def.shot_idle or 3.0,
        r = 1,
        c = clamp(col, ENEMY_COL_MIN, ENEMY_COL_MAX),
        move_interval = speed_to_interval(def.speed / 4),
        next_move_at = state.frame + speed_to_interval(def.speed / 4),
        next_shot_at = state.frame + sec_to_frames(def.shot_active or 1.5),
    }
    state.enemies[#state.enemies + 1] = e
    state.dirty = true
end

local function maybe_spawn_enemy()
    if state.boss.active or state.frame < state.next_enemy_spawn_at then return end
    if state.frame < state.enemy_spawn_block_until then return end

    local s = stage_level()
    local interval = math.max(1.0, 3.0 - s * 0.12)
    state.next_enemy_spawn_at = state.frame + sec_to_frames(interval)

    local cap = math.min(12, 1 + state.stage * 2)
    if #state.enemies >= cap then return end

    local pick = choose_weighted({
        { id = "normal", w = 60 },
        { id = "fast",   w = 20 },
        { id = "tank",   w = 10 },
        { id = "heavy",  w = 10 },
    })
    spawn_enemy(pick.id, rand_range(ENEMY_COL_MIN, ENEMY_COL_MAX))
end

local function activate_attack_buff(symbol)
    local def = ATTACK_BUFF[symbol]
    if def == nil then return end

    state.attack_symbol = symbol
    state.attack_until = state.frame + sec_to_frames(def.dur)

    if def.mode == "missile" then
        state.missile_shots = 3
        state.missile_last = state.frame - sec_to_frames(2)
    else
        state.missile_shots = 0
    end

    show_message(tr("game.shooter.item." .. symbol) .. " " .. tr("game.shooter.msg_buff_on"), "green", 3, false)
end

local function activate_function_item(symbol)
    local def = FUNC_BUFF[symbol]
    if def == nil then return end

    if symbol == "*" then
        state.buff_until["*"] = state.frame + sec_to_frames(def.dur)
        touch_buff_order("*")
    elseif symbol == "c" then
        local extra = sec_to_frames(def.dur)
        if state.buff_until["c"] > state.frame then
            state.buff_until["c"] = state.buff_until["c"] + extra
        else
            state.buff_until["c"] = state.frame + extra
        end
        touch_buff_order("c")
    elseif symbol == "+" then
        state.buff_until["+"] = state.frame + sec_to_frames(def.dur)
        touch_buff_order("+")
    elseif symbol == "~" then
        if state.hp < 10 then
            state.hp = state.hp + 1
        end
    elseif symbol == "o" then
        state.score = state.score + 10
    elseif symbol == "G" then
        if state.nuke_stock < 3 then
            state.nuke_stock = state.nuke_stock + 1
        end
    end

    show_message(tr("game.shooter.item." .. symbol) .. " " .. tr("game.shooter.msg_item_get"), "light_cyan", 3, false)
end

local function nuke_spawn_permille()
    if state.boss.active then return 0 end
    if state.nuke_stock <= 0 then return 10 end
    if state.nuke_stock == 1 then return 5 end
    if state.nuke_stock == 2 then return 1 end
    return 0
end

local function spawn_item()
    if state.frame < state.next_item_spawn_at then return end
    state.next_item_spawn_at = state.frame + sec_to_frames(rand_range(5, 10))

    local symbol, color, kind = "@", "rgb(170,255,170)", "attack"

    local nuke_p = nuke_spawn_permille()
    if nuke_p > 0 and rand_int(1000) < nuke_p then
        symbol, color, kind = "G", "magenta", "function"
    else
        local group = choose_weighted({
            { kind = "attack",   w = 60 },
            { kind = "function", w = 40 }
        })
        kind = group.kind
        if group.kind == "attack" then
            local pick = choose_weighted({
                { sym = "@", w = 20 },
                { sym = "%", w = 20 },
                { sym = "$", w = 20 },
                { sym = "#", w = 20 },
                { sym = "&", w = 20 }
            })
            symbol = pick.sym
            color = "rgb(170,255,170)"
        else
            local pick = choose_weighted({
                { sym = "*", w = 25 },
                { sym = "~", w = 5 },
                { sym = "o", w = 50 },
                { sym = "c", w = 10 },
                { sym = "+", w = 10 }
            })
            symbol = pick.sym
            color = "light_cyan"
        end
    end

    state.items[#state.items + 1] = {
        symbol = symbol,
        color = color,
        kind = kind,
        r = 1,
        c = rand_range(ENEMY_COL_MIN, ENEMY_COL_MAX),
        next_move_at = state.frame + sec_to_frames(1)
    }
    state.dirty = true
end

local function attack_mode_name()
    if state.attack_symbol == nil then return "normal" end
    local def = ATTACK_BUFF[state.attack_symbol]
    if def == nil then return "normal" end
    return def.mode
end

local function create_player_bullet(c, mode)
    local speed_mul = has_bullet_speed() and 2 or 1
    local b = {
        owner = "player",
        r = PLAYER_ROW - 1,
        c = c,
        ch = "^",
        color = "green",
        damage = scale_player_dmg(1),
        kind = "normal",
        pierce = false,
        tracking = false,
        is_missile = false,
        missile_hp = 0,
        move_interval = speed_to_interval(2 * speed_mul),
        next_move_at = state.frame,
    }

    if mode == "laser" then
        b.ch = "|"
        b.kind = "laser"
        b.pierce = true
        b.damage = scale_player_dmg(1)
    elseif mode == "double" then
        b.ch = ":"
        b.kind = "double"
        b.damage = scale_player_dmg(2)
    elseif mode == "single" then
        b.ch = "."
        b.kind = "single"
        b.damage = scale_player_dmg(3)
    elseif mode == "missile" then
        b.ch = "!"
        b.kind = "missile"
        b.damage = scale_player_dmg(4)
        b.tracking = true
        b.move_interval = speed_to_interval(1 * speed_mul)
        b.is_missile = true
        b.missile_hp = 4
    end
    return b
end

local function nearest_enemy_col(c)
    local best_c, best_d = nil, 10 ^ 9
    for i = 1, #state.enemies do
        local e = state.enemies[i]
        local d = math.abs(e.c - c) + math.abs(e.r - PLAYER_ROW)
        if d < best_d then
            best_d, best_c = d, e.c
        end
    end
    if state.boss.active then
        local d = math.abs(state.boss.center_c - c)
        if d < best_d then
            best_c = state.boss.center_c
        end
    end
    return best_c
end

local function fire_player_if_needed(force_once)
    if state.phase ~= "playing" or state.confirm_mode ~= nil then return end
    if state.fire_mode == "manual" and force_once ~= true then return end

    local mode = attack_mode_name()
    local rapid = (mode == "rapid")
    local interval = rapid and sec_to_frames(0.5) or sec_to_frames(1)

    if state.last_player_fire == nil then
        state.last_player_fire = state.frame - interval
    end

    if mode == "missile" then
        if state.frame - state.missile_last >= sec_to_frames(2) then
            state.player_bullets[#state.player_bullets + 1] = create_player_bullet(state.player_c, "missile")
            state.missile_last = state.frame
            state.dirty = true
        end
        return
    end

    if state.frame - state.last_player_fire < interval then return end
    state.last_player_fire = state.frame

    local fire_kind = rapid and "normal" or mode
    state.player_bullets[#state.player_bullets + 1] = create_player_bullet(state.player_c, fire_kind)
    state.dirty = true
end

local function kill_enemy(idx)
    local e = state.enemies[idx]
    if e ~= nil then
        state.score = state.score + (e.score or 0)
    end
    remove_idx(state.enemies, idx)
    state.dirty = true
end

local function find_enemy_at(r, c)
    for i = 1, #state.enemies do
        local e = state.enemies[i]
        if e.r == r and e.c == c then
            return i, e
        end
    end
    return nil, nil
end

local function boss_take_damage(dmg)
    if not state.boss.active then return end
    state.boss.hp = state.boss.hp - dmg
    if state.boss.hp <= 0 then
        state.boss.hp = 0
        state.boss.active = false
        state.score = state.score + 20
        state.stage = state.stage + 1
        state.next_boss_score = next_boss_threshold(state.next_boss_score, state.stage)
        show_message(tr("game.shooter.msg_boss_defeated"), "green", 3, false)
    end
    state.dirty = true
end

local function update_player_bullets()
    local i = 1
    while i <= #state.player_bullets do
        local b = state.player_bullets[i]
        local remove = false
        local can_hit = false

        if state.frame >= b.next_move_at then
            b.next_move_at = state.frame + b.move_interval
            if b.tracking then
                local target = nearest_enemy_col(b.c)
                if target ~= nil then
                    if target > b.c then
                        b.c = b.c + 1
                    elseif target < b.c then
                        b.c = b.c - 1
                    end
                end
            end
            b.r = b.r - 1
            can_hit = true
            state.dirty = true
        end

        if b.r < 1 or b.c < 1 or b.c > INNER_W then
            remove = true
        end

        if (not remove) and can_hit and state.boss.active and boss_contains(b.r, b.c) then
            local dmg = b.damage or 1
            if b.kind == "double" then
                local hp_before = state.boss.hp
                boss_take_damage(dmg)
                if hp_before == 1 then
                    b.kind = "double_remain"
                    b.ch = "·"
                    b.damage = scale_player_dmg(2)
                else
                    remove = true
                end
            elseif b.kind == "double_remain" then
                boss_take_damage(dmg)
                remove = true
            else
                boss_take_damage(dmg)
                if not b.pierce then
                    remove = true
                end
            end
        end

        if (not remove) and can_hit then
            local ei, e = find_enemy_at(b.r, b.c)
            if ei ~= nil and e ~= nil then
                local dmg = b.damage or 1
                if b.kind == "double" then
                    local hp_before = e.hp
                    e.hp = e.hp - dmg
                    if e.hp <= 0 then kill_enemy(ei) end
                    if hp_before == 1 then
                        b.kind = "double_remain"
                        b.ch = "·"
                        b.damage = scale_player_dmg(2)
                    else
                        remove = true
                    end
                elseif b.kind == "double_remain" then
                    e.hp = e.hp - dmg
                    if e.hp <= 0 then kill_enemy(ei) end
                    remove = true
                else
                    e.hp = e.hp - dmg
                    if e.hp <= 0 then kill_enemy(ei) end
                    if not b.pierce then
                        remove = true
                    end
                end
            end
        end

        if remove then
            remove_idx(state.player_bullets, i)
        else
            i = i + 1
        end
    end
end

local function spawn_enemy_bullet(c, ch, base_dmg, tracking, row)
    local is_missile = ch == "u"
    state.enemy_bullets[#state.enemy_bullets + 1] = {
        owner = "enemy",
        r = row or 2,
        c = clamp(c, ENEMY_COL_MIN, ENEMY_COL_MAX),
        ch = ch,
        color = "magenta",
        damage = scale_enemy_bullet_dmg(base_dmg),
        tracking = tracking == true,
        target_c = state.player_c,
        is_missile = is_missile,
        missile_hp = is_missile and 8 or 0,
        move_interval = speed_to_interval(1),
        next_move_at = state.frame,
    }
end

local function update_enemy_bullets()
    local i = 1
    while i <= #state.enemy_bullets do
        local b = state.enemy_bullets[i]

        if state.frame >= b.next_move_at then
            b.next_move_at = state.frame + b.move_interval
            if b.tracking then
                local tc = b.target_c or b.c
                if tc > b.c then
                    b.c = b.c + 1
                elseif tc < b.c then
                    b.c = b.c - 1
                end
                b.c = clamp(b.c, ENEMY_COL_MIN, ENEMY_COL_MAX)
            end
            b.r = b.r + 1
            state.dirty = true
        end

        local remove = false
        if b.r > INNER_H then
            remove = true
        elseif b.r == PLAYER_ROW and b.c == state.player_c then
            apply_player_damage(b.damage or 1, false)
            remove = true
        end

        if remove then
            remove_idx(state.enemy_bullets, i)
        else
            i = i + 1
        end
    end
end

local function has_enemy_bullet_too_close(col, spawn_row)
    for i = 1, #state.enemy_bullets do
        local b = state.enemy_bullets[i]
        if b.c == col and math.abs(b.r - spawn_row) <= 1 then
            return true
        end
    end
    return false
end

local function boss_muzzle_blocked(col)
    local muzzle_row = state.boss.row + 2
    for i = 1, #state.enemy_bullets do
        local b = state.enemy_bullets[i]
        if b.c == col and b.r >= muzzle_row and b.r <= (muzzle_row + 2) then
            return true
        end
    end
    return false
end

local function update_enemies()
    local i = 1
    while i <= #state.enemies do
        local e = state.enemies[i]
        local removed = false

        if state.frame >= e.next_move_at then
            e.next_move_at = state.frame + e.move_interval
            e.r = e.r + 1
            state.dirty = true
        end

        if e.r > INNER_H then
            remove_idx(state.enemies, i)
            removed = true
        elseif e.r == PLAYER_ROW and e.c == state.player_c then
            apply_player_damage(e.collide or 1, false)
            remove_idx(state.enemies, i)
            removed = true
        end

        if not removed then
            if e.shooter and state.frame >= e.next_shot_at then
                local spawn_row = e.r + 1
                local active_interval = sec_to_frames(e.shot_active or 1.5)
                local idle_interval = sec_to_frames(e.shot_idle or 3.0)
                local next_interval = (e.c == state.player_c) and active_interval or idle_interval

                if has_enemy_bullet_too_close(e.c, spawn_row) then
                    e.next_shot_at = state.frame + math.max(1, math.floor(next_interval / 2))
                else
                    spawn_enemy_bullet(e.c, e.bullet, e.base_dmg, e.tracking, spawn_row)
                    e.next_shot_at = state.frame + next_interval
                end
            end
            i = i + 1
        end
    end
end

local function find_nearest_player_bullet_col()
    local best_c, best_d = nil, 10 ^ 9
    for i = 1, #state.player_bullets do
        local b = state.player_bullets[i]
        local d = math.abs(b.c - state.boss.center_c) + math.abs(b.r - state.boss.row)
        if d < best_d then
            best_d, best_c = d, b.c
        end
    end
    return best_c
end

local function boss_summon_wave()
    local n = rand_range(2, 5)
    for _ = 1, n do
        local pick = choose_weighted({
            { id = "normal", w = 45 },
            { id = "fast",   w = 25 },
            { id = "tank",   w = 15 },
            { id = "heavy",  w = 15 }
        })
        spawn_enemy(pick.id, rand_range(ENEMY_COL_MIN, ENEMY_COL_MAX))
    end
end

local function choose_boss_mode()
    local chase_w = (state.frame >= (state.boss.chase_cd_until or 0)) and 10 or 0
    return choose_weighted({
        { mode = "attack",  w = 35 },
        { mode = "predict", w = 30 },
        { mode = "dodge",   w = 15 },
        { mode = "summon",  w = 10 },
        { mode = "chase",   w = chase_w },
    }).mode
end

local function boss_fire(mode)
    local w = mode == "predict"
        and {
            { ch = "v", base = 1, tr = false, p = 60 },
            { ch = ".", base = 2, tr = false, p = 20 },
            { ch = "u", base = 4, tr = true,  p = 20 }
        }
        or {
            { ch = "v", base = 1, tr = false, p = 70 },
            { ch = ".", base = 2, tr = false, p = 20 },
            { ch = "u", base = 4, tr = true,  p = 10 }
        }

    local pick = choose_weighted({
        { idx = 1, w = w[1].p },
        { idx = 2, w = w[2].p },
        { idx = 3, w = w[3].p }
    })
    local spec = w[pick.idx]

    local c = state.boss.center_c
    if mode == "predict" then
        c = clamp(state.player_c + state.player_last_dir * 2, ENEMY_COL_MIN, ENEMY_COL_MAX)
    elseif mode == "attack" then
        c = state.player_c
    elseif mode == "chase" then
        c = state.player_c
    end

    if boss_muzzle_blocked(c) then return false end
    spawn_enemy_bullet(c, spec.ch, spec.base, spec.tr, state.boss.row + 2)
    return true
end

local function enter_boss_battle()
    clear_world_entities()
    clear_buffs()
    state.boss.active = true
    state.boss.row = 1
    state.boss.center_c = math.floor((PLAYER_MIN_C + PLAYER_MAX_C) / 2)
    state.boss.max_hp = scale_boss_hp(30)
    state.boss.hp = state.boss.max_hp
    state.boss.mode = "attack"
    state.boss.mode_until = state.frame + sec_to_frames(3)
    state.boss.next_move_at = state.frame + sec_to_frames(2)
    state.boss.next_shot_at = state.frame + sec_to_frames(1)
    state.boss.start_frame = state.frame
    state.boss.chase_cd_until = state.frame
    show_message(tr("game.shooter.msg_boss_incoming"), "yellow", 3, false)
end

local function maybe_trigger_boss()
    if state.boss.active then return end
    if state.score >= state.next_boss_score then
        enter_boss_battle()
    end
end

local function update_boss()
    if not state.boss.active then return end

    if state.frame - state.boss.start_frame >= sec_to_frames(180) then
        apply_player_damage(99, true)
        return
    end

    if state.frame >= state.boss.mode_until then
        if state.boss.mode == "chase" then
            state.boss.chase_cd_until = state.frame + sec_to_frames(10)
        end

        state.boss.mode = choose_boss_mode()
        if state.boss.mode == "chase" then
            state.boss.mode_until = state.frame + sec_to_frames(2)
            state.boss.next_move_at = state.frame
            state.boss.next_shot_at = state.frame
        else
            state.boss.mode_until = state.frame + sec_to_frames(rand_range(3, 6))
            if state.boss.mode == "summon" then
                boss_summon_wave()
            end
        end
    end

    if state.frame >= state.boss.next_move_at then
        local move_interval = sec_to_frames(2)
        if state.boss.mode == "chase" then
            move_interval = sec_to_frames(0.35)
        end
        state.boss.next_move_at = state.frame + move_interval

        local target = state.boss.center_c
        if state.boss.mode == "attack" then
            target = state.player_c
        elseif state.boss.mode == "predict" then
            target = clamp(state.player_c + state.player_last_dir * 3, 3, 28)
        elseif state.boss.mode == "dodge" then
            local bc = find_nearest_player_bullet_col()
            if bc ~= nil then
                if bc <= state.boss.center_c then
                    target = state.boss.center_c + 2
                else
                    target = state.boss.center_c - 2
                end
            else
                target = clamp(state.player_c - state.player_last_dir * 3, 3, 28)
            end
        elseif state.boss.mode == "summon" then
            if state.player_c < state.boss.center_c then
                target = 27
            else
                target = 4
            end
        elseif state.boss.mode == "chase" then
            target = state.player_c
        end

        target = clamp(target, 3, 28)
        if target > state.boss.center_c then
            state.boss.center_c = state.boss.center_c + 1
        elseif target < state.boss.center_c then
            state.boss.center_c = state.boss.center_c - 1
        end
        state.boss.center_c = clamp(state.boss.center_c, 3, 28)
        state.dirty = true
    end

    if state.frame >= state.boss.next_shot_at then
        local shot_interval = sec_to_frames(1)
        if state.boss.mode == "chase" then
            shot_interval = sec_to_frames(0.5)
        end

        if state.boss.mode ~= "summon" and state.boss.mode ~= "dodge" then
            boss_fire(state.boss.mode)
        end

        state.boss.next_shot_at = state.frame + shot_interval
    end
end

local function update_buffs()
    local changed = false
    if state.attack_symbol ~= nil and state.frame >= state.attack_until then
        state.attack_symbol = nil
        state.attack_until = 0
        state.missile_shots = 0
        changed = true
    end
    for _, sym in ipairs({ "*", "c", "+" }) do
        if state.buff_until[sym] > 0 and state.frame >= state.buff_until[sym] then
            state.buff_until[sym] = 0
            remove_buff_order(sym)
            changed = true
        end
    end
    if changed then
        state.dirty = true
    end
end

local function resolve_bullet_vs_bullet_collisions()
    if #state.player_bullets == 0 or #state.enemy_bullets == 0 then
        return
    end

    local changed = false

    for pi = 1, #state.player_bullets do
        local pb = state.player_bullets[pi]
        for ei = 1, #state.enemy_bullets do
            local eb = state.enemy_bullets[ei]
            if pb.r == eb.r and pb.c == eb.c then
                if pb.is_missile then
                    pb.missile_hp = (pb.missile_hp or 4) - 1
                    changed = true
                end
                if eb.is_missile then
                    eb.missile_hp = (eb.missile_hp or 8) - 1
                    changed = true
                end
            end
        end
    end

    if not changed then
        return
    end

    for i = #state.player_bullets, 1, -1 do
        local b = state.player_bullets[i]
        if b.is_missile and (b.missile_hp or 0) <= 0 then
            table.remove(state.player_bullets, i)
        end
    end

    for i = #state.enemy_bullets, 1, -1 do
        local b = state.enemy_bullets[i]
        if b.is_missile and (b.missile_hp or 0) <= 0 then
            table.remove(state.enemy_bullets, i)
        end
    end

    state.dirty = true
end

local function update_items()
    local i = 1
    while i <= #state.items do
        local it = state.items[i]
        local magnet_tracking = has_magnet()
            and math.abs(it.c - state.player_c) <= 4
            and math.abs(it.r - PLAYER_ROW) <= 4

        if state.frame >= it.next_move_at then
            it.next_move_at = state.frame + sec_to_frames(1)
            if magnet_tracking then
                if it.c < state.player_c then
                    it.c = it.c + 1
                elseif it.c > state.player_c then
                    it.c = it.c - 1
                end
                if it.r < PLAYER_ROW then
                    it.r = it.r + 1
                elseif it.r > PLAYER_ROW then
                    it.r = it.r - 1
                end
            else
                it.r = it.r + 1
            end
            state.dirty = true
        end

        local picked = false
        if it.r == PLAYER_ROW and it.c == state.player_c then
            picked = true
        end

        if picked then
            if it.kind == "attack" then
                activate_attack_buff(it.symbol)
            else
                activate_function_item(it.symbol)
            end
            remove_idx(state.items, i)
        elseif it.r > INNER_H then
            remove_idx(state.items, i)
        else
            i = i + 1
        end
    end
end

local function use_nuke()
    if state.nuke_stock <= 0 then
        show_message(tr("game.shooter.msg_nuke_empty"), "dark_gray", 2, false)
        return
    end

    state.nuke_stock = state.nuke_stock - 1
    clear_world_entities()
    if state.boss.active then
        boss_take_damage(50)
    end
    apply_player_damage(2, true)

    state.enemy_spawn_block_until = state.frame + sec_to_frames(3)
    if state.enemy_spawn_block_until > state.next_enemy_spawn_at then
        state.next_enemy_spawn_at = state.enemy_spawn_block_until
    end
    state.boom_until = state.frame + sec_to_frames(3)
    state.dirty = true
end

local function gameplay_update()
    if state.phase ~= "playing" or state.confirm_mode ~= nil then return end

    update_buffs()
    maybe_trigger_boss()
    maybe_spawn_enemy()
    spawn_item()
    fire_player_if_needed(false)

    update_player_bullets()
    update_enemy_bullets()
    resolve_bullet_vs_bullet_collisions()
    update_enemies()
    update_boss()
    update_items()

    if state.boom_until > 0 and state.frame >= state.boom_until then
        state.boom_until = 0
        state.dirty = true
    end

    if state.hp <= 0 and state.phase ~= "lost" then
        set_lost_state()
    end
end

local function board_to_term(layout, c, r)
    return layout.board_x + c, layout.board_y + r
end

local function draw_board_frame(layout)
    local x, y = layout.board_x, layout.board_y
    draw_text(x, y, CH_DBL_TL .. string.rep(CH_DBL_H, BOARD_W - 2) .. CH_DBL_TR, "white", "black")
    for r = 1, BOARD_H - 2 do
        draw_text(x, y + r, CH_DBL_V, "white", "black")
        draw_text(x + BOARD_W - 1, y + r, CH_DBL_V, "white", "black")
    end
    draw_text(x, y + BOARD_H - 1, CH_DBL_BL .. string.rep(CH_DBL_H, BOARD_W - 2) .. CH_DBL_BR, "white", "black")
end

local function build_board_buffer()
    local buf = {}
    for r = 1, INNER_H do
        buf[r] = {}
        for c = 1, INNER_W do
            buf[r][c] = { ch = " ", fg = "white", bg = "black" }
        end
    end

    buf[PLAYER_ROW][1] = { ch = CH_BLOCK, fg = "white", bg = "black" }
    buf[PLAYER_ROW][INNER_W] = { ch = CH_BLOCK, fg = "white", bg = "black" }

    for i = 1, #state.items do
        local it = state.items[i]
        if it.r >= 1 and it.r <= INNER_H and it.c >= 1 and it.c <= INNER_W then
            buf[it.r][it.c] = { ch = it.symbol, fg = it.color, bg = "black" }
        end
    end

    for i = 1, #state.player_bullets do
        local b = state.player_bullets[i]
        if b.r >= 1 and b.r <= INNER_H and b.c >= 1 and b.c <= INNER_W then
            buf[b.r][b.c] = { ch = b.ch, fg = b.color, bg = "black" }
        end
    end

    for i = 1, #state.enemy_bullets do
        local b = state.enemy_bullets[i]
        if b.r >= 1 and b.r <= INNER_H and b.c >= 1 and b.c <= INNER_W then
            buf[b.r][b.c] = { ch = b.ch, fg = b.color, bg = "black" }
        end
    end

    for i = 1, #state.enemies do
        local e = state.enemies[i]
        if e.r >= 1 and e.r <= INNER_H and e.c >= 1 and e.c <= INNER_W then
            buf[e.r][e.c] = { ch = e.glyph, fg = e.color, bg = "black" }
        end
    end

    if state.boss.active then
        local cells = boss_cells()
        for i = 1, #cells do
            local cell = cells[i]
            if cell.r >= 1 and cell.r <= INNER_H and cell.c >= 1 and cell.c <= INNER_W then
                buf[cell.r][cell.c] = { ch = CH_BLOCK, fg = "rgb(255,170,170)", bg = "black" }
            end
        end
    end

    buf[PLAYER_ROW][state.player_c] = { ch = "A", fg = player_color(), bg = "black" }
    return buf
end

local function draw_board_content(layout)
    local buf = build_board_buffer()
    for r = 1, INNER_H do
        for c = 1, INNER_W do
            local cell = buf[r][c]
            local tx, ty = board_to_term(layout, c, r)
            draw_text(tx, ty, cell.ch, cell.fg, cell.bg)
        end
    end

    if state.boom_until > state.frame then
        local boom = "BOOM!!!"
        local bx = centered_x(boom, layout.board_x + 1, INNER_W)
        local by = layout.board_y + math.floor(INNER_H / 2)
        draw_text(bx, by, boom, "rgb(255,165,0)", "black")
    end
end

local function draw_boss_bar(layout)
    local term_w, _ = terminal_size()
    draw_text(1, layout.boss_bar_y, string.rep(" ", term_w), "white", "black")
    if not state.boss.active then return end

    local pct = clamp(math.floor((state.boss.hp / state.boss.max_hp) * 100 + 0.5), 0, 100)
    local filled = clamp(math.floor((state.boss.hp / state.boss.max_hp) * BOSS_HP_BAR_W + 0.5), 0, BOSS_HP_BAR_W)

    local x = centered_x(string.rep(CH_BLOCK, BOSS_HP_BAR_W) .. " 100%", layout.x, layout.total_w)
    if filled > 0 then
        draw_text(x, layout.boss_bar_y, string.rep(CH_BLOCK, filled), "green", "black")
    end
    if filled < BOSS_HP_BAR_W then
        draw_text(x + filled, layout.boss_bar_y, string.rep(CH_BLOCK, BOSS_HP_BAR_W - filled), "red", "black")
    end
    draw_text(x + BOSS_HP_BAR_W, layout.boss_bar_y, string.format(" %d%%", pct), "white", "black")
end

local function draw_life_block(x, y)
    draw_text(x, y, tr("game.shooter.hp") .. ":", "white", "black")
    local a = math.min(5, state.hp)
    local b = math.max(0, state.hp - 5)
    draw_text(x, y + 1, string.rep("A", a), "yellow", "black")
    if a < 5 then
        draw_text(x + a, y + 1, string.rep("-", 5 - a), "dark_gray", "black")
    end
    draw_text(x, y + 2, string.rep("A", b), "yellow", "black")
    if b < 5 then
        draw_text(x + b, y + 2, string.rep("-", 5 - b), "dark_gray", "black")
    end
end

local function draw_buff_line(x, y, sym, remain, total)
    local blocks = 6
    local filled = 0
    if total > 0 then
        filled = math.floor((remain / total) * blocks + 0.999)
    end
    filled = clamp(filled, 0, blocks)

    draw_text(x, y, sym, "white", "black")
    if filled > 0 then
        draw_text(x + 2, y, string.rep(CH_BLOCK, filled), "green", "black")
    end
    if filled < blocks then
        draw_text(x + 2 + filled, y, string.rep(CH_BLOCK, blocks - filled), "dark_gray", "black")
    end
    draw_text(x + 2 + blocks + 1, y, tostring(remain) .. tr("game.shooter.seconds"), "white", "black")
end

local function draw_info(layout)
    local x, y, w = layout.info_x, layout.info_y, layout.info_w
    fill_rect(x, y, w, BOARD_H, "black")

    draw_text(x, y + 0, tr("game.shooter.best_score") .. ": " .. tostring(state.best_score), "dark_gray", "black")
    draw_text(x, y + 1, tr("game.shooter.best_stage") .. ": " .. tostring(state.best_stage), "dark_gray", "black")
    draw_text(x, y + 2, tr("game.shooter.score") .. ": " .. tostring(state.score), "white", "black")
    draw_text(x, y + 3, tr("game.shooter.time") .. ": " .. format_duration(elapsed_seconds()), "light_cyan", "black")

    draw_text(x, y + 4,
        tr("game.shooter.fire_mode") ..
        ": " .. tr(state.fire_mode == "manual" and "game.shooter.fire_mode_manual" or "game.shooter.fire_mode_auto"),
        "white", "black")
    draw_text(x, y + 5, tr("game.shooter.stage") .. ": " .. tostring(state.stage), "white", "black")
    draw_life_block(x, y + 6)

    local slot = string.rep("G", state.nuke_stock) .. string.rep("-", 3 - state.nuke_stock)
    draw_text(x, y + 10, tr("game.shooter.magazine") .. ": " .. slot, "white", "black")

    local line_y = y + 12
    if state.attack_symbol ~= nil and state.attack_until > state.frame then
        local remain = math.max(0, math.ceil((state.attack_until - state.frame) / FPS))
        local total = ATTACK_BUFF[state.attack_symbol] and ATTACK_BUFF[state.attack_symbol].dur or 1
        draw_buff_line(x, line_y, state.attack_symbol, remain, total)
        line_y = line_y + 1
    end

    for i = 1, #state.buff_order do
        local sym = state.buff_order[i]
        local until_frame = state.buff_until[sym] or 0
        if until_frame > state.frame then
            local total = FUNC_BUFF[sym] and FUNC_BUFF[sym].dur or 1
            local remain = math.max(0, math.ceil((until_frame - state.frame) / FPS))
            draw_buff_line(x, line_y, sym, remain, total)
            line_y = line_y + 1
        end
    end
end

local function current_msg()
    if state.confirm_mode == "restart" then
        return tr("game.shooter.confirm_restart"), "yellow"
    end
    if state.confirm_mode == "exit" then
        return tr("game.shooter.confirm_exit"), "yellow"
    end
    return state.msg_text, state.msg_color
end

local function shooter_controls_text()
    return tr("game.shooter.controls")
end

local function draw_message_controls(layout)
    local term_w, _ = terminal_size()
    draw_text(1, layout.message_y, string.rep(" ", term_w), "white", "black")

    local m, c = current_msg()
    if m ~= nil and m ~= "" then
        draw_text(centered_x(m, 1, term_w), layout.message_y, m, c or "dark_gray", "black")
    end

    local txt = shooter_controls_text()
    local lines = wrap_words(txt, math.max(12, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 0, 2 do
        draw_text(1, layout.controls_y + i, string.rep(" ", term_w), "white", "black")
    end
    local off = (#lines < 3) and math.floor((3 - #lines) / 2) or 0
    for i = 1, #lines do
        draw_text(centered_x(lines[i], 1, term_w), layout.controls_y + off + i - 1, lines[i], "white", "black")
    end
end

local function build_layout()
    local term_w, term_h = terminal_size()
    local info_w, gap = 28, 4

    local content_w = BOARD_W + gap + info_w
    local controls_w = min_width_for_lines(shooter_controls_text(), 3, 28)
    local msg_w = math.max(
        text_width(tr("game.shooter.lose_banner") .. " " .. tr("game.shooter.result_controls")),
        text_width(tr("game.shooter.confirm_restart")),
        text_width(tr("game.shooter.confirm_exit")),
        text_width(tr("game.shooter.msg_boss_incoming")),
        text_width(tr("game.shooter.msg_boss_defeated"))
    )

    local total_w = math.max(content_w, controls_w, msg_w, BOSS_HP_BAR_W + 8)
    local total_h = BOARD_H + 2 + 3

    local x = math.floor((term_w - total_w) / 2) + 1
    local y = math.floor((term_h - total_h) / 2) + 1
    if x < 1 then x = 1 end
    if y < 2 then y = 2 end

    return {
        x = x,
        y = y,
        total_w = total_w,
        total_h = total_h,
        board_x = x,
        board_y = y,
        info_x = x + BOARD_W + gap,
        info_y = y,
        info_w = info_w,
        boss_bar_y = y - 1,
        message_y = y + BOARD_H,
        controls_y = y + BOARD_H + 1,
    }
end

local function clear_last_area()
    if state.last_area == nil then return end
    fill_rect(state.last_area.x, state.last_area.y, state.last_area.w, state.last_area.h, "black")
end

local function force_full_refresh()
    clear()
    state.last_area = nil
    state.dirty = true
end

local function render_frame()
    local layout = build_layout()
    local area = { x = layout.x, y = layout.boss_bar_y, w = layout.total_w, h = BOARD_H + 1 + 3 }

    if state.last_area == nil then
        fill_rect(area.x, area.y, area.w, area.h, "black")
    elseif state.last_area.x ~= area.x or state.last_area.y ~= area.y
        or state.last_area.w ~= area.w or state.last_area.h ~= area.h then
        clear_last_area()
        fill_rect(area.x, area.y, area.w, area.h, "black")
    end
    state.last_area = area

    draw_boss_bar(layout)
    draw_board_frame(layout)
    draw_board_content(layout)
    draw_info(layout)
    draw_message_controls(layout)
end

local function minimum_required_size()
    local content_w = BOARD_W + 4 + 28
    local controls_w = min_width_for_lines(shooter_controls_text(), 3, 28)
    local msg_w = math.max(
        text_width(tr("game.shooter.confirm_restart")),
        text_width(tr("game.shooter.confirm_exit")),
        text_width(tr("game.shooter.lose_banner") .. " " .. tr("game.shooter.result_controls"))
    )

    local min_w = math.max(content_w, controls_w, msg_w, BOSS_HP_BAR_W + 8) + 2
    local min_h = BOARD_H + 1 + 3 + 2
    return min_w, min_h
end

local function draw_size_warning(term_w, term_h, min_w, min_h)
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint"),
        tr("warning.back_to_game_list_hint"),
    }

    clear()
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then top = 1 end

    for i = 1, #lines do
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, line, "white", "black")
    end
end

local function ensure_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then
            clear()
            state.last_area = nil
            state.dirty = true
        end
        state.size_warning_active = false
        return true
    end

    local changed = (not state.size_warning_active)
        or state.last_warn_term_w ~= term_w
        or state.last_warn_term_h ~= term_h
        or state.last_warn_min_w ~= min_w
        or state.last_warn_min_h ~= min_h
    if changed then
        draw_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w, state.last_warn_term_h = term_w, term_h
        state.last_warn_min_w, state.last_warn_min_h = min_w, min_h
    end
    state.size_warning_active = true
    return false
end

local function sync_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w, state.last_term_h = w, h
        force_full_refresh()
    end
end

local function refresh_dirty_time()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed then
        state.last_elapsed = elapsed
        state.dirty = true
    end
end

local function handle_confirm_key(key)
    if state.confirm_mode == nil then return false end
    if key == "y" or key == "enter" then
        if state.confirm_mode == "restart" then
            reset_run()
        else
            commit_result_once()
            exit_game()
        end
        return true
    end
    if key == "n" or key == "q" or key == "esc" then
        state.confirm_mode = nil
        state.dirty = true
        return true
    end
    return true
end

local function handle_input(key)
    if key == nil or key == "" then return end

    if state.confirm_mode ~= nil then
        handle_confirm_key(key)
        return
    end

    if state.phase == "lost" then
        if key == "r" then
            reset_run()
            return
        end
        if key == "q" or key == "esc" then
            commit_result_once()
            exit_game()
            return
        end
        return
    end

    if key == "left" then
        state.player_c = clamp(state.player_c - 1, PLAYER_MIN_C, PLAYER_MAX_C)
        state.player_last_dir = -1
        state.dirty = true
        return
    end
    if key == "right" then
        state.player_c = clamp(state.player_c + 1, PLAYER_MIN_C, PLAYER_MAX_C)
        state.player_last_dir = 1
        state.dirty = true
        return
    end

    if key == "z" then
        if state.fire_mode == "auto" then
            state.fire_mode = "manual"
            show_message(tr("game.shooter.msg_fire_mode_manual"), "yellow", 2, false)
        else
            state.fire_mode = "auto"
            show_message(tr("game.shooter.msg_fire_mode_auto"), "yellow", 2, false)
        end
        state.dirty = true
        return
    end

    if key == "space" then
        if state.fire_mode == "manual" then
            fire_player_if_needed(true)
        end
        return
    end

    if key == "x" then
        use_nuke()
        return
    end

    if key == "s" then
        save_game_state(true)
        return
    end

    if key == "r" then
        state.confirm_mode = "restart"
        state.dirty = true
        return
    end
    if key == "q" or key == "esc" then
        state.confirm_mode = "exit"
        state.dirty = true
        return
    end
end

local function bootstrap_game()
    clear()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    state.launch_mode = read_launch_mode()

    load_best_records()
    reset_run()
    if state.launch_mode == "continue" then
        if not load_game_state() then
            reset_run()
        end
    end

    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
    return state
end

function init_game()
    return bootstrap_game()
end

function handle_event(state_arg, event)
    state = state_arg or state
    sync_resize()
    local key = normalize_key(event)

    if ensure_size_ok() then
        if type(event) == "table" and event.type == "tick" then
            gameplay_update()
            update_message_timer()
            refresh_dirty_time()
            state.frame = state.frame + 1
        else
            handle_input(key)
        end
    else
        if key == "q" or key == "esc" then
            commit_result_once()
            exit_game()
        end
    end

    return state
end

function render(state_arg)
    state = state_arg or state
    sync_resize()
    if ensure_size_ok() then
        render_frame()
        state.dirty = false
    end
end

function best_score(state_arg)
    local current = state_arg or state
    return {
        best_string = "game.shooter.best_block",
        score = current.best_score or 0,
        stage = current.best_stage or 1,
    }
end
