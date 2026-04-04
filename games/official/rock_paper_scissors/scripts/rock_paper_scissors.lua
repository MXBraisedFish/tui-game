local FRAME_MS = 16

local CHOICES = {
    [1] = { symbol = "Y", key = "game.rock_paper_scissors.choice.scissors", fallback = "Scissors" },
    [2] = { symbol = "O", key = "game.rock_paper_scissors.choice.rock", fallback = "Rock" },
    [3] = { symbol = "U", key = "game.rock_paper_scissors.choice.paper", fallback = "Paper" }
}

local state = {
    player_pick = nil,
    ai_pick = nil,
    current_streak = 0,
    best_streak = 0,
    loss_streak = 0,
    message = "",
    message_color = "dark_gray",
    dirty = true,
    last_term_w = 0,
    last_term_h = 0,
    size_warning_active = false,
    last_warn_term_w = 0,
    last_warn_term_h = 0,
    last_warn_min_w = 0,
    last_warn_min_h = 0
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

local function text_width(text)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, text)
        if ok and type(w) == "number" then
            return w
        end
    end
    return #text
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
            return "esc"
        end
        if key.type == "key" and type(key.name) == "string" then
            return string.lower(key.name)
        end
        if key.type == "action" and type(key.name) == "string" then
            local map = {
                pick_scissors = "1",
                pick_rock = "2",
                pick_paper = "3",
                restart = "r",
                quit_action = "q",
            }
            return map[key.name] or ""
        end
    end
    return tostring(key):lower()
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

local function wrap_words(text, max_width)
    if max_width <= 1 then
        return { text }
    end
    local lines = {}
    local current = ""
    local had = false
    for token in string.gmatch(text, "%S+") do
        had = true
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
    if not had then
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

local function centered_x(text, area_x, area_w)
    local x = area_x + math.floor((area_w - text_width(text)) / 2)
    if x < area_x then
        x = area_x
    end
    return x
end

local function draw_center_split_line(y, left_text, right_text, fg, bg)
    local term_w = select(1, terminal_size())
    local center_x = math.floor(term_w / 2)
    local left_w = text_width(left_text)
    local left_x = center_x - 2 - left_w
    local right_x = center_x + 3
    if left_x < 1 then
        left_x = 1
    end
    draw_text(left_x, y, left_text, fg, bg)
    draw_text(center_x, y, "|", fg, bg)
    draw_text(right_x, y, right_text, fg, bg)
end

local function save_best()
    if type(save_data) == "function" then
        pcall(save_data, "rock_paper_scissors_best", { best_streak = state.best_streak })
    end
    if type(request_refresh_best_score) == "function" then
        pcall(request_refresh_best_score)
    end
end

local function load_best()
    if type(load_data) ~= "function" then
        return
    end
    local ok, data = pcall(load_data, "rock_paper_scissors_best")
    if not ok or type(data) ~= "table" then
        return
    end
    local v = tonumber(data.best_streak)
    if v ~= nil and v >= 0 then
        state.best_streak = math.floor(v)
    end
end

local function choice_text(index)
    if index == nil or CHOICES[index] == nil then
        return "-"
    end
    local info = CHOICES[index]
    return info.symbol .. " " .. tr(info.key)
end

local function resolve_round(player_idx, ai_idx)
    if player_idx == ai_idx then
        return 0
    end
    if (player_idx == 1 and ai_idx == 3)
        or (player_idx == 2 and ai_idx == 1)
        or (player_idx == 3 and ai_idx == 2) then
        return 1
    end
    return -1
end

local function player_win_bias(loss_streak)
    if loss_streak <= 0 then
        return 0
    end
    if loss_streak >= 7 then
        return 1
    end
    return loss_streak / 8
end

local function pick_ai_choice(player_idx)
    local bias = player_win_bias(state.loss_streak)
    if bias > 0 then
        local roll = (random(1000) + 1) / 1000
        if roll <= bias then
            if player_idx == 1 then return 3 end
            if player_idx == 2 then return 1 end
            return 2
        end
    end
    return random(3) + 1
end

local function play_round(player_idx)
    local ai_idx = pick_ai_choice(player_idx)
    state.player_pick = player_idx
    state.ai_pick = ai_idx

    local result = resolve_round(player_idx, ai_idx)
    local controls = tr("game.rock_paper_scissors.result_controls")
    if result > 0 then
        state.current_streak = state.current_streak + 1
        state.loss_streak = 0
        if state.current_streak > state.best_streak then
            state.best_streak = state.current_streak
            save_best()
        end
        state.message = tr("game.rock_paper_scissors.win_banner") .. " " .. controls
        state.message_color = "green"
    elseif result < 0 then
        state.current_streak = 0
        state.loss_streak = state.loss_streak + 1
        state.message = tr("game.rock_paper_scissors.lose_banner") .. " " .. controls
        state.message_color = "red"
    else
        state.current_streak = 0
        state.message = tr("game.rock_paper_scissors.draw_banner") .. " " .. controls
        state.message_color = "yellow"
    end
    state.dirty = true
end

local function reset_round()
    state.player_pick = nil
    state.ai_pick = nil
    state.current_streak = 0
    state.loss_streak = 0
    state.message = tr("game.rock_paper_scissors.ready_banner")
    state.message_color = "dark_gray"
    state.dirty = true
