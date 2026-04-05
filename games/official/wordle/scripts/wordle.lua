local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    request_exit()
end

local FPS, FRAME_MS = 60, 16
local MAX_ATTEMPTS = 6

local S = {
    words = {},
    secret = "",
    word_len = 5,
    guesses = {},
    marks = {},
    input = "",
    mode = "input",
    confirm = nil,
    settled = false,
    won = false,
    streak = 0,
    best_time_sec = 0,
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    toast = nil,
    toast_color = "green",
    toast_until = 0,
    dirty = true,
    time_dirty = false,
    last_elapsed = -1,
    last_time_line = "",
    tw = 0,
    th = 0,
    warn = false,
    lw = 0,
    lh = 0,
    lmw = 0,
    lmh = 0,
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

local function normalize_key(event)
    if event == nil then return "" end
    if type(event) == "string" then return string.lower(event) end
    if type(event) ~= "table" then return tostring(event):lower() end
    if event.type == "quit" then return "esc" end
    if event.type == "key" and type(event.name) == "string" then return string.lower(event.name) end
    if event.type ~= "action" or type(event.name) ~= "string" then return "" end
    local map = {
        submit = "enter",
        backspace = "backspace",
        toggle_mode = "tab",
    }
    return map[event.name] or string.lower(event.name)
end

local function text_width(t)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, t)
        if ok and type(w) == "number" then return w end
    end
    return #t
end

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

local function elapsed_seconds()
    local ef = S.end_frame or S.frame
    return math.max(0, math.floor((ef - S.start_frame) / FPS))
end

