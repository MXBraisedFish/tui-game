# Lua 控制语句

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 条件分支

使用 `if`、`elseif`、`else`，代码块不用 `{}`，而是 `then` 和 `end`。

只有 `false` 和 `nil` 为假，其他值均为真。

```lua
local score = 85

if score >= 90 then
  print("A+")
elseif score >= 60 then
  print("B")
else
  print("C")
end
-- 输出：B
```

---

## 循环

Lua 有三种循环。

### while

先判断，再执行，使用 `do` 和 `end`。

```lua
local i = 1
while i <= 5 do
  print(i)
  i = i + 1
end
-- 输出：1 2 3 4 5
```

### repeat ... until

先执行，再判断，条件为**真**时跳出。

until 可访问代码块内的局部变量。

```lua
local i = 1
repeat
  print(i)
  i = i + 1
until i > 5
-- 输出：1 2 3 4 5
```

### 数值 for

用于已知循环次数，三个表达式只在循环开始前求值一次。

步长为 0 会报错。

```lua
for i = 1, 5 do
  print(i)
end

for i = 10, 1, -2 do  -- 初值、终值、步长
  print(i)
end
-- 输出：1 2 3 4 5 10 8 6 4 2
```

### 泛型 for

配合迭代器使用，迭代器返回 `nil` 时结束。

自定义迭代器需返回一个函数，每次调用返回下一个值，结束时返回 `nil`。

```lua
-- ipairs：按索引 1,2,3... 顺序遍历，遇 nil 停止
local list = { "a", "b", "c" }
for i, v in ipairs(list) do
  print(i, v)
end

-- pairs：遍历所有键值对，顺序不保证
local person = { name = "Alice", age = 30 }
for k, v in pairs(person) do
  print(k, v)
end
-- 输出：1 a / 2 b / 3 c / name Alice / age 30
```

---

## 跳转语句

### break

提前跳出当前循环。

```lua
for i = 1, 10 do
  if i == 5 then
    break
  end
  print(i)
end
-- 输出：1 2 3 4
```

### goto 与标签

使用 `goto` 与 `::...::` 在语句间跳转，可模拟 `continue`。

Lua 没有 `continue` 关键字。

`goto` 只能在同一函数内跳转，限制在当前或外层作用域。

同名标签从当前作用域逐级向上查找，跳转到最近的那个。

```lua
for i = 1, 5 do
  if i % 2 == 0 then
    goto continue
  end
  print(i)
  ::continue::
end
-- 输出：1 3 5
```

---

| 上一篇                                        | 下一篇                                        |
| ------------------------------------------ | ------------------------------------------ |
| [Lua 运算符](./2_operator.md)                 | [Lua 函数](./4_function.md)                  |
