表（table）是 Lua 中**唯一的数据结构**，它同时充当数组、字典、集合、对象等角色。理解表是掌握 Lua 的关键。

## 1. 表的本质

表是**关联数组**（associative array）：任何类型的值（除 `nil` 外）都可以作键，任何类型的值都可以作值。它内部用哈希表 + 数组实现，兼具字典和数组的性能。

```lua
local t = {}   -- 创建空表
```

表是**引用类型**，变量存储的是引用：

```lua
local a = {1, 2}
local b = a
b[1] = 99
print(a[1])   -- 99（a 和 b 指向同一个表）
```

---

## 2. 构造表

```lua
-- 数组式（列表式）
local arr = {10, 20, 30}          -- arr[1]=10, arr[2]=20, arr[3]=30

-- 记录式（字典式）
local person = {name = "Alice", age = 25}

-- 混合式
local mixed = {1, 2, x = "a", y = "b"}

-- 显式键
local t = {[1] = "a", ["key"] = "b", [true] = "c"}

-- 嵌套
local matrix = {
  {1, 2, 3},
  {4, 5, 6},
}
print(matrix[2][3])   -- 6
```

> ⚠️ 数组索引**从 1 开始**，这是 Lua 的传统（受 Sol 语言影响）。

---

## 3. 访问与赋值

```lua
local t = {name = "Lua", version = 5.4}

-- 点号访问（键是合法标识符时）
print(t.name)        -- Lua
t.version = 5.5

-- 方括号访问（键是任意值时）
print(t["name"])     -- Lua
t["version"] = 5.6

-- 数字键
local arr = {10, 20}
print(arr[1])        -- 10
arr[3] = 30

-- 嵌套访问
local cfg = {db = {host = "localhost"}}
print(cfg.db.host)          -- localhost
print(cfg["db"]["host"])    -- 等价
```

**赋 `nil` 即删除键**：

```lua
local t = {a = 1, b = 2}
t.a = nil
print(t.a)   -- nil（键 a 被删除）
```

---

## 4. 长度运算符 `#`

`#` 返回表的**数组长度**（连续整数键的个数）。

```lua
print(#{10, 20, 30})   -- 3
print(#{})             -- 0
print(#{1, 2, nil, 4}) -- 不确定！可能是 2 或 4
```

> ⚠️ **重要陷阱**：当数组中间有 `nil`（空洞）时，`#` 的行为未定义，结果可能是任意边界。**应避免在数组中插入 `nil`**，需要删除元素时用 `table.remove`。

```lua
local t = {1, 2, 3}
table.remove(t, 2)   -- 删除第2个，后面元素前移
print(#t)            -- 2，t = {1, 3}
```

---

## 5. 遍历

```lua
local t = {10, 20, 30, name = "Lua"}

-- ipairs：只遍历数组部分，从 1 开始，遇到 nil 停止
for i, v in ipairs(t) do
  print(i, v)     -- 1 10 / 2 20 / 3 30
end

-- pairs：遍历所有键值对，顺序不保证
for k, v in pairs(t) do
  print(k, v)     -- 1 10 / 2 20 / 3 30 / name Lua（顺序不定）
end
```

| 迭代器 | 遍历范围 | 顺序 | 遇到 nil |
|--------|----------|------|----------|
| `ipairs` | 仅数组部分（整数键 1,2,3...） | 保证从 1 递增 | 停止 |
| `pairs` | 所有键值对 | 不保证 | 跳过 |

> Lua 5.2 之前 `pairs` 用 `next`；5.2+ 支持 `__pairs` 元方法；5.3+ 还可用 `pairs` 配合自定义迭代。

---

## 6. table 标准库

```lua
local t = {3, 1, 2}

-- 插入
table.insert(t, 4)       -- 末尾插入 -> {3,1,2,4}
table.insert(t, 1, 0)    -- 位置1插入 -> {0,3,1,2,4}

-- 删除
table.remove(t)          -- 删除末尾
table.remove(t, 1)       -- 删除位置1

-- 排序（默认升序）
table.sort(t)
table.sort(t, function(a, b) return a > b end)   -- 降序

-- 拼接
table.concat({"a", "b", "c"}, "-")   -- "a-b-c"
table.concat({1, 2, 3})               -- "123"

-- 解包（5.2+ 在 table 库中）
table.unpack({1, 2, 3})   -- 返回 1, 2, 3

-- 打包（5.2+）
table.pack(1, 2, 3)       -- {1, 2, 3, n = 3}（含 n 字段）
```

