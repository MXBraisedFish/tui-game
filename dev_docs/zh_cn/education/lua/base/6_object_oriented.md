元表（metatable）是 Lua 最强大的特性之一，它让表（以及 userdata）可以**自定义行为**：运算符重载、`tostring`、`__index` 继承、面向对象等。下面系统讲解。

## 1. 什么是元表

普通表只有键值对。给表附加一个"元表"后，就能改变它的行为（如 `+`、`==`、索引访问等）。元表本身也是一个表，里面存放**元方法**（以 `__` 开头的特殊键）。

```lua
local t = {}
local mt = {}                      -- 元表
setmetatable(t, mt)                -- 给 t 设置元表
print(getmetatable(t) == mt)       -- true
setmetatable(t, nil)               -- 移除元表
```

- `setmetatable(t, mt)`：设置元表（`t` 必须是表）。
- `getmetatable(t)`：获取元表。
- 只有表能改元表；其他类型只能通过 `debug.setmetatable`（不推荐）。

---

## 2. 常用元方法一览

| 元方法 | 触发时机 |
|--------|----------|
| `__index` | 访问不存在的键 |
| `__newindex` | 给不存在的键赋值 |
| `__call` | 把表当函数调用 |
| `__tostring` | `tostring(t)` / `print(t)` |
| `__len` | `#t` |
| `__eq` `__lt` `__le` | `==` `<` `<=` |
| `__add` `__sub` `__mul` `__div` `__mod` `__pow` `__unm` | 算术运算 |
| `__concat` | `..` |
| `__pairs` | `pairs(t)`（5.2+） |
| `__metatable` | 保护元表 |
| `__gc` | 垃圾回收（5.2+） |
| `__mode` | 弱引用表 |

---

## 3. `__index`：访问不存在的键

这是最重要的元方法，是**继承**和**面向对象**的基础。

当访问 `t[k]` 而 `t` 中不存在 `k` 时，Lua 会查找 `t` 元表的 `__index`：

- 若 `__index` 是**表**，则在该表中继续查找 `k`。
- 若 `__index` 是**函数**，则调用 `__index(t, k)`，用其返回值。

```lua
-- __index 是表
local base = {greet = "hello"}
local t = setmetatable({}, {__index = base})
print(t.greet)   -- hello（从 base 找到）

-- __index 是函数
local t2 = setmetatable({}, {
  __index = function(tbl, key)
    return "默认值:" .. key
  end,
})
print(t2.foo)    -- 默认值:foo
```

**实现默认值**：

```lua
local function withDefault(default)
  return setmetatable({}, {
    __index = function(_, k) return default end,
  })
end

local cfg = withDefault(0)
print(cfg.port)   -- 0
```

---

## 4. `__newindex`：赋值不存在的键

当给 `t[k]` 赋值而 `t` 中原本没有 `k` 时触发（已有键的赋值不触发）。

- 若是**表**：赋值转发到该表。
- 若是**函数**：调用 `__newindex(t, k, v)`。

```lua
local t = setmetatable({}, {
  __newindex = function(tbl, key, value)
    print("设置 " .. key .. " = " .. tostring(value))
    rawset(tbl, key, value)   -- 真正写入，避免无限递归
  end,
})

t.x = 10    -- 设置 x = 10
print(t.x)  -- 10
```

**实现只读表**：

```lua
local function readonly(t)
  return setmetatable({}, {
    __index = t,
    __newindex = function()
      error("表是只读的", 2)
    end,
  })
end

local ro = readonly({a = 1})
print(ro.a)    -- 1
-- ro.a = 2    -- 报错：表是只读的
```

> ⚠️ **递归陷阱**：在 `__newindex` 里用 `tbl[key] = value` 会再次触发 `__newindex`，必须用 `rawset`。同理，`__index` 里读取用 `rawget`。

**`rawget` / `rawset` / `rawequal` / `rawlen`**：绕过元表的原始操作。

```lua
local t = setmetatable({}, {__index = function() return 0 end})
print(t.x)          -- 0（走元表）
print(rawget(t, "x")) -- nil（绕过元表）
```

---

## 5. 算术与比较元方法

