函数是 Lua 的核心概念之一，它是一等值（first-class value），可以赋值、传参、返回。下面系统讲解。

## 1. 定义与调用

```lua
-- 方式一：标准定义
function add(a, b)
  return a + b
end

-- 方式二：匿名函数赋给变量
local sub = function(a, b)
  return a - b
end

print(add(3, 5))   -- 8
print(sub(3, 5))   -- -2
```

> 两者几乎等价，但有个细微区别：`function add()` 定义的是（默认）全局函数，`local function` 定义的是局部函数。**推荐用 `local`**，避免污染全局。

---

## 2. 参数与返回值

Lua 的参数和返回值都非常灵活。

### 多返回值

```lua
local function minmax(t)
  local min, max = t[1], t[1]
  for _, v in ipairs(t) do
    if v < min then min = v end
    if v > max then max = v end
  end
  return min, max   -- 返回两个值
end

local lo, hi = minmax({3, 1, 4, 1, 5})
print(lo, hi)   -- 1   5
```

### 多返回值的"截断"规则

只有当函数调用处于**表达式列表最后位置**时，才会展开多个返回值：

```lua
local function f() return 1, 2, 3 end

local a, b, c = f()       -- a=1 b=2 c=3
print(f())                -- 1  2  3

local t = {f()}           -- {1, 2, 3}（在末尾，展开）
local t2 = {f(), 10}      -- {1, 10}（不在末尾，只取第一个）
local x, y = f(), 10      -- x=1 y=10（f() 只取第一个值）
print((f()))              -- 1（用括号包裹，强制只返回一个值）
```

### 可变参数 `...`

```lua
local function sum(...)
  local s = 0
  for _, v in ipairs({...}) do
    s = s + v
  end
  return s
end

print(sum(1, 2, 3, 4))   -- 10

-- 用 select 处理可变参数
local function count(...)
  return select("#", ...)   -- 参数个数
end
print(count(1, 2, 3))       -- 3

local function tail(...)
  return select(2, ...)     -- 从第2个开始返回
end
print(tail("a", "b", "c"))  -- b  c
```

### 参数默认值

Lua 没有内置默认参数，用 `or` 实现：

```lua
local function greet(name)
  name = name or "world"
  print("Hello, " .. name)
end
greet()        -- Hello, world
greet("Lua")   -- Hello, Lua
```

---

## 3. 函数是第一类值

函数可以赋给变量、作为参数、作为返回值。

```lua
-- 作为参数（高阶函数）
local function apply(fn, x, y)
  return fn(x, y)
end
print(apply(add, 3, 4))   -- 7

-- 作为返回值
local function makeAdder(n)
  return function(x)      -- 闭包
    return x + n
  end
end

local add10 = makeAdder(10)
print(add10(5))   -- 15
```

---

## 4. 闭包（Closure）

闭包是"能访问其定义时所在作用域变量的函数"。它是 Lua 非常重要的特性。

```lua
local function counter()
  local count = 0
  return function()
    count = count + 1
    return count
  end
end

local c1 = counter()
print(c1())   -- 1
print(c1())   -- 2
print(c1())   -- 3

local c2 = counter()   -- 新的独立闭包
print(c2())   -- 1
```

`count` 是 `counter` 的局部变量，但内部函数一直"记住"它，形成了独立的状态。

**注意循环中创建闭包的陷阱**：

```lua
-- 错误：所有函数共享同一个 i
local fns = {}
for i = 1, 3 do
  fns[i] = function() return i end
end
print(fns[1](), fns[2](), fns[3]())   -- 1  2  3（数值 for 每次都新建 i，其实没问题）

-- while 循环中才会共享
local fns2 = {}
local i = 1
while i <= 3 do
  fns2[i] = function() return i end
  i = i + 1
end
print(fns2[1](), fns2[2](), fns2[3]())  -- 4  4  4（共享同一个 i）
```

---

## 5. 方法（面向对象）

用冒号 `:` 定义和调用方法，会自动传递 `self`。

