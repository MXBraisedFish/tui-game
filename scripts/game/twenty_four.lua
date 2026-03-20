-- 24点游戏元数据
GAME_META = {
    name = "24 Points",
    description = "Use A/J/Q/K and + - * / () to form an expression equal to 24."
}

-- 游戏常量
local FPS, FRAME_MS, EPS = 60, 16, 1e-6            -- EPS: 浮点数比较误差范围
local M_CLASSIC, M_FIXED_NEG, M_FLEX_NEG = 1, 2, 3 -- 游戏模式
local OP_EMPTY = "_"                               -- 空运算符

-- 括号颜色池
local PAREN_COLORS = { "magenta", "light_cyan", "light_green", "orange" }

-- 游戏状态表（使用简短变量名，此处添加注释）
local S = {
    -- 游戏模式
    mode = M_CLASSIC, -- 当前模式

    -- 数字和运算符
    base_nums = { 1, 2, 3, 4 },             -- 原始数字（用于重置）
    nums = { 1, 2, 3, 4 },                  -- 当前数字（可能带符号）
    ops = { OP_EMPTY, OP_EMPTY, OP_EMPTY }, -- 运算符
    pairs = {},                             -- 括号对 { l, r }

    -- 光标
    cursor = 1,         -- 当前光标位置
    cursor_mode = "op", -- 光标模式："op" 或 "num"

    -- 时间相关
    frame = 0,
    start_frame = 0,
    end_frame = nil,
    steps = 0,     -- 操作步数
    ready = false, -- 表达式是否完整
    value = nil,   -- 表达式计算结果
    win = false,   -- 是否获胜

    -- UI状态
    confirm = nil,    -- 确认模式
    input_mode = nil, -- 输入模式
    input_buf = "",   -- 输入缓冲区
    toast = nil,      -- 提示消息
    toast_color = "green",
    toast_until = 0,

    -- 最佳记录
    best_time = 0,
    committed = false, -- 是否已提交结果

    -- 渲染脏标记
    dirty = true,
    last_elapsed = -1,
    last_toast = false,
    time_dirty = false,
    last_stat = "",

    -- 终端尺寸
    tw = 0,
    th = 0,
    warn = false,
    lw = 0,
    lh = 0,
    lmw = 0,
    lmh = 0,
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

-- 规范化按键
local function normalize_key(k)
    if k == nil then return "" end
    if type(k) == "string" then return string.lower(k) end
    if type(k) == "table" and type(k.code) == "string" then return string.lower(k.code) end
    return tostring(k):lower()
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

-- 计算已过秒数
local function elapsed_seconds()
    local e = S.end_frame or S.frame
    return math.max(0, math.floor((e - S.start_frame) / FPS))
end

-- 格式化时间
local function format_duration(s)
    local h = math.floor(s / 3600)
    local m = math.floor((s % 3600) / 60)
    local x = s % 60
    return string.format("%02d:%02d:%02d", h, m, x)
end

-- 随机整数 [0, n-1]
local function rand_int(n)
    if n <= 0 or type(random) ~= "function" then return 0 end
    return random(n)
end

-- 计算文本居中位置
local function centered_x(text, l, r)
    local x = l + math.floor(((r - l + 1) - text_width(text)) / 2)
    if x < l then x = l end
    return x
end

-- 按单词换行
local function wrap_words(t, mw)
    if mw <= 1 then return { t } end
    local ls, cur, had = {}, "", false
    for tok in string.gmatch(t, "%S+") do
        had = true
        if cur == "" then
            cur = tok
        else
            local c = cur .. " " .. tok
            if text_width(c) <= mw then
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
local function min_width_for_lines(t, ml, hm)
    local f = text_width(t)
    local w = hm
    while w <= f do
        if #wrap_words(t, w) <= ml then return w end
        w = w + 1
    end
    return f
end

-- 获取模式名称
local function mode_name(m)
    if m == M_FIXED_NEG then return tr("game.twenty_four.mode.fixed_negative") end
    if m == M_FLEX_NEG then return tr("game.twenty_four.mode.flex_negative") end
    return tr("game.twenty_four.mode.classic")
end

-- 获取当前光标对应的项目列表
local function active_list()
    if S.cursor_mode == "num" then
        return {
            { k = "num", i = 1 },
            { k = "num", i = 2 },
            { k = "num", i = 3 },
            { k = "num", i = 4 },
        }
    end
    return {
        { k = "op", i = 1 },
        { k = "op", i = 2 },
        { k = "op", i = 3 },
    }
end

-- 获取当前光标位置
local function focus()
    local ls = active_list()
    if S.cursor < 1 then S.cursor = 1 end
    if S.cursor > #ls then S.cursor = #ls end
    return ls[S.cursor], #ls
end

-- 递归检查是否能得到24
local function can24(nums)
    if #nums == 1 then return math.abs(nums[1] - 24) < EPS end
    for i = 1, #nums do
        for j = i + 1, #nums do
            local a, b = nums[i], nums[j]
            local rest = {}
            for k = 1, #nums do
                if k ~= i and k ~= j then rest[#rest + 1] = nums[k] end
            end
            local cand = { a + b, a - b, b - a, a * b }
            if math.abs(b) > EPS then cand[#cand + 1] = a / b end
            if math.abs(a) > EPS then cand[#cand + 1] = b / a end
            for c = 1, #cand do
                local n = { table.unpack(rest) }
                n[#n + 1] = cand[c]
                if can24(n) then return true end
            end
        end
    end
    return false
end

-- 检查指定模式是否有解
local function has_solution(nums, mode)
    if mode == M_FLEX_NEG then
        local absn = { math.abs(nums[1]), math.abs(nums[2]), math.abs(nums[3]), math.abs(nums[4]) }
        for mask = 0, 15 do
            local t = {}
            for i = 1, 4 do
                local bit = math.floor(mask / (2 ^ (i - 1))) % 2
                t[i] = (bit == 1) and -absn[i] or absn[i]
            end
            if can24(t) then return true end
        end
        return false
    end
    return can24({ nums[1], nums[2], nums[3], nums[4] })
end

-- 随机生成一个数字（根据模式决定正负）
local function rand_num(mode)
    local v = rand_int(13) + 1
    if mode == M_CLASSIC then return v end
    return (rand_int(100) < 50) and -v or v
end

-- 加载最佳记录
local function load_best()
    S.best_time = 0
    if type(load_data) ~= "function" then return end
    local ok, d = pcall(load_data, "twenty_four_best_time")
    if not ok then return end
    if type(d) == "number" then
        S.best_time = math.max(0, math.floor(d))
        return
    end
    if type(d) == "table" then
        local s = tonumber(d.time_sec) or tonumber(d.best_time_sec) or 0
        S.best_time = math.max(0, math.floor(s))
    end
end

-- 保存最佳记录
local function save_best()
    if type(save_data) == "function" then
        pcall(save_data, "twenty_four_best_time", { time_sec = S.best_time })
    end
end

-- 提交游戏结果
local function commit_once()
    if S.committed then return end
    S.committed = true
    local t = elapsed_seconds()
    if S.best_time <= 0 or t < S.best_time then
        S.best_time = t
        save_best()
    end
    if type(update_game_stats) == "function" then
        local score = math.max(0, 1000000 - t * 100 - S.steps)
        pcall(update_game_stats, "twenty_four", score, t)
    end
end

-- 重置一局
local function reset_round(mode)
    S.mode = mode or S.mode
    local guard = 0
    while true do
        guard = guard + 1
        local n = { rand_num(S.mode), rand_num(S.mode), rand_num(S.mode), rand_num(S.mode) }
        if has_solution(n, S.mode) or guard > 2000 then
            S.base_nums = { n[1], n[2], n[3], n[4] }
            S.nums = { n[1], n[2], n[3], n[4] }
            break
        end
    end
    S.ops = { OP_EMPTY, OP_EMPTY, OP_EMPTY }
    S.pairs = {}
    S.cursor = 1
    S.cursor_mode = "op"
    S.steps = 0
    S.ready = false
    S.value = nil
    S.win = false
    S.confirm = nil
    S.input_mode = nil
    S.input_buf = ""
    S.end_frame = nil
    S.start_frame = S.frame
    S.committed = false
    S.dirty = true
end

-- 检查两个区间是否交叉
local function cross(l1, r1, l2, r2)
    return (l1 < l2 and l2 < r1 and r1 < r2) or (l2 < l1 and l1 < r2 and r2 < r1)
end

-- 构建括号映射
local function pair_map()
    local L, R = {}, {}
    for i = 1, 8 do L[i], R[i] = {}, {} end
    for c = 1, 4 do
        local p = S.pairs[c]
        if p then
            L[p.l][#L[p.l] + 1] = { c = c, l = p.l, r = p.r }
            R[p.r][#R[p.r] + 1] = { c = c, l = p.l, r = p.r }
        end
    end
    for i = 1, 8 do
        table.sort(L[i], function(a, b)
            if a.l ~= b.l then return a.l < b.l end
            return a.r > b.r
        end)
        table.sort(R[i], function(a, b)
            if a.r ~= b.r then return a.r < b.r end
            return a.l > b.l
        end)
    end
    return L, R
end

-- 添加括号对
local function add_pair(l, r)
    if l < 1 or r > 8 or l >= r then
        S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_order"), "red", S.frame + FPS * 2
        S.dirty = true
        return false
    end
    local nums, ops = 0, 0
    for p = l, r - 1 do
        if p % 2 == 1 then
            nums = nums + 1
        else
            ops = ops + 1
        end
    end

    -- 括号必须包含至少两个数字和一个运算符
    local valid_shape = (l % 2 == 1) and (r % 2 == 0) and (nums >= 2) and (ops >= 1) and (nums == ops + 1)
    if not valid_shape then
        S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_single"), "red", S.frame + FPS * 2
        S.dirty = true
        return false
    end
    -- 检查交叉
    for i = 1, 4 do
        local p = S.pairs[i]
        if p and cross(l, r, p.l, p.r) then
            S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_cross"), "red", S.frame + FPS * 2
            S.dirty = true
            return false
        end
        if p and p.l == l and p.r == r then
            S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_duplicate"), "red", S.frame + FPS * 2
            S.dirty = true
            return false
        end
    end
    -- 找空位添加
    for i = 1, 4 do
        if S.pairs[i] == nil then
            S.pairs[i] = { l = l, r = r }
            S.steps = S.steps + 1
            S.dirty = true
            return true
        end
    end
    S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_paren_full"), "red", S.frame + FPS * 2
    S.dirty = true
    return false
end

-- 计算表达式
local function eval_expr()
    S.ready, S.value = false, nil
    for i = 1, 3 do if S.ops[i] == OP_EMPTY then return end end

    local toks = {
        tostring(S.nums[1]), S.ops[1],
        tostring(S.nums[2]), S.ops[2],
        tostring(S.nums[3]), S.ops[3],
        tostring(S.nums[4]),
    }

    local L, R = pair_map()
    local parts = {}
    for b = 1, 8 do
        for i = 1, #R[b] do parts[#parts + 1] = ")" end
        for i = 1, #L[b] do parts[#parts + 1] = "(" end
        if b <= 7 then parts[#parts + 1] = toks[b] end
    end

    local expr = table.concat(parts, "")
    S.ready = true
    local fn = load("return " .. expr)
    if fn == nil then return end
    local ok, v = pcall(fn)
    if (not ok) or type(v) ~= "number" or v ~= v or v == math.huge or v == -math.huge then return end
    S.value = v
    if math.abs(v - 24) < EPS then
        S.win = true
        S.end_frame = S.frame
        commit_once()
    end
end

-- 设置运算符
local function set_op(i, op)
    if S.ops[i] ~= op then
        S.ops[i] = op
        S.steps = S.steps + 1
        eval_expr()
        S.dirty = true
    end
end

-- 设置数字符号
local function set_num_sign(i, sign)
    local v = math.abs(S.nums[i])
    local t = (sign < 0) and -v or v
    if S.nums[i] ~= t then
        S.nums[i] = t
        S.steps = S.steps + 1
        eval_expr()
        S.dirty = true
    end
end

-- 交换两个数字的位置
local function swap_nums(a, b)
    if a < 1 or a > 4 or b < 1 or b > 4 or a == b then return end
    local t = S.nums[a]
    S.nums[a] = S.nums[b]
    S.nums[b] = t
    S.steps = S.steps + 1
    eval_expr()
    S.dirty = true
end

-- 获取当前消息
local function current_message()
    if S.confirm == "restart" then
        return tr("game.twenty_four.confirm_restart"), "yellow"
    end
    if S.confirm == "exit" then
        return tr("game.twenty_four.confirm_exit"), "yellow"
    end
    if S.input_mode == "paren_add" then
        if S.input_buf == "" then
            return tr("game.twenty_four.prompt_add_paren"), "dark_gray"
        end
        return S.input_buf, "yellow"
    end
    if S.input_mode == "paren_remove" then
        return "", "yellow"
    end
    if S.input_mode == "difficulty" then
        return tr("game.twenty_four.prompt_difficulty"), "dark_gray"
    end
    if S.win then
        return tr("game.twenty_four.win_banner") .. "  " .. tr("game.twenty_four.result_controls"), "green"
    end
    if S.toast and S.frame <= S.toast_until then
        return S.toast, S.toast_color
    end
    return tr("game.twenty_four.ready"), "dark_gray"
end

-- 获取计算结果文本
local function result_text()
    if not S.ready then return "?", "blue" end
    if S.value == nil then return "NaN", "red" end
    local iv = math.floor(S.value + 0.5)
    local t = (math.abs(S.value - iv) < 1e-9) and tostring(iv) or string.format("%.6g", S.value)
    return t, (math.abs(S.value - 24) < EPS) and "green" or "red"
end

-- 渲染中间区域（表达式）
local function render_mid(y, tw)
    local f = focus()
    local toks = {
        tostring(S.nums[1]), S.ops[1],
        tostring(S.nums[2]), S.ops[2],
        tostring(S.nums[3]), S.ops[3],
        tostring(S.nums[4]),
    }

    local L, R = pair_map()
    local seg, bx, cur = {}, {}, 1

    local function boundary_chars(b)
        local out = {}
        for i = 1, #R[b] do
            out[#out + 1] = { t = ")", fg = PAREN_COLORS[R[b][i].c], bg = "black" }
        end
        for i = 1, #L[b] do
            out[#out + 1] = { t = "(", fg = PAREN_COLORS[L[b][i].c], bg = "black" }
        end
        return out
    end

    local function push_boundary_slot(b)
        local bc = boundary_chars(b)
        local used = math.min(#bc, 2)
        local align_right = (b % 2 == 1)
        if align_right and used < 2 then
            local sp = 2 - used
            seg[#seg + 1] = { t = string.rep(" ", sp), fg = "white", bg = "black" }
            cur = cur + sp
        end

        for i = 1, used do
            seg[#seg + 1] = bc[i]
            cur = cur + 1
        end

        if (not align_right) and used < 2 then
            local sp = 2 - used
            seg[#seg + 1] = { t = string.rep(" ", sp), fg = "white", bg = "black" }
            cur = cur + sp
        end
    end

    bx[1] = cur
    push_boundary_slot(1)

    for b = 1, 7 do
        local is_num = (b % 2 == 1)
        local fg, bg = "white", "black"

        if is_num then
            local ni = math.floor((b + 1) / 2)
            local hit = (f.k == "num" and f.i == ni)
            fg = hit and "black" or "white"
            if hit then bg = "light_yellow" end
        else
            local oi = math.floor(b / 2)
            local hit = (f.k == "op" and f.i == oi)
            if toks[b] == OP_EMPTY then
                fg = "yellow"
            else
                fg = hit and "#3f48cc" or "cyan"
            end
            if hit then bg = "light_yellow" end
        end

        seg[#seg + 1] = { t = toks[b], fg = fg, bg = bg }
        cur = cur + text_width(toks[b])

        bx[b + 1] = cur
        push_boundary_slot(b + 1)
    end

    local rv, rc = result_text()
    seg[#seg + 1] = { t = "= ", fg = "white", bg = "black" }
    seg[#seg + 1] = { t = rv, fg = rc, bg = "black" }

    local sw = 0
    for i = 1, #seg do sw = sw + text_width(seg[i].t) end

    -- 计算起始位置
    local sx_axis = centered_x(string.rep(" ", sw), 1, tw)
    local max_sx = math.max(1, tw - sw + 1)
    if sx_axis > max_sx then sx_axis = max_sx end
    local sx_expr = sx_axis
    if sx_expr > max_sx then sx_expr = max_sx end

    draw_text(1, y, string.rep(" ", tw), "white", "black")
    draw_text(1, y + 1, string.rep(" ", tw), "white", "black")
    draw_text(1, y + 2, string.rep(" ", tw), "white", "black")

    for i = 1, 8 do
        local x = sx_axis + bx[i] - 1
        draw_text(x, y, tostring(i), "white", "black")
        draw_text(x, y + 1, "├┐", "white", "black")
    end

    local x = sx_expr
    for i = 1, #seg do
        draw_text(x, y + 2, seg[i].t, seg[i].fg, seg[i].bg)
        x = x + text_width(seg[i].t)
    end
end

-- 获取控制说明文本
local function controls_text()
    return tr("game.twenty_four.controls")
end

-- 计算最小所需终端尺寸
local function minimum_required_size()
    local cw = min_width_for_lines(controls_text(), 3, 56)
    local mw = math.max(
        text_width(tr("game.twenty_four.confirm_restart")),
        text_width(tr("game.twenty_four.confirm_exit")),
        text_width(tr("game.twenty_four.prompt_difficulty"))
    )
    local tw = math.max(
        text_width(tr("game.twenty_four.best_time") .. " " .. format_duration(0)),
        text_width(tr("game.twenty_four.time") ..
        " " .. format_duration(0) .. "  " .. tr("game.twenty_four.steps") .. " 9999")
    )
    return math.max(cw, mw, tw, 64) + 2, 13
end

-- 绘制终端尺寸警告
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

-- 确保终端尺寸足够
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
    local chg = (not S.warn) or S.lw ~= tw or S.lh ~= th or S.lmw ~= mw or S.lmh ~= mh
    if chg then
        draw_terminal_size_warning(tw, th, mw, mh)
        S.lw, S.lh, S.lmw, S.lmh = tw, th, mw, mh
    end
    S.warn = true
    return false
end

-- 获取状态行文本
local function status_line_text()
    return tr("game.twenty_four.time") ..
    " " .. format_duration(elapsed_seconds()) .. "  " .. tr("game.twenty_four.steps") .. " " .. tostring(S.steps)
end

-- 绘制括号移除提示
local function draw_paren_remove_prompt(y, tw)
    local segs = {}
    for i = 1, 4 do
        if S.pairs[i] then
            if #segs > 0 then
                segs[#segs + 1] = { t = "   ", fg = "white", bg = "black" }
            end
            segs[#segs + 1] = { t = tostring(i) .. " ", fg = "yellow", bg = "black" }
            segs[#segs + 1] = { t = "(", fg = PAREN_COLORS[i], bg = "black" }
            segs[#segs + 1] = { t = ")", fg = PAREN_COLORS[i], bg = "black" }
        end
    end

    if #segs == 0 then
        local t = tr("game.twenty_four.no_parens")
        draw_text(centered_x(t, 1, tw), y, t, "dark_gray", "black")
        return
    end

    local w = 0
    for i = 1, #segs do w = w + text_width(segs[i].t) end
    local x = centered_x(string.rep(" ", w), 1, tw)
    for i = 1, #segs do
        draw_text(x, y, segs[i].t, segs[i].fg, segs[i].bg)
        x = x + text_width(segs[i].t)
    end
end

-- 主渲染函数
local function render()
    local tw, th = terminal_size()
    local lines = wrap_words(controls_text(), math.max(20, tw - 2))
    if #lines > 3 then lines = { lines[1], lines[2], lines[3] } end
    local top = math.floor((th - 10 - #lines) / 2)
    if top < 1 then top = 1 end
    local best = tr("game.twenty_four.best_time") ..
    "  " .. ((S.best_time > 0) and format_duration(S.best_time) or tr("game.twenty_four.none"))
    local stat = status_line_text()
    local m, mc = current_message()
    for i = 0, 2 do draw_text(1, top + i, string.rep(" ", tw), "white", "black") end
    draw_text(centered_x(best, 1, tw), top, best, "dark_gray", "black")
    draw_text(centered_x(stat, 1, tw), top + 1, stat, "light_cyan", "black")
    S.last_stat = stat
    if S.input_mode == "paren_remove" then
        draw_paren_remove_prompt(top + 2, tw)
    else
        draw_text(centered_x(m, 1, tw), top + 2, m, mc, "black")
    end
    render_mid(top + 4, tw)
    local cy = top + 8
    for i = 0, 2 do draw_text(1, cy + i, string.rep(" ", tw), "white", "black") end
    local off = math.floor((3 - #lines) / 2)
    if off < 0 then off = 0 end
    for i = 1, #lines do
        draw_text(centered_x(lines[i], 1, tw), cy + off + i - 1, lines[i], "white", "black")
    end
end

-- 仅更新时间显示（优化）
local function render_time_only()
    local tw, th = terminal_size()
    local lines = wrap_words(controls_text(), math.max(20, tw - 2))
    if #lines > 3 then lines = { lines[1], lines[2], lines[3] } end
    local top = math.floor((th - 10 - #lines) / 2)
    if top < 1 then top = 1 end
    local stat = status_line_text()
    local old = S.last_stat or ""
    local cw = math.max(text_width(old), text_width(stat))
    local clear_x = centered_x(string.rep(" ", cw), 1, tw)
    draw_text(clear_x, top + 1, string.rep(" ", cw), "white", "black")
    draw_text(centered_x(stat, 1, tw), top + 1, stat, "light_cyan", "black")
    S.last_stat = stat
end

-- 处理确认模式按键
local function handle_confirm_key(k)
    if k == "y" or k == "enter" then
        if S.confirm == "restart" then
            S.confirm = nil
            reset_round(S.mode)
            return "changed"
        end
        if S.confirm == "exit" then
            return "exit"
        end
    end
    if k == "n" or k == "q" or k == "esc" then
        S.confirm = nil
        S.dirty = true
        return "changed"
    end
    return "none"
end

-- 处理输入模式按键
local function handle_input_mode(k)
    if k == "esc" or k == "q" then
        S.input_mode = nil
        S.input_buf = ""
        S.dirty = true
        return "changed"
    end
    if k == "backspace" or k == "delete" then
        if #S.input_buf > 0 then
            S.input_buf = string.sub(S.input_buf, 1, #S.input_buf - 1)
            S.dirty = true
        end
        return "changed"
    end
    if S.input_mode == "difficulty" then
        if k:match("^[1-3]$") then
            local d = tonumber(k)
            S.input_mode, S.input_buf = nil, ""
            reset_round(d)
            return "changed"
        end
        return "changed"
    end
    if S.input_mode == "paren_add" then
        if (k:match("^%d$") or k == "space") and #S.input_buf < 5 then
            S.input_buf = S.input_buf .. ((k == "space") and " " or k)
            S.dirty = true
            return "changed"
        end
        if k == "enter" then
            local a, b = S.input_buf:match("^(%d+)%s+(%d+)$")
            S.input_mode, S.input_buf = nil, ""
            if a and b then
                add_pair(math.min(tonumber(a), tonumber(b)), math.max(tonumber(a), tonumber(b)))
                eval_expr()
            else
                S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_input"), "red", S.frame + FPS * 2
            end
            S.dirty = true
            return "changed"
        end
        return "changed"
    end
    if S.input_mode == "paren_remove" then
        if k:match("^[1-4]$") then
            local i = tonumber(k)
            if i and S.pairs[i] then
                S.pairs[i] = nil
                S.steps = S.steps + 1
                eval_expr()
                S.input_mode, S.input_buf = nil, ""
            else
                S.toast, S.toast_color, S.toast_until = tr("game.twenty_four.err_remove_paren"), "red", S.frame + FPS * 2
            end
            S.dirty = true
            return "changed"
        end
        return "changed"
    end
    return "none"
end

-- 处理普通按键
local function handle_input(k)
    local f, n = focus()

    if k == "left" then
        if S.cursor > 1 then
            S.cursor = S.cursor - 1
            S.dirty = true
        end
        return "changed"
    end
    if k == "right" then
        if S.cursor < n then
            S.cursor = S.cursor + 1
            S.dirty = true
        end
        return "changed"
    end

    if S.win then
        if k == "r" then
            reset_round(S.mode)
            return "changed"
        end
        if k == "q" or k == "esc" then
            return "exit"
        end
        return "none"
    end

    if k == "c" then
        if S.cursor_mode == "op" then
            S.cursor_mode = "num"
            if S.cursor > 4 then S.cursor = 4 end
        else
            S.cursor_mode = "op"
            if S.cursor > 3 then S.cursor = 3 end
        end
        S.dirty = true
        return "changed"
    end

    if k == "up" and f.k == "num" then
        if f.i > 1 then
            swap_nums(f.i, f.i - 1)
            S.cursor = S.cursor - 1
        end
        return "changed"
    end
    if k == "down" and f.k == "num" then
        if f.i < 4 then
            swap_nums(f.i, f.i + 1)
            S.cursor = S.cursor + 1
        end
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
    if k == "p" then
        S.input_mode, S.input_buf = "difficulty", ""
        S.dirty = true
        return "changed"
    end
    if k == "z" then
        S.input_mode, S.input_buf = "paren_add", ""
        S.dirty = true
        return "changed"
    end
    if k == "x" then
        local has_pair = false
        for i = 1, 4 do
            if S.pairs[i] then
                has_pair = true
                break
            end
        end
        if has_pair then
            S.input_mode, S.input_buf = "paren_remove", ""
        else
            S.toast = tr("game.twenty_four.no_parens")
            S.toast_color = "dark_gray"
            S.toast_until = S.frame + FPS * 2
        end
        S.dirty = true
        return "changed"
    end

    if k == "space" and f.k == "op" then
        set_op(f.i, OP_EMPTY)
        return "changed"
    end

    if k == "1" or k == "+" then
        if f.k == "op" then
            set_op(f.i, "+")
        elseif f.k == "num" and S.mode == M_FLEX_NEG then
            set_num_sign(f.i, 1)
        end
        return "changed"
    end

    if k == "2" or k == "-" then
        if f.k == "op" then
            set_op(f.i, "-")
        elseif f.k == "num" and S.mode == M_FLEX_NEG then
            set_num_sign(f.i, -1)
        end
        return "changed"
    end

    if k == "3" or k == "*" then
        if f.k == "op" then
            set_op(f.i, "*")
        end
        return "changed"
    end
    if k == "4" or k == "/" then
        if f.k == "op" then
            set_op(f.i, "/")
        end
        return "changed"
    end

    return "none"
end

-- 刷新脏标记
local function refresh_dirty_flags()
    local e = elapsed_seconds()
    if e ~= S.last_elapsed then
        S.last_elapsed = e
        S.time_dirty = true
    end
    local tv = S.toast ~= nil and S.frame <= S.toast_until
    if tv ~= S.last_toast then
        S.last_toast = tv
        S.dirty = true
    end
    if (not tv) and S.toast ~= nil then
        S.toast = nil
        S.dirty = true
    end
end

-- 游戏初始化
local function init_game()
    clear()
    load_best()
    S.tw, S.th = terminal_size()
    reset_round(M_CLASSIC)
    S.dirty = true
    if type(clear_input_buffer) == "function" then
        pcall(clear_input_buffer)
    end
end

-- 主游戏循环
local function game_loop()
    while true do
        local k = normalize_key(get_key(false))
        if ensure_terminal_size_ok() then
            local a = "none"
            if S.confirm then
                a = handle_confirm_key(k)
            elseif S.input_mode then
                a = handle_input_mode(k)
            else
                a = handle_input(k)
            end
            if a == "exit" then
                return
            end
            refresh_dirty_flags()
            if S.dirty then
                render()
                S.dirty = false
                S.time_dirty = false
            elseif S.time_dirty then
                render_time_only()
                S.time_dirty = false
            end
            S.frame = S.frame + 1
        else
            if k == "q" or k == "esc" then
                return
            end
        end
        sleep(FRAME_MS)
    end
end

-- 启动游戏
init_game()
game_loop()