end

local function minimum_required_size()
    local top1 = tr("game.rock_paper_scissors.best_streak") .. ": 9999"
    local top2 = tr("game.rock_paper_scissors.current_streak") .. ": 9999"
    local header = tr("game.rock_paper_scissors.player") .. "   |   " .. tr("game.rock_paper_scissors.system")
    local picks = "Y " .. tr("game.rock_paper_scissors.choice.scissors") .. "   |   O " .. tr("game.rock_paper_scissors.choice.rock")
    local msg = tr("game.rock_paper_scissors.win_banner") .. " " .. tr("game.rock_paper_scissors.result_controls")
    local controls = tr("game.rock_paper_scissors.controls")
    local controls_w = min_width_for_lines(controls, 3, 24)
    local min_w = math.max(text_width(top1), text_width(top2), text_width(header), text_width(picks), text_width(msg), controls_w) + 2
    local min_h = 10
    return min_w, min_h
end

local function draw_terminal_size_warning(term_w, term_h, min_w, min_h)
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
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then
            x = 1
        end
        draw_text(x, top + i - 1, line, "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local term_w, term_h = terminal_size()
    local min_w, min_h = minimum_required_size()

    if term_w >= min_w and term_h >= min_h then
        if state.size_warning_active then
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
        clear()
        draw_terminal_size_warning(term_w, term_h, min_w, min_h)
        state.last_warn_term_w = term_w
        state.last_warn_term_h = term_h
        state.last_warn_min_w = min_w
        state.last_warn_min_h = min_h
    end

    state.size_warning_active = true
    return false
end

local function draw_controls(y)
    local controls = tr("game.rock_paper_scissors.controls")
    local term_w = terminal_size()
    local lines = wrap_words(controls, math.max(10, term_w - 2))
    if #lines > 3 then
        lines = { lines[1], lines[2], lines[3] }
    end
    for i = 1, 3 do
        draw_text(1, y + i - 1, string.rep(" ", term_w), "white", "black")
    end
    local offset = 0
    if #lines < 3 then
        offset = math.floor((3 - #lines) / 2)
    end
    for i = 1, #lines do
        local line = lines[i]
        local x = math.floor((term_w - text_width(line)) / 2)
        if x < 1 then
            x = 1
        end
        draw_text(x, y + offset + i - 1, line, "white", "black")
    end
end

local function render_scene()
    local term_w, term_h = terminal_size()
    local total_h = 8
    local y0 = math.floor((term_h - total_h) / 2) + 1
    if y0 < 1 then
        y0 = 1
    end

    clear()

    local top1 = tr("game.rock_paper_scissors.best_streak") .. ": " .. tostring(state.best_streak)
    local top2 = tr("game.rock_paper_scissors.current_streak") .. ": " .. tostring(state.current_streak)
    draw_text(centered_x(top1, 1, term_w), y0, top1, "dark_gray", "black")
    draw_text(centered_x(top2, 1, term_w), y0 + 1, top2, "light_cyan", "black")

    if state.message ~= "" then
        draw_text(centered_x(state.message, 1, term_w), y0 + 2, state.message, state.message_color, "black")
    end

    local left_header = tr("game.rock_paper_scissors.player")
    local right_header = tr("game.rock_paper_scissors.system")
    local left_pick = choice_text(state.player_pick)
    local right_pick = choice_text(state.ai_pick)
    draw_center_split_line(y0 + 4, left_header, right_header, "white", "black")
    draw_center_split_line(y0 + 5, left_pick, right_pick, "white", "black")

    draw_controls(y0 + 7)
end

local function handle_input(key)
    if key == nil or key == "" then
        return "none"
    end
    if key == "q" or key == "esc" then
        return "exit"
    end
    if key == "r" then
        reset_round()
        return "changed"
    end
    if key == "1" or key == "2" or key == "3" then
        play_round(tonumber(key))
        return "changed"
    end
    return "none"
end

local function sync_terminal_resize()
    local w, h = terminal_size()
    if w ~= state.last_term_w or h ~= state.last_term_h then
        state.last_term_w = w
        state.last_term_h = h
        state.dirty = true
    end
end

function init_game()
    local w, h = terminal_size()
    state.last_term_w = w
    state.last_term_h = h
    load_best()
    reset_round()
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
    return state
end

function handle_event(state_arg, event)
    state = state_arg or state
    local key = normalize_key(event)
    if event ~= nil and event.type == "tick" then
        key = ""
    end

    if ensure_terminal_size_ok() then
        local action = handle_input(key)
        if action == "exit" then
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
            return state
        end
        sync_terminal_resize()
        if action == "changed" then
            state.dirty = true
        end
    else
        if key == "q" or key == "esc" then
            if type(request_exit) == "function" then
                pcall(request_exit)
            end
            return state
        end
    end

    return state
end

function render(state_arg)
    state = state_arg or state
    if ensure_terminal_size_ok() then
        render_scene()
        state.dirty = false
    end
end

function best_score(state_arg)
    state = state_arg or state
    if type(state.best_streak) ~= "number" or state.best_streak <= 0 then
        return nil
    end
    return {
        best_string = "game.rock_paper_scissors.best_block",
        streak = state.best_streak,
    }
end
