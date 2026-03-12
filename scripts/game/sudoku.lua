-- 数独游戏元数据
GAME_META = { name = "Sudoku", description = "Fill each row, column, and 3x3 box with digits 1-9 exactly once." }

-- 游戏常量定义
local N, B, FPS, MS, UL = 9, 3, 60, 16, 100 -- N: 网格大小9x9, B: 宫大小3x3, UL: 最大撤销步数

-- 各难度对应的空格数量
local HOLES = { [1] = 30, [2] = 40, [3] = 50, [4] = 60, [5] = 70 }

-- 界面显示的固定字符串
local H1 = "      1 2 3  4 5 6  7 8 9" -- 列号标题
local H2 = "      | | |  | | |  | | |" -- 列分隔线
local BT = "    ╔══════╤══════╤══════╗" -- 顶部边框
local BM = "    ╟──────┼──────┼──────╢" -- 中间分隔线
local BB = "    ╚══════╧══════╧══════╝" -- 底部边框

-- 游戏状态表（使用简短变量名以节省空间，但此处添加注释说明）
local S = {
    d = 3,          -- difficulty 当前难度
    p = {},         -- puzzle 原始谜题
    sol = {},       -- solution 完整解
    b = {},         -- board 当前棋盘
    g = {},         -- given 初始给定的格子（不可修改）
    cf = {},        -- conflict 冲突标记
    r = 1,          -- row 光标行
    c = 1,          -- col 光标列
    undo = {},      -- 撤销栈
    f = 0,          -- frame 当前帧计数
    sf = 0,         -- start_frame 游戏开始帧
    ef = nil,       -- end_frame 游戏结束帧
    win = false,    -- 是否获胜
    bc = false,     -- best_committed 是否已提交最佳记录
    im = nil,       -- input_mode 输入模式
    ib = "",        -- input_buffer 输入缓冲区
    cm = nil,       -- confirm_mode 确认模式
    toast = nil,    -- 提示消息
    tu = 0,         -- toast_until 提示消息截止帧
    as = 0,         -- auto_save_sec 上次自动保存时间
    best = nil,     -- 最佳记录 {d, t}
    dirty = true,   -- 是否需要重绘
    le = -1,        -- last_elapsed_sec 上次已过秒数
    lt = false,     -- last_toast_visible 上次提示是否可见
    launch = "new", -- 启动模式
    area = nil,     -- 上次渲染区域
    tw = 0,         -- last_term_w 上次终端宽度
    th = 0,         -- last_term_h 上次终端高度
    hl = false      -- highlight 是否高亮相关单元格
}

-- 翻译函数（安全调用）
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

-- 获取文本显示宽度
local function text_width(t)
    if type(get_text_width) == "function" then
        local ok, w = pcall(get_text_width, t)
        if ok and type(w) == "number" then return w end
    end
    return #t
end

-- 获取终端尺寸
local function terminal_size()
    local w, h = 120, 40
    if type(get_terminal_size) == "function" then
        local tw, th = get_terminal_size()
        if type(tw) == "number" and type(th) == "number" then w, h = tw, th end
    end
    return w, h
end

-- 规范化按键
local function normalize_key(k)
    if k == nil then return "" end
    if type(k) == "string" then return string.lower(k) end
    return tostring(k):lower()
end

-- 随机整数 [1, n]
local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

-- 创建新矩阵（9x9）
local function mx(v)
    local m = {}
    for r = 1, N do
        m[r] = {}
        for c = 1, N do m[r][c] = v end
    end
    return m
end

-- 复制矩阵
local function cp(a)
    local m = {}
    for r = 1, N do
        m[r] = {}
        for c = 1, N do m[r][c] = a[r][c] end
    end
    return m
end

-- 打乱数组
local function sh(t)
    for i = #t, 2, -1 do
        local j = rand_int(i) + 1
        t[i], t[j] = t[j], t[i]
    end
end

