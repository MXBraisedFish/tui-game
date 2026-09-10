# Lua 控制语句

Lua 的控制语句比较精简，主要有**条件分支**、**循环**和**跳转**三类。

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 一、条件分支

### if / elseif / else

```lua
local score = 85

if score >= 90 then
  print("优秀")
elseif score >= 60 then
  print("及格")
else
  print("不及格")
end
```

要点：

- 必须包含 `then` 和 `end`，没有 `{}`。
- `elseif` 是一个单词，不是 `else if`。
- 条件表达式里，**只有 `nil` 和 `false` 为假**，其余一切值（包括 `0`、空字符串 `""`）都为真。这一点和很多语言不同，要特别注意。

```lua
if 0 then print("0 为真") end        -- 会输出
if "" then print("空串为真") end     -- 会输出
```

---

## 二、循环

Lua 有四种循环写法。

### 1. while

先判断条件，再执行。

```lua
local i = 1
while i <= 5 do
  print(i)
  i = i + 1
end
```

### 2. repeat ... until

先执行一次，再判断条件；**条件为假时继续循环**，为真时退出。注意判断在末尾，所以循环体至少执行一次。

注意：`until` 后面的条件作用域包含循环体内的局部变量：

```lua
local i = 1
repeat
  print(i)
  i = i + 1
until i > 5 -- 这里的 i 可以访问到
```

### 3. 数值 for

用于已知次数的循环，步长可正可负，默认 1。

```lua
for i = 1, 5 do
  print(i)          -- 1 2 3 4 5
end

for i = 10, 1, -2 do
  print(i)          -- 10 8 6 4 2
end
```

说明：

- 三个表达式（初值、终值、步长）只在循环开始前求值一次。
- 循环变量生命周期为局部变量，循环结束后不可见，不要在循环外使用。
- 步长为 0 会报错。

### 4. 泛型 for

配合迭代器使用，最典型的是 `ipairs` 和 `pairs`。

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

自定义迭代器示例：

```lua
local function range(n)
  local i = 0
  return function()
    i = i + 1
    if i <= n then return i end
  end
end

for i in range(3) do
  print(i)   -- 1 2 3
end
```

---

## 三、break 与 goto

### break

用于**提前跳出当前循环**（只跳一层），不能用在循环外。

```lua
for i = 1, 10 do
  if i == 5 then break end
  print(i)      -- 1 2 3 4
end
```

注意：Lua 没有 `continue`。想跳过某次迭代，通常用 `if ... then ... end` 包住剩余逻辑，或用 `goto`。

### goto 与 label

Lua 5.2 起支持 `goto`，标签用 `::name::` 表示。常用来模拟 `continue`：

```lua
for i = 1, 5 do
  if i % 2 == 0 then
    goto continue   -- 偶数跳过
  end
  print(i)            -- 1 3 5
  ::continue::
end
```

限制：

- 不能跳进局部变量的作用域内。
- 不能跳出函数。
- 标签遵循可见性规则（同一个块内唯一）。

---

## 四、几个易错点

1. **`0` 和 `""` 为真**，写 `if x then` 时若 x 可能是 0，要显式写 `if x ~= 0 then`。
2. **`and` / `or` 做条件简写**：

```lua
local name = input or "默认名"       -- input 为 nil/false 时取默认值
local ok = a and b                    -- 相当于 a ? b : a
```

3. **没有 continue**，用 goto 或重构逻辑代替。
4. **`repeat` 的条件是“退出条件”**，逻辑和 `while` 相反，别搞混。
5. **`pairs` 顺序不确定**，需要顺序就用 `ipairs` 或先排序键。

---

| 上一篇                               | 下一篇                                   |
| ------------------------------------ | ---------------------------------------- |
| [Lua 运算符](./2_operator.md) | [Lua 控制语句](./4_control_statement.md) |
