# Lua 好习惯建议

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 少用全局，变量、函数和模块尽可能使用 `local`

如果不加 `local` 会被标记为全局变量或全局函数，这会导致生命周期难以管理和全局污染。

```lua
local x = 1 -- 好
y = 1       -- 坏，全局变量

local function helper() end -- 好
function helper() end       -- 坏，全局函数


local M = {} -- 避免引用后污染全局
return M
```

## 判断 `nil` 使用显式判断

```lua
if x == nil then end
if x ~= nil then end
```

## 区分 `and` / `or` 的短路返回值

`and` 和 `or` 返回的是操作数本身，不一定是布尔值。

```lua
local name = input or "default" -- 常用默认值写法
```

## 字符串拼接用 `..`，数字两侧留空格

```lua
local s = x .. y -- 好
local s = x..y   -- 坏，易歧义
```

## 不要依赖 `#`

`#` 返回的是“边界”，不是元素个数，遇到有数据空洞的表，返回值不可控。

## 用 `:` 或 `.` 定义的函数，应当也用同样的操作符调用

```lua
function Obj:method() end
obj:method()

function Obj.method(self) end
obj.method(obj)
```

## 及时把不用的引用置 `nil`

```lua
obj = nil -- 帮助 GC 回收
```

## 常写注释来标记代码作用

```lua
-- 给予玩家默认名字
local name = input or "default"
```

---

| 上一篇                                        |
| ------------------------------------------ |
| [Lua 模块](./7_module.md)                    |
