# base 库

`base` 提供 Lua 的基础操作。

---

# 目录

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `ipairs` | 按连续整数索引遍历表 | [ipairs](#ipairs) |
| `pairs` | 遍历表中的键值对 | [pairs](#pairs) |
| `next` | 获取后续键值对 | [next](#next) |
| `select` | 选择变参或查询变参数量 | [select](#select) |
| `rawequal` | 不触发元方法地比较两个值 | [rawequal](#rawequal) |
| `rawget` | 不触发元方法地读取表 | [rawget](#rawget) |
| `rawset` | 不触发元方法地修改表 | [rawset](#rawset) |
| `rawlen` | 获取字符串字节数或表边界 | [rawlen](#rawlen) |
| `tonumber` | 将值转换为数字 | [tonumber](#tonumber) |
| `tostring` | 将值转换为字符串 | [tostring](#tostring) |
| `type` | 查询 Lua 值的类型 | [type](#type) |
| `setmetatable` | 设置或移除表的元表 | [setmetatable](#setmetatable) |
| `getmetatable` | 获取表的元表或保护值 | [getmetatable](#getmetatable) |

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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要遍历的表。 |

## 返回值

返回迭代函数、状态表和初始控制值 `0`。每次迭代返回索引和值。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `iterator` | function | 获取下一项的迭代函数。 |
| `state` | table | 被遍历的表。 |
| `control` | integer | 初始控制值 `0`。 |

### 示例

```lua
local values = {"a", "b", "c", [10] = "ignored"}
for index, value in ipairs(values) do
  debug.print(tostring(index) .. " " .. value)
end
```

**输出：**

```lua
```

---

## `pairs`

遍历表中的键值对。若表定义了 `__pairs`，使用该方法提供的迭代协议。

### 调用

```lua
pairs
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要遍历的表。 |

## 返回值

返回迭代函数、状态值和初始控制值；每次迭代返回键和值。`__pairs` 的返回结果按 Lua 5.4 的三返回值协议使用。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `iterator` | function | 获取下一项的迭代函数。 |
| `state` | any | 迭代状态。 |
| `control` | any | 初始控制值。 |

### 示例

```lua
local values = {"a", "b", name = "TUI GAME"}
for key, value in pairs(values) do
  debug.print(tostring(key) .. " " .. tostring(value))
end
```

**输出：**

```lua
```

---

## `next`

从指定键之后读取表中的下一个键值对。省略键或传入 nil 时从表的开头开始。

### 调用

```lua
next
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要遍历的表。 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `key` | any / nil | 当前键；默认 `nil`。 |

## 返回值

找到下一项时返回键和值；没有后续项时返回一个 nil。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `key` | any | 下一项的键。 |
| `value` | any | 下一项的值。 |

### 示例

```lua
local values = {"a", "b", name = "TUI GAME"}
local key, value = next(values)
while key ~= nil do
  debug.print(tostring(key) .. " " .. tostring(value))
  key, value = next(values, key)
end
```

**输出：**

```lua
```

### 额外说明

- 选填参数直接按位置传入，不使用末尾选项表。

---

## `select`

从变参中选择指定位置开始的所有值，或查询变参数量。

### 调用

```lua
select
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `index` | integer / `"#"` | 起始位置；`"#"` 用于查询变参数量。 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `...` | any... | 要选择的值。 |

## 返回值

`index` 为 `"#"` 时返回一个整数；否则返回从指定位置开始的原始多返回值。

### 示例

```lua
local count = select("#", "a", nil, "c")
local first, second = select(2, "a", "b", "c")
```

**输出：**

```lua
```

### 额外说明

- 选填参数直接按位置传入，不使用末尾选项表。

---

## `rawequal`

比较两个值是否相等，不调用 `__eq` 元方法。

### 调用

```lua
rawequal
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `left` | any | 左侧值。 |
| `right` | any | 右侧值。 |

## 返回值

返回一个布尔值。

| 类型 | 说明 |
| --- | --- |
| boolean | 两个值是否相等。 |

### 示例

```lua
debug.print(tostring(rawequal(1, 1.0)))
```

**输出：**

```lua
```

---

## `rawget`

读取表中的原始键值，不调用 `__index` 元方法。

### 调用

```lua
rawget
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要读取的表。 |
| `key` | any | 要查询的键。 |

## 返回值

返回键对应的值；键不存在时返回 nil。

| 类型 | 说明 |
| --- | --- |
| any / nil | 表中保存的原始值。 |

### 示例

```lua
local value = rawget({answer = 42}, "answer")
debug.print(tostring(value))
```

**输出：**

```lua
```

---

## `rawset`

直接设置表中的键值，不调用 `__newindex` 元方法。

### 调用

```lua
rawset
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要修改的表。 |
| `key` | any | 要设置的键。 |
| `value` | any / nil | 要写入的值；nil 会移除该键。 |

## 返回值

返回被修改的原表。

| 类型 | 说明 |
| --- | --- |
| table | 被修改的表。 |

### 示例

```lua
local values = {}
rawset(values, "answer", 42)
debug.print(tostring(values.answer))
```

**输出：**

```lua
```

---

## `rawlen`

获取字符串的 UTF-8 字节数或表的原始数组长度。

### 调用

```lua
rawlen
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | string / table | 要测量的字符串或表。 |

## 返回值

返回整数长度。

| 类型 | 说明 |
| --- | --- |
| integer | 字节数或表边界。 |

### 示例

```lua
debug.print(tostring(rawlen("你好")))
```

**输出：**

```lua
```

---

## `tonumber`

尝试将值转换为数字。无法转换时返回 nil。

### 调用

```lua
tonumber
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | string / number | 要转换的值。 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `base` | integer | 字符串数字的进制，范围为 2 到 36；默认 `10`。 |

## 返回值

返回转换后的数字；无法转换时返回 nil。

| 类型 | 说明 |
| --- | --- |
| number / nil | 转换结果。 |

### 示例

```lua
local decimal = tonumber("42")
local hexadecimal = tonumber("2A", 16)
```

**输出：**

```lua
```

### 额外说明

- 选填参数直接按位置传入，不使用末尾选项表。

---

## `tostring`

将值转换为字符串。表可通过 `__tostring` 提供自己的转换结果。

### 调用

```lua
tostring
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要转换的值。 |

## 返回值

返回字符串。

| 类型 | 说明 |
| --- | --- |
| string | 转换结果。 |

### 示例

```lua
debug.print(tostring({name = "TUI GAME"}))
```

**输出：**

```lua
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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要查询的值。 |

## 返回值

返回类型名称字符串。

| 类型 | 说明 |
| --- | --- |
| string | 类型名称。 |

### 示例

```lua
debug.print(type(42))
```

**输出：**

```lua
```

---

## `setmetatable`

为表设置元表，或传入 nil 移除元表。受保护的元表不能被替换。

### 调用

```lua
setmetatable
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `table` | table | 要设置元表的表。 |
| `metatable` | table / nil | 新元表；nil 会移除元表。 |

## 返回值

返回原表。

| 类型 | 说明 |
| --- | --- |
| table | 设置元表后的原表。 |

### 示例

```lua
local values = setmetatable({}, {__index = {answer = 42}})
debug.print(tostring(values.answer))
```

**输出：**

```lua
```

---

## `getmetatable`

获取表的元表，或获取它设置的保护值。

### 调用

```lua
getmetatable
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `value` | any | 要查询的值。 |

## 返回值

返回元表、保护值或 nil。

| 类型 | 说明 |
| --- | --- |
| table / any / nil | 元表、保护值或不存在元表时的 nil。 |

### 示例

```lua
local meta = getmetatable(setmetatable({}, {}))
debug.print(type(meta))
```

**输出：**

```lua
```

### 额外说明

- 参数不是表（例如字符串）时，返回 `nil`。

- `ipairs`、`pairs`、`next`、`select`、`rawequal`、`rawget`、`rawset`、`rawlen`、`tonumber`、`tostring`、`type`、`setmetatable` 和 `getmetatable` 同时作为 Lua 全局函数提供。
- `rawset` 只用于修改脚本自己的表。
