# Lua 基本数据类型

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 总览

Lua 中有 **8 种**基本数据类型。

本教程只讲解最常用的 **6 种**。

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

`nil` 是 Lua 中唯一的空类型，直接表示“没有值”。

在 Lua 解释器中，某一个变量值为 `nil`，对于解释器就表示这个变量不存在。

### 示例

```lua
local x
print(x)
```

输出：

```text
nil
```

### 额外补充

在 Lua 解释器中，若某个变量的值为 nil，则对该解释器而言，这个变量即视为不存在。

---

## boolean

布尔值只有 `true` 和 `false`，分别代表**真**和**假**。

### 示例

```lua
local a = true
local b = false

print(a and b)
print(a or b)
print(not a)
```

输出：

```text
false
true
false
```

### 额外补充

- Lua 中只有 `false` 和 `nil` 为**假**，其他类型的值均为**真**。

---

## number

Lua 5.3 版本前只有 `number` 一个数字类型，均保存为双精度浮点类型。
Lua 5.3 版本后新增了 `integer` 整型和 `float` 双精度浮点类型用于区分两种数字，但依旧统一归类为 `number` 类型。

### 示例

```lua
local i = 10
local f = 3.14
local h = 0xFF
local e = 1e3

print(type(i))
print(math.type(3))
print(math.type(3.0))
```

输出:

```text
number
integer
float
```

### 额外补充

- 在 Lua 中字符串与数字之间的运算会将字符串转换为数字处理，而非拼接字符串。

```lua
print("10" + 5) -- 输出 15
```

---

## string

Lua 中的字符串可以使用 `''` 或 `""` 包裹。

除此之外，还有一种特殊的长字符串，使用 `[[]]` 包裹，其中的字符串允许多行书写，且转义失效。如果字符串内包含 `[[` 或 `]]`，可以在两边的方括号之间添加 `=`，例如 `[=[]=]`，`=` 的数量任意，但左右两侧必须相等。

### 示例

```lua
local s1 = 'hello'
local s2 = "world"
local s3 = [[多行
字符串]]
local s4 = [==[ 内含 ]] 的字符串 ]==]
```

### 额外补充

- Lua 中的字符串是不可变对象，任何处理都会重建一个新的字符串。

---

## table

Lua 中唯一的数据结构被称为**表**（table）。

表的强大之处有以下几点：

1. 数组索引从 `1` 开始，更符合直觉。
2. 数组之间允许存在数据空洞。
3. 数组结构与字典（对象）结构可以混用。
4. 长度可任意变化。
5. 操作简单。

### 示例

```lua
local t = {1, [3] = 3, n = 2} -- 基础表，允许下标 1 和 3 之间不存在 下标 2 数据，即数据空洞

print(t[1], t[3], t.n) -- 直接调取，数组从下标 1 开始

t[4] = 4 -- 添加数组下标 4
print(t[4]) -- 数组下标 4 正确添加

t.x = "hello" -- 添加键 x
print(t.x) -- 键 x 正确添加

t[1] = nil -- 直接设为空值（该值不存在）
t.n = nil -- 直接设为空值（该值不存在）
```

输出：

```text
1	3	2
4
hello
```

---

## function

Lua 中函数也属于第一类值，即也可以被赋给变量、作为参数、作为返回值等。

函数有以下几种声明方式：

1. 显式声明

```lua
function add(a, b) then
  return a + b
end
```

2. 匿名函数

```lua
add = function(a, b)
  return a + b
end
```

3. 立即调用函数

```lua
result = (function(a, b)
  return a + b
end)(3, 5)
```

---

| 上一篇                          | 下一篇                        |
| ------------------------------- | ----------------------------- |
| [Lua 简介](./0_introduction.md) | [Lua 运算符](./2_operator.md) |
