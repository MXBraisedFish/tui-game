# base 库

`base` 提供 Lua 的基础操作。

---

# 目录

## 方法

| 方法           | 说明                                                     | 定位                          |
| -------------- | -------------------------------------------------------- | ----------------------------- |
| `ipairs`       | 按从 1 开始的连续整数索引遍历表，遇到第一个 nil 值时结束 | [ipairs](#ipairs)             |
| `pairs`        | 遍历表中的键值对                                         | [pairs](#pairs)               |
| `next`         | 从指定键之后读取表中的下一个键值对                       | [next](#next)                 |
| `select`       | 从变参中选择指定位置开始的所有值，或查询变参数量         | [select](#select)             |
| `rawequal`     | 比较两个值是否相等                                       | [rawequal](#rawequal)         |
| `rawget`       | 读取表中的原始键值                                       | [rawget](#rawget)             |
| `rawset`       | 直接设置表中的键值                                       | [rawset](#rawset)             |
| `rawlen`       | 获取字符串的 UTF-8 字节数或表的原始数组长度              | [rawlen](#rawlen)             |
| `tonumber`     | 尝试将值转换为数字                                       | [tonumber](#tonumber)         |
| `tostring`     | 将值转换为字符串                                         | [tostring](#tostring)         |
| `type`         | 返回值的 Lua 类型名                                      | [type](#type)                 |
| `setmetatable` | 为表设置或移除元表，或传入 nil 移除元表                  | [setmetatable](#setmetatable) |
| `getmetatable` | 获取表的元表，或获取它设置的保护值                       | [getmetatable](#getmetatable) |

---

# 方法

## `ipairs`

按从 1 开始的连续整数索引遍历表，遇到第一个 nil 值时结束。

### 调用

```lua
ipairs
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明       |
| ------- | ----- | ---------- |
| `table` | table | 要遍历的表 |

## 返回值

返回三个值。

| 值名       | 类型     | 说明                 |
| ---------- | -------- | -------------------- |
| `iterator` | function | 获取下一项的迭代函数 |
| `state`    | table    | 被遍历的表           |
| `control`  | integer  | 初始控制值 `0`       |

**迭代函数**每次返回两个值。

| 值名    | 类型    | 说明         |
| ------- | ------- | ------------ |
| `index` | integer | 当前索引     |
| `value` | any     | 当前索引的值 |

### 示例

```lua
local values = {"a", "b", "c", [10] = "ignored"}
for index, value in ipairs(values) do
  debug.print(tostring(index) .. " " .. value)
end
```

**输出：**

```lua
1 a
2 b
3 c
```

---

## `pairs`

遍历表中的键值对。

### 调用

```lua
pairs
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明       |
| ------- | ----- | ---------- |
| `table` | table | 要遍历的表 |

## 返回值

返回三个值。

| 值名       | 类型     | 说明                 |
| ---------- | -------- | -------------------- |
| `iterator` | function | 获取下一项的迭代函数 |
| `state`    | any      | 迭代状态             |
| `control`  | any      | 初始控制值           |

**迭代函数**每次返回两个值。

| 值名    | 类型 | 说明         |
| ------- | ---- | ------------ |
| `key`   | any  | 当前键       |
| `value` | any  | 当前索引的值 |

**若定义`__pairs`元方法**，返回数个自定值。

| 类型   | 说明     |
| ------ | -------- |
| any... | 自定义值 |

### 示例

```lua
local values = {"a", "b", name = "TUI GAME"}
for key, value in pairs(values) do
  debug.print(tostring(key) .. " " .. tostring(value))
end
```

**输出：**

```lua
1 a
2 b
name TUI GAME
```

---

## `next`

从指定键之后读取表中的下一个键值对。

### 调用

```lua
next
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明       |
| ------- | ----- | ---------- |
| `table` | table | 要遍历的表 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明   |
| ------ | ---- | ------ | ------ |
| `key`  | any  | `nil`  | 当前键 |

## 返回值

**找到下一项时**，返回两个值。

| 值名    | 类型 | 说明       |
| ------- | ---- | ---------- |
| `key`   | any  | 下一项的键 |
| `value` | any  | 下一项的值 |

**未找到下一项时**，返回一个值。

| 类型 | 说明       |
| ---- | ---------- |
| nil  | 没有下一项 |

### 示例

```lua
local values = {"a", "b", name = "TUI GAME"}
local key, value = next(values)
while key ~= nil do
  debug.print(tostring(key) .. " " .. tostring(value))
  key, value = next(values, { key = key })
end
```

**输出：**

```lua
1 a
2 b
name TUI GAME
```

## 额外说明

- 选填参数 `key` 为 `nil` 时表示从头开始。

---

## `select`

从变参中选择指定位置开始的所有值，或查询变参数量。

### 调用

```lua
select
```

## 参数

### 必填参数

| 参数名  | 类型            | 说明                         |
| ------- | --------------- | ---------------------------- |
| `index` | integer / `"#"` | 起始位置；`"#"` 查询变参数量 |
| `...`   | any...          | 变参                         |

## 返回值

**必填参数 `index` 为 `integer` 时**，返回任意数量值。

| 类型   | 说明                   |
| ------ | ---------------------- |
| any... | 从指定为开始的数个变参 |

**必填参数 `index` 为 `"#"` 时**，返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| integer | 变参数量 |

### 示例

```lua
local count = select("#", "a", nil, "c")
debug.print(count)
local first, second = select(2, "a", "b", "c")
debug.print(first)
debug.print(second)
```

**输出：**

```lua
3
b
c
```

## 额外说明

- `nil` 也占参数位置。

---

## `rawequal`

比较两个值是否相等。

### 调用

```lua
rawequal
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明   |
| ------- | ---- | ------ |
| `left`  | any  | 左侧值 |
| `right` | any  | 右侧值 |

## 返回值

返回一个值。

| 类型    | 说明           |
| ------- | -------------- |
| boolean | 两个值是否相等 |

### 示例

```lua
debug.print(tostring(rawequal(1, 1.0)))
```

**输出：**

```lua
true
```

## 额外补充

- 不触发 `__eq` 元方法。

---

## `rawget`

读取表中的原始键值。

### 调用

```lua
rawget
```

## 参数

### 必填参数

| 参数名  | 类型  | 说明       |
| ------- | ----- | ---------- |
| `table` | table | 要读取的表 |
| `key`   | any   | 要查询的键 |

## 返回值

**若键存在**，返回一个值。

| 类型 | 说明   |
| ---- | ------ |
| any  | 原始值 |

**若键不存在**，返回一个值。

| 类型 | 说明   |
| ---- | ------ |
| any  | 原始值 |

### 示例

```lua
local value = rawget({answer = 42}, "answer")
debug.print(tostring(value))
```

**输出：**

```lua
42
```

## 额外补充

- 不触发 `__index` 元方法。

---

## `rawset`

直接设置表中的键值。

### 调用

```lua
rawset
```

## 参数

### 必填参数

| 参数名  | 类型      | 说明                       |
| ------- | --------- | -------------------------- |
| `table` | table     | 要修改的表                 |
| `key`   | any       | 要设置的键                 |
| `value` | any / nil | 要写入的值；nil 会移除该键 |

## 返回值

返回一个值。

| 类型  | 说明         |
| ----- | ------------ |
| table | 修改后的原表 |

### 示例

```lua
local values = {}
rawset(values, "answer", 42)
debug.print(tostring(values.answer))
```

**输出：**

```lua
42
```

## 额外补充

- 不触发 `__newindex` 元方法。

---

## `rawlen`

获取字符串的 UTF-8 字节数或表的原始数组长度。

### 调用

```lua
rawlen
```

## 参数

### 必填参数

| 参数名  | 类型           | 说明               |
| ------- | -------------- | ------------------ |
| `value` | string / table | 要测量的字符串或表 |

## 返回值

返回一个值。

| 类型    | 说明           |
| ------- | -------------- |
| integer | 字节数或表边界 |

### 示例

```lua
debug.print(tostring(rawlen("你好")))
```

**输出：**

```lua
6
```

---

## `tonumber`

尝试将值转换为数字。

### 调用

```lua
tonumber
```

## 参数

### 必填参数

| 参数名  | 类型                     | 说明       |
| ------- | ------------------------ | ---------- |
| `value` | string / integer / float | 要转换的值 |

### 选填参数

| 参数名 | 类型    | 默认值   | 说明             |
| ------ | ------- | -------- | ---------------- |
| `base` | integer | 自动识别 | 字符串数字的进制 |

## 返回值

**转换成功**，返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| number | 转换结果 |

**转换失败**，返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 转换失败 |

### 示例

```lua
local decimal = tonumber("42")
debug.print(type(decimal))
local hexadecimal = tonumber("2A", { base = 16 })
debug.print(type(hexadecimal))
```

**输出：**

```lua
number
number
```

## 额外说明

- 选填参数 `base` 取值范围为 $[2, 36]$。

---

## `tostring`

将值转换为字符串。

### 调用

```lua
tostring
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明       |
| ------- | ---- | ---------- |
| `value` | any  | 要转换的值 |

## 返回值

返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 转换结果 |

### 示例

```lua
debug.print(tostring({name = "TUI GAME"}))
debug.print(type(tostring(20)))
```

**输出：**

```lua
table: 0x1eb9cd3c2c0
string
```

---

## `type`

返回值的 Lua 类型名。

### 调用

```lua
type
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明       |
| ------- | ---- | ---------- |
| `value` | any  | 要查询的值 |

## 返回值

返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 类型名称 |

### 示例

```lua
debug.print(type(42))
debug.print(type(3.14))
debug.print(type({}))
debug.print(type("Hi"))
debug.print(type(nil))
```

**输出：**

```lua
number
number
table
string
nil
```

---

## `setmetatable`

为表设置或移除元表，或传入 nil 移除元表。

### 调用

```lua
setmetatable
```

## 参数

### 必填参数

| 参数名      | 类型        | 说明                   |
| ----------- | ----------- | ---------------------- |
| `table`     | table       | 要设置元表的表         |
| `metatable` | table / nil | 新元表；nil 会移除元表 |

## 返回值

返回一个值。

| 类型  | 说明             |
| ----- | ---------------- |
| table | 设置元表后的原表 |

### 示例

```lua
local values = setmetatable({}, {__index = {answer = 42}})
debug.print(tostring(values.answer))
```

**输出：**

```lua
42
```

## 额外补充

- 受保护的元表不能被设置或移除。

---

## `getmetatable`

获取表的元表，或获取它设置的保护值。

### 调用

```lua
getmetatable
```

## 参数

### 必填参数

| 参数名  | 类型 | 说明       |
| ------- | ---- | ---------- |
| `value` | any  | 要查询的值 |

## 返回值

**若元表存在**，返回一个值。

| 类型  | 说明 |
| ----- | ---- |
| table | 元表 |

**若元表受保护**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| any  | 元表的保护值 |

**若元表不存在**，返回一个值。

| 类型 | 说明       |
| ---- | ---------- |
| nil  | 元表不存在 |

### 示例

```lua
local meta = getmetatable(setmetatable({}, {}))
debug.print(type(meta))
```

**输出：**

```lua
table
```

## 额外说明

- 参数不是表时，返回 `nil`。
