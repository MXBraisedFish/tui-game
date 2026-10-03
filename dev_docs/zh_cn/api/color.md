# color 库

`color` 提供基础颜色与标准颜色字符串构造。

---

# 目录

## 常量

| 常量               | 说明                     | 定位                                |
| ---------------- | ---------------------- | --------------------------------- |
| `GRAY`           | 灰色                     | [GRAY](#gray)                     |
| `BRIGHT_GRAY`    | 亮灰色                    | [BRIGHT_GRAY](#bright_gray)       |
| `BLACK`          | 黑色                     | [BLACK](#black)                   |
| `RED`            | 红色                     | [RED](#red)                       |
| `GREEN`          | 绿色                     | [GREEN](#green)                   |
| `YELLOW`         | 黄色                     | [YELLOW](#yellow)                 |
| `BLUE`           | 蓝色                     | [BLUE](#blue)                     |
| `MAGENTA`        | 品红                     | [MAGENTA](#magenta)               |
| `CYAN`           | 青色                     | [CYAN](#cyan)                     |
| `BRIGHT_RED`     | 亮红                     | [BRIGHT_RED](#bright_red)         |
| `BRIGHT_GREEN`   | 亮绿                     | [BRIGHT_GREEN](#bright_green)     |
| `BRIGHT_YELLOW`  | 亮黄                     | [BRIGHT_YELLOW](#bright_yellow)   |
| `BRIGHT_BLUE`    | 亮蓝                     | [BRIGHT_BLUE](#bright_blue)       |
| `BRIGHT_MAGENTA` | 亮品红                    | [BRIGHT_MAGENTA](#bright_magenta) |
| `BRIGHT_CYAN`    | 亮青                     | [BRIGHT_CYAN](#bright_cyan)       |
| `WHITE`          | 白色                     | [WHITE](#white)                   |
| `NONE`           | 默认颜色                   | [NONE](#none)                     |
| `TRANSPARENT`    | 透明背景                   | [TRANSPARENT](#transparent)       |
| `GREY`           | 灰色，GRAY 的另一种拼写         | [GREY](#grey)                     |
| `BRIGHT_GREY`    | 亮灰色，BRIGHT_GRAY 的另一种拼写 | [BRIGHT_GREY](#bright_grey)       |

## 方法

| 方法 | 说明 | 定位 |
| --- | --- | --- |
| `rgb` | 根据 RGB 分量构造颜色字符串 `rgb(r,g,b)` | [rgb](#rgb) |
| `hex` | 根据 RGB 分量构造十六进制颜色字符串 `#rrggbb` | [hex](#hex) |


---

# 常量

## `BLACK`

黑色。

### 调用

```lua
color.BLACK
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.WHITE})
draw.text(3, 1, "FG", {fg = color.BLACK, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BLACK})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"black"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `RED`

红色。

### 调用

```lua
color.RED
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.RED, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.RED})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"red"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `GREEN`

绿色。

### 调用

```lua
color.GREEN
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.GREEN, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.GREEN})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"green"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `YELLOW`

黄色。

### 调用

```lua
color.YELLOW
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.YELLOW, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.YELLOW})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"yellow"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BLUE`

蓝色。

### 调用

```lua
color.BLUE
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BLUE, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BLUE})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"blue"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `MAGENTA`

品红。

### 调用

```lua
color.MAGENTA
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.MAGENTA, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.MAGENTA})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"magenta"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `CYAN`

青色。

### 调用

```lua
color.CYAN
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.CYAN, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.CYAN})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"cyan"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `GRAY`

灰色。

### 调用

```lua
color.GRAY
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.GRAY, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.GREY})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"gray"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_GRAY`

亮灰。

### 调用

```lua
color.BRIGHT_GRAY
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_GRAY, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_GREY})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_gray"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_RED`

亮红。

### 调用

```lua
color.BRIGHT_RED
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_RED, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_RED})
draw.text(3, 4, "BG", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_red"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_GREEN`

亮绿。

### 调用

```lua
color.BRIGHT_GREEN
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_GREEN, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_GREEN})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_green"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_YELLOW`

亮黄。

### 调用

```lua
color.BRIGHT_YELLOW
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_YELLOW, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_YELLOW})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_yellow"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_BLUE`

亮蓝。

### 调用

```lua
color.BRIGHT_BLUE
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_BLUE, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_BLUE})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_blue"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_MAGENTA`

亮品红。

### 调用

```lua
color.BRIGHT_MAGENTA
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_MAGENTA, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_MAGENTA})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_magenta"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `BRIGHT_CYAN`

亮青。

### 调用

```lua
color.BRIGHT_CYAN
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.BRIGHT_CYAN, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.BRIGHT_CYAN})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"bright_cyan"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `WHITE`

白色。

### 调用

```lua
color.WHITE
```

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 示例

```lua
draw.fill_rect(0, 0, 8, 3, {bg = color.NONE})
draw.text(3, 1, "FG", {fg = color.WHITE, bg = color.TRANSPARENT})

draw.fill_rect(0, 3, 8, 3, {bg = color.WHITE})
draw.text(3, 4, "BG", {fg = color.BLACK, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"white"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `NONE`

默认颜色。

### 调用

```lua
color.NONE
```

### 可用于

- 参数 `fg`
- 参数 `bg`

### 示例

```lua
draw.text(3, 1, "NONE", {fg = color.NONE, bg = color.NONE})
```

**输出：**

```lua
```

### 等值

```text
"none"
```

### 额外说明

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `TRANSPARENT`

透明背景。

### 调用

```lua
color.TRANSPARENT
```

### 可用于

- 参数 `bg`

### 示例

```lua
draw.fill_rect(0, 0, 8, 6, {bg = color.RED})

draw.text(2, 1, "NONE", {fg = color.WHITE, bg = color.NONE})

draw.text(2, 4, "TRAN", {fg = color.WHITE, bg = color.TRANSPARENT})
```

**输出：**

```lua
```

### 等值

```text
"transparent"
```

---

## `GREY`

灰色，GRAY 的另一种拼写。

### 调用

```lua
color.GREY
```

### 可用于

- 任意

### 示例

```lua
debug.print(color.GREY == color.GRAY)
```

**输出：**

```lua
```

### 等值

```text
"gray"
```

### 额外说明

- 与 `color.GRAY` 的值相同。

---

## `BRIGHT_GREY`

亮灰色，BRIGHT_GRAY 的另一种拼写。

### 调用

```lua
color.BRIGHT_GREY
```

### 可用于

- 任意

### 示例

```lua
debug.print(color.BRIGHT_GREY == color.BRIGHT_GRAY)
```

**输出：**

```lua
```

### 等值

```text
"bright_gray"
```

### 额外说明

- 与 `color.BRIGHT_GRAY` 的值相同。

---

# 方法

## `rgb`

根据 RGB 分量构造颜色字符串 `rgb(r,g,b)`。

### 调用

```lua
color.rgb
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `r` | integer | 红色分量 |
| `g` | integer | 绿色分量 |
| `b` | integer | 蓝色分量 |

## 返回值

直接返回一个值。

| 类型   | 说明                  |
| ------ | --------------------- |
| string | 形如 `"rgb(255,0,0)"` |

### 示例

```lua
local rgb = color.rgb(123, 128, 200)
draw.text(0, 0, rgb, {fg = rgb})
```

**输出：**

```lua
```

---

## `hex`

根据 RGB 分量构造十六进制颜色字符串 `#rrggbb`。

### 调用

```lua
color.hex
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `r` | integer | 红色分量 |
| `g` | integer | 绿色分量 |
| `b` | integer | 蓝色分量 |

## 返回值

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| string | 形如 `"#ff0000"` |

### 示例

```lua
local hex = color.hex(176, 238, 222)
draw.text(0, 0, hex, {fg = hex})
```

**输出：**

```lua
```