**`table.unpack` 与可变参数配合**：

```lua
local function f(...)
  local args = {...}
  return table.unpack(args)
end
print(f(1, 2, 3))   -- 1  2  3
```

---

## 7. 表作为数组

```lua
local arr = {}

-- 添加元素
arr[#arr + 1] = "a"       -- 手动追加（等价 table.insert(arr, "a")）
table.insert(arr, "b")

-- 遍历
for i = 1, #arr do
  print(arr[i])
end

-- 二维数组（矩阵）
local matrix = {}
for i = 1, 3 do
  matrix[i] = {}
  for j = 1, 3 do
    matrix[i][j] = i * j
  end
end
print(matrix[2][3])   -- 6
```

---

## 8. 表作为字典/记录

```lua
local config = {
  host = "localhost",
  port = 3306,
  options = {ssl = true, timeout = 30},
}

-- 访问
print(config.port)             -- 3306
print(config.options.ssl)      -- true

-- 遍历键值
for k, v in pairs(config) do
  print(k, type(v))
end

-- 检查键是否存在
if config.host ~= nil then
  print("有 host 配置")
end

-- 删除键
config.port = nil
```

---

## 9. 表作为集合

```lua
-- 用表的键去重
local function unique(t)
  local seen = {}
  local result = {}
  for _, v in ipairs(t) do
    if not seen[v] then
      seen[v] = true
      result[#result + 1] = v
    end
  end
  return result
end

print(table.concat(unique({1, 2, 2, 3, 1}), ","))   -- 1,2,3
```

---

## 10. 表作为对象

配合元表（metatable）可实现面向对象。

```lua
local Account = {}
Account.__index = Account

function Account.new(balance)
  local obj = setmetatable({balance = balance}, Account)
  return obj
end

function Account:deposit(n)
  self.balance = self.balance + n
end

function Account:show()
  print("余额: " .. self.balance)
end

local acc = Account.new(100)
acc:deposit(50)
acc:show()   -- 余额: 150
```

---

## 11. 元表简介（预告）

元表能改变表的行为，比如支持 `+`、`==`、`tostring` 等。

```lua
local mt = {
  __add = function(a, b)
    return {x = a.x + b.x, y = a.y + b.y}
  end,
  __tostring = function(v)
    return "(" .. v.x .. ", " .. v.y .. ")"
  end,
}

local v1 = setmetatable({x = 1, y = 2}, mt)
local v2 = setmetatable({x = 3, y = 4}, mt)
local v3 = v1 + v2
print(v3)   -- (4, 6)
```

---

## 12. 常见陷阱

**1. `#` 遇 nil 未定义**
```lua
local t = {1, 2, nil, 4}
print(#t)   -- 可能 2 或 4，避免这种结构
```

**2. `pairs` 遍历时修改表**
```lua
-- 不要在 pairs 循环中新增键，行为不确定
-- 删除当前键是允许的
for k in pairs(t) do
  if k == "x" then t[k] = nil end   -- 安全
end
```

**3. 引用共享**
```lua
local a = {1, 2}
local b = a          -- b 是引用，不是拷贝
-- 浅拷贝
local copy = {}
for k, v in pairs(a) do copy[k] = v end
```

**4. 数字键与字符串键不等价**
```lua
local t = {}
t[1] = "a"
t["1"] = "b"
print(t[1], t["1"])   -- a   b（是两个不同的键）
```

---

## 13. 综合示例：词频统计

```lua
local text = "the quick brown fox jumps over the lazy dog the fox"

local freq = {}
for word in text:gmatch("%a+") do
  freq[word] = (freq[word] or 0) + 1
end

-- 转成数组后排序输出
local list = {}
for word, count in pairs(freq) do
  list[#list + 1] = {word = word, count = count}
end
table.sort(list, function(a, b) return a.count > b.count end)

for _, item in ipairs(list) do
  print(item.word, item.count)
end
```

---

## 关键要点

1. **表是 Lua 唯一的数据结构**，既是数组也是字典。
2. **数组索引从 1 开始**。
3. **表是引用类型**，赋值共享同一份数据。
4. **`#` 只对连续数组部分可靠**，有 `nil` 空洞时结果未定义。
5. **`ipairs` 遍历数组，`pairs` 遍历所有键**。
6. **赋 `nil` 即删除键**。
7. **`table.insert/remove/sort/concat` 是常用操作**。
8. **配合元表可实现运算符重载和面向对象**。

需要我继续讲 **元表与元方法**（table 最强大的进阶特性）吗？