-- 计算已过秒数
local function elapsed_seconds()
    local e = S.ef or S.f
    return math.max(0, math.floor((e - S.sf) / FPS))
end

-- 格式化时间
local function format_duration(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

-- 获取难度名称
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

-- 检查在指定位置放置数字n是否有效
local function okv(b, r, c, n)
    -- 检查行
    for i = 1, N do if i ~= c and b[r][i] == n then return false end end
    -- 检查列
    for i = 1, N do if i ~= r and b[i][c] == n then return false end end
    -- 检查宫
    local br = math.floor((r - 1) / B) * B + 1
    local bc = math.floor((c - 1) / B) * B + 1
    for i = br, br + B - 1 do
        for j = bc, bc + B - 1 do
            if (i ~= r or j ~= c) and b[i][j] == n then return false end
        end
    end
    return true
end

-- 获取指定位置的可能数字列表
local function cand(b, r, c)
    local t = {}
    for n = 1, 9 do
        if okv(b, r, c, n) then t[#t + 1] = n end
    end
    return t
end

-- 选择下一个要填的格子（最少候选数优先）
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

-- 递归填充数独（用于生成完整解）
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

-- 计数解法数量（限制最大计数）
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

-- 生成一个完整的数独解
local function gen_solved()
    local b = mx(0)
    fill(b)
    return b
end

-- 挖洞法生成唯一解谜题
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

-- 任意挖洞（不保证唯一解）
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

-- 生成指定难度的数独
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

-- 重新计算冲突标记
local function recf()
    S.cf = mx(false)
    local function mark(ls)
        if #ls > 1 then
            for i = 1, #ls do S.cf[ls[i].r][ls[i].c] = true end
        end
    end
    -- 检查行冲突
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
    -- 检查列冲突
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
    -- 检查宫冲突
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

-- 检查是否已完成
local function done()
    for r = 1, N do
        for c = 1, N do
            if S.b[r][c] == 0 or S.cf[r][c] then return false end
        end
    end
    return true
end

-- 判断是否应替换最佳记录
local function rep(old, new)
    if old == nil then return true end
    if new.d ~= old.d then return new.d > old.d end
    return new.t < old.t
end

-- 加载最佳记录
local function load_best()
    if type(load_data) ~= "function" then return nil end
    local ok, d = pcall(load_data, "sudoku_best")
    if (not ok) or type(d) ~= "table" then return nil end
    local lv = math.floor(tonumber(d.d) or tonumber(d.difficulty) or 0)
    local tm = math.floor(tonumber(d.t) or tonumber(d.min_time_sec) or 0)
    if lv < 1 or lv > 5 or tm <= 0 then return nil end
    return { d = lv, t = tm }
end

-- 保存最佳记录
local function save_best(x)
    if type(save_data) == "function" then
        pcall(save_data, "sudoku_best", { d = x.d, t = x.t, difficulty = x.d, min_time_sec = x.t })
    end
end

-- 提交最佳记录
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

-- 检查胜利状态
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

-- 压入撤销栈
local function pushu(r, c, o, n)
    if o == n then return end
    S.undo[#S.undo + 1] = { r = r, c = c, o = o, n = n }
    while #S.undo > UL do table.remove(S.undo, 1) end
end

-- 创建游戏快照
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

-- 验证矩阵有效性
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

-- 恢复游戏快照
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

-- 保存游戏状态
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

-- 加载游戏状态
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

-- 读取启动模式
local function lmode()
    if type(get_launch_mode) ~= "function" then return "new" end
    local ok, m = pcall(get_launch_mode)
    if (not ok) or type(m) ~= "string" then return "new" end
    m = string.lower(m)
    if m == "continue" then return "continue" end
    return "new"
end

-- 重置游戏
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

-- 设置指定位置的值
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

-- 撤销一步
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

-- 按单词换行
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

-- 计算最小宽度
local function min_width_for_lines(t, m, h)
    local f = text_width(t)
    local w = h
    while w <= f do
        if #wrap_words(t, w) <= m then return w end
        w = w + 1
    end
    return f
end

-- 填充矩形区域
local function fill_rect(x, y, w, h, bg)
    if w <= 0 or h <= 0 then return end
    local ln = string.rep(" ", w)
    for i = 0, h - 1 do
        draw_text(x, y + i, ln, "white", bg or "black")
    end
end

-- 计算界面几何布局
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

-- 绘制一行数独
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

-- 绘制数独棋盘
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

-- 绘制提示信息
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

-- 绘制控制说明
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

-- 清除上次渲染区域
local function clear_area()
    if S.area then
        fill_rect(S.area.x, S.area.y, S.area.w, S.area.h, "black")
    end
end

-- 主渲染函数
local function render()
    local g = geo()
    local a = { x = 1, y = g.ty, w = g.tw, h = g.thh }
    if S.area == nil then
        fill_rect(a.x, a.y, a.w, a.h, "black")
    elseif S.area.w ~= a.w or S.area.h ~= a.h or S.area.y ~= a.y then
        clear_area()
        fill_rect(a.x, a.y, a.w, a.h, "black")
    end
    S.area = a

    -- 清空顶部区域
    draw_text(1, g.ty, string.rep(" ", g.tw), "white", "black")
    draw_text(1, g.ty + 1, string.rep(" ", g.tw), "white", "black")

    -- 显示最佳记录
    local bx = math.floor((g.tw - text_width(g.best)) / 2)
    if bx < 1 then bx = 1 end
    draw_text(bx, g.ty, g.best, "dark_gray", "black")

    -- 显示时间和难度
    draw_text(g.tx, g.ty + 1, g.st, "light_cyan", "black")

    -- 绘制棋盘和界面
    draw_board(g)
    dnotice(g)
    draw_controls(g)
end

-- 绘制尺寸警告
local function draw_terminal_size_warning(tw, th, mw, mh)
    local ls = {
        tr("warning.size_title"),
        string.format("%s: %dx%d", tr("warning.required"), mw, mh),
        string.format("%s: %dx%d", tr("warning.current"), tw, th),
        tr("warning.enlarge_hint")
    }
    local top = math.floor((th - #ls) / 2)
    if top < 1 then top = 1 end
    for i = 1, #ls do
        local x = math.floor((tw - text_width(ls[i])) / 2)
        if x < 1 then x = 1 end
        draw_text(x, top + i - 1, ls[i], "white", "black")
    end
end

-- 检查终端尺寸是否足够
local function ensure_terminal_size_ok()
    local g = geo()
    local tw, th, mw, mh = g.tw, g.th, g.rw, g.rh
    if tw >= mw and th >= mh then return true end
    clear()
    draw_terminal_size_warning(tw, th, mw, mh)
    return false
end

-- 同步终端尺寸变化
local function sync_resize()
    local w, h = terminal_size()
    if w ~= S.tw or h ~= S.th then
        S.tw, S.th = w, h
        clear()
        S.area = nil
        S.dirty = true
    end
end

-- 刷新脏标记
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

-- 自动保存
local function autosave()
    if S.win then return end
    local e = elapsed_seconds()
    if e - S.as >= 60 then
        save_state(false)
        S.as = e
    end
end

-- 处理输入模式下的按键
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

-- 处理确认模式下的按键
local function hconfirm(k)
    if k == "y" then
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
    if k == "n" or k == "q" or k == "esc" then
        S.cm = nil
        S.dirty = true
        return "changed"
    end
    return "changed"
end

-- 主输入处理函数
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

-- 游戏初始化
local function init_game()
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

-- 主游戏循环
local function game_loop()
    while true do
        local k = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            local a = input(k)
            if a == "exit" then return end
            sync_resize()
            autosave()
            refresh()
            if S.dirty then
                render()
                S.dirty = false
            end
            S.f = S.f + 1
        else
            if k == "q" or k == "esc" then return end
        end
        sleep(MS)
    end
end

-- 启动游戏
init_game()
game_loop()