local function format_duration(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

local function rand_int(n)
    if n <= 0 then return 0 end
    if type(random) == "function" then return random(n) end
    return math.random(0, n - 1)
end

local function centered_x(text, l, r)
    local x = l + math.floor(((r - l + 1) - text_width(text)) / 2)
    if x < l then x = l end
    return x
end

local function clear_line(y, tw)
    draw_text(1, y, string.rep(" ", tw), "white", "black")
end

local function load_words()
    local words, seen = {}, {}
    if type(read_json) == "function" then
        local ok, data = pcall(read_json, "data/word.json")
        if ok and type(data) == "table" then
            for i = 1, #data do
                local value = data[i]
                if type(value) == "string" then
                    local lw = string.lower(value)
                    if #lw >= 2 and lw:match("^[a-z]+$") and not seen[lw] then
                        seen[lw] = true
                        words[#words + 1] = lw
                    end
                end
            end
        end
    end
    if #words == 0 then
        words = { "apple", "water", "green", "house", "sound", "light", "story", "music", "table", "clock" }
    end
    S.words = words
end

local function pick_word()
    if #S.words == 0 then load_words() end
    local idx = rand_int(#S.words) + 1
    local w = S.words[idx]
    S.secret = string.lower(w)
    S.word_len = #S.secret
end

local function char_at(str, i)
    return string.sub(str, i, i)
end

local function evaluate_guess(secret, guess)
    local n = #secret
    local marks, pool = {}, {}
    for i = 1, n do
        local c = char_at(secret, i)
        pool[c] = (pool[c] or 0) + 1
        marks[i] = "absent"
    end
    for i = 1, n do
        local g = char_at(guess, i)
        local s = char_at(secret, i)
        if g == s then
            marks[i] = "correct"
            pool[g] = (pool[g] or 0) - 1
        end
    end
    for i = 1, n do
        if marks[i] ~= "correct" then
            local g = char_at(guess, i)
            local cnt = pool[g] or 0
            if cnt > 0 then
                marks[i] = "present"
                pool[g] = cnt - 1
            else
                marks[i] = "absent"
            end
        end
    end
    return marks
end

local function save_best_time()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_best_time_sec", S.best_time_sec)
    end
end

local function save_streak()
    if type(save_data) == "function" then
        pcall(save_data, "wordle_streak", S.streak)
    end
end

local function load_meta()
    if type(load_data) ~= "function" then return end
    local ok1, bt = pcall(load_data, "wordle_best_time_sec")
    if ok1 and type(bt) == "number" and bt > 0 then
        S.best_time_sec = math.floor(bt)
    end
    local ok2, st = pcall(load_data, "wordle_streak")
    if ok2 and type(st) == "number" and st >= 0 then
        S.streak = math.floor(st)
    end
end

local function save_slot()
    local payload = {
        secret = S.secret,
        guesses = S.guesses,
        input = S.input,
        mode = S.mode,
        streak = S.streak,
        best_time_sec = S.best_time_sec,
        elapsed_sec = elapsed_seconds(),
        settled = S.settled,
        won = S.won,
    }
    local ok = false
    if type(save_game_slot) == "function" then
        local s, ret = pcall(save_game_slot, "wordle", payload)
        ok = s and ret ~= false
    elseif type(save_data) == "function" then
        local s, ret = pcall(save_data, "wordle_slot", payload)
        ok = s and ret ~= false
    end
    S.toast = ok and tr("game.wordle.saved") or tr("game.wordle.saved")
    S.toast_color = "green"
    S.toast_until = S.frame + FPS * 2
end

local function load_slot_if_continue()
    if type(get_launch_mode) ~= "function" then return false end
    local ok_mode, mode = pcall(get_launch_mode)
    mode = ok_mode and string.lower(tostring(mode)) or "new"
    if mode ~= "continue" then return false end

    local ok, slot = false, nil
    if type(load_game_slot) == "function" then
        local s, ret = pcall(load_game_slot, "wordle")
        ok = s and ret ~= nil
        slot = ret
    elseif type(load_data) == "function" then
        local s, ret = pcall(load_data, "wordle_slot")
        ok = s and ret ~= nil
        slot = ret
    end
    if not ok or type(slot) ~= "table" then return false end
    if type(slot.secret) ~= "string" or slot.secret == "" then return false end
    if slot.settled then return false end

    S.secret = string.lower(slot.secret)
    S.word_len = #S.secret
    S.guesses = {}
    S.marks = {}
    if type(slot.guesses) == "table" then
        for i = 1, #slot.guesses do
            local g = tostring(slot.guesses[i]):lower()
            if #g == S.word_len then
                S.guesses[#S.guesses + 1] = g
                S.marks[#S.marks + 1] = evaluate_guess(S.secret, g)
            end
        end
    end
    S.input = type(slot.input) == "string" and string.lower(slot.input) or ""
    if #S.input > S.word_len then
        S.input = string.sub(S.input, 1, S.word_len)
    end
    S.mode = (slot.mode == "action") and "action" or "input"
    if type(slot.streak) == "number" and slot.streak >= 0 then
        S.streak = math.floor(slot.streak)
    end
    if type(slot.best_time_sec) == "number" and slot.best_time_sec > 0 then
        S.best_time_sec = math.floor(slot.best_time_sec)
    end
    local elapsed = 0
    if type(slot.elapsed_sec) == "number" and slot.elapsed_sec >= 0 then
        elapsed = math.floor(slot.elapsed_sec)
    end
    S.start_frame = S.frame - elapsed * FPS
    S.end_frame = nil
    S.settled = false
    S.won = false
    return true
end

local function new_round(preserve_streak)
    if not preserve_streak then
        S.streak = 0
        save_streak()
    end
    pick_word()
    S.guesses = {}
    S.marks = {}
    S.input = ""
    S.mode = "input"
    S.confirm = nil
    S.settled = false
    S.won = false
    S.start_frame = S.frame
    S.end_frame = nil
    S.toast = nil
    S.dirty = true
end

local function settle(win)
    S.settled = true
    S.won = win
    S.end_frame = S.frame
    if win then
        S.streak = S.streak + 1
        local t = elapsed_seconds()
        if S.best_time_sec <= 0 or t < S.best_time_sec then
            S.best_time_sec = t
            save_best_time()
        end
        save_streak()
        if type(update_game_stats) == "function" then
            pcall(update_game_stats, "wordle", S.streak, t)
        end
    else
        S.streak = 0
        save_streak()
    end
end

local function status_text()
    if S.confirm == "restart" then
        return tr("game.wordle.confirm_restart"), "yellow"
    end
    if S.confirm == "exit" then
        return tr("game.wordle.confirm_exit"), "yellow"
    end
    if S.settled then
        if S.won then
            return tr("game.wordle.win") .. "  " .. tr("game.wordle.result_controls"), "green"
        end
        return tr("game.wordle.lose") .. "  " .. tr("game.wordle.result_controls"), "red"
    end
    if S.toast and S.frame <= S.toast_until then
        return S.toast, S.toast_color
    end
    if S.mode == "action" then
        return tr("game.wordle.mode_action"), "yellow"
    end
    return tr("game.wordle.mode_input"), "dark_gray"
end

local function controls_text()
    if S.settled then
        return tr("game.wordle.controls_result")
    end
    if S.mode == "action" then
        return tr("game.wordle.controls_action")
    end
    return tr("game.wordle.controls_input")
end

local function minimum_required_size()
    local cw = text_width(controls_text())
    local row_w = text_width("-> ") + S.word_len * 2 + 2
    local top_w = math.max(
        text_width(tr("game.wordle.best_time") .. " " .. format_duration(0)),
        text_width(tr("game.wordle.time") .. " " .. format_duration(0) .. "  " .. tr("game.wordle.streak") .. " 999")
    )
    local need_w = math.max(60, cw + 2, row_w + 8, top_w + 2)
    return need_w, 14
end

local function draw_terminal_size_warning(tw, th, mw, mh)
    clear()
    local ls = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), mw, mh),
        string.format("%s: %dx%d", tr("warning.current"), tw, th),
        tr("warning.enlarge_hint"),
        tr("warning.back_to_game_list_hint")
    }
    local top = math.floor((th - #ls) / 2)
    if top < 1 then top = 1 end
    for i = 1, #ls do
        draw_text(centered_x(ls[i], 1, tw), top + i - 1, ls[i], "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local tw, th = terminal_size()
    local mw, mh = minimum_required_size()
    if tw >= mw and th >= mh then
        if S.warn then
            clear()
            S.dirty = true
        end
        if tw ~= S.tw or th ~= S.th then
            clear()
            S.dirty = true
        end
        S.tw, S.th, S.warn = tw, th, false
        return true
    end
    local changed = (not S.warn) or S.lw ~= tw or S.lh ~= th or S.lmw ~= mw or S.lmh ~= mh
    if changed then
        draw_terminal_size_warning(tw, th, mw, mh)
        S.lw, S.lh, S.lmw, S.lmh = tw, th, mw, mh
    end
    S.warn = true
    return false
end

local function top_time_line()
    return tr("game.wordle.time") .. " " .. format_duration(elapsed_seconds()) .. "  " .. tr("game.wordle.streak") .. " " .. tostring(S.streak)
end

local function draw_guess_row(y, tw, idx)
    local prefix = "-> "
    local guess = S.guesses[idx]
    local marks = S.marks[idx]
    local width = text_width(prefix) + S.word_len * 2
    local x = centered_x(string.rep(" ", width), 1, tw)
    draw_text(x, y, prefix, "white", "black")
    x = x + text_width(prefix)
    for i = 1, S.word_len do
        local ch = " "
        local fg, bg = "white", "black"
        if type(guess) == "string" then
            ch = string.upper(char_at(guess, i))
            local mark = marks and marks[i] or "absent"
            if mark == "correct" then
                fg, bg = "black", "green"
            elseif mark == "present" then
                fg, bg = "black", "yellow"
            else
                fg, bg = "dark_gray", "black"
            end
        end
        draw_text(x, y, ch, fg, bg)
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

local function draw_input_row(y, tw)
    local prefix = "  "
    local width = text_width(prefix) + S.word_len * 2
    local x = centered_x(string.rep(" ", width), 1, tw)
    draw_text(x, y, prefix, "white", "black")
    x = x + text_width(prefix)
    local show = S.input
    if S.settled then
        show = S.secret
    end
    for i = 1, S.word_len do
        local ch, fg = "_", "dark_gray"
        if i <= #show then
            ch = string.upper(char_at(show, i))
            if S.settled then
                fg = S.won and "green" or "red"
            else
                fg = "white"
            end
        end
        draw_text(x, y, ch, fg, "black")
        draw_text(x + 1, y, " ", "white", "black")
        x = x + 2
    end
end

local function render_frame()
    local tw, th = terminal_size()
    local top = math.floor((th - 12) / 2)
    if top < 1 then top = 1 end
    local best = tr("game.wordle.best_time") .. " " .. ((S.best_time_sec > 0) and format_duration(S.best_time_sec) or "--:--:--")
    local tline = top_time_line()
    local msg, mc = status_text()
    for i = 0, 2 do clear_line(top + i, tw) end
    draw_text(centered_x(best, 1, tw), top, best, "dark_gray", "black")
    draw_text(centered_x(tline, 1, tw), top + 1, tline, "light_cyan", "black")
    S.last_time_line = tline
    draw_text(centered_x(msg, 1, tw), top + 2, msg, mc, "black")
    local y0 = top + 4
    for i = 0, MAX_ATTEMPTS do clear_line(y0 + i, tw) end
    for i = 1, MAX_ATTEMPTS do
        draw_guess_row(y0 + i - 1, tw, i)
    end
    draw_input_row(y0 + MAX_ATTEMPTS, tw)
    local controls = controls_text()
    clear_line(y0 + MAX_ATTEMPTS + 2, tw)
    draw_text(centered_x(controls, 1, tw), y0 + MAX_ATTEMPTS + 2, controls, "white", "black")
end

local function apply_guess()
    if #S.input ~= S.word_len then
        S.toast = tr("game.wordle.need_letters")
        S.toast_color = "red"
        S.toast_until = S.frame + FPS * 2
        S.dirty = true
        return
    end
    local guess = string.lower(S.input)
    local marks = evaluate_guess(S.secret, guess)
    S.guesses[#S.guesses + 1] = guess
    S.marks[#S.marks + 1] = marks
    S.input = ""
    if guess == S.secret then
        settle(true)
    elseif #S.guesses >= MAX_ATTEMPTS then
        settle(false)
    end
    S.dirty = true
end

local function refresh_dirty_flags()
    local e = elapsed_seconds()
    if e ~= S.last_elapsed then
        S.last_elapsed = e
        S.time_dirty = true
    end
    local tv = S.toast ~= nil and S.frame <= S.toast_until
    if (not tv) and S.toast ~= nil then
        S.toast = nil
        S.dirty = true
    end
end

local function handle_confirm_key(k)
    if k == "y" or k == "enter" then
        if S.confirm == "restart" then
            S.confirm = nil
            new_round(false)
            return "changed"
        end
        if S.confirm == "exit" then
            return "exit"
        end
    end
    if k == "q" or k == "esc" or k == "n" then
        S.confirm = nil
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_playing_key(k)
    if S.mode == "input" then
        if k == "tab" then
            S.mode = "action"
            S.dirty = true
            return "changed"
        end
        if k == "backspace" or k == "delete" then
            if #S.input > 0 then
                S.input = string.sub(S.input, 1, #S.input - 1)
                S.dirty = true
            end
            return "changed"
        end
        if k == "enter" then
            apply_guess()
            return "changed"
        end
        if k:match("^[a-z]$") then
            if #S.input < S.word_len then
                S.input = S.input .. k
                S.dirty = true
            end
            return "changed"
        end
        return "none"
    end
    if k == "tab" then
        S.mode = "input"
        S.dirty = true
        return "changed"
    end
    if k == "s" then
        save_slot()
        S.dirty = true
        return "changed"
    end
    if k == "r" then
        S.confirm = "restart"
        S.dirty = true
        return "changed"
    end
    if k == "q" or k == "esc" then
        S.confirm = "exit"
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function handle_settled_key(k)
    if k == "r" then
        new_round(S.won)
        return "changed"
    end
    if k == "q" or k == "esc" then
        return "exit"
    end
    if k == "tab" then
        S.mode = (S.mode == "input") and "action" or "input"
        S.dirty = true
        return "changed"
    end
    return "none"
end

local function bootstrap_game()
    clear()
    if type(clear_input_buffer) == "function" then pcall(clear_input_buffer) end
    load_meta()
    load_words()
    if not load_slot_if_continue() then
        new_round(true)
    end
    S.frame = 0
    S.last_elapsed = elapsed_seconds()
    S.time_dirty = false
    S.dirty = true
end

function init_game()
    bootstrap_game()
    return S
end

function handle_event(state_arg, event)
    S = state_arg or S
    local k = normalize_key(event)
    if not ensure_terminal_size_ok() then
        if k == "q" or k == "esc" then
            exit_game()
            return S
        end
        if type(event) == "table" and event.type == "tick" then
            S.frame = S.frame + 1
        end
        return S
    end

    if k ~= "" then
        local a = "none"
        if S.confirm then
            a = handle_confirm_key(k)
        elseif S.settled then
            a = handle_settled_key(k)
        else
            a = handle_playing_key(k)
        end
        if a == "exit" then
            exit_game()
            return S
        end
    end

    refresh_dirty_flags()
    if type(event) == "table" and event.type == "tick" then
        S.frame = S.frame + 1
    end
    return S
end

function render(state_arg)
    S = state_arg or S
    if not ensure_terminal_size_ok() then
        return
    end
    render_frame()
end

function best_score(state_arg)
    S = state_arg or S
    local best_time = (S.best_time_sec > 0) and format_duration(S.best_time_sec) or "--:--:--"
    return {
        best_string = "game.wordle.best_block",
        streak = S.streak,
        time = best_time,
    }
end
