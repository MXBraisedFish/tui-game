# Lua 模式与正则表达式语法规范

## 前言

Lua 模式与正则表达式是字符串匹配操作的核心参数，用于精确描述如何在 Lua 字符串中执行文本检索与捕获。本文档旨在为开发者提供一份语法速查手册。

> 模式串的语法规则与 Lua 官方原版完全一致，Tui Game 仅在签名的 API 使用上有修改。如需完整的规范说明，请参阅 Lua 语言官方文档。

---

## 目录

## Lua 模式

| 章节       | 说明                         | 索引                      |
| ---------- | ---------------------------- | ------------------------- |
| 魔法字符   | Lua 模式中的特殊字符及其含义 | [魔法字符](#魔法字符)     |
| 转义符号   | 使用 `%` 转义特殊字符        | [转义符号](#转义符号)     |
| 量词       | 匹配前一字符的次数控制       | [量词](#量词)             |
| 字符类     | 预定义字符类别               | [字符类](#字符类)         |
| 字符集     | 自定义字符集合               | [字符集](#字符集)         |
| 捕获与分组 | 使用 `(...)` 捕获匹配内容    | [捕获与分组](#捕获与分组) |
| 锚点与边界 | 匹配输入开头和结尾           | [锚点与边界](#锚点与边界) |
| 平衡匹配   | 匹配平衡的成对符号           | [平衡匹配](#平衡匹配)     |
| 捕获引用   | 在替换中引用捕获组           | [捕获引用](#捕获引用)     |

## 正则表达式

| 章节            | 说明                            | 索引                                |
| --------------- | ------------------------------- | ----------------------------------- |
| Lua 长字符串    | 避免转义地狱的字符串语法        | [Lua 长字符串](#lua-长字符串)       |
| 元字符          | 正则表达式中的特殊字符          | [元字符](#元字符)                   |
| 转义符号        | 使用 `\` 转义特殊字符           | [转义符号](#转义符号-1)             |
| 特殊转义        | 常用转义序列                    | [特殊转义](#特殊转义)               |
| 量词            | 贪婪与懒惰量词                  | [量词](#量词-1)                     |
| Perl 风格字符类 | `\d`、`\s`、`\w` 等             | [Perl 风格字符类](#perl-风格字符类) |
| POSIX 字符类    | `[[:ascii:]]`、`[[:alnum:]]` 等 | [POSIX 字符类](#posix-字符类)       |
| 字符集          | 自定义字符集合                  | [字符集](#字符集-1)                 |
| Unicode 属性    | `\p{...}` 匹配 Unicode 属性     | [Unicode 属性](#unicode-属性)       |
| 捕获与分组      | 命名捕获与分组                  | [捕获与分组](#捕获与分组-1)         |
| 锚点与边界      | `\A`、`\z`、`\b` 等             | [锚点与边界](#锚点与边界-1)         |
| 模式标志        | `(?i)`、`(?m)` 等模式开关       | [模式标志](#模式标志)               |
| 捕获引用        | `$0`、`$1` 等在替换中引用捕获   | [捕获引用](#捕获引用-1)             |

## 附录

| 章节             | 说明                     | 索引                                  |
| ---------------- | ------------------------ | ------------------------------------- |
| Unicode 属性参数 | `\p{...}` 可用的属性参数 | [Unicode 属性参数](#unicode-属性参数) |
| 模式标志参数     | `(?...)` 可用的模式标志  | [模式标志参数](#模式标志参数)         |

---

## Lua 模式

Lua 模式是 Lua 语言内置的轻量级匹配语法，相较于正则表达式语法更简洁。

---

## 魔法字符

### 语法

| 语法    | 说明                                                      |
| ------- | --------------------------------------------------------- |
| `%`     | 转义字符                                                  |
| `.`     | 匹配除换行符外的任意字符                                  |
| `+`     | 匹配前一字符 1 次或多次                                   |
| `*`     | 匹配前一字符 0 次或多次                                   |
| `-`     | 匹配前一字符 0 次或多次（非贪婪）；在字符集内取连字符范围 |
| `?`     | 匹配前一字符 0 次或 1 次                                  |
| `^`     | 匹配字符串开头；在字符集内取反                            |
| `$`     | 匹配字符串结尾                                            |
| `(...)` | 用于捕获分组                                              |
| `[...]` | 定义字符集                                                |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.find(s1, "%d+")
debug.print(start1 .. " " .. finish1)

s2 = "abc"
local start2, finish2, captures2 = string.find(s2, "%d*")
debug.print(start2 .. " " .. finish2)
```

输出：

```text
4 6
1 0
```

---

## 转义符号

### 语法

| 语法 | 说明     |
| ---- | -------- |
| `%`  | 转义符号 |

### 示例

```lua
s1 = "100% complete"
local start1, finish1, captures1 = string.find(s1, "%%")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "file.txt"
local start2, finish2, captures2 = string.find(s2, "%.")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
4 4 %
5 5 .
```

---

## 量词

| 语法 | 说明                            |
| ---- | ------------------------------- |
| `+`  | 匹配前一字符 1 次或多次         |
| `*`  | 匹配前一字符 0 次或多次         |
| `-`  | 匹配前一字符 0 次或多次（懒惰） |
| `?`  | 匹配前一字符 0 次或 1 次        |

### 示例

```lua
s1 = "abc123def"
local start1, finish1, captures1 = string.find(s1, "%d+")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "abc"
local start2, finish2, captures2 = string.find(s2, "%d*")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])

s3 = "a<b>c<d>e"
local start3, finish3, captures3 = string.find(s3, "<.->")
debug.print(start3 .. " " .. finish3 .. " " .. captures3[1])

s4 = "color"
local start4, finish4, captures4 = string.find(s4, "colou?r")
debug.print(start4 .. " " .. finish4 .. " " .. captures4[1])
```

输出：

```text
4 6 123
1 0
2 4 <b>
1 5 color
```

### 额外补充

- 未添加特殊标注的语法均为贪婪匹配。

---

## 字符类

### 语法

| 语法 | 说明                      |
| ---- | ------------------------- |
| `.`  | 匹配除换行符外的任意字符  |
| `%a` | 匹配任意字母              |
| `%c` | 匹配任意控制字符          |
| `%d` | 匹配任意数字              |
| `%l` | 匹配任意小写字母          |
| `%p` | 匹配任意标点符号          |
| `%s` | 匹配任意空白字符          |
| `%u` | 匹配任意大写字母          |
| `%w` | 匹配任意字母数字          |
| `%x` | 匹配任意十六进制数字      |
| `%z` | 匹配任意 ASCII NUL 字符   |
| `%A` | 匹配任意非字母            |
| `%C` | 匹配任意非控制字符        |
| `%D` | 匹配任意非数字            |
| `%L` | 匹配任意非小写字母        |
| `%P` | 匹配任意非标点符号        |
| `%S` | 匹配任意非空白字符        |
| `%U` | 匹配任意非大写字母        |
| `%W` | 匹配任意非字母数字        |
| `%X` | 匹配任意非十六进制数字    |
| `%Z` | 匹配任意非 ASCII NUL 字符 |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.find(s1, "%d+")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "123abc"
local start2, finish2, captures2 = string.find(s2, "%D+")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
4 6 123
4 6 abc
```

---

## 字符集

### 语法

| 语法    | 说明               |
| ------- | ------------------ |
| `[...]` | 匹配所定义字符集   |
| `-`     | 连字符范围         |
| `^`     | 取反字符集（补集） |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.find(s1, "[abc]+")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "hello123"
local start2, finish2, captures2 = string.find(s2, "[a-z]+")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])

s3 = "abc123"
local start3, finish3, captures3 = string.find(s3, "[^a-z]+")
debug.print(start3 .. " " .. finish3 .. " " .. captures3[1])
```

输出：

```text
1 3 abc
1 5 hello
4 6 123
```

### 额外补充

- `-` 两侧字符范围为 ASCII 字符表。
- `^` 仅放在字符集开头时代表取反字符集，若需要开头使用普通字符需转义 `%^`。

---

## 捕获与分组

### 语法

| 语法    | 说明               |
| ------- | ------------------ |
| `(...)` | 捕获匹配内容并分组 |

### 示例

```lua
s1 = "Name: Alice"
local start1, finish1, captures1 = string.find(s1, "Name: (%w+)")
debug.print(captures1[1] .. " " .. captures1.n)

s2 = "Alice, 30"
local start2, finish2, captures2 = string.find(s2, "(%w+), (%d+)")
debug.print(captures2[1] .. " " .. captures2[2] .. " " .. captures2.n)

s3 = "Today is 2024-12-25"
local start3, finish3, captures3 = string.find(s3, "(%d+)-(%d+)-(%d+)")
debug.print(captures3[1] .. " " .. captures3[2] .. " " .. captures3[3] .. " " .. captures3.n)

s4 = "hello"
local start4, finish4, captures4 = string.find(s4, "()hello()")
debug.print(captures4[1] .. " " .. captures4[2] .. " " .. captures4.n)
```

输出：

```text
Alice 1
Alice 30 2
2024 12 25 3
1 6 2
```

### 额外补充

- `()` 空捕获返回捕获起始位置。
- 分组按照从首个字符位置开向后记录 `(` 出现的顺序计数。

---

## 锚点与边界

| 语法 | 说明         |
| ---- | ------------ |
| `^`  | 匹配输入开头 |
| `$`  | 匹配输入结尾 |

### 示例

```lua
s1 = "hello world"
local start1, finish1, captures1 = string.find(s1, "^hello")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "hello world"
local start2, finish2, captures2 = string.find(s2, "world$")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
1 5 hello
7 11 world
```

---

## 平衡匹配

### 语法

| 语法   | 说明                                   |
| ------ | -------------------------------------- |
| `%bxy` | 匹配从 `x` 开始到 `y` 结束的平衡字符串 |

### 示例

```lua
s1 = "a(b(c)d)e"
local start1, finish1, captures1 = string.find(s1, "%b()")
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "x[1,2,[3,4]]y"
local start2, finish2, captures2 = string.find(s2, "%b[]")
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
2 8 (b(c)d)
2 12 [1,2,[3,4]]
2 8 {b{c}d}
2 8 <b<c>d>
```

---

## 捕获引用

### 语法

| 语法      | 说明                 |
| --------- | -------------------- |
| `%0`      | 完整匹配             |
| `%1`-`%9` | 第 1 至第 9 个捕获组 |
| `%%`      | 字面量 `%`           |

### 示例

```lua
s1 = "hello world"
local result1, count1 = string.gsub(s1, "%a+", "[%0]")
debug.print(result1 .. " " .. count1)

s2 = "2024-12-25"
local result2, count2 = string.gsub(s2, "(%d+)-(%d+)-(%d+)", "%1/%2/%3")
debug.print(result2 .. " " .. count2)
```

```text
[hello] [world] 2
2024/12/25 1
```

---

## 正则表达式

正则表达式是描述字符模式的形式语言，相较于 Lua 模式语法功能更加全面。

---

## Lua 长字符串

> 在正则表达式中可能需要频繁的使用 `\`做转义字符，为了避免 Lua 中的 `\\` 导致的转义地狱，可以使用 Lua 长字符串语法

### 语法

| 语法      | 说明     |
| --------- | -------- |
| `[[...]]` | 长字符串 |

### 示例

```lua
str1 = [[string \d \" \[]]
debug.print(str1)

str2 = [=[string [[]] string]=]
debug.print(str2)
```

输出：

```text
string \d \" \[
string [[]] string
```

### 额外补充

- 带 `=` 符号的是长字符串的特殊格式，当文本中出现 `[[` 和 `]]` 时可以使用该格式，`=` 不限数量，但必须左右两侧数量相等。

---

## 元字符

### 语法

| 语法    | 说明                                     |
| ------- | ---------------------------------------- |
| `\`     | 转义字符                                 |
| `.`     | 匹配除换行符外的任意字符                 |     |
| `*`     | 匹配前一字符 0 次或多次                  |
| `+`     | 匹配前一字符 1 次或多次                  |
| `?`     | 匹配前一字符 0 次或 1 次；或更多使用操作 |
| `-`     | 在字符集内取连字符范围；或更多使用操作   |
| `^`     | 匹配字符串开头；在字符集内取反           |
| `$`     | 匹配输入字符串的结尾；或更多使用操作     |
| `\|`    | 逻辑或                                   |
| `(...)` | 用于捕获分组                             |
| `[...]` | 定义字符集                               |
| `{...}` | 精确指定重复次数                         |

### 示例

```lua
s1 = "a.b"
local start1, finish1, captures1 = string.regex_find(s1, [[\.]])
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "abc"
local start2, finish2, captures2 = string.regex_find(s2, [[.]])
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
2 2 .
1 1 a
```

---

## 转义符号

### 语法

| 语法 | 说明     |
| ---- | -------- |
| `\`  | 转义字符 |

### 示例

```lua
s1 = "a.b"
local start1, finish1, captures1 = string.regex_find(s1, [[\.]])
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "a+b"
local start2, finish2, captures2 = string.regex_find(s2, [[\+]])
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])
```

输出：

```text
2 2 .
2 2 +
```

---

## 特殊转义

### 语法

| 语法      | 含义                      |
| --------- | ------------------------- |
| `\a`      | 响铃符                    |
| `\f`      | 换页符                    |
| `\t`      | 水平制表符                |
| `\v`      | 垂直制表符                |
| `\n`      | 换行符                    |
| `\r`      | 回车符                    |
| `\x...`   | 两位十六进制 Unicode 码点 |
| `\x{...}` | 可变长度 Unicode 码点     |
| `\u...`   | 四位十六进制 Unicode 码点 |
| `\u{...}` | 可变长度 Unicode 码点     |
| `\U...`   | 八位十六进制 Unicode 码点 |
| `\U...}`  | 可变长度 Unicode 码点     |

### 示例

```lua
s1 = "hello\nworld"
local start1, finish1, captures1 = string.regex_find(s1, [[\n]])
debug.print(start1 .. " " .. finish1 .. " " .. captures1[1])

s2 = "ABC"
local start2, finish2, captures2 = string.regex_find(s2, [[\x41]])
debug.print(start2 .. " " .. finish2 .. " " .. captures2[1])

s3 = "😀"
local start3, finish3, captures3 = string.regex_find(s3, [[\u{1F600}]])
debug.print(start3 .. " " .. finish3 .. " " .. captures3[1])
```

输出：

```text
6 6

1 1 A
1 1 😀
```

---

## 量词

### 语法

| 语法     | 含义                             |
| -------- | -------------------------------- |
| `+`      | 匹配前一字符 1 次或多次          |
| `*`      | 匹配前一字符 0 次或多次          |
| `?`      | 匹配前一字符 0 次或 1 次         |
| `{n}`    | 匹配前一字符 n 次                |
| `{n,}`   | 匹配前一字符至少 n 次            |
| `{n,m}`  | 匹配前一字符 n 至 m 次           |
| `+?`     | 匹配前一字符 1 次或多次（懒惰）  |
| `*?`     | 匹配前一字符 0 次或多次（懒惰）  |
| `??`     | 匹配前一字符 0 次或 1 次（懒惰） |
| `{n,m}?` | 匹配前一字符 n 至 m 次（懒惰）   |

### 示例

```lua
s1 = "aaab"
local start1, finish1, captures1 = string.regex_find(s1, [[a+]])
debug.print(start1 .. " " .. finish1)

s2 = "aaabc"
local start2, finish2, captures2 = string.regex_find(s2, [[a{2,3}]])
debug.print(start2 .. " " .. finish2)

s3 = "aaab"
local start3, finish3, captures3 = string.regex_find(s3, [[a*?]])
debug.print(start3 .. " " .. finish3)
```

输出：

```text
1 3
1 3
1 0
```

### 额外补充

- 未添加特殊标注的语法均为贪婪匹配。

---

## Perl 风格字符类

### 语法

| 语法 | 说明               |
| ---- | ------------------ |
| `\d` | 匹配任意数字       |
| `\s` | 匹配任意空白字符   |
| `\w` | 匹配任意字母数字   |
| `\D` | 匹配任意非数字     |
| `\S` | 匹配任意非空白字符 |
| `\W` | 匹配任意非字母数字 |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.regex_find(s1, [[\d+]])
debug.print(start1 .. " " .. finish1)

s2 = "a b"
local start2, finish2, captures2 = string.regex_find(s2, [[\s]])
debug.print(start2 .. " " .. finish2)

s3 = "hello!"
local start3, finish3, captures3 = string.regex_find(s3, [[\W]])
debug.print(start3 .. " " .. finish3)
```

输出：

```text
4 6
2 2
6 6
```

---

## POSIX 字符类

### 语法

| 语法           | 说明                      |
| -------------- | ------------------------- |
| `[[:ascii:]]`  | 匹配任意 ASCII 字符       |
| `[[:cntrl:]]`  | 匹配任意 ASCII 控制字符   |
| `[[:graph:]]`  | 匹配任意 ASCII 可见字符   |
| `[[:print:]]`  | 匹配任意 ASCII 可打印字符 |
| `[[:punct:]]`  | 匹配任意 ASCII 标点符号   |
| `[[:space:]]`  | 匹配任意 ASCII 空白字符   |
| `[[:blank:]]`  | 匹配任意空白字符          |
| `[[:alnum:]]`  | 匹配任意字母数字          |
| `[[:alpha:]]`  | 匹配任意字母              |
| `[[:digit:]]`  | 匹配任意数字              |
| `[[:lower:]]`  | 匹配任意小写字母          |
| `[[:upper:]]`  | 匹配任意大写字母          |
| `[[:word:]]`   | 匹配任意单词字符          |
| `[[:xdigit:]]` | 匹配任意十六进制数字      |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.regex_find(s1, [=[[[:digit:]]+]=])
debug.print(start1 .. " " .. finish1)

s2 = "123abc"
local start2, finish2, captures2 = string.regex_find(s2, [=[[[:alpha:]]+]=])
debug.print(start2 .. " " .. finish2)

s3 = "a b"
local start3, finish3, captures3 = string.regex_find(s3, [=[[[:space:]]]=])
debug.print(start3 .. " " .. finish3)

s4 = "hello!"
local start4, finish4, captures4 = string.regex_find(s4, [=[[[:punct:]]]=])
debug.print(start4 .. " " .. finish4)
```

输出：

```text
4 6
4 6
2 2
6 6
```

---

## 字符集

### 语法

| 语法    | 说明               |
| ------- | ------------------ |
| `[...]` | 匹配所定义字符集   |
| `-`     | 连字符范围         |
| `^`     | 取反字符集（补集） |
| `&&`    | 交集               |
| `--`    | 差集               |
| `~~`    | 对称差             |

### 示例

```lua
s1 = "defabc"
local start1, finish1, captures1 = string.regex_find(s1, [=[[abc]+]=])
debug.print(start1 .. " " .. finish1)

s3 = "abc123"
local start3, finish3, captures3 = string.regex_find(s3, [=[[^0-9]+]=])
debug.print(start3 .. " " .. finish3)

s4 = "abcdef"
local start4, finish4, captures4 = string.regex_find(s4, [=[[a-z&&[def]]+]=])
debug.print(start4 .. " " .. finish4)

s5 = "123456"
local start5, finish5, captures5 = string.regex_find(s5, [=[[0-9--4]+]=])
debug.print(start5 .. " " .. finish5)

s6 = "abgh"
local start6, finish6, captures6 = string.regex_find(s6, [=[[a-g~~b-h]+]=])
debug.print(start6 .. " " .. finish6)
```

输出：

```text
4 6
1 3
4 6
1 3
1 1
```

### 额外补充

- `-` 两侧字符范围为 ASCII 字符表。
- `^` 仅放在字符集开头时代表取反字符集，若需要开头使用普通字符需转义 `%^`。
- 运算符优先级：范围 > 并集 > 交集/差集/对称差（从左到右） > 取反。

---

## Unicode 属性

### 语法

| 语法      | 说明                              |
| --------- | --------------------------------- |
| `\p{...}` | 匹配具有特别 Unicode 属性的字符   |
| `\P{...}` | 匹配不具有特别 Unicode 属性的字符 |

### 示例

```lua
s1 = "abc123"
local start1, finish1, captures1 = string.regex_find(s1, [[\p{N}+]])
debug.print(start1 .. " " .. finish1)

s2 = "Hello"
local start2, finish2, captures2 = string.regex_find(s2, [[\p{Lu}]])
debug.print(start2 .. " " .. finish2)

s3 = "αβγ123"
local start3, finish3, captures3 = string.regex_find(s3, [[\p{Greek}+]])
debug.print(start3 .. " " .. finish3)

s4 = "abc123!@#"
local start4, finish4, captures4 = string.regex_find(s4, [[\P{L}+]])
debug.print(start4 .. " " .. finish4)
```

输出：

```text
4 6
1 1
1 3
4 9
```

### 额外补充

- 可填属性参数见「[附录-Unicode 属性参数](#unicode-属性参数)」

---

## 捕获与分组

### 语法

| 语法           | 说明                           |
| -------------- | ------------------------------ |
| `(...)`        | 捕获匹配内容并分组             |
| `(?:..)`       | 匹配内容并分组                 |
| `(?P<...>...)` | 捕获匹配内容并分组，且进行命名 |

### 示例

```lua
s1 = "Name: Alice"
local start1, finish1, captures1 = string.regex_find(s1, [[Name: (\w+)]])
debug.print(captures1[1] .. " " .. captures1.n)

s2 = "2024-12-25"
local start2, finish2, captures2 = string.regex_find(s2, [[(\d+)-(\d+)-(\d+)]])
debug.print(captures2[1] .. " " .. captures2[2] .. " " .. captures2[3] .. " " .. captures2.n)

s3 = "abc123"
local start3, finish3, captures3 = string.regex_find(s3, [[(?:abc)(\d+)]])
debug.print(captures3[1] .. " " .. captures3.n)

s4 = "Name: Alice"
local start4, finish4, captures4 = string.regex_find(s4, [[Name: (?P<user>\w+)]])
debug.print(captures4[1] .. " " .. captures4.n)
```

输出：

```text
Alice 1
2024 12 25 3
123 1
Alice 1
```

### 额外补充

- `()` 空捕获返回空字符串 `""`。
- 分组按照从首个字符位置开向后记录 `(` 出现的顺序计数。
- `(?P<...>...)` 在实际程序中无命名引用作用，仅作为增加可读性的扩充语法。

---

## 锚点与边界

### 语法

| 语法 | 说明               |
| ---- | ------------------ |
| `^`  | 匹配行输入开头     |
| `$`  | 匹配行输入结尾     |
| `\A` | 匹配整体输入开头   |
| `\z` | 匹配整体输入结尾   |
| `\b` | 匹配任意单词边界   |
| `\B` | 匹配任意非单词边界 |
| `\<` | 匹配单词起始边界   |
| `\>` | 匹配单词结束边界   |

### 示例

```lua
s1 = "hello world"
local start1, finish1, captures1 = string.regex_find(s1, [[^hello]])
debug.print(start1 .. " " .. finish1)

s2 = "hello world"
local start2, finish2, captures2 = string.regex_find(s2, [[world$]])
debug.print(start2 .. " " .. finish2)
```

输出：

```text
1 5
7 11
```

---

## 模式标志

### 语法

| 语法     | 说明           |
| -------- | -------------- |
| `(?...)` | 启用模式       |
| `-`      | 关闭模式       |
| `:`      | 仅在分组内生效 |

### 示例

```lua
s1 = "Hello"
local start1, finish1, captures1 = string.regex_find(s1, [[(?i)hELLo]])
debug.print(start1 .. " " .. finish1)

s2 = "Hello"
t2 = string.regex_test(s2, [[(?-i)hELLo]])
debug.print(tostring(t2))
```

输出：

```text
1 5
false
```

### 额外提示

- 可填属性参数见「[附录-模式标志参数](#模式标志参数)」

---

## 捕获引用

### 语法

| 语法      | 说明                 |
| --------- | -------------------- |
| `$0`      | 完整匹配             |
| `$1`-`$9` | 第 1 至第 9 个捕获组 |
| `$$`      | 字面量 `$`           |

### 示例

```lua
s1 = "hello world"
local result1, count1 = string.regex_gsub(s1, [[\w+]], "[$0]")
debug.print(result1 .. " " .. count1)

s2 = "2024-12-25"
local result2, count2 = string.regex_gsub(s2, [[(\d+)-(\d+)-(\d+)]], "$1/$2/$3")
debug.print(result2 .. " " .. count2)
```

```text
[hello] [world] 2
2024/12/25 1
```

---

## 附录

### Unicode 属性参数

- `L` 字母

| 子类 | 说明     |
| ---- | -------- |
| `Lu` | 大写字母 |
| `Ll` | 小写字母 |
| `Lt` | 标题字母 |
| `Lm` | 修饰字母 |
| `Lo` | 其它字母 |

- `M` 组合标记

| 子类 | 说明         |
| ---- | ------------ |
| `Mn` | 非间距标记   |
| `Mc` | 间距组合标记 |
| `Me` | 包围标记     |

- `N` 数字

| 子类 | 说明       |
| ---- | ---------- |
| `Nd` | 十进制数字 |
| `Nl` | 字母数字   |
| `Np` | 包其它数字 |

- `P` 标点符号

| 子类 | 说明     |
| ---- | -------- |
| `Pc` | 连接标点 |
| `Pd` | 破折号   |
| `Ps` | 开括号   |
| `Pe` | 闭括号   |
| `Pi` | 前引号   |
| `Pf` | 后引号   |
| `Po` | 其它标点 |

- `S` 符号

| 子类 | 说明     |
| ---- | -------- |
| `Sm` | 数学符号 |
| `Sc` | 货币符号 |
| `Sk` | 修饰符号 |
| `So` | 其它符号 |

- `Z` 分隔符

| 子类 | 说明       |
| ---- | ---------- |
| `Zs` | 空格分隔符 |
| `Zl` | 行分隔符   |
| `Zp` | 段落分隔符 |

- `C` 其它

| 子类 | 说明         |
| ---- | ------------ |
| `Cc` | 空格分隔符   |
| `Cf` | 格式控制字符 |
| `Cs` | 代理对       |
| `Co` | 私用区字符   |
| `Cn` | 未分配字符   |

- 脚本列表

| 脚本名                   | 说明                     |
| ------------------------ | ------------------------ |
| `Adlam`                  | 阿德拉姆文               |
| `Ahom`                   | 阿霍姆文                 |
| `Anatolian_Hieroglyphs`  | 安纳托利亚象形文字       |
| `Arabic`                 | 阿拉伯文                 |
| `Armenian`               | 亚美尼亚文               |
| `Avestan`                | 阿维斯陀文               |
| `Balinese`               | 巴厘文                   |
| `Bamum`                  | 巴穆姆文                 |
| `Bassa_Vah`              | 巴萨瓦文                 |
| `Batak`                  | 巴塔克文                 |
| `Bengali`                | 孟加拉文                 |
| `Bhaiksuki`              | 拜克舒基文               |
| `Bopomofo`               | 注音符号                 |
| `Brahmi`                 | 婆罗米文                 |
| `Braille`                | 盲文                     |
| `Buginese`               | 布吉文                   |
| `Buhid`                  | 布希德文                 |
| `Canadian_Aboriginal`    | 加拿大原住民音节文字     |
| `Carian`                 | 卡里亚文                 |
| `Caucasian_Albanian`     | 高加索阿尔巴尼亚文       |
| `Chakma`                 | 查克马文                 |
| `Cham`                   | 占文                     |
| `Cherokee`               | 切罗基文                 |
| `Chorasmian`             | 花剌子模文               |
| `Coptic`                 | 科普特文                 |
| `Cypriot`                | 塞浦路斯音节文字         |
| `Cyrillic`               | 西里尔字母               |
| `Deseret`                | 德瑟雷特文               |
| `Devanagari`             | 天城文                   |
| `Dives_Akuru`            | 迪维斯阿库鲁文           |
| `Dogra`                  | 多格拉文                 |
| `Duployan`               | 迪普洛伊安速记           |
| `Egyptian_Hieroglyphs`   | 埃及象形文字             |
| `Elbasan`                | 埃尔巴桑文               |
| `Elymaic`                | 埃利迈文                 |
| `Ethiopic`               | 埃塞俄比亚文             |
| `Georgian`               | 格鲁吉亚文               |
| `Glagolitic`             | 格拉哥里字母             |
| `Gothic`                 | 哥特文                   |
| `Grantha`                | 格兰塔文                 |
| `Greek`                  | 希腊文                   |
| `Gujarati`               | 古吉拉特文               |
| `Gunjala_Gondi`          | 贡贾拉贡迪文             |
| `Gurmukhi`               | 古木基文                 |
| `Han`                    | 汉字（含中日韩统一汉字） |
| `Hangul`                 | 韩文音节                 |
| `Hanunoo`                | 哈努诺文                 |
| `Hatran`                 | 哈特拉文                 |
| `Hebrew`                 | 希伯来文                 |
| `Hiragana`               | 平假名                   |
| `Imperial_Aramaic`       | 帝国阿拉姆文             |
| `Javanese`               | 爪哇文                   |
| `Kaithi`                 | 凯提文                   |
| `Kannada`                | 卡纳达文                 |
| `Katakana`               | 片假名                   |
| `Kayah_Li`               | 克耶黎文                 |
| `Kharoshthi`             | 佉卢文                   |
| `Khitan_Small_Script`    | 契丹小字                 |
| `Khmer`                  | 高棉文                   |
| `Khojki`                 | 科奇基文                 |
| `Lao`                    | 老挝文                   |
| `Latin`                  | 拉丁字母                 |
| `Lepcha`                 | 雷布查文                 |
| `Limbu`                  | 林布文                   |
| `Linear_A`               | 线性文字A                |
| `Linear_B`               | 线性文字B                |
| `Lisu`                   | 傈僳文                   |
| `Lycian`                 | 吕基亚文                 |
| `Lydian`                 | 吕底亚文                 |
| `Mahajani`               | 马哈贾尼文               |
| `Makasar`                | 望加锡文                 |
| `Mandaic`                | 曼达文                   |
| `Manichaean`             | 摩尼文                   |
| `Marchen`                | 玛钦文                   |
| `Masaram_Gondi`          | 马萨拉姆贡迪文           |
| `Medefaidrin`            | 梅德法伊德林文           |
| `Mende_Kikakui`          | 门德基卡库伊文           |
| `Nyiakeng_Puachue_Hmong` | 尼亚肯普阿丘赫蒙文       |
| `Old_Hungarian`          | 古匈牙利文               |
| `Old_Italic`             | 古意大利文               |
| `Pahawh_Hmong`           | 帕哈赫蒙文               |
| `Tai_Tham`               | 傣坦文                   |

- 二进制属性

| 属性名                               | 简短别名  | 说明                                  |
| ------------------------------------ | --------- | ------------------------------------- |
| `Alphabetic`                         | —         | 字母及字母类字符                      |
| `Alnum`                              | —         | Alphabetic + 十进制数字（POSIX 风格） |
| `ASCII_Hex_Digit`                    | `AHex`    | ASCII 十六进制数字（0-9, A-F, a-f）   |
| `Bidi_Control`                       | `Bidi_C`  | 双向文本控制字符                      |
| `Bidi_Mirrored`                      | `Bidi_M`  | 双向镜像字符                          |
| `Blank`                              | —         | 空格或水平制表符                      |
| `Cased`                              | —         | 有大写/小写/标题大小写形式的字符      |
| `Dash`                               | —         | 破折号类字符                          |
| `Default_Ignorable_Code_Point`       | `DI`      | 默认可忽略的码点                      |
| `Deprecated`                         | `Dep`     | 已弃用的字符                          |
| `Diacritic`                          | `Dia`     | 变音符号                              |
| `Emoji`                              | —         | Emoji 字符                            |
| `Emoji_Component`                    | —         | Emoji 组成元素                        |
| `Emoji_Modifier`                     | —         | Emoji 肤色修饰符                      |
| `Emoji_Modifier_Base`                | —         | 可被肤色修饰的 Emoji 基础字符         |
| `Emoji_Presentation`                 | —         | 默认以 Emoji 形式展示的字符           |
| `Extended_Pictographic`              | —         | 扩展象形文字                          |
| `Full_Composition_Exclusion`         | —         | 完全合成排除                          |
| `Grapheme_Base`                      | `Gr_Base` | 字形基础字符                          |
| `Grapheme_Extend`                    | `Gr_Ext`  | 字形扩展字符                          |
| `Hex_Digit`                          | `Hex`     | 十六进制数字（含全角）                |
| `Hyphen`                             | —         | 连字符                                |
| `ID_Continue`                        | `IDC`     | 标识符继续字符                        |
| `ID_Start`                           | `IDS`     | 标识符起始字符                        |
| `Ideographic`                        | `Ideo`    | 表意文字字符                          |
| `Join_Control`                       | `Join_C`  | 连接控制字符                          |
| `Logical_Order_Exception`            | `LOE`     | 逻辑顺序例外                          |
| `Lowercase`                          | `Lower`   | 小写字符                              |
| `Math`                               | —         | 数学符号字符                          |
| `Noncharacter_Code_Point`            | `NChar`   | 非字符码点                            |
| `Other_Alphabetic`                   | `OAlpha`  | 其他字母类字符                        |
| `Other_Default_Ignorable_Code_Point` | `ODI`     | 其他默认可忽略码点                    |
| `Other_Grapheme_Extend`              | `OGr_Ext` | 其他字形扩展字符                      |
| `Other_ID_Continue`                  | `OIDC`    | 其他标识符继续字符                    |
| `Other_ID_Start`                     | `OIDS`    | 其他标识符起始字符                    |
| `Other_Lowercase`                    | `OLower`  | 其他小写字符                          |
| `Other_Math`                         | `OMath`   | 其他数学符号字符                      |
| `Other_Uppercase`                    | `OUpper`  | 其他大写字符                          |
| `Pattern_Syntax`                     | `Pat_Syn` | 模式语法字符                          |
| `Pattern_White_Space`                | `Pat_WS`  | 模式空白字符                          |
| `Quotation_Mark`                     | `QMark`   | 引号字符                              |
| `Radical`                            | —         | 部首字符                              |
| `Regional_Indicator`                 | `RI`      | 区域指示符                            |
| `Sentence_Terminal`                  | `STerm`   | 句子终止符                            |
| `Soft_Dotted`                        | `SD`      | 软点字符                              |
| `Terminal_Punctuation`               | `Term`    | 终止标点                              |
| `Unified_Ideograph`                  | `UIdeo`   | 统一表意文字                          |
| `Uppercase`                          | `Upper`   | 大写字符                              |
| `Variation_Selector`                 | `VS`      | 变体选择符                            |
| `White_Space`                        | `WSpace`  | 空白字符                              |
| `XID_Continue`                       | `XIDC`    | 扩展标识符继续字符                    |
| `XID_Start`                          | `XIDS`    | 扩展标识符起始字符                    |

---

### 模式标志参数

| 标识符 | 说明             |
| ------ | ---------------- |
| `i`    | 大小写不敏感匹配 |
| `m`    | 多行模式         |
| `s`    | DOTALL 模式      |
| `R`    | CRLFF 模式       |
| `U`    | 默认懒惰模式     |
| `u`    | Unicode 模式     |
| `x`    | 扩展模式         |
