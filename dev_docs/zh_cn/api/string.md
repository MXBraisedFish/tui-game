# string 库

`string` 提供字符串处理。Lua 字符串值没有额外的原生方法，请使用此库公开的 `string.*` 接口。

---

# 目录

## 常量

| 常量       | 说明                 | 定位                      |
| ------------ | -------------------- | ------------------------- |
| `AUTO`       | 自动检查文本类型     | [AUTO](#auto)             |
| `PLAIN_TEXT` | 强制按普通文本解析   | [PLAIN_TEXT](#plain_text) |
| `RICH_TEXT`  | 强制按富文本语法解析 | [RICH_TEXT](#rich_text)   |

## 方法

| 方法                    | 说明                                             | 定位                                                |
| ------------------------- | ------------------------------------------------ | --------------------------------------------------- |
| `lower`                   | 将字符串全部转为小写                             | [lower](#lower)                                     |
| `upper`                   | 将字符串全部转为大写                             | [upper](#upper)                                     |
| `reverse`                 | 按字符反转字符串                                 | [reverse](#reverse)                                 |
| `split`                   | 按指定分割字符分割目标字符串                     | [split](#split)                                     |
| `sub`                     | 按字符位置截取子串                               | [sub](#sub)                                         |
| `rep`                     | 将字符串重复数次并按照指定分隔符拼接             | [rep](#rep)                                         |
| `find`                    | 按照模式字符串查找首个满足要求的内容或捕获组     | [find](#find)                                       |
| `match`                   | 匹配目标字符串中首个满足要求的内容或捕获组       | [match](#match)                                     |
| `gmatch`                  | 遍历并匹配目标字符串中所有满足要求的内容或捕获组 | [gmatch](#gmatch)                                   |
| `gsub`                    | 全局替换匹配内容                                 | [gsub](#gsub)                                       |
| `regex_escape`            | 转义字符串中的正则特殊字符为普通文本             | [regex_escape](#regex_escape)                       |
| `regex_find`              | 按照正则表达式查找首个满足要求的内容或捕获组     | [regex_find](#regex_find)                           |
| `regex_match`             | 按照正则表达式匹配首个满足要求的内容或捕获组     | [regex_match](#regex_match)                         |
| `regex_gmatch`            | 用正则迭代全部匹配                               | [regex_gmatch](#regex_gmatch)                       |
| `regex_gsub`              | 全局替换匹配内容                                 | [regex_gsub](#regex_gsub)                           |
| `regex_test`              | 判断文本是否匹配给定的正则表达式                 | [regex_test](#regex_test)                           |
| `regex_split`             | 按正则表达式分割目标字符串                       | [regex_split](#regex_split)                         |
| `format`                  | 按格式串格式化值列表                             | [format](#format)                                   |
| `rich_text_to_plain_text` | 将富文本转换为普通文本                           | [rich_text_to_plain_text](#rich_text_to_plain_text) |

---

# 常量

## `AUTO`

自动检查文本类型。

### 调用

```lua
string.AUTO
```

### 可用于

- 参数 `text_mode`

### 示例

```lua
local p_str = "Hello Tui Game"
local r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

local len1 = measurement.get_text_width(p_str, {text_mode = string.AUTO})
local len2 = measurement.get_text_width(r_str, {text_mode = string.AUTO})

debug.print(tostring(len1))
debug.print(tostring(len2))
```

**输出：**

```lua
```

### 等值

```text
"auto"
```

---

## `PLAIN_TEXT`

强制按普通文本解析；头部声明 `f%`，富文本标签会被强制保留。

### 调用

```lua
string.PLAIN_TEXT
```

### 可用于

- 参数 `text_mode`

### 示例

```lua
local p_str = "Hello Tui Game"
local r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

local len1 = measurement.get_text_width(p_str, {text_mode = string.PLAIN_TEXT})
local len2 = measurement.get_text_width(r_str, {text_mode = string.PLAIN_TEXT})

debug.print(tostring(len1))
debug.print(tostring(len2))
```

**输出：**

```lua
```

### 等值

```text
"plain_text"
```

---

## `RICH_TEXT`

强制按富文本语法解析；头部声明 `f%` 会被强制保留。

### 调用

```lua
string.RICH_TEXT
```

### 可用于

- 文本参数 `text_mode`

### 示例

```lua
local nh_r_str = "<fg:red>Hello<fg:yellow> Tui Game</fg>"
local r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

local len1 = measurement.get_text_width(nh_r_str, {text_mode = string.RICH_TEXT})
local len2 = measurement.get_text_width(r_str, {text_mode = string.RICH_TEXT})

debug.print(tostring(len1))
debug.print(tostring(len2))
```

**输出：**

```lua
```

### 等值

```text
"rich_text"
```

---

# 方法

## `lower`

将字符串全部转为小写。

### 调用

```lua
string.lower
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |

## 返回值

| 类型   | 说明     |
| ------ | -------- |
| string | 小写结果 |

### 示例

```lua
local u_str = "HELLO TUI GAME"
local str = string.lower(u_str)

debug.print(str)
```

**输出：**

```lua
```

### 额外说明

本库的字符串输入和单次输出最多为 1 MiB；模式最多为 8 KiB，捕获组最多 32 个。Lua 模式最多执行 512 个操作和 1,000,000 次累计匹配步骤；正则表达式构建最多使用 1 MiB。适用时，遍历、替换和分割最多产生 10,000 项。一次性操作超限会返回带接口名称的错误；`gmatch` 类迭代器触限时返回错误，已经取出的结果不会撤回。字符位置、反转和大小写按 Unicode 字符处理；Lua 模式中的字母、大小写、单词和空白类别使用 Unicode 属性，数字、十六进制和标点类别仍按 ASCII 判断。

---

## `upper`

将字符串全部转为大写。

### 调用

```lua
string.upper
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |

## 返回值

返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 大写结果 |

### 示例

```lua
local l_str = "hello tui game"
local str = string.upper(l_str)

debug.print(str)
```

**输出：**

```lua
```

---

## `reverse`

按字符反转字符串。

### 调用

```lua
string.reverse
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |

## 返回值

| 类型   | 说明     |
| ------ | -------- |
| string | 反转结果 |

### 示例

```lua
local r_str = "emaG iuT olleH"
local str = string.reverse(r_str)

debug.print(str)
```

**输出：**

```lua
```

---

## `split`

按指定分割字符分割目标字符串。

### 调用

```lua
string.split
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `sep` | string | 分割字符 |

## 返回值

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
| table | 分割结果 |

### 示例

```lua
local parts = string.split("apple,banana,grape", ",")

for _, value in ipairs(parts) do
  debug.print(value)
end
```

**输出：**

```lua
```

---

## `sub`

按字符位置截取子串。

### 调用

```lua
string.sub
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `start` | integer | 起始字符位置 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `finish` | integer | 目标字符串长度 | 结束字符位置 |

## 返回值

返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 截取结果 |

### 示例

```lua
local sub_str = "Hello Tui Game"
local str = string.sub(sub_str, 1, {finish = 5})

debug.print(str)
```

**输出：**

```lua
```

---

## `rep`

将字符串重复数次并按照指定分隔符拼接。

### 调用

```lua
string.rep
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 要重复的字符串 |
| `times` | integer | 重复次数 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `sep` | string | `""` | 相邻副本间的分隔符 |

## 返回值

返回一个值。

| 类型   | 说明         |
| ------ | ------------ |
| string | 重复拼接结果 |

### 示例

```lua
local rep_str = "ABC"
local str = string.rep(rep_str, 3, {sep = " | "})

debug.print(str)
```

**输出：**

```lua
```

---

## `find`

按照模式字符串，从起始搜索位置开始，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
string.find
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 模式字符串 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `init` | integer | `1` | 起始搜索位置 |
| `plain` | boolean | `false` | 是否按普通文本查找 |

## 返回值

查找成功时返回三个值：匹配起点、匹配终点和捕获数组表。查找失败时返回一个 nil。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `start` | integer | 匹配起点 |
| `finish` | integer | 匹配终点 |
| `captures` | table | 匹配到的字符串或捕获组数组表 |

### 示例

```lua
local start1, finish1, captures1 = string.find("Hello Tui Game", "Tui")
debug.print(tostring(start1))
debug.print(tostring(finish1))
debug.print(captures1[1])
debug.print(tostring(captures1.n) .. "\n")

local start2, finish2, captures2 = string.find("Name: Alice, Age: 30", "Name: (%w+), Age: (%d+)")
debug.print(tostring(start2))
debug.print(tostring(finish2))
debug.print(captures2[1])
debug.print(tostring(captures2[2]))
debug.print(tostring(captures2.n))
```

**输出：**

```lua
```

### 额外说明

- 这是项目的字符串接口：`init`、`plain` 放在末尾选项表中；起止位置按从 1 开始的 Unicode 字符计数。第三个返回值始终是捕获表，不把捕获组展开为更多返回值。

- 返回值 `captures` 表结构如下：

```lua
local captures = {
  [1] = "Alice", -- string
  [2] = "30", -- string
  n = 2, -- integer; the number of captures
}
```

- 匹配结果为零长字符时，返回值中的 $finish = start - 1$

---

## `match`

按照模式字符串，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
string.match
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 模式字符串 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `init` | integer | `1` | 起始搜索位置 |

## 返回值

若**查找成功**，返回一个混合表。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 匹配到的字符串或捕获组 |

若**查找失败**，返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
local match1 = string.match("Hello 123", "%d+")
debug.print(match1[1])
debug.print(tostring(match1.n) .. "\n")

local match2 = string.match("Product: Apple, Price: 5.99", "Product: (%w+), Price: ([%d.]+)")
debug.print(match2[1])
debug.print(match2[2])
debug.print(tostring(match2.n))
```

**输出：**

```lua
```

### 额外说明

- 返回值混合表结构如下：

```lua
local captures = {
  [1] = "Alice",
  [2] = "30",
  n = 2,
}
```

---

## `gmatch`

按照模式字符串，遍历并匹配目标字符串中所有满足要求的内容或捕获组。

### 调用

```lua
string.gmatch
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 模式字符串 |

## 返回值

返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个混合表。

| 字段      | 类型      | 说明         |
| --------- | --------- | ------------ |
| [integer] | string / integer / nil | 捕获结果     |
| `n`       | integer   | 捕获结果数量 |

### 示例

```lua
local iter1 = string.gmatch("a1 b2 c3", "%w+")

for m in iter1 do
  debug.print(m[1] .. " " .. m.n)
end

debug.print("")

local iter2 = string.gmatch("A-1 B-2 C-3", "(%w+)-(%d+)")

for caps in iter2 do
  debug.print(caps[1] .. " " .. caps[2] .. " " .. caps.n)
end
```

**输出：**

```lua
```

### 额外说明

- 每次迭代只返回一个捕获表；使用 `for captures in string.gmatch(text, pattern) do ... end` 读取。没有捕获组时，表的第 1 项是完整匹配；位置捕获 `()` 的值为整数。

- 迭代器返回值元素混合表结构：

```lua
local captures = {
  [1] = "Alice",
  [2] = "30",
  n = 2,
}
```

---

## `gsub`

全局替换匹配内容。

### 调用

```lua
string.gsub
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 模式字符串 |
| `repl` | string / table / function | 替换内容 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `limit` | integer | `-1` | 最大替换次数 |

## 返回值

返回两个值，依次为替换后的字符串和实际替换次数。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 替换结果 |
| `count` | integer | 实际替换次数 |

### 示例

```lua
local result1, count1 = string.gsub("one two three", "%a+", "X")
debug.print(result1 .. " " .. count1 .. "\n")

local result2, count2 = string.gsub("2023-2024-2025", "(%d+)", "[$1]", {limit = 2})
debug.print(result2 .. " " .. count2 .. "\n")

local result3, count3 = string.gsub("apple banana apple", "(%w+)", { apple = "fruit", banana = "berry" })
debug.print(result3 .. " " .. count3 .. "\n")

local result4, count4 = string.gsub("a1 b2 c3", "(%w)(%d)", function(letter, num) return letter .. string.rep("x", tonumber(num)) end)
debug.print(result4 .. " " .. count4)
```

**输出：**

```lua
```

### 额外说明

- 参数 `limit` 为 -1 时代表不限次数。

---

## `regex_escape`

转义字符串中的正则特殊字符为普通文本。

### 调用

```lua
string.regex_escape
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |

## 返回值

返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 转义后的字符串 |

### 示例

```lua
local e1 = string.regex_escape("hello")
debug.print(e1)

local e2 = string.regex_escape("a+b*c?")
debug.print(e2)
```

**输出：**

```lua
```

---

## `regex_find`

按照正则表达式，从起始搜索位置开始，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
string.regex_find
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `init` | integer | `1` | 起始搜索位置 |

## 返回值

查找成功时返回三个值：匹配起点、匹配终点和捕获数组表。查找失败时返回一个 nil。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `start` | integer | 匹配起点 |
| `finish` | integer | 匹配终点 |
| `captures` | table | 匹配到的字符串或捕获组数组表 |

### 示例

```lua
local start1, finish1, captures1 = string.regex_find("Hello 123", [[\d+]])
debug.print(tostring(start1))
debug.print(tostring(finish1))
debug.print(captures1[1])
debug.print(tostring(captures1.n) .. "\n")

local start2, finish2, captures2 = string.regex_find("Name: Alice, Age: 30", [[Name: (\w+), Age: (\d+)]])
debug.print(tostring(start2))
debug.print(tostring(finish2))
debug.print(captures2[1])
debug.print(captures2[2])
debug.print(tostring(captures2.n))
```

**输出：**

```lua
```

### 额外说明

- 返回值 `captures` 表结构如下：

```lua
local captures = {
  [1] = "Alice",
  [2] = "30",
  n = 2,
}
```

- 匹配结果为零长字符时，返回值中的 $finish = start - 1$；`captures.n` 表示捕获数量，没有捕获组时为 1，保存完整匹配（可以是空字符串）。

---

## `regex_match`

按照正则表达式，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
string.regex_match
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `init` | integer | `1` | 起始搜索位置 |

## 返回值

若**查找成功**，返回一个混合表。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 匹配到的字符串或捕获组 |

若**查找失败**，返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
local m1 = string.regex_match("Hello 123", [[\d+]])
debug.print(m1[1])
debug.print(tostring(m1.n) .. "\n")

local m2 = string.regex_match("Name: Alice, Age: 30", [[Name: (\w+), Age: (\d+)]])
debug.print(m2[1])
debug.print(m2[2])
debug.print(tostring(m2.n))
```

**输出：**

```lua
```

### 额外说明

- 返回值混合表结构如下：

```lua
local captures = {
  [1] = "Alice",
  [2] = "30",
  n = 2,
}
```

---

## `regex_gmatch`

返回用正则迭代全部匹配的迭代函数。

### 调用

```lua
string.regex_gmatch
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |

## 返回值

返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个混合表。

| 字段      | 类型      | 说明         |
| --------- | --------- | ------------ |
| [integer] | string / nil | 捕获结果     |
| `n`       | integer   | 捕获结果数量 |

### 示例

```lua
local iter1 = string.regex_gmatch("a1 b2 c3", [[\w+]])

for m in iter1 do
  debug.print(m[1] .. " " .. m.n)
end

debug.print("")

local iter2 = string.regex_gmatch("A-1 B-2 C-3", [[(\w+)-(\d+)]])
for caps in iter2 do
  debug.print(caps[1] .. " " .. caps[2] .. " " .. caps.n)
end
```

**输出：**

```lua
```

### 额外说明

- 迭代器返回值元素混合表结构：

```lua
local captures = {
  [1] = "Alice",
  [2] = "30",
  n = 2,
}
```

---

## `regex_gsub`

全局替换匹配内容。

### 调用

```lua
string.regex_gsub
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |
| `repl` | string / table / function | 替换内容 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `limit` | integer | `-1` | 最大替换次数 |

## 返回值

返回两个值，依次为替换后的字符串和实际替换次数。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `result` | string | 替换结果 |
| `count` | integer | 实际替换次数 |

### 示例

```lua
local result1, count1 = string.regex_gsub("one two three", [[\w+]], "X")
debug.print(result1 .. " " .. count1 .. "\n")

local result2, count2 = string.regex_gsub("2023-2024-2025", [[(\d+)]], "[$1]", {limit = 2})
debug.print(result2 .. " " .. count2 .. "\n")

local result3, count3 = string.regex_gsub("apple banana apple", [[(\w+)]], { apple = "fruit", banana = "berry" })
debug.print(result3 .. " " .. count3 .. "\n")

local result4, count4 = string.regex_gsub("a1 b2 c3", [[(\w)(\d)]], function(letter, num)
    return letter .. string.rep("x", tonumber(num))
  end)
debug.print(result4 .. " " .. count4)
```

**输出：**

```lua
```

---

## `regex_test`

判断文本是否匹配给定的正则表达式。

### 调用

```lua
string.regex_test
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |

## 返回值

返回一个值。

| 类型 | 说明 |
| --- | --- |
| boolean | 是否存在匹配 |

### 示例

```lua
local t1 = string.regex_test("abc123", [[\d+]])
debug.print(tostring(t1))

local t2 = string.regex_test("hello", [[\d+]])
debug.print(tostring(t2))
```

**输出：**

```lua
```

---

## `regex_split`

按正则表达式分割目标字符串。

### 调用

```lua
string.regex_split
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 目标字符串 |
| `pattern` | string | 正则表达式 |

## 返回值

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
| table | 分割结果 |

### 示例

```lua
local parts1 = string.regex_split("a b c", [[\s+]])
debug.print(parts1[1])
debug.print(parts1[2])
debug.print(parts1[3])

debug.print("")

local parts2 = string.regex_split("one, two;three", [[\s*[,;]\s*]])
debug.print(parts2[1])
debug.print(parts2[2])
debug.print(parts2[3])
```

**输出：**

```lua
```

---

## `format`

按格式串格式化值列表。

### 调用

```lua
string.format
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `format_string` | string | 格式串 |

## 返回值

返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | 格式化结果 |

### 示例

```lua
local f1 = string.format("Hello %s!", table.unpack({ "World" }))
debug.print(f1)

local f2 = string.format("%s is %d years old.", table.unpack({ "Alice", 30 }))
debug.print(f2)

local f3 = string.format("Pi ≈ %.2f", table.unpack({ math.PI }))
debug.print(f3)

local f4 = string.format("Hello, Tui Game!", table.unpack({}))
debug.print(f4)
```

**输出：**

```lua
```

### 额外说明

- 格式串后依次传入要格式化的值，数量与类型必须匹配对应占位符；这些值是格式化数据，不作为选项表解析。

---

## `rich_text_to_plain_text`

将富文本转换为普通文本。

### 调用

```lua
string.rich_text_to_plain_text
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 富文本字符串 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `rich_params` | table | `nil` | 富文本参数表 |
| `key_params` | boolean | `true` | 是否解析按键参数 |
| `strip_header` | boolean | `true` | 是否剥离 `f%` 头 |

## 返回值

返回一个字符串。

| 类型 | 说明 |
| --- | --- |
| string | 转换后的普通文本 |

### 示例

```lua
local plain1 = string.rich_text_to_plain_text("f%<fg:red>Hello</fg>")
debug.print(plain1)

local plain2 = string.rich_text_to_plain_text("f%<fg:red>Hello {value:name}</fg>", {rich_params = { name = "World" }})
debug.print(plain2)

local plain3 = string.rich_text_to_plain_text("f%<fg:red>{key:exit}</fg>", {key_params = false})
debug.print(plain3)

local plain4 = string.rich_text_to_plain_text("f%<fg:red>Hello</fg>", {strip_header = false})
debug.print(plain4)
```

**输出：**

```lua
```

### 额外说明

- 该 API 返回的普通文本会去掉所有的富文本标签，并按需解析相关参数标签（未被解析的参数标签会保留）。
