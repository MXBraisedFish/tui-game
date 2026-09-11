# Lua 模块

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 什么是模块

模块就是一个**返回表的文件**，表里装着你想暴露的函数、变量。别人 `require` 这个文件，就能拿到这个表直接使用。

类似于其他语言的 `import` 导入。

---

## 基本写法

模块名通常就是文件名（不带 `.lua`）。

```lua
-- mymodule.lua
local M = {}

function M.add(a, b)
  return a + b
end

function M.sub(a, b)
  return a - b
end

return M
```

使用：

```lua
local mymodule = require("mymodule")

print(mymodule.add(1, 2))  -- 3
print(mymodule.sub(5, 3))  -- 2
```

---

## 模块的几种写法

**写法一：返回表（最常用）**

```lua
local M = {}
function M.foo() end
return M
```

**写法二：先定义再挂载**

```lua
local foo = function() end
local bar = function() end

return {
  foo = foo,
  bar = bar,
}
```

**写法三：模块名直接当表名（不推荐）**

```lua
mymodule = {} -- 全局变量，容易污染全局表
function mymodule.foo() end
return mymodule
```

---

## 用 `:` 定义模块方法

如果模块方法需要操作自身状态：

```lua
local M = {}
M.count = 0

function M:inc()
  self.count = self.count + 1
end

return M
```

调用：

```lua
local m = require("mymodule")
m:inc()
print(m.count)  -- 1
```

---

| 上一篇                                        | 下一篇                                        |
| ------------------------------------------ | ------------------------------------------ |
| [Lua 元方法、元表与伪面向对象](./6_object_oriented.md) | [Lua 好习惯建议](./8_good_habits.md)            |
