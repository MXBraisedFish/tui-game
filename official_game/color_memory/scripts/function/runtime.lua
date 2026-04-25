local Constants = load_function("/constants.lua")

local FPS = Constants.FPS
local FRAME_MS = Constants.FRAME_MS
local SHOW_ON_MS = Constants.SHOW_ON_MS
local SHOW_OFF_MS = Constants.SHOW_OFF_MS

local BOX_W = Constants.BOX_W
local BOX_H = Constants.BOX_H
local BOX_GAP = Constants.BOX_GAP
local INPUT_GAP = Constants.INPUT_GAP
local FRAME_H = Constants.FRAME_H

local FRAME_TL = utf8.char(9556)
local FRAME_TR = utf8.char(9559)
local FRAME_BL = utf8.char(9562)
local FRAME_BR = utf8.char(9565)
local FRAME_HL = utf8.char(9552)
local FRAME_VL = utf8.char(9553)
local BOX_TL = utf8.char(9484)
local BOX_TR = utf8.char(9488)
local BOX_BL = utf8.char(9492)
local BOX_BR = utf8.char(9496)
local BOX_HL = utf8.char(9472)
local BOX_VL = utf8.char(9474)

local COLORS = Constants.COLORS

local state = {
    score = 0,
    round = 1,
    sequence = {},
    input_colors = {},
    highlight_idx = 0,

    best_score = 0,
    best_time_sec = 0,

    phase = "input",
    lost = false,
    confirm_mode = nil,
    committed = false,

    frame = 0,
    start_frame = 0,
    end_frame = nil,
    running = true,
    dirty = true,

    last_elapsed_sec = -1,

    last_term_w = 0,
    last_term_h = 0,

    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0,

    sequence_anim = nil,
    sequence_timer_id = nil
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

local KEY_DISPLAY = {
    up = "↑",
    down = "↓",
    left = "←",
    right = "→",
    enter = "Enter",
    esc = "Esc",
    space = "Space",
    backspace = "Bksp",
    del = "Del",
    tab = "Tab",
    back_tab = "BTab"
}

local function display_key_name(key)
    key = tostring(key or "")
    if key == "" then return "" end
    if KEY_DISPLAY[key] ~= nil then return KEY_DISPLAY[key] end
    if #key == 1 then return string.upper(key) end
    if string.sub(key, 1, 1) == "f" and tonumber(string.sub(key, 2)) ~= nil then
        return string.upper(key)
    end
    return key
end

local function key_label(action)
    if type(get_key) ~= "function" then
        return "[]"
    end
    local ok, info = pcall(get_key, action)
    if not ok or type(info) ~= "table" then
        return "[]"
    end
    if info[action] ~= nil and type(info[action]) == "table" then
        info = info[action]
    end
    local keys = info.key_user or info.key
    if type(keys) ~= "table" then
        keys = { keys }
    end
    local out = {}
    for i = 1, #keys do
        local label = display_key_name(keys[i])
        if label ~= "" then
            out[#out + 1] = "[" .. label .. "]"
        end
    end
    if #out == 0 then return "[]" end
    return table.concat(out, "/")
end


local function replace_prompt_keys(text)
    text = tostring(text or "")
    text = string.gsub(text, "%[Y%]", key_label("confirm_yes"))
    text = string.gsub(text, "%[N%]", key_label("confirm_no"))
    text = string.gsub(text, "%[Q%]/%[ESC%]", key_label("quit_action"))
    return text
end

local function controls_text()
    return table.concat({
        key_label("pick_1") .. "/" .. key_label("pick_2") .. "/" .. key_label("pick_3") .. "/" .. key_label("pick_4") .. " " .. tr("game.color_memory.action.pick_1"),
        key_label("confirm") .. " " .. tr("game.color_memory.action.confirm"),
        key_label("remove_last") .. " " .. tr("game.color_memory.action.remove_last"),
        key_label("restart") .. " " .. tr("game.color_memory.action.restart"),
        key_label("quit_action") .. " " .. tr("game.color_memory.action.quit")
    }, "  ")
end

local function restart_quit_controls_text()
    return key_label("restart") .. " " .. tr("game.color_memory.action.restart")
        .. "  " .. key_label("quit_action") .. " " .. tr("game.color_memory.action.quit")
end


local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
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

local function random_index(n)
    if type(n) ~= "number" or n <= 0 then
        return 0
    end
    return random(n - 1)
end

local function kill_sequence_timer()
    if state.sequence_timer_id ~= nil and type(timer_kill) == "function" then
        pcall(timer_kill, state.sequence_timer_id)
    end
    state.sequence_timer_id = nil
end

local function start_sequence_timer(delay_ms)
    kill_sequence_timer()
    state.sequence_timer_id = timer_create(math.max(1, delay_ms or 1), "color_memory_sequence")
    timer_start(state.sequence_timer_id)
end

local function sequence_timer_completed()
    return state.sequence_timer_id ~= nil and is_timer_completed(state.sequence_timer_id)
end

local function normalize_key(key)
    if key == nil then
        return ""
    end
    if type(key) == "string" then
        return string.lower(key)
    end
    if type(key) == "table" then
        if key.type == "quit" then
            return "quit_action"
        end
        if key.type == "key" and type(key.name) == "string" then
            return string.lower(key.name)
        end
        if key.type == "action" and type(key.name) == "string" then
            local map = {
                pick_1 = "pick_1",
                pick_2 = "pick_2",
                pick_3 = "pick_3",
                pick_4 = "pick_4",
                confirm = "confirm",
                confirm_yes = "confirm_yes",
                confirm_no = "confirm_no",
                remove_last = "remove_last",
                restart = "restart",
                quit_action = "quit_action"
            }
            return map[key.name] or ""
        end
    end
    return tostring(key):lower()
end

local function flush_input_buffer()
end

local function elapsed_seconds()
    local end_frame = state.end_frame
    if end_frame == nil then
        end_frame = state.frame
    end
    return math.floor((end_frame - state.start_frame) / FPS)
end

local function format_duration(sec)
    local h = math.floor(sec / 3600)
    local m = math.floor((sec % 3600) / 60)
    local s = sec % 60
    return string.format("%02d:%02d:%02d", h, m, s)
end

local function fill_line(y, width)
    draw_text(1, y, string.rep(" ", width), "white", "black")
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then
        return
    end
    canvas_fill_rect(math.max(0, x - 1), math.max(0, y - 1), w, h, " ", nil, bg or "black")
end

local function draw_outer_frame(x, y, w, h)
    draw_text(x, y, FRAME_TL .. string.rep(FRAME_HL, w - 2) .. FRAME_TR, "white", "black")
    for i = 1, h - 2 do
        draw_text(x, y + i, FRAME_VL, "white", "black")
        draw_text(x + w - 1, y + i, FRAME_VL, "white", "black")
    end
    draw_text(x, y + h - 1, FRAME_BL .. string.rep(FRAME_HL, w - 2) .. FRAME_BR, "white", "black")
end

local function draw_color_fill_slot(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    fill_rect(x, y, BOX_W, BOX_H, "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
end

local function draw_highlight_box(x, y, color_idx)
    local bg = COLORS[color_idx].bg
    draw_text(x, y, BOX_TL .. string.rep(BOX_HL, 2) .. BOX_TR, "white", "black")
    draw_text(x, y + 1, BOX_VL, "white", "black")
    draw_text(x + 1, y + 1, "  ", "white", bg)
    draw_text(x + 3, y + 1, BOX_VL, "white", "black")
    draw_text(x, y + 2, BOX_BL .. string.rep(BOX_HL, 2) .. BOX_BR, "white", "black")
end

local function load_best_record()
    local ok, data = pcall(get_best_score)
    if not ok or type(data) ~= "table" then
        return
    end
    local bs = tonumber(data.score or data.best_score)
    local bt = tonumber(data.time_sec or data.best_time_sec)
    if bs ~= nil and bs >= 0 then
        state.best_score = math.floor(bs)
    end
    if bt ~= nil and bt >= 0 then
        state.best_time_sec = math.floor(bt)
    end
end

local function commit_stats_if_needed()
    if state.committed then
        return
    end
    local dur = elapsed_seconds()
    local changed = false
    if state.score > state.best_score then
        state.best_score = state.score
        changed = true
    end
    if dur > state.best_time_sec then
        state.best_time_sec = dur
        changed = true
    end
    if changed and type(request_save_best_score) == "function" then
        pcall(request_save_best_score)
    end
    state.committed = true
end

local function centered_x(text, left_x, right_x)
    local width = text_width(text)
    local x = left_x + math.floor(((right_x - left_x + 1) - width) / 2)
    if x < left_x then
        x = left_x
    end
    if x > right_x - width + 1 then
        x = math.max(left_x, right_x - width + 1)
    end
    return x
end

local function minimum_required_size()
    local controls = controls_text()
    local controls_w = min_width_for_lines(controls, 3, 40)

    local best_line = tr("game.color_memory.best_score") .. " 99999  "
        .. tr("game.color_memory.best_time") .. " 00:00:00"
    local curr_line = tr("game.color_memory.time") .. " 00:00:00  "
        .. tr("game.color_memory.score") .. " 99999"

    local info_w = math.max(
        text_width(replace_prompt_keys(tr("game.color_memory.confirm_restart"))),
        text_width(replace_prompt_keys(tr("game.color_memory.confirm_exit"))),
        text_width(tr("game.color_memory.lose_banner") .. " " .. restart_quit_controls_text())
    )

    local boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local frame_w = math.max(48, boxes_w + 10, info_w + 2)
    local min_w = math.max(frame_w + 2, controls_w + 2, text_width(best_line) + 2, text_width(curr_line) + 2)
    local min_h = FRAME_H + 7
    return min_w, min_h
end

local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
    clear()
    local lines = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), min_w, min_h),
        string.format("%s: %dx%d", tr("warning.current"), term_w, term_h),
        tr("warning.enlarge_hint"),
        tr("warning.back_to_game_list_hint")
    }
    local top = math.floor((term_h - #lines) / 2)
    if top < 1 then
        top = 1
    end
    for i = 1, #lines do
        local x = math.floor((term_w - text_width(lines[i])) / 2)
        if x < 1 then
            x = 1
        end
        draw_text(x, top + i - 1, lines[i], "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        local resized = (term_w ~= state.last_term_w) or (term_h ~= state.last_term_h)
        state.last_term_w = term_w
        state.last_term_h = term_h
        if state.size_warning_active or resized then
            clear()
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
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end
    state.size_warning_active = true
    return false
end

local function frame_geometry()
    local term_w, term_h = terminal_size()
    local frame_w = math.max(48, 4 * BOX_W + 3 * BOX_GAP + 10)
    local top_h = 3
    local bottom_h = 3
    local block_h = top_h + FRAME_H + bottom_h

    local top = math.floor((term_h - block_h) / 2) + 1
    if top < 1 then
        top = 1
    end

    local x = math.floor((term_w - frame_w) / 2)
    if x < 1 then
        x = 1
    end

    return {
        best_y = top,
        current_y = top + 1,
        info_y = top + 2,
        game_x = x,
        game_y = top + 3,
        frame_w = frame_w,
        frame_h = FRAME_H,
        controls_y = top + 3 + FRAME_H + 1,
        term_w = term_w
    }
end

local function game_inner(g)
    return g.game_x + 1, g.game_y + 1, g.frame_w - 2
end

local function format_round_text()
    local tmpl = tr("game.color_memory.round")
    if string.find(tmpl, "{n}", 1, true) ~= nil then
        return string.gsub(tmpl, "{n}", tostring(state.round))
    end
    if tmpl == "第几局" then
        return "第" .. tostring(state.round) .. "局"
    end
    return tmpl .. " " .. tostring(state.round)
end

local function draw_show_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    fill_rect(inner_x, inner_y, inner_w, 7, "black")

    local round_text = format_round_text()
    draw_text(centered_x(round_text, inner_x, inner_x + inner_w - 1), inner_y, round_text, "yellow", "black")

    local total_boxes_w = 4 * BOX_W + 3 * BOX_GAP
    local row_x = inner_x + math.floor((inner_w - total_boxes_w) / 2)
    local show_y = inner_y + 2
    for i = 1, 4 do
        local bx = row_x + (i - 1) * (BOX_W + BOX_GAP)
        if state.highlight_idx == i then
            draw_highlight_box(bx, show_y, i)
        else
            draw_color_fill_slot(bx, show_y, i)
        end
    end

    local status_text = ""
    if state.phase == "show" then
        status_text = tr("game.color_memory.status_observe")
    elseif state.phase == "input" then
        status_text = tr("game.color_memory.status_input")
    end
    draw_text(centered_x(status_text, inner_x, inner_x + inner_w - 1), inner_y + 6, status_text, "dark_gray", "black")
end

local function draw_input_section(g)
    local inner_x = g.game_x + 1
    local inner_y = g.game_y + 1
    local inner_w = g.frame_w - 2
    local input_y = inner_y + 7
    fill_rect(inner_x, input_y, inner_w, 3, "black")

    local max_slots = math.max(1, math.floor((inner_w + INPUT_GAP) / (BOX_W + INPUT_GAP)))
    local start_idx = 1
    if #state.input_colors > max_slots then
        start_idx = #state.input_colors - max_slots + 1
    end
    local visible = #state.input_colors - start_idx + 1
    local input_w = visible * BOX_W + math.max(0, visible - 1) * INPUT_GAP
    local input_x = inner_x + math.floor((inner_w - input_w) / 2)

    for i = start_idx, #state.input_colors do
        local slot = i - start_idx
        local bx = input_x + slot * (BOX_W + INPUT_GAP)
        draw_color_fill_slot(bx, input_y, state.input_colors[i])
    end

    draw_text(
        g.game_x,
        g.game_y + g.frame_h - 1,
        FRAME_BL .. string.rep(FRAME_HL, g.frame_w - 2) .. FRAME_BR,
        "white",
        "black"
    )
end

local function draw_header(g)
    fill_line(g.best_y, g.term_w)
    fill_line(g.current_y, g.term_w)
    fill_line(g.info_y, g.term_w)

    local best_line = tr("game.color_memory.best_score") .. ": " .. tostring(state.best_score)
        .. "  "
        .. tr("game.color_memory.best_time") .. ": " .. format_duration(state.best_time_sec)
    draw_text(centered_x(best_line, 1, g.term_w), g.best_y, best_line, "dark_gray", "black")

    local current_line = tr("game.color_memory.time") .. ": " .. format_duration(elapsed_seconds())
        .. "  "
        .. tr("game.color_memory.score") .. ": " .. tostring(state.score)
    draw_text(centered_x(current_line, 1, g.term_w), g.current_y, current_line, "light_cyan", "black")

    local info = ""
    local info_color = "yellow"
    if state.confirm_mode == "restart" then
        info = replace_prompt_keys(tr("game.color_memory.confirm_restart"))
    elseif state.confirm_mode == "exit" then
        info = replace_prompt_keys(tr("game.color_memory.confirm_exit"))
    elseif state.lost then
        info = tr("game.color_memory.lose_banner") .. " " .. restart_quit_controls_text()
        info_color = "red"
    end
    if info ~= "" then
        draw_text(centered_x(info, 1, g.term_w), g.info_y, info, info_color, "black")
    end
end

local function draw_controls(g)
    local controls = controls_text()
    local lines = wrap_words(controls, math.max(10, g.term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end

    for i = 0, 2 do
        fill_line(g.controls_y + i, g.term_w)
    end

    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end

    for i = 1, #lines do
        local line = lines[i]
        draw_text(centered_x(line, 1, g.term_w), g.controls_y + offset + i - 1, line, "white", "black")
    end
end

local function render_full(g)
    draw_header(g)
    draw_outer_frame(g.game_x, g.game_y, g.frame_w, g.frame_h)
    local inner_x, inner_y, inner_w = game_inner(g)
    fill_rect(inner_x, inner_y, inner_w, g.frame_h - 2, "black")
    draw_show_section(g)
    draw_input_section(g)
    draw_controls(g)
end

local function generate_sequence(round_no)
    local out = {}
    for _ = 1, round_no do
        out[#out + 1] = random_index(4) + 1
    end
    return out
end

local function start_sequence_animation()
    state.phase = "show"
    state.highlight_idx = 0
    state.sequence_anim = {
        step = "initial_off",
        index = 1
    }
    start_sequence_timer(SHOW_OFF_MS)
    state.dirty = true
end

local function start_next_round()
    state.input_colors = {}
    state.sequence = generate_sequence(state.round)
    start_sequence_animation()
end

local function start_new_run()
    state.score = 0
    state.round = 1
    state.sequence = {}
    state.input_colors = {}
    state.highlight_idx = 0

    state.phase = "input"
    state.lost = false
    state.confirm_mode = nil
    state.committed = false

    state.start_frame = state.frame
    state.end_frame = nil
    state.sequence_anim = nil
    state.dirty = true

    start_next_round()
end

local function mark_lost()
    if state.lost then
        return
    end
    state.lost = true
    state.phase = "lost"
    state.end_frame = state.frame
    state.confirm_mode = nil
    state.sequence_anim = nil
    kill_sequence_timer()
    commit_stats_if_needed()
    state.dirty = true
end

local function on_round_success()
    state.score = state.score + state.round
    state.round = state.round + 1
    start_next_round()
end

local function refresh_dirty_flags()
    local elapsed = elapsed_seconds()
    if elapsed ~= state.last_elapsed_sec then
        state.last_elapsed_sec = elapsed
        state.dirty = true
    end
end

local function advance_sequence_animation(dt_ms)
    local anim = state.sequence_anim
    if anim == nil or not sequence_timer_completed() then
        return
    end

    if anim.step == "initial_off" or anim.step == "off" then
        if anim.index > #state.sequence then
            flush_input_buffer()
            state.phase = "input"
            state.highlight_idx = 0
            state.sequence_anim = nil
            kill_sequence_timer()
            state.dirty = true
            return
        end
        state.highlight_idx = state.sequence[anim.index]
        anim.step = "on"
        start_sequence_timer(SHOW_ON_MS)
        state.dirty = true
    else
        state.highlight_idx = 0
        anim.index = anim.index + 1
        anim.step = "off"
        start_sequence_timer(SHOW_OFF_MS)
        state.dirty = true
    end
end

local function handle_confirm_key(key)
    if key == "confirm_yes" then
        if state.confirm_mode == "restart" then
            commit_stats_if_needed()
            start_new_run()
            return "changed"
        end
        if state.confirm_mode == "exit" then
            commit_stats_if_needed()
            return "exit"
        end
    elseif key == "confirm_no" or key == "quit_action" then
        state.confirm_mode = nil
        state.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end

    if state.confirm_mode ~= nil then
        return handle_confirm_key(key)
    end

    if state.lost then
        if key == "restart" then
            start_new_run()
            return "changed"
        end
        if key == "quit_action" then
            commit_stats_if_needed()
            return "exit"
        end
        return "none"
    end

    if key == "quit_action" then
        state.confirm_mode = "exit"
        state.dirty = true
        return "changed"
    end
    if key == "restart" then
        state.confirm_mode = "restart"
        state.dirty = true
        return "changed"
    end

    if state.phase ~= "input" then
        return "none"
    end

    if key == "remove_last" then
        if #state.input_colors > 0 then
            table.remove(state.input_colors)
            state.dirty = true
        end
        return "changed"
    end

    local color_idx = nil
    if key == "pick_1" then color_idx = 1 end
    if key == "pick_2" then color_idx = 2 end
    if key == "pick_3" then color_idx = 3 end
    if key == "pick_4" then color_idx = 4 end

    if color_idx ~= nil then
        state.input_colors[#state.input_colors + 1] = color_idx
        state.dirty = true
        return "changed"
    end

    if key == "confirm" then
        local ok = #state.input_colors == #state.sequence
        if ok then
            for i = 1, #state.sequence do
                if state.input_colors[i] ~= state.sequence[i] then
                    ok = false
                    break
                end
            end
        end

        if not ok then
            mark_lost()
        else
            on_round_success()
        end
        return "changed"
    end

    return "none"
end

local function runtime_init_game(saved_state)
    clear()
    flush_input_buffer()
    local w, h = terminal_size()
    state.last_term_w, state.last_term_h = w, h
    state.frame = 0
    state.last_elapsed_sec = -1
    state.size_warning_active = false
    load_best_record()
    start_new_run()
    return state
end

local function handle_tick(dt_ms)
    if not ensure_terminal_size_ok() then
        state.frame = state.frame + 1
        refresh_dirty_flags()
        return
    end
    advance_sequence_animation(dt_ms or FRAME_MS)
    refresh_dirty_flags()
    state.frame = state.frame + 1
end

local function runtime_handle_event(state_arg, event)
    state = state_arg or state

    if event ~= nil and event.type == "resize" then
        state.last_term_w = event.width or state.last_term_w
        state.last_term_h = event.height or state.last_term_h
        state.dirty = true
        return state
    end

    if event ~= nil and event.type == "tick" then
        handle_tick(event.dt_ms)
        return state
    end

    local key = normalize_key(event)
    if not ensure_terminal_size_ok() then
        if key == "quit_action" then
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
        end
        return state
    end

    local action = handle_input(key)
    if action == "exit" then
        if type(request_exit) == "function" then
            pcall(request_exit)
        end
    end
    return state
end

local function runtime_render(state_arg)
    state = state_arg or state
    if not ensure_terminal_size_ok() then
        return
    end
    render_full(frame_geometry())
end

local function runtime_save_best_score(state_arg)
    state = state_arg or state
    if state.best_score <= 0 and state.best_time_sec <= 0 then
        return { best_string = "game.color_memory.best_none_block" }
    end
    return {
        best_string = "game.color_memory.best_block",
        score = state.best_score,
        time_sec = state.best_time_sec,
        time = format_duration(state.best_time_sec)
    }
end


local function runtime_exit_game(state_arg)
    state = state_arg or state
    kill_sequence_timer()
    commit_stats_if_needed()
    return state
end

local Runtime = {
    init_game = runtime_init_game,
    handle_event = runtime_handle_event,
    render = runtime_render,
    exit_game = runtime_exit_game,
    save_best_score = runtime_save_best_score,
}

_G.COLOR_MEMORY_RUNTIME = Runtime
return Runtime
