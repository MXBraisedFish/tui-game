Lua 的错误处理机制简洁但灵活，核心是 `error`、`assert`、`pcall`、`xpcall` 四个函数，配合 `error` 对象的类型和错误信息传递。下面系统讲解。

## 1. 两类错误

Lua 中的错误分两类：

- **语法错误（compile-time）**：代码无法编译，通常在加载时抛出。
- **运行时错误（run-time）**：执行中出错，如访问 `nil`、类型错误、除零等。

```lua
-- 语法错误：加载时就失败
-- local x = 
-- 报错：unexpected symbol near '<eof>'

-- 运行时错误
local t = nil
-- print(t.x)   -- 报错：attempt to index a nil value (local 't')
```

---

## 2. `error`：主动抛出错误

```lua
error(message [, level])
```

- `message`：任意类型（通常是字符串，也可以是表）。
- `level`：错误位置信息级别（默认 1）。

```lua
local function divide(a, b)
  if b == 0 then
    error("除数不能为 0")
  end
  return a / b
end

divide(10, 0)
-- 报错：lua: test.lua:3: 除数不能为 0
```

**`level` 参数**控制错误信息里显示的位置：

```lua
local function f()
  error("出错了", 1)   -- 位置指向 f 内部（error 调用处）
end

local function g()
  error("出错了", 2)   -- 位置指向调用 g 的地方
end
```

| level | 位置指向 |
|-------|----------|
| 0 | 不添加位置信息 |
| 1（默认） | 调用 `error` 的位置 |
| 2 | 调用 `error` 所在函数的调用者 |

```lua
-- level = 0 时不加前缀
error("纯错误信息", 0)   -- 报错：纯错误信息
```

**抛出非字符串错误对象**（推荐用于结构化错误）：

```lua
local function validate(x)
  if type(x) ~= "number" then
    error({code = 400, msg = "类型错误", value = x})
  end
end
```

---

## 3. `assert`：断言

```lua
assert(value [, message])
```

若 `value` 为假（`nil` 或 `false`），抛出 `message`（默认 `"assertion failed!"`）；否则返回所有参数。

```lua
local function divide(a, b)
  assert(b ~= 0, "除数不能为 0")
  return a / b
end

print(divide(10, 2))   -- 5
-- divide(10, 0)        -- 报错：除数不能为 0

-- 返回值特性
local x = assert(tonumber("42"), "转换失败")
print(x)   -- 42

-- 多返回值
local a, b = assert(1, 2, 3)
print(a, b)   -- 1  2
```

`assert` 常用于**参数校验**和**前置条件检查**，简洁且性能好。

---

## 4. `pcall`：保护调用

```lua
local ok, result = pcall(f, arg1, arg2, ...)
```

- `ok`：布尔值，`true` 表示成功，`false` 表示出错。
- 成功时：`ok = true`，后续是函数的所有返回值。
- 失败时：`ok = false`，后续是错误对象。

```lua
local function divide(a, b)
  if b == 0 then error("除零错误") end
  return a / b
end

-- 成功
local ok, res = pcall(divide, 10, 2)
print(ok, res)      -- true  5.0

-- 失败
local ok2, err = pcall(divide, 10, 0)
print(ok2, err)     -- false  除零错误
```

**多返回值**：

```lua
local function f() return 1, 2, 3 end
local ok, a, b, c = pcall(f)
print(ok, a, b, c)   -- true  1  2  3
```

**错误对象透传**：

```lua
local ok, err = pcall(function()
  error({code = 500, msg = "服务器错误"})
end)
if not ok then
  print(err.code, err.msg)   -- 500  服务器错误
end
```

**保护调用中停止执行**：`pcall` 内的代码出错后会立即停止，但不影响 `pcall` 外的代码继续执行。

```lua
print("开始")
local ok, err = pcall(function()
  error("内部错误")
end)
print("继续执行")   -- 会打印
print("结果:", ok, err)   -- false  内部错误
```

---

## 5. `xpcall`：带错误处理器的保护调用

```lua
xpcall(f, msgh [, arg1, ...])
```

和 `pcall` 类似，但可以传入一个**消息处理器**（message handler），在错误发生时先处理错误（如加堆栈信息），再返回。

```lua
local function handler(err)
  return "【错误】" .. tostring(err)
end

local ok, err = xpcall(function()
  error("出问题了")
end, handler)

print(ok, err)   -- false  【错误】出问题了
```

**常用：获取堆栈信息**

```lua
local ok, err = xpcall(function()
  error("出错了")
end, debug.traceback)      -- 用 debug.traceback 作处理器

print(err)
-- 出错了
-- stack traceback:
--   [C]: in function 'error'
--   ...
```

> `pcall` 只返回错误信息（不含堆栈），`xpcall` 配合 `debug.traceback` 可拿到完整调用栈，调试时非常有用。

**传参**：

```lua
local ok, err = xpcall(divide, debug.traceback, 10, 0)
```

---

## 6. `error`、`assert`、`pcall` 的关系

| 函数 | 作用 | 是否捕获错误 |
|------|------|--------------|
| `error` | 抛出错误 | 否 |
| `assert` | 条件不满足时抛出错误 | 否 |
| `pcall` | 保护调用，捕获错误 | 是 |
| `xpcall` | 保护调用 + 错误处理器 | 是 |

