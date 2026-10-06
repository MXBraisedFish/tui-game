# char 库

`char` 提供常用的字符集常量。

---

# 目录

## 常量

| 常量              | 说明                        | 定位                                |
| ----------------- | --------------------------- | ----------------------------------- |
| `LINE`            | 单线边框字符表              | [LINE](#line)                       |
| `BOLD_LINE`       | 粗线边框字符表              | [BOLD_LINE](#bold_line)             |
| `DOUBLE_LINE`     | 双线边框字符表              | [DOUBLE_LINE](#double_line)         |
| `ROUNDED_LINE`    | 圆角线边框字符表            | [ROUNDED_LINE](#rounded_line)       |
| `ASCII_NUMBER`    | 数字字符数组表              | [ASCII_NUMBER](#ascii_number)       |
| `ASCII_LOWERCASE` | 小写字母字符数组表          | [ASCII_LOWERCASE](#ascii_lowercase) |
| `ASCII_UPPERCASE` | 大写字母字符数组表          | [ASCII_UPPERCASE](#ascii_uppercase) |
| `ASCII_LETTER`    | 全部大小写字母字符数组表    | [ASCII_LETTER](#ascii_letter)       |
| `ASCII_CHARACTER` | ASCII 符号字符数组表        | [ASCII_CHARACTER](#ascii_character) |
| `ASCII`           | 全量可打印 ASCII 字符数组表 | [ASCII](#ascii)                     |

---

# 常量

## `LINE`

单线边框字符表。

### 调用

```lua
char.LINE
```

### 可用于

- 参数 `border_char`

### 示例

```lua
draw.stroke_rect(15, 1, 12, 5, {border_char = char.LINE})
```

**输出：**

![char_LINE_example](../image/char_LINE_example.png)

### 等值

```lua
{
  top = "─",
  left_top = "┌",
  left = "│",
  left_bottom = "└",
  bottom = "─",
  right_bottom = "┘",
  right = "│",
  right_top = "┐",
  t_left = "├",
  t_bottom = "┴",
  t_right = "┤",
  t_top = "┬",
  center = "┼",
}
```

---

## `BOLD_LINE`

粗线边框字符表。

### 调用

```lua
char.BOLD_LINE
```

### 可用于

- 参数 `border_char`

### 示例

```lua
draw.stroke_rect(15, 1, 12, 5, {border_char = char.BOLD_LINE})
```

**输出：**

![char_BOLD_LINE_example](../image/char_BOLD_LINE_example.png)

### 等值

```lua
{
  top = "━",
  left_top = "┏",
  left = "┃",
  left_bottom = "┗",
  bottom = "━",
  right_bottom = "┛",
  right = "┃",
  right_top = "┓",
  t_left = "┣",
  t_bottom = "┻",
  t_right = "┫",
  t_top = "┳",
  center = "╋",
}
```

---

## `DOUBLE_LINE`

双线边框字符表。

### 调用

```lua
char.DOUBLE_LINE
```

### 可用于

- 参数 `border_char`

### 示例

```lua
draw.stroke_rect(15, 1, 12, 5, {border_char = char.DOUBLE_LINE})
```

**输出：**

![char_DOUBLE_LINE_example](../image/char_DOUBLE_LINE_example.png)

### 等值

```lua
{
  top = "═",
  left_top = "╔",
  left = "║",
  left_bottom = "╚",
  bottom = "═",
  right_bottom = "╝",
  right = "║",
  right_top = "╗",
  t_left = "╠",
  t_bottom = "╩",
  t_right = "╣",
  t_top = "╦",
  center = "╬",
}
```

---

## `ROUNDED_LINE`

圆角线边框字符表。

### 调用

```lua
char.ROUNDED_LINE
```

### 可用于

- 参数 `border_char`

### 示例

```lua
draw.stroke_rect(15, 1, 12, 5, {border_char = char.ROUNDED_LINE})
```

**输出：**

![char_ROUNDED_LINE_example](../image/char_ROUNDED_LINE_example.png)

### 等值

```lua
{
  top = "─",
  left_top = "╭",
  left = "│",
  left_bottom = "╰",
  bottom = "─",
  right_bottom = "╯",
  right = "│",
  right_top = "╮",
  t_left = "├",
  t_bottom = "┴",
  t_right = "┤",
  t_top = "┬",
  center = "┼",
}
```

---

## `ASCII_NUMBER`

数字字符数组表。

### 调用

```lua
char.ASCII_NUMBER
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII_NUMBER) do
  x = x + 2
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_NUMBER_example](../image/char_ASCII_NUMBER_example.png)

### 等值

```lua
{
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
}
```

## 额外说明

- 所有数字均为**字符串**类型，而非数字。

---

## `ASCII_LOWERCASE`

小写字母字符数组表。

### 调用

```lua
char.ASCII_LOWERCASE
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII_LOWERCASE) do
  x = x + 2
  if x >= 20 then
    x = 2
    y = y + 1
  end
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_LOWERCASE_example](../image/char_ASCII_LOWERCASE_example.png)

### 等值

```lua
{
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
}
```

---

## `ASCII_UPPERCASE`

大写字母字符数组表。

### 调用

```lua
char.ASCII_UPPERCASE
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII_UPPERCASE) do
  x = x + 2
  if x >= 20 then
    x = 2
    y = y + 1
  end
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_UPPERCASE_example](../image/char_ASCII_UPPERCASE_example.png)

### 等值

```lua
{
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
}
```

---

## `ASCII_LETTER`

全部大小写字母字符数组表。

### 调用

```lua
char.ASCII_LETTER
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII_LETTER) do
  x = x + 2
  if x >= 20 then
    x = 2
    y = y + 1
  end
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_LETTER_example](../image/char_ASCII_LETTER_example.png)

### 等值

```lua
{
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
}
```

---

## `ASCII_CHARACTER`

ASCII 符号字符数组表。

### 调用

```lua
char.ASCII_CHARACTER
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII_CHARACTER) do
  x = x + 2
  if x >= 20 then
    x = 2
    y = y + 1
  end
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_CHARACTER_example](../image/char_ASCII_CHARACTER_example.png)

### 等值

```lua
{
  "!",
  "\"",
  "#",
  "$",
  "%",
  "&",
  "'",
  "(",
  ")",
  "*",
  "+",
  ",",
  "-",
  ".",
  "/",
  ":",
  ";",
  "<",
  "=",
  ">",
  "?",
  "@",
  "[",
  "\\",
  "]",
  "^",
  "_",
  "`",
  "{",
  "|",
  "}",
  "~",
}
```

---

## `ASCII`

全量可打印 ASCII 字符数组表。

### 调用

```lua
char.ASCII
```

### 可用于

- 任意。

### 示例

```lua
local x, y = 0, 0

for i, item in ipairs(char.ASCII) do
  x = x + 2
  if x >= 20 then
    x = 2
    y = y + 1
  end
  draw.text(x, y, item)
end
```

**输出：**

![char_ASCII_example](../image/char_ASCII_example.png)

### 等值

```lua
{
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
  "!",
  "\"",
  "#",
  "$",
  "%",
  "&",
  "'",
  "(",
  ")",
  "*",
  "+",
  ",",
  "-",
  ".",
  "/",
  ":",
  ";",
  "<",
  "=",
  ">",
  "?",
  "@",
  "[",
  "\\",
  "]",
  "^",
  "_",
  "`",
  "{",
  "|",
  "}",
  "~",
}
```