```lua
local Vec = {}
Vec.__index = Vec

function Vec.new(x, y)
  return setmetatable({x = x, y = y}, Vec)
end

-- 算术运算
Vec.__add = function(a, b) return Vec.new(a.x + b.x, a.y + b.y) end
Vec.__sub = function(a, b) return Vec.new(a.x - b.x, a.y - b.y) end
Vec.__mul = function(a, b)
  if type(a) == "number" then return Vec.new(a * b.x, a * b.y) end
  if type(b) == "number" then return Vec.new(a.x * b, a.y * b) end
  return a.x * b.x + a.y * b.y
end
Vec.__unm = function(a) return Vec.new(-a.x, -a.y) end   -- 一元负号

-- 比较
Vec.__eq = function(a, b) return a.x == b.x and a.y == b.y end
Vec.__lt = function(a, b) return a.x * a.x + a.y * a.y < b.x * b.x + b.y * b.y end
Vec.__le = function(a, b) return not (b < a) end

-- 拼接与长度
Vec.__concat = function(a, b) return tostring(a) .. tostring(b) end
Vec.__len = function(a) return 2 end

-- tostring
Vec.__tostring = function(v) return "(" .. v.x .. ", " .. v.y .. ")" end

-- 调用
Vec.__call = function(self, dx, dy) return Vec.new(self.x + dx, self.y + dy) end

-- 测试
local a = Vec.new(1, 2)
local b = Vec.new(3, 4)
print(a + b)       -- (4, 6)
print(b - a)       -- (2, 2)
print(a * 2)       -- (2, 4)
print(a * b)       -- 11（点积）
print(-a)          -- (-1, -2)
print(a == Vec.new(1, 2))   -- true
print(a < b)       -- true
print(#a)          -- 2
local c = a(10, 20)  -- 调用：类似构造函数
print(c)           -- (11, 22)
```

> **注意**：`__eq` 只对**两个都是表**（或都是 userdata）且**元表相同**时生效；`__lt`/`__le` 可混合类型，但需自行处理。

---

## 6. `__call`：把表当函数

```lua
local counter = setmetatable({count = 0}, {
  __call = function(self, step)
    self.count = self.count + (step or 1)
    return self.count
  end,
})

print(counter())     -- 1
print(counter(5))    -- 6
```

---

## 7. `__tostring`

```lua
local point = setmetatable({x = 3, y = 4}, {
  __tostring = function(p) return "Point(" .. p.x .. ", " .. p.y .. ")" end,
})
print(point)          -- Point(3, 4)
print(tostring(point))-- Point(3, 4)
```

---

## 8. `__metatable`：保护元表

```lua
local t = setmetatable({}, {
  __metatable = "locked",     -- getmetatable 返回这个值
})
print(getmetatable(t))        -- locked
-- setmetatable(t, {})        -- 报错：cannot change a protected metatable
```

---

## 9. 面向对象实现

### 9.1 基本类与实例

核心思路：用 `__index` 把实例的方法查找指向类。

```lua
local Animal = {}
Animal.__index = Animal     -- 实例找不到的键，去 Animal 找

function Animal.new(name)
  local self = setmetatable({}, Animal)   -- 创建实例，元表是 Animal
  self.name = name
  return self
end

function Animal:speak()
  print(self.name .. " makes a sound")
end

function Animal:getName()
  return self.name
end

local a = Animal.new("Dog")
a:speak()          -- Dog makes a sound
print(a:getName()) -- Dog
```

**调用链**：`a:speak()` → `a` 中无 `speak` → 查 `__index`（即 Animal）→ 找到 `Animal.speak` → 以 `a` 为 `self` 调用。

### 9.2 继承

```lua
local Dog = setmetatable({}, {__index = Animal})   -- Dog 继承 Animal
Dog.__index = Dog

function Dog.new(name, breed)
  local self = Animal.new(name)      -- 调用父类构造
  self.breed = breed
  return setmetatable(self, Dog)     -- 改元表为 Dog
end

function Dog:speak()                 -- 重写
  print(self.name .. " barks!")
end

function Dog:fetch()
  print(self.name .. " fetches the ball")
end

local d = Dog.new("Rex", "Labrador")
d:speak()     -- Rex barks!（重写生效）
d:fetch()     -- Rex fetches the ball（子类方法）
print(d:getName())   -- Rex（继承父类方法）
```

