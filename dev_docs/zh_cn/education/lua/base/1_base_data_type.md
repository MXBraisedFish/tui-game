# Lua 基本数据类型

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

Lua 有 **8 种**基本数据类型，最常用的是以下 **6 种**：

| 类型       | 说明   | 示例                |
| ---------- | ------ | ------------------- |
| `nil`      | 空值   | `nil`               |
| `boolean`  | 布尔   | `true`, `false`     |
| `number`   | 数字   | `3`, `3.14`, `0x1A` |
| `string`   | 字符串 | `"hello"`           |
| `table`    | 表     | `{1, 2, 3}`         |
| `function` | 函数   | `function() end`    |

---

## nil

`nil` 是唯一的空类型，表示“没有值”。变量值为 `nil` 时，对解释器而言该变量不存在。

```lua
local x
print(x)  -- nil
```

---

## boolean

只有 `true` 和 `false`。

并且只有 `false` 和 `nil` 为假，其他值均为真。

```lua
local a, b = true, false
print(a and b)  -- false
print(a or b)   -- true
print(not a)    -- false
```

---

## number

Lua 5.3 后区分 `integer` 和 `float`，但统一归类为 `number`。Tui Game 使用 5.4 版本。

```lua
local i, f, h, e = 10, 3.14, 0xFF, 1e3
print(type(i))      -- number
print(math.type(3))   -- integer
print(math.type(3.0)) -- float
print("10" + 5)       -- 15（字符串自动转数字）
```

---

## string

可用 `''`、`""` 或长字符串 `[[]]` 包裹。长字符串支持多行且转义失效，内含 `]]` 时可用 `[==[ ]==]`，`=` 数量任意，但必须左右数量相等。

```lua
local s1 = 'hello'
local s2 = "world"
local s3 = [[多行
字符串]]
local s4 = [==[ 内含 ]] 的字符串 ]==]
```

---

## table

Lua 唯一的数据结构，数组从 `1` 开始，允许数据空洞，数组与字典可混用，长度任意变化。

```lua
local t = {1, [3] = 3, n = 2}
print(t[1], t[3], t.n) -- 1  3  2

t[4] = 4
print(t[4]) -- 4

t.x = "hello"
print(t.x) -- hello

t[1] = nil
t.n = nil
```

---

## function

函数是第一类值，可赋给变量、作为参数或返回值。

```lua
-- 显式声明
function add(a, b)
  return a + b
end

-- 匿名函数
add = function(a, b)
  return a + b
end

-- 立即调用
result = (function(a, b)
  return a + b
end)(3, 5)
```

---

## 注释

Lua 注释使用 `--` 开头。

```lua
print("Hello Lua") -- 打印语句
```

---

| 上一篇                                        | 下一篇                                        |
| ------------------------------------------ | ------------------------------------------ |
| [前言与 Lua 简介](./0_preface.md)               | [Lua 运算符](./2_operator.md)                 |
