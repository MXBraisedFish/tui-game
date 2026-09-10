# Lua 基本数据类型

> 该教程文档使用的是原生 Lua 语言，语法本身完全合规，但 Tui Game 程序并非原生 Lua 运行器，因此无法直接运行该文档中的代码。

---

## 总览

Lua 中有**8 种**基本数据类型。

本教程只讲解最常用的 6 种。

| 类型       | 说明     | 示例                    |
| ---------- | -------- | ----------------------- |
| `nil`      | 空值     | `nil`                   |
| `boolean`  | 布尔     | `true`, `false`         |
| `number`   | 数字     | `3`, `3.14`, `0x1A`     |
| `string`   | 字符串   | `"hello"`               |
| `table`    | 表       | `{1, 2, 3}`             |
| `function` | 函数     | `function() end`        |

---

## nil

`nil` 是 Lua 中唯一的空类型，直接表示“没有值”。

也就是说如果某一个变量值为 `nil`，对于解释器就表示这个变量不存在。

### 示例

```lua
local x
print(x) -- nil
```

输出：

```text
nil
```

---

## boolean

布尔值只有 `true` 和 `false`，分别代表**真**和**假**。

```lua
local a = true
local b = false

print(a and b)   -- false
print(a or b)    -- true
print(not a)     -- false
```

---

## 3. number（数字）

Lua 5.3 起区分 **integer（整数）** 和 **float（浮点数）**，但类型名统一为 `number`。

```lua
local i = 10          -- 整数
local f = 3.14        -- 浮点数
local h = 0xFF        -- 十六进制 = 255
local e = 1e3         -- 科学计数法 = 1000.0

print(type(i))        -- number
print(3 / 2)          -- 1.5（除法总是浮点）
print(7 // 2)         -- 3  （整除，5.3+）
print(7 % 3)          -- 1
print(2 ^ 10)         -- 1024.0（幂运算返回浮点）
print(math.type(3))   -- integer
print(math.type(3.0)) -- float
```

**字符串与数字自动转换**：

```lua
print("10" + 5)       -- 15（字符串自动转数字）
print(10 .. 20)       -- "1020"（数字自动转字符串拼接）
```

**常用数学库**：

```lua
math.floor(3.7)   -- 3
math.ceil(3.2)    -- 4
math.abs(-5)      -- 5
math.max(1, 9, 3) -- 9
math.random(1, 6) -- 1~6 随机整数
math.huge         -- 无穷大
```

---

## 4. string（字符串）

字符串是不可变的字节序列，可以用单引号、双引号或长括号定义。

```lua
local s1 = 'hello'
local s2 = "world"
local s3 = [[多行
字符串]]
local s4 = [==[ 内含 ]] 的字符串 ]==]  -- 自定义分隔符
```

**转义字符**：`\n`（换行）、`\t`（制表符）、`\\`（反斜杠）、`\"`（引号）、`\ddd`（十进制 ASCII 码）。

**常用操作**：

```lua
local s = "Hello Lua"

print(#s)                 -- 9（长度）
print(s .. "!" )          -- Hello Lua!（拼接）
print(s:upper())          -- HELLO LUA
print(s:lower())          -- hello lua
print(s:sub(1, 5))        -- Hello（截取，索引从1开始）
print(s:find("Lua"))      -- 7  9（起始和结束位置）
print(s:rep(2))           -- Hello LuaHello Lua
print(s:gsub("Lua", "World"))  -- Hello World  替换次数
```

**字符串格式化**：

```lua
print(string.format("姓名: %s, 年龄: %d", "Alice", 25))
-- 姓名: Alice, 年龄: 25
-- %s 字符串, %d 整数, %f 浮点, %x 十六进制
print(string.format("%.2f", 3.14159))  -- 3.14
```

**与数字转换**：

```lua
print(tonumber("123"))     -- 123
print(tonumber("abc"))     -- nil
print(tostring(456))       -- "456"
print(tonumber("FF", 16))  -- 255（指定进制）
```

---

## 5. table（表）

表是 Lua **唯一**的数据结构，既能当数组，也能当字典/对象，功能极其强大。

```lua
-- 数组（索引从 1 开始！）
local arr = {10, 20, 30}
print(arr[1])       -- 10

-- 字典
local person = {name = "Alice", age = 25}
print(person.name)  -- Alice
print(person["age"])-- 25

-- 混合
local t = {1, 2, x = "a"}
```

**引用语义**：table 是引用类型，赋值只是复制引用。

```lua
local a = {1, 2}
local b = a
b[1] = 99
print(a[1])   -- 99（a 和 b 指向同一个表）
```

**遍历**：

```lua
local t = {10, 20, name = "Lua"}

for i, v in ipairs(t) do   -- 只遍历数组部分，遇到 nil 停止
  print(i, v)              -- 1 10 / 2 20
end

for k, v in pairs(t) do    -- 遍历所有键值对
  print(k, v)              -- 1 10 / 2 20 / name Lua
end
```

**常用 table 库**：

```lua
local t = {3, 1, 2}
table.insert(t, 4)     -- 末尾插入 -> {3,1,2,4}
table.insert(t, 1, 0)  -- 在位置1插入 -> {0,3,1,2,4}
table.remove(t, 1)     -- 删除位置1
table.sort(t)          -- 排序
print(table.concat({"a","b","c"}, "-"))  -- a-b-c
print(#t)              -- 长度
```

---

## 6. function（函数）

函数是第一类值，可以赋给变量、作为参数传递、作为返回值。

```lua
local function add(a, b)
  return a + b
end

local f = add          -- 赋给变量
print(f(1, 2))         -- 3

-- 作为参数
local function apply(fn, x, y)
  return fn(x, y)
end
print(apply(add, 3, 4))  -- 7

-- 匿名函数
local double = function(x) return x * 2 end
print(double(5))         -- 10
```

---

## 7. userdata（用户数据）

用于表示由 C 语言创建、存储在 Lua 中的任意数据，Lua 本身无法直接创建或操作（需通过 C API）。常见于文件句柄、GUI 对象等。

```lua
-- io.open 返回的就是 userdata（文件句柄）
local f = io.open("test.txt", "r")
print(type(f))   -- userdata
f:close()
```

---

## 8. thread（线程/协程）

表示独立的执行线程，用于实现协程（coroutine），是 Lua 的协作式多任务机制。

```lua
local co = coroutine.create(function(a, b)
  print("开始", a, b)
  local c = coroutine.yield(a + b)  -- 挂起并返回值
  print("恢复，收到", c)
  return "结束"
end)

print(coroutine.resume(co, 1, 2))   -- 开始 1 2 / true 3
print(coroutine.resume(co, 10))     -- 恢复，收到 10 / true 结束
print(coroutine.status(co))         -- dead
```

---

|上一篇|下一篇|
|---|---|
|[Lua 简介](./0_introduction.md)|[Lua 运算符](./2_operator.md)|