**查找链**：`d:getName()` → `d` 无 → `Dog.__index`（Dog）无 → `Dog` 的元表 `__index`（Animal）有 → 调用。

### 9.3 封装（私有成员）

用闭包或命名约定实现私有：

```lua
local function Account(balance)
  local self = {}
  local _balance = balance       -- 私有变量

  function self.deposit(n)
    _balance = _balance + n
  end

  function self.getBalance()
    return _balance
  end

  return self
end

local acc = Account(100)
acc.deposit(50)
print(acc.getBalance())   -- 150
-- print(acc._balance)    -- nil（外部访问不到）
```

### 9.4 完整示例：带继承的图形系统

```lua
-- 基类 Shape
local Shape = {}
Shape.__index = Shape

function Shape.new(name)
  return setmetatable({name = name}, Shape)
end

function Shape:area()
  return 0
end

function Shape:describe()
  print(string.format("%s 面积 = %.2f", self.name, self:area()))
end

-- 子类 Circle
local Circle = setmetatable({}, {__index = Shape})
Circle.__index = Circle

function Circle.new(r)
  local self = Shape.new("圆形")
  self.r = r
  return setmetatable(self, Circle)
end

function Circle:area()
  return math.pi * self.r ^ 2
end

-- 子类 Rectangle
local Rectangle = setmetatable({}, {__index = Shape})
Rectangle.__index = Rectangle

function Rectangle.new(w, h)
  local self = Shape.new("矩形")
  self.w, self.h = w, h
  return setmetatable(self, Rectangle)
end

function Rectangle:area()
  return self.w * self.h
end

-- 使用
local shapes = {
  Circle.new(5),
  Rectangle.new(3, 4),
}
for _, s in ipairs(shapes) do
  s:describe()      -- 多态：调用各自 area
end
-- 圆形 面积 = 78.54
-- 矩形 面积 = 12.00
```

---

## 10. 其他元方法

### `__pairs`（5.2+）

自定义 `pairs` 行为：

```lua
local t = setmetatable({1, 2, 3}, {
  __pairs = function(self)
    local i = 0
    return function()
      i = i + 1
      if i <= #self then return i, self[i] end
    end
  end,
})
for k, v in pairs(t) do print(k, v) end
```

### `__mode`：弱引用表

```lua
local weak = setmetatable({}, {__mode = "v"})   -- 值是弱引用
weak[1] = {}     -- 这个表可能随时被 GC 回收
-- "k" 键弱引用，"kv" 键值都弱引用
```

### `__gc`：垃圾回收（5.2+）

```lua
local t = setmetatable({}, {
  __gc = function() print("被回收了") end,
})
t = nil
collectgarbage()   -- 触发回收
```

---

## 11. 关键陷阱

1. **`__index` 递归**：`__index` 指向自身会死循环。
2. **`__newindex` 递归**：里面必须用 `rawset`。
3. **`__eq` 限制**：仅当两值都是表且元表相同才生效。
4. **元表共享**：多个表可共用一个元表（类就是这种用法）。
5. **`__index` 是函数时性能略低**，热路径可考虑查表方式。

---

## 12. 速查模板

```lua
-- 定义类
local MyClass = {}
MyClass.__index = MyClass

-- 构造函数
function MyClass.new(...)
  local self = setmetatable({}, MyClass)
  -- 初始化
  return self
end

-- 方法
function MyClass:method()
  -- self.xxx
end

-- 继承
local SubClass = setmetatable({}, {__index = MyClass})
SubClass.__index = SubClass

function SubClass.new(...)
  local self = MyClass.new(...)
  return setmetatable(self, SubClass)
end

return MyClass
```

---

## 关键要点

1. **元表改变表的行为**，元方法以 `__` 开头。
2. **`__index`** 是继承和面向对象的核心，可为表或函数。
3. **`__newindex`** 控制赋值，可做只读表、代理；内部用 `rawset` 防递归。
4. **运算符元方法**实现重载：`__add`、`__eq`、`__lt`、`__concat`、`__tostring`、`__call` 等。
5. **面向对象三要素**：`__index` 指向类实现方法查找，`setmetatable` 创建实例，元表链实现继承。
6. **多态**由子类重写方法自然实现。

需要我继续讲 **协程**，或者 **Lua 与 C 的交互（C API）** 吗？