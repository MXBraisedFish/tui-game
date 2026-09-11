# Lua 控制语句

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 条件分支

Lua 中的条件分支使用 `if`、`elseif`、`else` 三个关键字。

### 示例

```lua
local score = 85

if score >= 90 then
  print("A+")
elseif score >= 60 then
  print("B")
else
  print("C")
end
```

输出：

```text
B
```

### 额外补充

- Lua 中的条件分支代码块不使用 `{}`，而是使用 `then` 和 `end` 关键字。
- Lua 中只有 `false` 和 `nil` 为**假**，其他类型的值均为**真**。

---

## 循环

Lua 中有三种循环。

## while

先判断条件，再执行。

### 示例

```lua
local i = 1
while i <= 5 do
  print(i)
  i = i + 1
end
```

输出：

```text
1
2
3
4
5
```

### 额外补充

- Lua 中的 while 循环代码块不使用 `{}`，而是使用 `do` 和 `end` 关键字。

---

## repeat ... until

先执行，再判断条件。

### 示例

```lua
local i = 1
repeat
  print(i)
  i = i + 1
until i > 5
```

输出：

```text
1
2
3
4
5
```

### 额外补充

- Lua 中的 repeat ... until 循环代码块不使用 `{}`，而是使用 `repeat` 和 `until` 关键字。
- until 关键字可以访问到代码块内的局部变量。
- repeat ... until 循环判断条件为**真**时跳出循环。

---

## for

## 数值 for

用于已知循环次数的可控循环。

### 示例

```lua
for i = 1, 5 do
  print(i)
end

for i = 10, 1, -2 do -- 初值、终值、步长
  print(i)
end
```

输出：

```text
1
2
3
4
5
10
8
6
4
2
```

### 额外补充

- 三个表达式（初值、终值、步长）只在循环开始前求值一次。
- 步长为 0 会报错。

---

## 泛型 for

配合迭代器使用。

### 示例

```lua
-- 数组：ipairs 按索引 1,2,3... 顺序遍历，遇到 nil 停止
local list = { "a", "b", "c" }
for i, v in ipairs(list) do
  print(i, v)
end

-- 字典：pairs 遍历所有键值对，顺序不保证
local person = { name = "Alice", age = 30 }
for k, v in pairs(person) do
  print(k, v)
end
```

输出：

```text
1	a
2	b
3	c
name	Alice
age	30
```

### 额外补充

- 泛型 for 依赖迭代器函数返回 `nil` 时结束循环。
- 自定义迭代器需返回一个函数，每次调用返回下一个值，结束时返回 `nil`。

---

## 跳转语句

## break

用于**提前跳出当前循环**。

### 示例

```lua
for i = 1, 10 do
  if i == 5 then 
    break 
  end
  print(i)
end
```

输出：

```text
1
2
3
4
```

---

## goto 与标签

Lua 中使用 `goto` 与标签 `::...::` 作为语句之间的跳转。

### 示例

```lua
for i = 1, 5 do
  if i % 2 == 0 then
    goto continue -- 模拟 continue 关键字
  end
  print(i)
  ::continue::
end
```

输出：

```text
1
2
5
```

### 额外补充

- Lua 中没有 `continue` 关键字。
- `goto` 跳转限制在当前作用域内或外层作用域内，且只能在同一函数内跳转。
- 同名标签会从当前作用域逐级向上查找，并跳转到最近的那个标签。

---

| 上一篇                               | 下一篇                                   |
| ------------------------------------ | ---------------------------------------- |
| [Lua 运算符](./2_operator.md) | [Lua 控制语句](./4_control_statement.md) |
