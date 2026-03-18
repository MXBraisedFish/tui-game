-- 文本命令注册表，存储所有自定义文本命令
-- 如果已存在则使用已有表，否则创建新表
TEXT_COMMANDS = TEXT_COMMANDS or {}

local function rt(key)
    if type(translate) == "function" then
        return translate(key)
    end
    return key
end

-- 注册背景色命令
-- 语法：{bg:颜色} 或 {bg:颜色>计数} 或 {bg:clear}
TEXT_COMMANDS.bg = function(params, _ctx)
    -- 获取第一个参数（颜色值），如果为空则返回错误
    local p1 = (params and params[1]) or ""
    if p1 == nil or p1 == "" then
        return { error = rt("rich_text.error.invalid_param") }
    end

    -- 处理清除命令
    if string.lower(p1) == "clear" then
        -- clear 命令不能带有额外参数
        if params[2] ~= nil and tostring(params[2]) ~= "" then
            return { error = rt("rich_text.error.invalid_param") }
        end
        return { clear = true }  -- 返回清除标记
    end

    -- 构建返回结果，设置颜色
    local out = { clear = false, color = tostring(p1) }
    
    -- 处理可选的计数参数（指定颜色生效的字符数）
    if params[2] ~= nil and tostring(params[2]) ~= "" then
        local n = tonumber(params[2])  -- 转换为数字
        if not n or n < 1 then  -- 必须是正整数
            return { error = rt("rich_text.error.invalid_param") }
        end
        out.count = math.floor(n)  -- 取整作为计数
    end
    
    return out  -- 返回颜色设置结果
end
