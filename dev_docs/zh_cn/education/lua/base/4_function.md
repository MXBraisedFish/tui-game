# Lua 函数

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 定义与调用

使用 `function` 关键字来定义函数，然后使用函数的名字直接调用。

## 示例

```lua
function add(a, b)
  return a + b
end

print(add(1, 3))
```

输出：

```text
4
```

---

## 参数与返回值

Lua 的参数和返回值都非常灵活。

参数和返回值允许所有基本数据类型，并且允许多返回值。

### 示例

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
print(lo .. " " .. hi)
```

输出：

```text
1 5
```

---

## 可变参数 `...`

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

| 定义              | 调用     | 等价形式    |
| ----------------- | -------- | ----------- |
| `function t.f(a)` | `t.f(x)` | —           |
| `function t:f(a)` | `t:f(x)` | `t.f(t, x)` |
