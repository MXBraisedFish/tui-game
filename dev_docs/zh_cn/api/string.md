# string 库

## 基本库说明

`string` 提供字符串处理。

## 目录

### 常量

| 常量名       | 说明                 | 索引                      |
| ------------ | -------------------- | ------------------------- |
| `AUTO`       | 自动检查文本类型     | [AUTO](#AUTO)             |
| `PLAIN_TEXT` | 强制按普通文本解析     | [PLAIN_TEXT](#PLAIN_TEXT) |
| `RICH_TEXT`  | 强制按富文本语法解析 | [RICH_TEXT](#RICH_TEXT)   |

### 方法

| 方法名                    | 说明                                             | 索引                                                |
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

## 常量

## `AUTO`

自动检查文本类型。

**可用于**

- 参数 `text_mode`

### 调用

```lua
string.AUTO
```

### 示例

```lua
p_str = "Hello Tui Game"
r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

len1 = measurement.get_text_width { text = p_str, text_mode = string.AUTO }
len2 = measurement.get_text_width { text = r_str, text_mode = string.AUTO }

debug.print { message = tostring(len1) }
debug.print { message = tostring(len2) }
```

输出：

```text
14
14
```

---

## `PLAIN_TEXT`

强制按普通文本解析；头部声明 `f%`，富文本标签会被强制保留。

**可用于**

- 参数 `text_mode`

### 调用

```lua
string.PLAIN_TEXT
```

### 示例

```lua
p_str = "Hello Tui Game"
r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

len1 = measurement.get_text_width { text = p_str, text_mode = string.PLAIN_TEXT }
len2 = measurement.get_text_width { text = r_str, text_mode = string.PLAIN_TEXT }

debug.print { message = tostring(len1) }
debug.print { message = tostring(len2) }
```

输出：

```text
14
40
```

---

## `RICH_TEXT`

强制按富文本语法解析；头部声明 `f%` 会被强制保留。

**可用于**

- 文本参数 `text_mode`

### 调用

```lua
string.RICH_TEXT
```

### 示例

```lua
nh_r_str = "<fg:red>Hello<fg:yellow> Tui Game</fg>"
r_str = "f%<fg:red>Hello<fg:yellow> Tui Game</fg>"

len1 = measurement.get_text_width { text = nh_r_str, text_mode = string.RICH_TEXT }
len2 = measurement.get_text_width { text = r_str, text_mode = string.RICH_TEXT }

debug.print { message = tostring(len1) }
debug.print { message = tostring(len2) }
```

输出：

```text
14
16
```

---

## 方法

## `lower`

将字符串全部转为小写。

### 调用

```lua
-- 单参数
string.lower()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

| 类型   | 说明     |
| ------ | -------- |
| string | 小写结果 |

### 示例

```lua
u_str = "HELLO TUI GAME"
str = string.lower(u_str)

debug.print { message = str }
```

输出：

直接返回一个值。

```text
hello tui game
```

---

## `upper`

将字符串全部转为大写。

### 调用

```lua
-- 单参数
string.upper()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 大写结果 |

### 示例

```lua
l_str = "hello tui game"
str = string.upper(l_str)

debug.print { message = str }
```

输出：

```text
HELLO TUI GAME
```

---

## `reverse`

按字符反转字符串。

### 调用

```lua
-- 单参数
string.reverse()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

| 类型   | 说明     |
| ------ | -------- |
| string | 反转结果 |

### 示例

```lua
r_str = "emaG iuT olleH"
str = string.reverse(r_str)

debug.print { message = str }
```

输出：

```text
Hello Tui Game
```

---

## `split`

按指定分割字符分割目标字符串。

### 调用

```lua
-- 表参数
string.split{}
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |
| `sep`  | string | 是   | -      | 分割字符   |

### 返回

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
| table | 分割结果 |

### 示例

```lua
parts = string.split { text = "apple,banana,grape", sep = "," }

for i in ipairs(parts) do
  debug.print { message = i.value }
end
```

输出：

```text
apple
banana
grape
```

---

## `sub`

按字符位置截取子串。

### 调用

```lua
-- 表参数
string.sub{}
```

### 参数

| 参数名   | 类型    | 必填 | 默认值     | 说明         |
| -------- | ------- | ---- | ---------- | ------------ |
| `text`   | string  | 是   | -          | 目标字符串   |
| `start`  | integer | 是   | -          | 起始字符位置 |
| `finish` | integer | 否   | 目标字符串长度 | 结束字符位置 |

### 返回

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 截取结果 |

### 示例

```lua
sub_str = "Hello Tui Game"
str = string.sub { text = sub_str, start = 1, finish = 5 }

debug.print { message = str }
```

输出：

```text
Hello
```

---

## `rep`

将字符串重复数次并按照指定分隔符拼接。

### 调用

```lua
-- 表参数
string.rep{}
```

### 参数

| 参数名  | 类型    | 必填 | 默认值 | 说明               |
| ------- | ------- | ---- | ------ | ------------------ |
| `text`  | string  | 是   | -      | 要重复的字符串     |
| `times` | integer | 是   | -      | 重复次数           |
| `sep`   | string  | 否   | `""`   | 相邻副本间的分隔符 |

### 返回

直接返回一个值。

| 类型   | 说明         |
| ------ | ------------ |
| string | 重复拼接结果 |

### 示例

```lua
rep_str = "ABC"
str = string.rep { text = rep_str, times = 3, sep = " | " }

debug.print { message = str }
```

输出：

```text
ABC | ABC | ABC
```

---

## `find`

按照模式字符串，从起始搜索位置开始，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
-- 表参数
string.find{}
```

### 参数

| 参数名    | 类型    | 必填 | 默认值  | 说明             |
| --------- | ------- | ---- | ------- | ---------------- |
| `text`    | string  | 是   | -       | 目标字符串       |
| `pattern` | string  | 是   | -       | 模式字符串       |
| `init`    | integer | 否   | `1`     | 起始搜索位置     |
| `plain`   | boolean | 否   | `false` | 是否按普通文本查找 |

### 返回

若**查找成功**，返回一个对象表。

| 字段       | 类型    | 说明                   |
| ---------- | ------- | ---------------------- |
| `start`    | integer | 匹配起点               |
| `finish`   | integer | 匹配终点               |
| `captures` | table   | 匹配到的字符串或捕获组 |

若**查找失败**，直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
result1 = string.find { text = "Hello Tui Game", pattern = "Tui" }
debug.print { message = tostring(result1.start) }
debug.print { message = tostring(result1.finish) }
debug.print { message = result1.captures[1] }
debug.print { message = tostring(result1.captures.n) .. "\n" }

result2 = string.find { text = "Name: Alice, Age: 30", pattern = "Name: (%w+), Age: (%d+)" }
debug.print { message = tostring(result2.start) }
debug.print { message = tostring(result2.finish) }
debug.print { message = result2.captures[1] }
debug.print { message = tostring(result2.captures[2]) }
debug.print { message = tostring(result2.captures.n) }
```

输出：

```text
7
9
Tui
1

1
20
Alice
30
2
```

### 额外补充

- 返回值 `captures` 表结构如下：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

- 匹配结果为零长字符时，返回值中的 $finish = start - 1$

---

## `match`

按照模式字符串，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
-- 表参数
string.match{}
```

### 参数

| 参数名    | 类型    | 必填 | 默认值 | 说明         |
| --------- | ------- | ---- | ------ | ------------ |
| `text`    | string  | 是   | -      | 目标字符串   |
| `pattern` | string  | 是   | -      | 模式字符串   |
| `init`    | integer | 否   | `1`    | 起始搜索位置 |

### 返回

若**查找成功**，返回一个混合表。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 匹配到的字符串或捕获组 |

若**查找失败**，直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
match1 = string.match { text = "Hello 123", pattern = "%d+" }
debug.print { message = match1[1] }
debug.print { message = tostring(match1.n) .. "\n" }

match2 = string.match { text = "Product: Apple, Price: 5.99", pattern = "Product: (%w+), Price: ([%d.]+)" }
debug.print { message = match2[1] }
debug.print { message = match2[2] }
debug.print { message = tostring(match2.n) }
```

输出：

```text
123
1

Apple
5.99
2
```

### 额外补充

- 返回值混合表结构如下：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

---

## `gmatch`

按照模式字符串，遍历并匹配目标字符串中所有满足要求的内容或捕获组。

### 调用

```lua
-- 表参数
string.gmatch{}
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明       |
| --------- | ------ | ---- | ------ | ---------- |
| `text`    | string | 是   | -      | 目标字符串 |
| `pattern` | string | 是   | -      | 模式字符串 |

### 返回

直接返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个混合表。

| 字段      | 类型      | 说明         |
| --------- | --------- | ------------ |
| [integer] | string... | 捕获结果     |
| `n`       | integer   | 捕获结果数量 |

### 示例

```lua
iter1 = string.gmatch { text = "a1 b2 c3", pattern = "%w+" }

for m in iter1 do
  debug.print { message = m[1] .. " " .. m.n }
end

debug.print { message = "" }

iter2 = string.gmatch { text = "A-1 B-2 C-3", pattern = "(%w+)-(%d+)" }

for caps in iter2 do
  debug.print { message = caps[1] .. " " .. caps[2] .. " " .. caps.n }
end
```

输出：

```text
a1 1
b2 1
c3 1

A 1 2
B 2 2
C 3 2
```

### 额外补充

- 迭代器返回值元素混合表结构：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

---

## `gsub`

全局替换匹配内容。

### 调用

```lua
-- 表参数
string.gsub{}
```

### 参数

| 参数名    | 类型                      | 必填 | 默认值 | 说明         |
| --------- | ------------------------- | ---- | ------ | ------------ |
| `text`    | string                    | 是   | -      | 目标字符串   |
| `pattern` | string                    | 是   | -      | 模式字符串   |
| `repl`    | string / table / function | 是   | -      | 替换内容     |
| `limit`   | integer                   | 否   | `-1`   | 最大替换次数 |

### 返回

返回一个对象表。

| 字段     | 类型    | 说明         |
| -------- | ------- | ------------ |
| `result` | string  | 替换结果     |
| `count`  | integer | 实际替换次数 |

### 示例

```lua
r1 = string.gsub { text = "one two three", pattern = "%a+", repl = "X" }
debug.print { message = r1.result .. " " .. r1.count .. "\n" }

r2 = string.gsub { text = "2023-2024-2025", pattern = "(%d+)", repl = "[$1]", limit = 2 }
debug.print { message = r2.result .. " " .. r2.count .. "\n" }

r3 = string.gsub { text = "apple banana apple", pattern = "(%w+)", repl = { apple = "fruit", banana = "berry" } }
debug.print { message = r3.result .. " " .. r3.count .. "\n" }

r4 = string.gsub {
  text = "a1 b2 c3",
  pattern = "(%w)(%d)",
  repl = function(letter, num) return letter .. string.rep { text = "x", times = tonumber { value = num } } end
}
debug.print { message = r4.result .. " " .. r4.count }
```

输出：

```text
X X X 3

[$1]-[$1]-2025 2

fruit berry fruit 3

ax bxx cxxx 3
```

### 额外补充

- 参数 `limit` 为 -1 时代表不限次数。

---

## `regex_escape`

转义字符串中的正则特殊字符为普通文本。

### 调用

```lua
-- 单参数
string.regex_escape()
```

### 参数

| 参数名 | 类型   | 必填 | 默认值 | 说明       |
| ------ | ------ | ---- | ------ | ---------- |
| `text` | string | 是   | -      | 目标字符串 |

### 返回

直接返回一个值。

| 类型   | 说明           |
| ------ | -------------- |
| string | 转义后的字符串 |

### 示例

```lua
local e1 = string.regex_escape("hello")
debug.print { message = e1 }

local e2 = string.regex_escape("a+b*c?")
debug.print { message = e2 }
```

输出：

```text
hello
a\+b\*c\?
```

---

## `regex_find`

按照正则表达式，从起始搜索位置开始，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
-- 表参数
string.regex_find{}
```

### 参数

| 参数名    | 类型    | 必填 | 默认值 | 说明         |
| --------- | ------- | ---- | ------ | ------------ |
| `text`    | string  | 是   | -      | 目标字符串   |
| `pattern` | string  | 是   | -      | 正则表达式   |
| `init`    | integer | 否   | `1`    | 起始搜索位置 |

### 返回

若**查找成功**，返回一个对象表。

| 字段       | 类型    | 说明                   |
| ---------- | ------- | ---------------------- |
| `start`    | integer | 匹配起点               |
| `finish`   | integer | 匹配终点               |
| `captures` | table   | 匹配到的字符串或捕获组 |

若**查找失败**，直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
f1 = string.regex_find { text = "Hello 123", pattern = [[\d+]] }
debug.print { message = tostring(f1.start) }
debug.print { message = tostring(f1.finish) }
debug.print { message = f1.captures[1] }
debug.print { message = tostring(f1.captures.n) .. "\n" }

f2 = string.regex_find { text = "Name: Alice, Age: 30", pattern = [[Name: (\w+), Age: (\d+)]] }
debug.print { message = tostring(f2.start) }
debug.print { message = tostring(f2.finish) }
debug.print { message = f2.captures[1] }
debug.print { message = f2.captures[2] }
debug.print { message = tostring(f2.captures.n) }
```

输出：

```text
7
9
123
1

1
20
Alice
30
2
```

### 额外补充

- 返回值 `captures` 表结构如下：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

- 匹配结果为零长字符时，返回值中的 $finish = start - 1$，且字段 `caputers.n` 为 0。

---

## `regex_match`

按照正则表达式，匹配目标字符串中首个满足要求的内容或捕获组。

### 调用

```lua
-- 表参数
string.regex_match{}
```

### 参数

| 参数名    | 类型    | 必填 | 默认值 | 说明         |
| --------- | ------- | ---- | ------ | ------------ |
| `text`    | string  | 是   | -      | 目标字符串   |
| `pattern` | string  | 是   | -      | 正则表达式   |
| `init`    | integer | 否   | `1`    | 起始搜索位置 |

### 返回

若**查找成功**，返回一个混合表。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 匹配到的字符串或捕获组 |

若**查找失败**，直接返回一个值。

| 类型 | 说明     |
| ---- | -------- |
| nil  | 查找失败 |

### 示例

```lua
m1 = string.regex_match { text = "Hello 123", pattern = [[\d+]] }
debug.print { message = m1[1] }
debug.print { message = tostring(m1.n) .. "\n" }

m2 = string.regex_match { text = "Name: Alice, Age: 30", pattern = [[Name: (\w+), Age: (\d+)]] }
debug.print { message = m2[1] }
debug.print { message = m2[2] }
debug.print { message = tostring(m2.n) }
```

### 额外补充

- 返回值混合表结构如下：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

---

## `regex_gmatch`

返回用正则迭代全部匹配的迭代函数。

### 调用

```lua
-- 表参数
string.regex_gmatch{}
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明       |
| --------- | ------ | ---- | ------ | ---------- |
| `text`    | string | 是   | -      | 目标字符串 |
| `pattern` | string | 是   | -      | 正则表达式 |

### 返回

直接返回一个值。

| 类型     | 说明       |
| -------- | ---------- |
| function | 迭代器函数 |

**迭代器函数**，返回一个混合表。

| 字段      | 类型      | 说明         |
| --------- | --------- | ------------ |
| [integer] | string... | 捕获结果     |
| `n`       | integer   | 捕获结果数量 |

### 示例

```lua
iter1 = string.regex_gmatch { text = "a1 b2 c3", pattern = [[\w+]] }

for m in iter1 do
  debug.print { message = m[1] .. " " .. m.n }
end

debug.print { message = "" }

iter2 = string.regex_gmatch { text = "A-1 B-2 C-3", pattern = [[(\w+)-(\d+)]] }
for caps in iter2 do
  debug.print { message = caps[1] .. " " .. caps[2] .. " " .. caps.n }
end
```

输出：

```text
a1 1
b2 1
c3 1

A 1 2
B 2 2
C 3 2
```

### 额外补充

- 迭代器返回值元素混合表结构：

```lua
{
  [1] = ..., -- string
  [2] = ...,
  ...
  [x] = ..., -- string
  n = x      -- integer
} -- 共有 x+1 个元素，所有捕获结果连续排序，最后 n 为捕获结果数量
```

---

## `regex_gsub`

全局替换匹配内容。

### 调用

```lua
-- 表参数
string.regex_gsub{}
```

### 参数

| 参数名    | 类型                      | 必填 | 默认值 | 说明         |
| --------- | ------------------------- | ---- | ------ | ------------ |
| `text`    | string                    | 是   | -      | 目标字符串   |
| `pattern` | string                    | 是   | -      | 正则表达式   |
| `repl`    | string / table / function | 是   | -      | 替换内容     |
| `limit`   | integer                   | 否   | `-1`   | 最大替换次数 |

### 返回

返回一个对象表。

| 字段     | 类型    | 说明         |
| -------- | ------- | ------------ |
| `result` | string  | 替换结果     |
| `count`  | integer | 实际替换次数 |

### 示例

```lua
g1 = string.regex_gsub { text = "one two three", pattern = [[\w+]], repl = "X" }
debug.print { message = g1.result .. " " .. g1.count .. "\n" }

g2 = string.regex_gsub { text = "2023-2024-2025", pattern = [[(\d+)]], repl = "[$1]", limit = 2 }
debug.print { message = g2.result .. " " .. g2.count .. "\n" }

g3 = string.regex_gsub { text = "apple banana apple", pattern = [[(\w+)]], repl = { apple = "fruit", banana = "berry" } }
debug.print { message = g3.result .. " " .. g3.count .. "\n" }

g4 = string.regex_gsub {
  text = "a1 b2 c3",
  pattern = [[(\w)(\d)]],
  repl = function(letter, num)
    return letter .. string.rep {
      text = "x",
      times = tonumber { value = num }
    }
  end
}
debug.print { message = g4.result .. " " .. g4.count }
```

输出：

```text
X X X 3

[2023]-[2024]-2025 2

fruit berry fruit 3

ax bxx cxxx 3
```

---

## `regex_test`

判断文本是否匹配给定的正则表达式。

### 调用

```lua
-- 表参数
string.regex_test{}
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明       |
| --------- | ------ | ---- | ------ | ---------- |
| `text`    | string | 是   | -      | 目标字符串 |
| `pattern` | string | 是   | -      | 正则表达式 |

### 返回

直接返回一个值。

| 返回值名  | 类型    | 说明         |
| --------- | ------- | ------------ |
| `matched` | boolean | 是否存在匹配 |

### 示例

```lua
t1 = string.regex_test { text = "abc123", pattern = [[\d+]] }
debug.print { message = tostring(t1) }

t2 = string.regex_test { text = "hello", pattern = [[\d+]] }
debug.print { message = tostring(t2) }
```

输出：

```text
true
false
```

---

## `regex_split`

按正则表达式分割目标字符串。

### 调用

```lua
-- 表参数
string.regex_split{}
```

### 参数

| 参数名    | 类型   | 必填 | 默认值 | 说明       |
| --------- | ------ | ---- | ------ | ---------- |
| `text`    | string | 是   | -      | 目标字符串 |
| `pattern` | string | 是   | -      | 正则表达式 |

### 返回

返回一个数组表。

| 类型  | 说明     |
| ----- | -------- |
| table | 分割结果 |

### 示例

```lua
parts1 = string.regex_split { text = "a b c", pattern = [[\s+]] }
debug.print { message = parts1[1] }
debug.print { message = parts1[2] }
debug.print { message = parts1[3] }

debug.print { message = "" }

parts2 = string.regex_split { text = "one, two;three", pattern = [[\s*[,;]\s*]] }
debug.print { message = parts2[1] }
debug.print { message = parts2[2] }
debug.print { message = parts2[3] }
```

输出：

```text
a
b
c

one
two
three
```

---

## `format`

按格式串格式化值列表。

### 调用

```lua
-- 表参数
string.format{}
```

### 参数

| 参数名          | 类型   | 必填 | 默认值 | 说明       |
| --------------- | ------ | ---- | ------ | ---------- |
| `format_string` | string | 是   | -      | 格式串     |
| `values`        | table  | 否   | `nil`  | 参数值数组表 |

### 返回

直接返回一个值。

| 类型   | 说明       |
| ------ | ---------- |
| string | 格式化结果 |

### 示例

```lua
f1 = string.format { format_string = "Hello %s!", values = { "World" } }
debug.print { message = f1 }

f2 = string.format { format_string = "%s is %d years old.", values = { "Alice", 30 } }
debug.print { message = f2 }

f3 = string.format { format_string = "Pi ≈ %.2f", values = { math.PI } }
debug.print { message = f3 }

f4 = string.format { format_string = "Hello, Tui Game!" }
debug.print { message = f4 }
```

输出：

```text
Hello World!
Alice is 30 years old.
Pi ≈ 3.14
Hello, Tui Game!
```

---

## `rich_text_to_plain_text`

将富文本转换为普通文本。

### 调用

```lua
-- 表参数
string.rich_text_to_plain_text{}
```

### 参数

| 参数名         | 类型    | 必填 | 默认值 | 说明             |
| -------------- | ------- | ---- | ------ | ---------------- |
| `text`         | string  | 是   | -      | 富文本字符串     |
| `rich_params`  | table   | 否   | `nil`  | 富文本参数表     |
| `key_params`   | boolean | 否   | `true` | 是否解析按键参数 |
| `strip_header` | boolean | 否   | `true` | 是否剥离 `f%` 头 |

### 返回

| 返回值名 | 类型   | 说明             |
| -------- | ------ | ---------------- |
| `text`   | string | 转换后的普通文本 |

### 示例

```lua
plain1 = string.rich_text_to_plain_text { text = "f%<fg:red>Hello</fg>" }
debug.print { message = plain1 }

plain2 = string.rich_text_to_plain_text { text = "f%<fg:red>Hello {value:name}</fg>", rich_params = { name = "World" } }
debug.print { message = plain2 }

plain3 = string.rich_text_to_plain_text { text = "f%<fg:red>{key:exit}</fg>", key_params = false }
debug.print { message = plain3 }

plain4 = string.rich_text_to_plain_text { text = "f%<fg:red>Hello</fg>", strip_header = false }
debug.print { message = plain4 }
```

输出：

```text
Hello
Hello World
{key:exit}
f%Hello
```

### 额外补充

- 该 API 返回的普通文本会去掉所有的富文本标签，并按需解析相关参数标签（未被解析的参数标签会保留）。
