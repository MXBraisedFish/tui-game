local function draw_text(x, y, text, fg, bg)
    canvas_draw_text(math.max(0, x - 1), math.max(0, y - 1), text or "", fg, bg)
end

local function clear()
    canvas_clear()
end

local function exit_game()
    request_exit()
end

local N, B, FPS, MS, UL = 9, 3, 60, 16, 100 

local HOLES = { [1] = 30, [2] = 40, [3] = 50, [4] = 60, [5] = 70 }

local H1 = "      1 2 3  4 5 6  7 8 9" 
local H2 = "      | | |  | | |  | | |" 
local BT = "    ╔══════╤══════╤══════╗" 
local BM = "    ╟──────┼──────┼──────╢" 
local BB = "    ╚══════╧══════╧══════╝" 

local S = {
    d = 3,          
    p = {},         
    sol = {},       
    b = {},         
    g = {},         
    cf = {},        
    r = 1,          
    c = 1,          
    undo = {},      
    f = 0,          
    sf = 0,         
    ef = nil,       
    win = false,    
    bc = false,     
    im = nil,       
    ib = "",        
    cm = nil,       
    toast = nil,    
    tu = 0,         
    as = 0,         
    best = nil,     
    dirty = true,   
    le = -1,        
    lt = false,     
    launch = "new", 
    area = nil,     
    tw = 0,         
    th = 0,         
    hl = false      
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

local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

local function normalize_key(event)
    if event == nil then return "" end
    if type(event) == "string" then return string.lower(event) end
    if type(event) ~= "table" then return tostring(event):lower() end
    if event.type == "quit" then return "esc" end
    if event.type == "key" and type(event.name) == "string" then return string.lower(event.name) end
    if event.type ~= "action" or type(event.name) ~= "string" then return "" end
    local map = {
        move_up = "up",
        move_down = "down",
        move_left = "left",
        move_right = "right",
        undo = "a",
        jump = "d",
        change_difficulty = "p",
        toggle_locator = "z",
        restart = "r",
        save = "s",
        quit_action = "q",
        confirm_yes = "enter",
        confirm_no = "esc",
    }
    return map[event.name] or ""
end

local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

local function mx(v)
    local m = {}
    for r = 1, N do
        m[r] = {}
        for c = 1, N do m[r][c] = v end
    end
    return m
end

local function cp(a)
    local m = {}
    for r = 1, N do
        m[r] = {}
        for c = 1, N do m[r][c] = a[r][c] end
    end
    return m
end

local function sh(t)
    for i = #t, 2, -1 do
        local j = rand_int(i) + 1
        t[i], t[j] = t[j], t[i]
    end
end

local function elapsed_seconds()
    local e = S.ef or S.f
    return math.max(0, math.floor((e - S.sf) / FPS))
end

local function format_duration(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

local function dn(d)
    if d == 1 then
        return tr("game.sudoku.difficulty.1")
    elseif d == 2 then
        return tr("game.sudoku.difficulty.2")
    elseif d == 3 then
        return tr("game.sudoku.difficulty.3")
    elseif d == 4 then
        return tr("game.sudoku.difficulty.4")
    end
    return tr("game.sudoku.difficulty.5")
end

local function okv(b, r, c, n)
    
    for i = 1, N do if i ~= c and b[r][i] == n then return false end end
    
    for i = 1, N do if i ~= r and b[i][c] == n then return false end end
    
    local br = math.floor((r - 1) / B) * B + 1
    local bc = math.floor((c - 1) / B) * B + 1
    for i = br, br + B - 1 do
        for j = bc, bc + B - 1 do
            if (i ~= r or j ~= c) and b[i][j] == n then return false end
        end
    end
    return true
end

local function cand(b, r, c)
    local t = {}
    for n = 1, 9 do
        if okv(b, r, c, n) then t[#t + 1] = n end
    end
    return t
end

local function pick(b)
    local br, bc, bt = nil, nil, nil
    for r = 1, N do
        for c = 1, N do
            if b[r][c] == 0 then
                local t = cand(b, r, c)
                if #t == 0 then return r, c, t end
                if bt == nil or #t < #bt then
                    br, bc, bt = r, c, t
                    if #bt == 1 then return br, bc, bt end
                end
            end
        end
    end
    return br, bc, bt
end

local function fill(b)
    local r, c, t = pick(b)
    if r == nil then return true end
    if t == nil or #t == 0 then return false end
    sh(t)
    for i = 1, #t do
        b[r][c] = t[i]
        if fill(b) then return true end
    end
    b[r][c] = 0
    return false
end

local function csol(b, lim)
    local cnt = 0
    local function dfs()
        if cnt >= lim then return end
        local r, c, t = pick(b)
        if r == nil then
            cnt = cnt + 1
            return
        end
        if t == nil or #t == 0 then return end
        for i = 1, #t do
            b[r][c] = t[i]
            dfs()
            if cnt >= lim then
                b[r][c] = 0
                return
            end
        end
        b[r][c] = 0
    end
    dfs()
    return cnt
end

local function gen_solved()
    local b = mx(0)
    fill(b)
    return b
end

local function dig_unique(sol, h)
    local p = cp(sol)
    local co = {}
    for r = 1, N do for c = 1, N do co[#co + 1] = { r = r, c = c } end end
    sh(co)
    local rm = 0
    for i = 1, #co do
        if rm >= h then break end
        local r, c = co[i].r, co[i].c
        local o = p[r][c]
        p[r][c] = 0
        local t = cp(p)
        if csol(t, 2) == 1 then
            rm = rm + 1
        else
            p[r][c] = o
        end
    end
    return p, rm
end

local function dig_any(sol, h)
    local p = cp(sol)
    local co = {}
    for r = 1, N do for c = 1, N do co[#co + 1] = { r = r, c = c } end end
    sh(co)
    for i = 1, math.min(h, #co) do
        p[co[i].r][co[i].c] = 0
    end
    return p
end

local function gen(d)
    local h = HOLES[d] or 50
    local fs = nil
    for _ = 1, 6 do
        local s = gen_solved()
        fs = s
        local p, rm = dig_unique(s, h)
        if rm >= h then return p, s end
    end
    local s = fs or gen_solved()
    return dig_any(s, h), s
end

local function recf()
    S.cf = mx(false)
    local function mark(ls)
        if #ls > 1 then
            for i = 1, #ls do S.cf[ls[i].r][ls[i].c] = true end
        end
    end
    
    for r = 1, N do
        local mp = {}
        for c = 1, N do
            local v = S.b[r][c]
            if v > 0 then
                mp[v] = mp[v] or {}
                mp[v][#mp[v] + 1] = { r = r, c = c }
            end
        end
        for _, ls in pairs(mp) do mark(ls) end
    end
    
    for c = 1, N do
        local mp = {}
        for r = 1, N do
            local v = S.b[r][c]
            if v > 0 then
                mp[v] = mp[v] or {}
                mp[v][#mp[v] + 1] = { r = r, c = c }
            end
        end
        for _, ls in pairs(mp) do mark(ls) end
    end
    
    for br = 1, N, 3 do
        for bc = 1, N, 3 do
            local mp = {}
            for r = br, br + 2 do
                for c = bc, bc + 2 do
                    local v = S.b[r][c]
                    if v > 0 then
                        mp[v] = mp[v] or {}
                        mp[v][#mp[v] + 1] = { r = r, c = c }
                    end
                end
            end
            for _, ls in pairs(mp) do mark(ls) end
        end
    end
end

local function done()
    for r = 1, N do
        for c = 1, N do
            if S.b[r][c] == 0 or S.cf[r][c] then return false end
        end
    end
    return true
end

local function rep(old, new)
    if old == nil then return true end
    if new.d ~= old.d then return new.d > old.d end
    return new.t < old.t
end

local function load_best()
    if type(load_data) ~= "function" then return nil end
    local ok, d = pcall(load_data, "sudoku_best")
    if (not ok) or type(d) ~= "table" then return nil end
    local lv = math.floor(tonumber(d.d) or tonumber(d.difficulty) or 0)
    local tm = math.floor(tonumber(d.t) or tonumber(d.min_time_sec) or 0)
    if lv < 1 or lv > 5 or tm <= 0 then return nil end
    return { d = lv, t = tm }
end

local function save_best(x)
    if type(save_data) == "function" then
        pcall(save_data, "sudoku_best", { d = x.d, t = x.t, difficulty = x.d, min_time_sec = x.t })
    end
end

local function cbest()
    if S.bc or not S.win then return end
    local n = { d = S.d, t = elapsed_seconds() }
    if rep(S.best, n) then
        S.best = n
        save_best(n)
    end
    if type(update_game_stats) == "function" then
        local sc = S.d * 100000 - n.t
        if sc < 0 then sc = 0 end
        pcall(update_game_stats, "sudoku", sc, n.t)
    end
    S.bc = true
end

local function chk()
    local w = done()
    if w then
        if not S.win then
            S.win = true
            S.ef = S.f
            S.bc = false
            cbest()
        end
    else
        S.win = false
        S.ef = nil
        S.bc = false
    end
end

local function pushu(r, c, o, n)
    if o == n then return end
    S.undo[#S.undo + 1] = { r = r, c = c, o = o, n = n }
    while #S.undo > UL do table.remove(S.undo, 1) end
end

local function snap()
    return {
        d = S.d,
        p = cp(S.p),
        s = cp(S.sol),
        b = cp(S.b),
        g = cp(S.g),
        r = S.r,
        c = S.c,
        e = elapsed_seconds(),
        w = S.win,
        u = S.undo,
        a = S.as,
        hl = S.hl
    }
end

local function vm(m)
    if type(m) ~= "table" then return false end
    for r = 1, N do
        if type(m[r]) ~= "table" then return false end
        for c = 1, N do
            local v = math.floor(tonumber(m[r][c]) or -1)
            if v < 0 or v > 9 then return false end
        end
    end
    return true
end

local function restore(x)
    if type(x) ~= "table" then return false end
    local d = math.floor(tonumber(x.d) or tonumber(x.difficulty) or 0)
    if d < 1 or d > 5 then return false end
    local p = x.p or x.puzzle
    local b = x.b or x.board
    if (not vm(p)) or (not vm(b)) then return false end
    S.d = d
    S.p = cp(p)
    S.b = cp(b)
    local s = x.s or x.solution
    if vm(s) then S.sol = cp(s) else S.sol = cp(S.p) end
    local g = x.g or x.given
    S.g = mx(false)
    if vm(g) then
        for r = 1, N do for c = 1, N do S.g[r][c] = g[r][c] and true or false end end
    else
        for r = 1, N do for c = 1, N do S.g[r][c] = S.p[r][c] ~= 0 end end
    end
    S.r = math.max(1, math.min(9, math.floor(tonumber(x.r) or 1)))
    S.c = math.max(1, math.min(9, math.floor(tonumber(x.c) or 1)))
    S.hl = (x.hl == true)
    S.undo = {}
    local u = x.u or x.undo_stack
    if type(u) == "table" then
        for i = 1, #u do
            local e = u[i]
            if type(e) == "table" then
                local r = math.floor(tonumber(e.r) or 0)
                local c = math.floor(tonumber(e.c) or 0)
                local o = math.floor(tonumber(e.o) or tonumber(e.old_v) or -1)
                local n = math.floor(tonumber(e.n) or tonumber(e.new_v) or -1)
                if r >= 1 and r <= 9 and c >= 1 and c <= 9 and o >= 0 and o <= 9 and n >= 0 and n <= 9 then
                    S.undo[#S.undo + 1] = { r = r, c = c, o = o, n = n }
                end
            end
        end
    end
    while #S.undo > UL do table.remove(S.undo, 1) end
    local e = math.max(0, math.floor(tonumber(x.e) or tonumber(x.elapsed_sec) or 0))
    S.sf = S.f - e * FPS
    S.as = math.max(0, math.floor(tonumber(x.a) or tonumber(x.last_auto_save_sec) or e))
    S.toast = tr("game.sudoku.continue_loaded")
    S.tu = S.f + 3 * FPS
    S.im = nil
    S.ib = ""
    S.cm = nil
    S.ef = nil
    S.win = (x.w == true or x.won == true)
    S.bc = false
    recf()
    chk()
    S.area = nil
    S.dirty = true
    return true
end

local function save_state(show)
    local ok = false
    local x = snap()
    if type(save_game_slot) == "function" then
        local s, r = pcall(save_game_slot, "sudoku", x)
        ok = s and r ~= false
    elseif type(save_data) == "function" then
        local s, r = pcall(save_data, "sudoku", x)
        ok = s and r ~= false
    end
    if show then
        local k = ok and "game.sudoku.save_success" or "game.sudoku.save_unavailable"
        local d = ok and "Save successful!" or "Save API unavailable."
        S.toast = tr(k)
        S.tu = S.f + 3 * FPS
        S.dirty = true
    end
end

local function load_state()
    local ok, x = false, nil
    if type(load_game_slot) == "function" then
        local s, r = pcall(load_game_slot, "sudoku")
        ok = s and r ~= nil
        x = r
    elseif type(load_data) == "function" then
        local s, r = pcall(load_data, "sudoku")
        ok = s and r ~= nil
        x = r
    end
    if ok then return restore(x) end
    return false
end

local function lmode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, m = pcall(get_launch_mode)
    if (not ok) or type(m) ~= "string" then return "new" end
    m = string.lower(m)
    if m == "continue" then return "continue" end
    return "new"
end

local function reset(d)
    S.d = math.max(1, math.min(5, d or S.d))
    local p, s = gen(S.d)
    S.p, S.sol, S.b = p, s, cp(p)
    S.g = mx(false)
    for r = 1, N do for c = 1, N do S.g[r][c] = S.p[r][c] ~= 0 end end
    S.cf = mx(false)
    S.r, S.c = 1, 1
    S.undo = {}
    S.sf = S.f
    S.ef = nil
    S.win = false
    S.bc = false
    S.im = nil
    S.ib = ""
    S.cm = nil
    S.toast = nil
    S.tu = 0
    S.as = 0
    S.area = nil
    S.dirty = true
    recf()
end

local function setv(r, c, n)
    if S.g[r][c] then return false end
    local o = S.b[r][c]
    if o == n then return false end
    pushu(r, c, o, n)
    S.b[r][c] = n
    recf()
    chk()
    S.dirty = true
    return true
end

local function undo()
    local n = #S.undo
    if n <= 0 then
        S.toast = tr("game.sudoku.undo_empty")
        S.tu = S.f + 3 * FPS
        S.dirty = true
        return
    end
    local e = table.remove(S.undo, n)
    if S.g[e.r][e.c] then return end
    S.b[e.r][e.c] = e.o
    recf()
    chk()
    S.toast = tr("game.sudoku.undo_done")
    S.tu = S.f + 2 * FPS
    S.dirty = true
end

local function wrap_words(t, w)
    if w <= 1 then return { t } end
    local ls, cur, had = {}, "", false
    for tok in string.gmatch(t, "%S+") do
        had = true
        if cur == "" then
            cur = tok
        else
            local c = cur .. " " .. tok
            if text_width(c) <= w then
                cur = c
            else
                ls[#ls + 1] = cur
                cur = tok
            end
        end
    end
    if not had then return { "" } end
    if cur ~= "" then ls[#ls + 1] = cur end
    return ls
end

local function min_width_for_lines(t, m, h)
    local f = text_width(t)
    local w = h
    while w <= f do
        if #wrap_words(t, w) <= m then return w end
        w = w + 1
    end
    return f
end

local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local ln = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, ln, "white", bg or "black")
    end
end

local function geo()
    local tw, th = terminal_size()
    local ctl = tr("game.sudoku.controls")
    if not string.find(ctl, "%[Z%]") then
        ctl = ctl .. "  [Z] " .. tr("game.sudoku.toggle_locator")
    end
    local cl = wrap_words(ctl, math.max(10, tw - 2))
    if #cl > 3 then cl = { cl[1], cl[2], cl[3] } end
    local bw, bh = text_width(BB), 15
    local best = S.best and (tr("game.sudoku.best") .. " " .. dn(S.best.d) .. " " .. format_duration(S.best.t)) or
    tr("game.sudoku.best_none")
    local st = tr("game.sudoku.time") ..
    " " .. format_duration(elapsed_seconds()) .. "   " .. tr("game.sudoku.difficulty") .. " " .. dn(S.d)
    local topw = math.max(text_width(best), text_width(st))
    local nw = 0
    local ns = {
        tr("game.sudoku.win_banner") .. " " .. tr("game.sudoku.win_controls"),
        tr("game.sudoku.input_jump_hint"),
        tr("game.sudoku.input_difficulty_hint"),
        tr("game.sudoku.confirm_exit"),
        tr("game.sudoku.confirm_restart"),
        tr("game.sudoku.save_success"),
        tr("game.sudoku.save_unavailable")
    }
    for i = 1, #ns do nw = math.max(nw, text_width(ns[i])) end
    local thh = 3 + bh + 1 + math.max(1, #cl)
    local sy = math.floor((th - thh) / 2)
    if sy < 1 then sy = 1 end
    local bx = math.floor((tw - bw) / 2)
    if bx < 1 then bx = 1 end
    local tx = math.floor((tw - topw) / 2)
    if tx < 1 then tx = 1 end
    return {
        tw = tw,
        th = th,
        bx = bx,
        by = sy + 3,
        bw = bw,
        bh = bh,
        best = best,
        st = st,
        tx = tx,
        ty = sy,
        ny = sy + 3 + bh,
        cl = cl,
        thh = thh,
        rw = math.max(bw, topw, nw, min_width_for_lines(ctl, 3, 40)) + 2,
        rh = thh + 1
    }
end

local function drow(bx, y, r)
    local p = tostring(r) .. " - "
    draw_text(bx, y, p .. "║", "white", "black")
    local x = bx + text_width(p .. "║")
    local crb = math.floor((S.r - 1) / B)
    local ccb = math.floor((S.c - 1) / B)
    local rb = math.floor((r - 1) / B)
    for c = 1, N do
        local v = S.b[r][c]
        local t = (v > 0) and (" " .. tostring(v)) or "  "
        local fg = "white"
        local cur = (r == S.r and c == S.c)
        if v > 0 then
            if cur then
                if S.g[r][c] then fg = "black" else fg = "#3f48cc" end
            elseif S.cf[r][c] then
                fg = "red"
            elseif S.g[r][c] then
                fg = "white"
            else
                fg = "cyan"
            end
        end
        local bg = "black"
        if cur then
            bg = "light_yellow"
        elseif S.hl then
            local cb = math.floor((c - 1) / B)
            if r == S.r or c == S.c or (rb == crb and cb == ccb) then bg = "#B3B3B3" end
        end
        draw_text(x, y, t, fg, bg)
        x = x + 2
        if c == 3 or c == 6 then
            draw_text(x, y, "│", "white", "black")
            x = x + 1
        end
    end
    draw_text(x, y, "║", "white", "black")
end

local function draw_board(g)
    local x, y = g.bx, g.by
    draw_text(x, y, H1, "white", "black")
    draw_text(x, y + 1, H2, "white", "black")
    draw_text(x, y + 2, BT, "white", "black")
    local ry = y + 3
    for r = 1, N do
        drow(x, ry, r)
        ry = ry + 1
        if r == 3 or r == 6 then
            draw_text(x, ry, BM, "white", "black")
            ry = ry + 1
        end
    end
    draw_text(x, ry, BB, "white", "black")
end

local function dnotice(g)
    draw_text(1, g.ny, string.rep(" ", g.tw), "white", "black")
    local l, col = "", "white"
    if S.cm == "exit" then
        l = tr("game.sudoku.confirm_exit")
        col = "yellow"
    elseif S.cm == "restart" then
        l = tr("game.sudoku.confirm_restart")
        col = "yellow"
    elseif S.im == "difficulty" then
        if S.ib == "" then
            l = tr("game.sudoku.input_difficulty_hint")
            col = "dark_gray"
        else
            l = S.ib
        end
    elseif S.im == "jump" then
        if S.ib == "" then
            l = tr("game.sudoku.input_jump_hint")
            col = "dark_gray"
        else
            l = S.ib
        end
    elseif S.win then
        l = tr("game.sudoku.win_banner") .. " " .. tr("game.sudoku.win_controls")
        col = "yellow"
    elseif S.toast ~= nil and S.f <= S.tu then
        l = S.toast
        col = "green"
    end
    if l ~= "" then
        local x = math.floor((g.tw - text_width(l)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, g.ny, l, col, "black")
    end
end

local function draw_controls(g)
    local by = g.ny + 1
    for i = 0, 2 do
        draw_text(1, by + i, string.rep(" ", g.tw), "white", "black")
    end
    local off = 0
    if #g.cl < 3 then off = math.floor((3 - #g.cl) / 2) end
    for i = 1, #g.cl do
        local ln = g.cl[i]
        local x = math.floor((g.tw - text_width(ln)) / 2)
        if x < 1 then x = 1 end
        draw_text(x, by + off + i - 1, ln, "white", "black")
    end
end

local function clear_area()
    if S.area then
        fill_rect(S.area.x, S.area.y, S.area.w, S.area.h, "black")
    end
end

local function render_frame()
    local g = geo()
    local a = { x = 1, y = g.ty, w = g.tw, h = g.thh }
    if S.area == nil then
        fill_rect(a.x, a.y, a.w, a.h, "black")
    elseif S.area.w ~= a.w or S.area.h ~= a.h or S.area.y ~= a.y then
        clear_area()
        fill_rect(a.x, a.y, a.w, a.h, "black")
    end
    S.area = a

    
    draw_text(1, g.ty, string.rep(" ", g.tw), "white", "black")
    draw_text(1, g.ty + 1, string.rep(" ", g.tw), "white", "black")

    
    local bx = math.floor((g.tw - text_width(g.best)) / 2)
    if bx < 1 then bx = 1 end
    draw_text(bx, g.ty, g.best, "dark_gray", "black")

    
    draw_text(g.tx, g.ty + 1, g.st, "light_cyan", "black")

    
    draw_board(g)
    dnotice(g)
    draw_controls(g)
end

local function draw_terminal_size_warning(tw, th, mw, mh)
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
        local x = math.floor((tw - text_width(ls[i])) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, ls[i], "white", "black")
    end
end

local function ensure_terminal_size_ok()
    local g = geo()
    local tw, th, mw, mh = g.tw, g.th, g.rw, g.rh
    S.last_warn_term_w, S.last_warn_term_h = tw, th
    S.last_warn_min_w, S.last_warn_min_h = mw, mh
    S.size_warning_active = not (tw >= mw and th >= mh)
    return not S.size_warning_active
end

local function sync_resize()
    local w, h = terminal_size()
    if w ~= S.tw or h ~= S.th then
        S.tw, S.th = w, h
        clear()
        S.area = nil
        S.dirty = true
    end
end

local function refresh()
    local e = elapsed_seconds()
    if e ~= S.le then
        S.le = e
        S.dirty = true
    end
    local tv = (S.toast ~= nil and S.f <= S.tu)
    if tv ~= S.lt then
        S.lt = tv
        S.dirty = true
    end
end

local function autosave()
    if S.win then return end
    local e = elapsed_seconds()
    if e - S.as >= 60 then
        save_state(false)
        S.as = e
    end
end

local function hmode(k)
    if k == "esc" or k == "q" then
        S.im = nil
        S.ib = ""
        S.dirty = true
        return "changed"
    end
    if k == "backspace" or k == "delete" then
        if #S.ib > 0 then
            S.ib = string.sub(S.ib, 1, #S.ib - 1)
            S.dirty = true
        end
        return "changed"
    end
    if k == "enter" then
        if S.im == "difficulty" then
            local d = math.floor(tonumber(S.ib) or 0)
            S.im = nil
            S.ib = ""
            if d >= 1 and d <= 5 then reset(d) else S.dirty = true end
            return "changed"
        end
        if S.im == "jump" then
            local a, b = S.ib:match("^(%d+)%s+(%d+)$")
            S.im = nil
            S.ib = ""
            if a and b then
                S.r = math.max(1, math.min(9, math.floor(tonumber(a) or 1)))
                S.c = math.max(1, math.min(9, math.floor(tonumber(b) or 1)))
            end
            S.dirty = true
            return "changed"
        end
    end
    if S.im == "difficulty" then
        if k:match("^[1-5]$") and #S.ib < 1 then
            S.ib = k
            S.dirty = true
        end
        return "changed"
    end
    if S.im == "jump" then
        if k:match("^%d$") or k == "space" then
            local t = (k == "space") and " " or k
            if #S.ib < 8 then
                S.ib = S.ib .. t
                S.dirty = true
            end
        end
        return "changed"
    end
    return "none"
end

local function hconfirm(k)
    if k == "y" or k == "enter" then
        local m = S.cm
        S.cm = nil
        if m == "exit" then return "exit" end
        if m == "restart" then
            reset(S.d)
            return "changed"
        end
        S.dirty = true
        return "changed"
    end
    if k == "q" or k == "esc" then
        S.cm = nil
        S.dirty = true
        return "changed"
    end
    return "changed"
end

local function input(k)
    if k == nil or k == "" then return "none" end
    if S.cm ~= nil then return hconfirm(k) end
    if S.im ~= nil then return hmode(k) end

    if k == "q" or k == "esc" then
        if S.win then return "exit" end
        S.cm = "exit"
        S.dirty = true
        return "changed"
    end
    if k == "r" then
        if S.win then
            reset(S.d)
        else
            S.cm = "restart"
            S.dirty = true
        end
        return "changed"
    end
    if k == "s" then
        save_state(true)
        return "changed"
    end
    if k == "p" then
        S.im = "difficulty"
        S.ib = ""
        S.dirty = true
        return "changed"
    end
    if k == "d" then
        S.im = "jump"
        S.ib = ""
        S.dirty = true
        return "changed"
    end
    if k == "a" then
        undo()
        return "changed"
    end
    if k == "z" then
        S.hl = not S.hl
        S.toast = S.hl and tr("game.sudoku.locator_on") or tr("game.sudoku.locator_off")
        S.tu = S.f + 2 * FPS
        S.dirty = true
        return "changed"
    end
    if k == "up" then
        S.r = math.max(1, S.r - 1)
        S.dirty = true
        return "changed"
    end
    if k == "down" then
        S.r = math.min(9, S.r + 1)
        S.dirty = true
        return "changed"
    end
    if k == "left" then
        S.c = math.max(1, S.c - 1)
        S.dirty = true
        return "changed"
    end
    if k == "right" then
        S.c = math.min(9, S.c + 1)
        S.dirty = true
        return "changed"
    end
    if k:match("^[1-9]$") then
        setv(S.r, S.c, tonumber(k))
        return "changed"
    end
    if k == "space" then
        setv(S.r, S.c, 0)
        return "changed"
    end
    return "none"
end

local function bootstrap_game()
    clear()
    S.tw, S.th = terminal_size()
    S.best = load_best()
    S.launch = lmode()
    if S.launch == "continue" then
        if not load_state() then reset(3) end
    else
        reset(3)
    end
    S.dirty = true
end

function init_game()
    bootstrap_game()
    return S
end

function handle_event(state_arg, event)
    S = state_arg or S
    local k = normalize_key(event)
    if ensure_terminal_size_ok() then
        local a = "none"
        if k ~= "" then
            a = input(k)
            if a == "exit" then
                exit_game()
                return S
            end
        end
        sync_resize()
        autosave()
        refresh()
        if type(event) == "table" and event.type == "tick" then
            S.f = S.f + 1
        end
    else
        if k == "q" or k == "esc" then
            exit_game()
            return S
        end
        if type(event) == "table" and event.type == "tick" then
            S.f = S.f + 1
        end
    end
    return S
end

function render(state_arg)
    S = state_arg or S
    if S.size_warning_active then
        clear()
        draw_terminal_size_warning(
            S.last_warn_term_w or 80,
            S.last_warn_term_h or 24,
            S.last_warn_min_w or 0,
            S.last_warn_min_h or 0
        )
        return
    end
    render_frame()
end

function best_score(state_arg)
    S = state_arg or S
    if S.best == nil then return nil end
    return {
        best_string = "game.sudoku.best_block",
        difficulty = dn(S.best.d),
        time = format_duration(S.best.t),
    }
end
