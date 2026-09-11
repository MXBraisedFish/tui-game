# Lua 元方法、元表与伪面向对象

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 元方法

元方法是元表中以 \_\_ 开头的特殊键，用于控制表在特定条件下触发自定义行为。

简单来说，元方法就是规定表在遇到某些情况时如何执行的规则。

常用的元方法有以下四个：

| 类别   | 元方法       | 触发时机                   |
| ------ | ------------ | -------------------------- |
| 索引   | `__index`    | 读取不存在的键             |
| 索引   | `__newindex` | 给不存在的键赋值           |
| 调用   | `__call`     | 把表当函数调用             |
| 字符串 | `__tostring` | `tostring(t)` / `print(t)` |

---

## 元表

元表是一张具有特殊身份的表，它可以存储元方法，并让普通的表在特定条件下触发自定义行为。

换句话说，元表就是存储规则的清单。但它本身依旧是一张表，继续用来存储其他数据也不会有任何问题。

```lua
local t = {}
local mt = {}                      -- 元表
setmetatable(t, mt)                -- 将表 t 的元表设置为 mt
print(getmetatable(t) == mt)       -- 查询表 t 的元表为 mt
setmetatable(t, nil)               -- 移除元表
```

---

## 伪面向对象

Lua 中并没有 `class` 等与类相关的关键字，但可以借助元方法、元表和函数模拟出面向对象。

表内函数有两种调用操作符：

| 操作符       | 等价形式        | 说明                          |
| ------------ | --------------- | ----------------------------- |
| obj.method() | 无              | 需要手动传递 self 变量        |
| obj:method() | obj.method(obj) | 自动把 obj 作为第一个参数传入 |

下面逐步推导如何构建。

### 1. 创建两个实例

```lua
local player1 = {
  hp = 100,
  name = "Alice",
  say = function(name)
    print("I am " .. name)
  end
}

local player2 = {
  hp = 30,
  name = "Peter",
  say = function(name)
    print("I am " .. name)
  end
}
```

### 2. 提取公共方法

```lua
function say(name)
  print("I am " .. name)
end

local player1 = {
  hp = 100,
  name = "Alice"
}

local player2 = {
  hp = 30,
  name = "Peter"
}
```

### 3. 用一个表来构建构造方法，同时将公共方法放入表中

```lua
local Player = {} -- 这个表就充当了 class Player，用于声明类

function Player.new(name, hp) -- 注意是 . 操作符
  local obj = {}
  obj.name = name
  obj.hp = hp
  return obj
end

function Player.say(name) -- 注意是 . 操作符
  print("I am " .. name)
end

player1 = Player.new("Alice", 100)
player2 = Player.new("Peter", 30)
```

### 4. 构造一个元表，并放入一个元方法

```lua
local Player = {}

local MataTable = {}

MataTable.__index = Player -- __index 表示当表内不存在某个键时该怎么办，这里就是：如果不存在，就去 Player 这个表中查找

function Player.new(name, hp)
  local obj = {}
  obj.name = name
  obj.hp = hp
  return obj
end

function Player.say(name)
  print("I am " .. name)
end

player1 = Player.new("Alice", 100)
player2 = Player.new("Peter", 30)
```

### 5. 将元表设置给构造方法中的表

```lua
local Player = {}

local MataTable = {}

MataTable.__index = Player

function Player.new(name, hp)
  local obj = setmetatable({}, MataTable) -- 现在所构造出的实例对象，在遇到不存在的键时，还会去 Player 中查找
  obj.name = name
  obj.hp = hp
  return obj
end

function Player.say(name)
  print("I am " .. name)
end

player1 = Player.new("Alice", 100)
player2 = Player.new("Peter", 30)
```

### 6. 修改公共方法的操作符，让其自动传递 self 参数

```lua
local Player = {}

local MataTable = {}

MataTable.__index = Player

function Player.new(name, hp)
  local obj = setmetatable({}, MataTable)
  obj.name = name
  obj.hp = hp
  return obj
end

function Player:say()
  print("I am " .. self.name) -- 这里的 . 改为了 :，并将原本的 name 改为 self.name
end

player1 = Player.new("Alice", 100)
player2 = Player.new("Peter", 30)

player1.say(player1) -- . 调用必须传递自己
player2:say() -- : 调用自动传递自己作为第一个参数
```

### 7. 合并元表

```lua
local Player = {}

Player.__index = Player -- 元表和类合并。它们本身都是表，元表只是一个额外的称呼，直接把元方法放在自己的表内即可

function Player.new(name, hp)
  local obj = setmetatable({}, Player) -- 将自身设置为元表，并不会影响其他构造方法的使用
  obj.name = name
  obj.hp = hp
  return obj
end

function Player:say()
  print("I am " .. self.name) -- 这里的 . 改为了 :，并将原本的 name 改为 self.name
end

player1 = Player.new("Alice", 100)
player2 = Player.new("Peter", 30)

player1:say()
player2:say()
```

以上就构建出了一个既有公共方法、又有实例字段的面向对象模型。

---

| 上一篇                                        | 下一篇                                        |
| ------------------------------------------ | ------------------------------------------ |
| [Lua 关键字](./5_keyword.md)                  | [Lua 模块](./7_module.md)                    |