```lua
local Person = {}
Person.__index = Person

function Person.new(name)      -- 相当于 Person.new(self, name)
  local obj = setmetatable({}, Person)
  obj.name = name
  return obj
end

function Person:greet()        -- 冒号定义，隐式参数 self
  print("Hi, I'm " .. self.name)
end

local p = Person.new("Alice")
p:greet()                      -- Hi, I'm Alice
p.greet(p)                     -- 等价写法
```

**点与冒号的区别**：

```lua
function t.foo(a, b) end   -- 点：需要显式传 self
function t:foo(a) end      -- 冒号：等价于 t.foo(self, a)
```

| 定义 | 调用 | 等价形式 |
|------|------|----------|
| `function t.f(a)` | `t.f(x)` | — |
| `function t:f(a)` | `t:f(x)` | `t.f(t, x)` |

---

## 6. 函数与 table

```lua
local obj = {
  value = 10,
  get = function(self)
    return self.value
  end,
}
print(obj:get())   -- 10
```

---

## 7. 尾调用优化

如果函数的**最后一步**是调用另一个函数并直接返回其结果，Lua 会复用栈帧，避免栈溢出。

```lua
-- 尾调用（有优化）
local function loop(n)
  if n == 0 then return "done" end
  return loop(n - 1)   -- 最后一步是调用，直接返回
end
print(loop(1000000))   -- 不会栈溢出

-- 非尾调用（无优化）
local function loop2(n)
  if n == 0 then return "done" end
  return 1 + loop2(n - 1)   -- 调用后还要 +1，不是尾调用
end
-- loop2(1000000)  -- 会栈溢出
```

> 关键：`return f(...)` 是尾调用；`return f(...) + 1` 或 `local x = f(...)` 都不是。

---

## 8. 常用内置函数

```lua
-- 类型与转换
type(x)          -- 返回类型名字符串
tonumber("42")   -- 字符串转数字
tostring(42)     -- 转字符串

-- 迭代
pairs(t)         -- 遍历所有键值
ipairs(t)        -- 遍历数组部分

-- 元编程
setmetatable(t, mt)
getmetatable(t)

-- 函数调用
pcall(f, ...)    -- 保护调用，返回 ok, err
xpcall(f, handler, ...)

-- 其他
select("#", ...) -- 参数个数
error("msg")     -- 抛出错误
assert(cond, "msg")  -- 断言
```

---

## 9. 综合示例

```lua
-- 一个简单的函数式工具库
local M = {}

-- 映射：对每个元素应用函数
function M.map(t, fn)
  local result = {}
  for i, v in ipairs(t) do
    result[i] = fn(v)
  end
  return result
end

-- 过滤
function M.filter(t, pred)
  local result = {}
  for _, v in ipairs(t) do
    if pred(v) then
      result[#result + 1] = v
    end
  end
  return result
end

-- 归约
function M.reduce(t, fn, init)
  local acc = init
  for _, v in ipairs(t) do
    acc = fn(acc, v)
  end
  return acc
end

-- 使用
local nums = {1, 2, 3, 4, 5}
local doubled = M.map(nums, function(x) return x * 2 end)
local evens = M.filter(nums, function(x) return x % 2 == 0 end)
local total = M.reduce(nums, function(a, b) return a + b end, 0)

print(table.concat(doubled, ","))  -- 2,4,6,8,10
print(table.concat(evens, ","))    -- 2,4
print(total)                       -- 15

return M
```

---

## 关键要点

1. **函数是一等值**，可赋值、传参、返回。
2. **多返回值**只在表达式列表末尾展开，用括号可强制取单个值。
3. **闭包**能记住定义时的外部变量，是状态封装的基础。
4. **冒号 `:` 自动传递 `self`**，是面向对象方法的写法。
5. **尾调用优化**让递归不怕栈溢出，但只对 `return f(...)` 形式生效。
6. **推荐用 `local function`** 定义函数，避免全局污染。

需要我继续讲 **元表与元方法**（实现运算符重载、面向对象的核心）或 **协程** 吗？