**典型模式**：

```lua
local ok, err = pcall(function()
  assert(cond, "条件不满足")
  -- 可能出错的操作
end)
if not ok then
  print("失败:", err)
end
```

---

## 7. 错误信息的最佳实践

### 7.1 用字符串描述

```lua
error("文件不存在: " .. path)
```

### 7.2 用表表示结构化错误

```lua
local function readConfig(path)
  local f = io.open(path, "r")
  if not f then
    error({
      code = "ENOENT",
      msg = "配置文件不存在",
      path = path,
    })
  end
  local content = f:read("*a")
  f:close()
  return content
end

local ok, err = pcall(readConfig, "config.json")
if not ok then
  print(err.code, err.msg, err.path)
end
```

### 7.3 错误信息包含上下文

```lua
local function parseNumber(s)
  local n = tonumber(s)
  if not n then
    error("无法解析为数字: '" .. tostring(s) .. "'", 2)
  end
  return n
end
```

`level = 2` 让错误位置指向调用 `parseNumber` 的地方，而非函数内部，便于定位。

---

## 8. `error` 与 `debug.traceback` 组合

```lua
local function risky()
  error("业务错误")
end

local ok, trace = xpcall(risky, function(err)
  return err .. "\n" .. debug.traceback("", 2)
end)

print(trace)
```

---

## 9. 返回值错误 vs 抛出错误

Lua 有两种错误处理风格：

**风格一：返回 `nil, err`（类似 Go）**

```lua
local function parse(s)
  if not s then return nil, "输入为空" end
  return tonumber(s) or nil, "解析失败"
end

local n, err = parse("abc")
if not n then print(err) end
```

**风格二：抛出错误（用 `pcall` 捕获）**

```lua
local function parse(s)
  assert(s, "输入为空")
  return assert(tonumber(s), "解析失败")
end

local ok, n = pcall(parse, "abc")
```

| 风格 | 优点 | 缺点 |
|------|------|------|
| 返回 `nil, err` | 显式、可控、无异常开销 | 每处都要检查 |
| 抛出错误 | 简洁、错误自动冒泡 | 需 `pcall` 捕获，容易遗漏 |

> 一般：**可预期的错误**用返回 `nil, err`；**编程错误/不可恢复**用 `error`。

---

## 10. 错误在调用链中的传播

未捕获的错误会沿调用栈向上冒泡，直到被 `pcall`/`xpcall` 捕获，或到达顶层导致程序退出。

```lua
local function level3() error("底层错误") end
local function level2() level3() end
local function level1() level2() end

-- level1()   -- 直接调用会终止程序

local ok, err = pcall(level1)
print(ok, err)   -- false  底层错误
```

---

## 11. 完整示例：安全执行器

```lua
-- 带重试和日志的安全调用
local function safeRun(fn, ...)
  local args = {...}
  local attempt = 0
  local maxRetry = 3

  while attempt < maxRetry do
    attempt = attempt + 1
    local ok, result = xpcall(function()
      return fn(table.unpack(args))
    end, debug.traceback)

    if ok then
      return true, result
    end

    print(string.format("第 %d 次尝试失败: %s", attempt, result))
  end

  return false, "重试 " .. maxRetry .. " 次后仍失败"
end

-- 使用
local success, res = safeRun(function(a, b)
  if math.random() < 0.7 then
    error("随机失败")
  end
  return a + b
end, 3, 4)

print(success, res)
```

---

## 12. 常见陷阱

**1. `pcall` 内的语法错误无法捕获**

```lua
-- pcall 只能捕获运行时错误，语法错误在加载时就失败了
local ok, err = pcall(function()
  -- local x =   -- 语法错误，整个文件都加载不了
end)
```

若需动态执行可能含语法错误的代码，用 `load`：

```lua
local f, err = load("return 1 +")
print(f, err)   -- nil   [string "return 1 +"]:1: unexpected symbol near '<eof>'

local f2 = load("return 1 + 2")
print(f2())     -- 3
```

**2. 错误对象是表时，`tostring` 可能不友好**

```lua
error({code = 1})   -- pcall 返回的是表本身，不是字符串
```

**3. `assert` 的第二个参数只在不满足时求值**

```lua
-- 若拼接代价大，可先判断
assert(cond, "错误: " .. expensiveString())
```

**4. 忘记检查 `pcall` 返回值**

```lua
-- 错误：忽略了 ok
pcall(risky)   -- 出错了但没人知道

-- 正确
local ok, err = pcall(risky)
if not ok then print(err) end
```

---

## 关键要点

1. **`error`** 主动抛错，`level` 控制错误位置。
2. **`assert`** 条件不满足时抛错，且返回其参数，适合前置校验。
3. **`pcall`** 返回 `ok, ...`，保护调用不中断程序。
4. **`xpcall`** 可传消息处理器，配 `debug.traceback` 获取堆栈。
5. **错误对象可以是任意类型**，表适合结构化错误。
6. **语法错误**需用 `load` 捕获，`pcall` 管不了。
7. **可预期错误返回 `nil, err`，不可恢复错误用 `error`**。

需要我继续讲 **协程（coroutine）**，还是 **Lua 与 C 的交互（C API）**？