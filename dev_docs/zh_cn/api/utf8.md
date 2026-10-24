# utf8 库

`utf8` 提供 UTF-8 字符串处理。

---

# 目录

## 方法

| 方法                | 说明                                                                 | 定位                                    |
| ------------------- | -------------------------------------------------------------------- | --------------------------------------- |
| `len`               | 返回字符串包含的 Unicode 标量数量                                    | [len](#len)                             |
| `byte_len`          | 返回字符串经过 UTF-8 编码后的字节数                                  | [byte_len](#byte_len)                   |
| `is_ascii`          | 判断字符串是否全部为 ASCII 字符                                      | [is_ascii](#is_ascii)                   |
| `codepoint_to_char` | 将一组 Unicode 码点转换为字符串                                      | [codepoint_to_char](#codepoint_to_char) |
| `ascii_to_char`     | 将一组 ASCII 码转换为字符串                                          | [ascii_to_char](#ascii_to_char)         |
| `char_to_codepoint` | 将字符串指定区间的 Unicode 标量转换为 Unicode 码点                   | [char_to_codepoint](#char_to_codepoint) |
| `char_to_ascii`     | 将字符串指定区间的 Unicode 标量转换为 ASCII 码                       | [char_to_ascii](#char_to_ascii)         |
| `char_position`     | 定位文本中指定序号的 Unicode 标量的首个 UTF-8 字节位置               | [char_position](#char_position)         |
| `codepoints`        | 返回遍历字符串全部 Unicode 标量的迭代函数                            | [codepoints](#codepoints)               |
| `next`              | 获取指定 UTF-8 字节位置之后，下一个 Unicode 标量的起始字节位置和码点 | [next](#next)                           |

---

# 方法

## `len`

返回字符串包含的 Unicode 标量数量。

### 调用

```lua
utf8.len
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | Unicode 标量数量 |

### 示例

```lua
local s1 = "Hello"
debug.print(utf8.len(s1))

local s2 = "你好世界"
debug.print(utf8.len(s2))

local s3 = "😊👍"
debug.print(utf8.len(s3))

local s4 = "A😊B中"
debug.print(utf8.len(s4))
```

**输出：**

```lua
5
4
2
4
```

### 重定向

```lua
utf8.len
```

---

## `byte_len`

返回字符串经过 UTF-8 编码后的字节数。

### 调用

```lua
utf8.byte_len
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | UTF-8 字节数 |

### 示例

```lua
local s1 = "Hello"
debug.print(utf8.byte_len(s1))

local s2 = "你好"
debug.print(utf8.byte_len(s2))

local s3 = "😊👍"
debug.print(utf8.byte_len(s3))

local s4 = "A😊B中"
debug.print(utf8.byte_len(s4))
```

**输出：**

```lua
5
6
8
9
```

---

## `is_ascii`

判断字符串是否全部为 ASCII 字符。

### 调用

```lua
utf8.is_ascii
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

### 返回值

返回一个值。

| 类型    | 说明           |
| ------- | -------------- |
| boolean | 是否全为 ASCII |

### 示例

```lua
local s1 = "Hello"
debug.print(utf8.is_ascii(s1))

local s2 = "😊"
debug.print(utf8.is_ascii(s2))
```

**输出：**

```lua
true
false
```

---

## `codepoint_to_char`

将一组 Unicode 码点转换为字符串。

### 调用

```lua
utf8.codepoint_to_char
```

### 参数

#### 必填参数

| 参数名   | 类型  | 说明       |
| -------- | ----- | ---------- |
| `values` | table | 码点数组表 |

### 返回值

返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 拼接后的字符串 |

### 示例

```lua
local t1 = { 72, 101, 108, 108, 111 }
debug.print(utf8.codepoint_to_char(t1))

local t2 = { 20320, 22909, 19990, 30028 }
debug.print(utf8.codepoint_to_char(t2))
```

**输出：**

```lua
Hello
你好世界
```

### 重定向

```lua
utf8.char
```

### 额外说明

- 必填参数 `values` 取值范围为 $[0, 1114111]$；即 `U+0000` 至 `U+10FFFF`，不含代理区 $[55296, 57343]$。

---

## `ascii_to_char`

将一组 ASCII 码转换为字符串。

### 调用

```lua
utf8.ascii_to_char
```

### 参数

#### 必填参数

| 参数名   | 类型  | 说明         |
| -------- | ----- | ------------ |
| `values` | table | ASCII 码数组 |

### 返回值

返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 拼接后的字符串 |

### 示例

```lua
local t1 = { 65, 66, 67 }
debug.print(utf8.ascii_to_char(t1))

local t2 = { 72, 105, 10, 84, 104, 101, 114, 101 }
debug.print(utf8.ascii_to_char(t2))
```

**输出：**

```lua
ABC
Hi
There
```

### 额外说明

- 必填参数 `values` 取值范围为 $[0, 127]$。

---

## `char_to_codepoint`

将字符串指定区间的 Unicode 标量转换为 Unicode 码点。

### 调用

```lua
utf8.char_to_codepoint
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

#### 选填参数

| 参数名   | 类型    | 默认值           | 说明                  |
| -------- | ------- | ---------------- | --------------------- |
| `start`  | integer | `1`              | 起始 Unicode 标量位置 |
| `finish` | integer | Unicode 标量数量 | 结束 Unicode 标量位置 |

### 返回值

返回一个值。

| 类型  | 说明       |
| ----- | ---------- |
| table | 码点数组表 |

### 示例

```lua
local s1 = "Hello"
local r1 = utf8.char_to_codepoint(s1)
debug.print(table.pretty(r1))

local s2 = "你好世界"
local r2 = utf8.char_to_codepoint(s2, { start = 2, finish = 3 })
debug.print(table.pretty(r2))
```

**输出：**

```lua
{
  [1] = 72,
  [2] = 101,
  [3] = 108,
  [4] = 108,
  [5] = 111,
  n = 5
}
{
  [1] = 22909,
  [2] = 19990,
  n = 2
}
```

### 重定向

```lua
utf8.codepoint
```

### 额外说明

- 返回值表结构如下：

```lua
{
  [1] = 65,    -- integer（Unicode 码点）
  [2] = 20320, -- integer（Unicode 码点）
  n = 2,       -- integer（元素数量）
}
```

- 选填参数 `start` 取值范围为 $[1, n]$，也可取 $[-n, -1]$；$n$ 为文本的 Unicode 标量数量。
- 选填参数 `finish` 取值范围为 $[1, n]$，也可取 $[-n, -1]$；$n$ 为文本的 Unicode 标量数量，$finish$ 不得小于 $start$。

---

## `char_to_ascii`

将字符串指定区间的 Unicode 标量转换为 ASCII 码。

### 调用

```lua
utf8.char_to_ascii
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

#### 选填参数

| 参数名   | 类型    | 默认值           | 说明                  |
| -------- | ------- | ---------------- | --------------------- |
| `start`  | integer | `1`              | 起始 Unicode 标量位置 |
| `finish` | integer | Unicode 标量数量 | 结束 Unicode 标量位置 |

### 返回值

返回一个值。

| 类型  | 说明           |
| ----- | -------------- |
| table | ASCII 码数组表 |

### 示例

```lua
local s1 = "ABC"
local r1 = utf8.char_to_ascii(s1)
debug.print(table.pretty(r1))

local s2 = "A中B"
local r2 = utf8.char_to_ascii(s2, { start = 1, finish = 2 })
debug.print(table.pretty(r2))
```

**输出：**

```lua
{
  [1] = 65,
  [2] = 66,
  [3] = 67,
  n = 3
}
{
  [1] = 65,
  n = 2
}
```

### 额外说明

- 若 Unicode 标量不属于 ASCII 范围，对应结果位置标记为 `nil`。
- 返回值表结构如下：

```lua
{
  [1] = 65,  -- integer（ASCII 码）
  -- 非 ASCII 字符的位置为 nil
  n = 2,     -- integer（元素数量）
}
```

- 选填参数 `start` 取值范围为 $[1, n]$，也可取 $[-n, -1]$；$n$ 为文本的 Unicode 标量数量。
- 选填参数 `finish` 取值范围为 $[1, n]$，也可取 $[-n, -1]$；$n$ 为文本的 Unicode 标量数量，$finish$ 不得小于 $start$。

---

## `char_position`

定位文本中指定序号的 Unicode 标量的首个 UTF-8 字节位置。

### 调用

```lua
utf8.char_position
```

### 参数

#### 必填参数

| 参数名  | 类型    | 说明                                   |
| ------- | ------- | -------------------------------------- |
| `text`  | string  | 目标字符串                             |
| `index` | integer | 从 `start` 开始计算的 Unicode 标量序号 |

#### 选填参数

| 参数名  | 类型    | 默认值 | 说明                  |
| ------- | ------- | ------ | --------------------- |
| `start` | integer | `1`    | 起始 Unicode 标量位置 |

### 返回值

**找到字符时**，返回一个值。

| 类型    | 说明                                   |
| ------- | -------------------------------------- |
| integer | 目标 Unicode 标量的一基 UTF-8 字节位置 |

**超出范围时**，返回一个值。

| 类型 | 说明         |
| ---- | ------------ |
| nil  | 超出索引范围 |

### 示例

```lua
local s1 = "Hello"
debug.print(utf8.char_position(s1, 1))

local s2 = "你好世界"
debug.print(utf8.char_position(s2, 3))
```

**输出：**

```lua
1
7
```

### 额外说明

- 必填参数 `index` 取值范围为 $[1, +\infty)$；必须为有限数。
- 选填参数 `start` 取值范围为 $[1, n + 1]$；$n$ 为文本的 Unicode 标量数量。

---

## `codepoints`

返回遍历字符串全部 Unicode 标量的迭代函数。

### 调用

```lua
utf8.codepoints
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

### 返回值

返回一个值。

| 类型     | 说明     |
| -------- | -------- |
| function | 迭代函数 |

**每次调用迭代函数时**，返回两个值。

| 值名            | 类型    | 说明                                   |
| --------------- | ------- | -------------------------------------- |
| `byte_position` | integer | 当前 Unicode 标量的一基 UTF-8 字节位置 |
| `codepoint`     | integer | 当前 Unicode 标量对应的码点            |

### 示例

```lua
local s1 = "ABC"
for byte_position, codepoint in utf8.codepoints(s1) do
  debug.print(byte_position .. " " .. codepoint)
end

debug.print("")

local s2 = "你好"
for byte_position, codepoint in utf8.codepoints(s2) do
  debug.print(byte_position .. " " .. codepoint)
end
```

**输出：**

```lua
1 65
2 66
3 67

1 20320
4 22909
```

### 重定向

```lua
utf8.codes
```

---

## `next`

获取指定 UTF-8 字节位置之后，下一个 Unicode 标量的起始字节位置和码点。

### 调用

```lua
utf8.next
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明       |
| ------ | ------ | ---------- |
| `text` | string | 目标字符串 |

#### 选填参数

| 参数名 | 类型          | 默认值 | 说明                                                                  |
| ------ | ------------- | ------ | --------------------------------------------------------------------- |
| `pos`  | integer / nil | `nil`  | 当前一基 UTF-8 字节位置；省略或传入 `nil` 时从第一个 Unicode 标量开始 |

### 返回值

**找到下一个元素时**，返回两个值。

| 值名            | 类型    | 说明                                     |
| --------------- | ------- | ---------------------------------------- |
| `byte_position` | integer | 下一个 Unicode 标量的一基 UTF-8 字节位置 |
| `codepoint`     | integer | 下一个 Unicode 标量对应的码点            |

**没有后续元素时**，返回一个值。

| 类型 | 说明       |
| ---- | ---------- |
| nil  | 无后续元素 |

### 示例

```lua
local s = "A😊B中"
local byte_position, codepoint = utf8.next(s)
while byte_position ~= nil do
  debug.print(byte_position .. " " .. codepoint)
  byte_position, codepoint = utf8.next(s, { pos = byte_position })
end
```

**输出：**

```lua
1 65
2 128522
6 66
7 20013
```

### 额外说明

- 选填参数 `pos` 取值范围为 $[1, +\infty)$；必须为有限值。
