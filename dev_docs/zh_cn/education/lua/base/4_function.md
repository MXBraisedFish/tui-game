# Lua 函数

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

### 定义与调用

使用 `function` 关键字定义函数，然后直接用函数名调用。

```lua
function add(a, b)
  return a + b
end

print(add(1, 3))  -- 4
```

---

### 参数与返回值

Lua 的参数和返回值非常灵活，允许所有基本数据类型，并且支持多返回值。

```lua
local function minmax(t)
  local min, max = t[1], t[1]
  for _, v in ipairs(t) do
    if v < min then min = v end
    if v > max then max = v end
  end
  return min, max
end

local lo, hi = minmax({3, 1, 4, 1, 5})
print(lo .. " " .. hi)  -- 1 5
```

---

### 函数是第一类值

函数可以赋给变量、作为参数、作为返回值。

```lua
-- 作为参数（高阶函数）
local function apply(fn, x, y)
  return fn(x, y)
end
print(apply(add, 3, 4))  -- 7

-- 作为返回值（闭包）
local function makeAdder(n)
  return function(x)
    return x + n
  end
end

local add10 = makeAdder(10)
print(add10(5))  -- 15
```

---

| 上一篇                                        | 下一篇                                        |
| ------------------------------------------ | ------------------------------------------ |
| [Lua 控制语句](./3_control_statement.md)       | [Lua 关键字](./5_keyword.md)                  |
