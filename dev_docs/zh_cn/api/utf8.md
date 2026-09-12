# utf8 库

## 基本库说明

`utf8` 提供 UTF-8 字符串处理。

## 目录

### 方法

| 方法名              | 说明                                                                 | 索引                                    |
| ------------------- | -------------------------------------------------------------------- | --------------------------------------- |
| `len`               | 返回字符串包含的 Unicode 标量数量                                    | [len](#len)                             |
| `byte_len`          | 返回字符串经过 UTF-8 编码后的字节数                                  | [byte_len](#byte_len)                   |
| `is_ascii`          | 判断字符串是否全部为 ASCII 字符                                      | [is_ascii](#is_ascii)                   |
| `codepoint_to_char` | 将一组 Unicode 码点转换为字符串                                      | [codepoint_to_char](#codepoint_to_char) |
| `ascii_to_char`     | 将一组 ASCII 码转换为字符串                                          | [ascii_to_char](#ascii_to_char)         |
| `char_to_codepoint` | 将字符串指定区间的 Unicode 标量转换为 Unicode 码点                   | [char_to_codepoint](#char_to_codepoint) |
| `char_to_ascii`     | 将字符串指定区间的 Unicode 标量转换为 ASCII 码                       | [char_to_ascii](#char_to_ascii)         |
| `char_position`     | 定位文本中指定 Unicode 标量的首个 UTF-8 字节位置                     | [char_position](#char_position)         |
| `codepoints`        | 返回遍历字符串全部 Unicode 标量的迭代函数                            | [codepoints](#codepoints)               |
| `next`              | 获取指定 UTF-8 字节位置之后，下一个 Unicode 标量的起始字节位置和码点 | [next](#next)                           |

---

## 方法

## `len`

返回字符串包含的 Unicode 标量数量。

### 调用

```lua
-- 单参数
utf8.len()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | Unicode 标量数量 |

### 示例

```lua
local s1 = "Hello"
debug.print { message = utf8.len(s1) }

local s2 = "你好世界"
debug.print { message = utf8.len(s2) }

local s3 = "😊👍"
debug.print { message = utf8.len(s3) }

local s4 = "A😊B中"
debug.print { message = utf8.len(s4) }
```

输出：

```text
5
4
2
4
```

---

## `byte_len`

返回字符串经过 UTF-8 编码后的字节数。

### 调用

```lua
-- 单参数
utf8.byte_len()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | UTF-8 字节数 |

### 示例

```lua
local s1 = "Hello"
debug.print { message = utf8.byte_len(s1) }

local s2 = "你好"
debug.print { message = utf8.byte_len(s2) }

local s3 = "😊👍"
debug.print { message = utf8.byte_len(s3) }

local s4 = "A😊B中"
debug.print { message = utf8.byte_len(s4) }

```

输出：

```text
5
6
8
9
```

### 额外补充

- 字节数为 UTF-8 字节数。

---

## `is_ascii`

判断字符串是否全部为 ASCII 字符。

### 调用

```lua
-- 单参数
utf8.is_ascii()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型    | 说明           |
| ------- | -------------- |
| boolean | 是否全为 ASCII |

### 示例

```lua
local s1 = "Hello"
debug.print { message = utf8.is_ascii(s1) }

local s2 = "😊"
debug.print { message = utf8.is_ascii(s2) }
```

输出：

```text
true
false
```

---

## `codepoint_to_char`

将一组 Unicode 码点转换为字符串。

### 调用

```lua
-- 单参数
utf8.codepoint_to_char()
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明       |
| -------- | ----- | ---- | ------ | ---------- |
| `values` | table | 是   | -      | 码点数组表 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 拼接后的字符串 |

### 示例

```lua
local t1 = { 72, 101, 108, 108, 111 }
debug.print { message = utf8.codepoint_to_char(t1) }

local t2 = { 20320, 22909, 19990, 30028 }
debug.print { message = utf8.codepoint_to_char(t2) }
```

输出：

```text
Hello
你好世界
```

---

## `ascii_to_char`

将一组 ASCII 码转换为字符串。

### 调用

```lua
-- 单参数
utf8.ascii_to_char()
```

### 参数

| 参数名   | 类型  | 必填 | 默认值 | 说明         |
| -------- | ----- | ---- | ------ | ------------ |
| `values` | table | 是   | -      | ASCII 码数组 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 拼接后的字符串 |

### 示例

```lua
local t1 = { 65, 66, 67 }
debug.print { message = utf8.ascii_to_char(t1) }

local t2 = { 72, 105, 10, 84, 104, 101, 114, 101 }
debug.print { message = utf8.ascii_to_char(t2) }
```

输出：

```text
ABC
Hi
There
```

### 额外补充

- ASCII 码范围为 $[0..127]$。

---

## `char_to_codepoint`

将字符串指定区间的 Unicode 标量转换为 Unicode 码点。

### 调用

```lua
-- 表参数
utf8.char_to_codepoint{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值           | 说明                  |
| -------- | ------- | ---- | ---------------- | --------------------- |
| `text`   | string  | 是   | -                | 目标字符串            |
| `start`  | integer | 否   | `1`              | 起始 Unicode 标量位置 |
| `finish` | integer | 否   | Unicode 标量数量 | 结束 Unicode 标量位置 |

### 返回

返回一个数组表。

| 类型  | 说明       |
| ----- | ---------- |
| table | 码点数组表 |

### 示例

```lua
local s1 = "Hello"
local r1 = utf8.char_to_codepoint { text = s1 }
debug.print { message = table.pretty(r1) }

local s2 = "你好世界"
local r2 = utf8.char_to_codepoint { text = s2, start = 2, finish = 3 }
debug.print { message = table.pretty(r2) }

```

输出：

```lua
{ [1] = 72, [2] = 101, [3] = 108, [4] = 108, [5] = 111, n = 5 }

{ [1] = 22909, [2] = 19990, n = 2 }
```

### 额外补充

- 返回值表结构如下：

```lua
{
  [1] = ...,
  [2] = ...,
  ...
  [x] = ...,
  n = x
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

---

## `char_to_ascii`

将字符串指定区间的 Unicode 标量转换为 ASCII 码。

### 调用

```lua
-- 表参数
utf8.char_to_ascii{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值           | 说明                  |
| -------- | ------- | ---- | ---------------- | --------------------- |
| `text`   | string  | 是   | -                | 目标字符串            |
| `start`  | integer | 否   | `1`              | 起始 Unicode 标量位置 |
| `finish` | integer | 否   | Unicode 标量数量 | 结束 Unicode 标量位置 |

### 返回

返回一个数组表。

| 类型  | 说明           |
| ----- | -------------- |
| table | ASCII 码数组表 |

### 示例

```lua
local s1 = "ABC"
local r1 = utf8.char_to_ascii { text = s1 }
debug.print { message = table.pretty(r1) }

local s2 = "A中B"
local r2 = utf8.char_to_ascii { text = s2, start = 1, finish = 2 }
debug.print { message = table.pretty(r2) }
```

输出：

```lua
{ [1] = 65, [2] = 66, [3] = 67, n = 3 }
{ [1] = 65, n = 2 }
```

### 额外补充

- 若 Unicode 标量不属于 ASCII 范围，对应结果位置标记为 `nil`。

- 返回值表结构如下：

```lua
{
  [1] = ...,
  [2] = ...,
  ...
  [x] = ...,
  n = x
} -- 共有 x+1 个元素，所有返回值连续排序，最后 n 为返回值个数
```

---

## `char_position`

定位文本中指定 Unicode 标量的首个 UTF-8 字节位置。

### 调用

```lua
-- 表参数
utf8.char_position{}
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明                                   |
| ------- | ------- | ---- | ------ | -------------------------------------- |
| `text`  | string  | 是   | -      | 目标字符串                             |
| `index` | integer | 是   | -      | 从 `start` 开始计算的 Unicode 标量序号 |
| `start` | integer | 否   | `1`    | 起始 Unicode 标量位置                  |

### 返回

直接返回一个值。

| 类型    | 说明                                   |
| ------- | -------------------------------------- |
| integer | 目标 Unicode 标量的一基 UTF-8 字节位置 |

### 示例

```lua
local s1 = "Hello"
debug.print { message = utf8.char_position { text = s1, index = 1 } }

local s2 = "你好世界"
debug.print { message = utf8.char_position { text = s2, index = 3 } }
```

输出：

```text
1
7
```

---

## `codepoints`

返回遍历字符串全部 Unicode 标量的迭代函数。

### 调用

```lua
-- 单参数
utf8.codepoints()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型     | 说明     |
| -------- | -------- |
| function | 迭代函数 |

**迭代器函数**，返回一个对象表。

| 字段          | 类型    | 说明                                   |
| ------------- | ------- | -------------------------------------- |
| `byte_position` | integer | 当前 Unicode 标量的一基 UTF-8 字节位置 |
| `codepoint`     | integer | 当前 Unicode 标量对应的码点            |

### 示例

```lua
local s1 = "ABC"
for item in utf8.codepoints(s1) do
  debug.print { message = item.byte_position .. " " .. item.codepoint }
end

debug.print { message = "" }

local s2 = "你好"
for item in utf8.codepoints(s2) do
  debug.print { message = item.byte_position .. " " .. item.codepoint }
end
```

输出：

```text
1 65
2 66
3 67

1 20320
4 22909
```

---

## `next`

获取指定 UTF-8 字节位置之后，下一个 Unicode 标量的起始字节位置和码点。

### 调用

```lua
-- 表参数
utf8.next{}
```

### 参数

| 参数名 | 类型          | 必填 | 默认值 | 说明                                                                  |
| ------ | ------------- | ---- | ------ | --------------------------------------------------------------------- |
| `text` | string        | 是   | -      | 目标字符串                                                            |
| `pos`  | integer / nil | 否   | `nil`  | 当前一基 UTF-8 字节位置；省略或传入 `nil` 时从第一个 Unicode 标量开始 |

### 返回

**找到下一个元素时**，返回一个对象表。

| 返回值名    | 类型    | 说明                                     |
| ----------- | ------- | ---------------------------------------- |
| `codepoint` | integer | 下一个 Unicode 标量对应的码点            |
| `position`  | integer | 下一个 Unicode 标量的一基 UTF-8 字节位置 |

**没有后续元素时**，直接返回一个值。

| 类型 | 说明       |
| ---- | ---------- |
| nil  | 无后续元素 |

### 示例

```lua
local s = "A😊B中"
local pos = nil

while true do
  local item = utf8.next { text = s, pos = pos }
  
  if item == nil then
    break
  end
  
  debug.print { message = item.position .. " " .. item.codepoint }
  pos = item.position
end
```

输出：

```text
1 65
2 128522
6 66
7 20013
```