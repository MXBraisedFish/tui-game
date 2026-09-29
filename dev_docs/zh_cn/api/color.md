# color 库

`color` 提供基础颜色与标准颜色字符串构造。

---

# 目录

## 常量

| 常量                        | 说明     | 定位                              |
| ----------------------------- | -------- | --------------------------------- |
| `BLACK`                       | 黑色     | [BLACK](#BLACK)                   |
| `RED`                         | 红色     | [RED](#RED)                       |
| `GREEN`                       | 绿色     | [GREEN](#GREEN)                   |
| `YELLOW`                      | 黄色     | [YELLOW](#YELLOW)                 |
| `BLUE`                        | 蓝色     | [BLUE](#BLUE)                     |
| `MAGENTA`                     | 品红     | [MAGENTA](#MAGENTA)               |
| `CYAN`                        | 青色     | [CYAN](#CYAN)                     |
| `GRAY` / `GREY`               | 灰色     | [GRAY](#GRAY)                     |
| `BRIGHT_GRAY` / `BRIGHT_GREY` | 亮灰     | [BRIGHT_GRAY](#BRIGHT_GRAY)       |
| `BRIGHT_RED`                  | 亮红     | [BRIGHT_RED](#BRIGHT_RED)         |
| `BRIGHT_GREEN`                | 亮绿     | [BRIGHT_GREEN](#BRIGHT_GREEN)     |
| `BRIGHT_YELLOW`               | 亮黄     | [BRIGHT_YELLOW](#BRIGHT_YELLOW)   |
| `BRIGHT_BLUE`                 | 亮蓝     | [BRIGHT_BLUE](#BRIGHT_BLUE)       |
| `BRIGHT_MAGENTA`              | 亮品红   | [BRIGHT_MAGENTA](#BRIGHT_MAGENTA) |
| `BRIGHT_CYAN`                 | 亮青     | [BRIGHT_CYAN](#BRIGHT_CYAN)       |
| `WHITE`                       | 白色     | [WHITE](#WHITE)                   |
| `NONE`                        | 默认颜色 | [NONE](#NONE)                     |
| `TRANSPARENT`                 | 透明背景 | [TRANSPARENT](#TRANSPARENT)       |

## 方法

| 方法 | 说明                                          | 定位        |
| ------ | --------------------------------------------- | ----------- |
| `rgb`  | 根据 RGB 分量构造颜色字符串 `rgb(r,g,b)`      | [rgb](#rgb) |
| `hex`  | 根据 RGB 分量构造十六进制颜色字符串 `#rrggbb` | [hex](#hex) |

---

# 常量

## `BLACK`

黑色。

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BLACK
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.RED
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.GREEN
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.YELLOW
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BLUE
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.MAGENTA
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.CYAN
```

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

## `GRAY` / `GREY` {#GRAY}

灰色。

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.GRAY
```

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

## `BRIGHT_GRAY` / `BRIGHT_GREY` {#BRIGHT_GRAY}

亮灰。

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_GRAY
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_RED
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_GREEN
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_YELLOW
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_BLUE
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_MAGENTA
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_CYAN
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.WHITE
```

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

### 可用于

- 参数 `fg`
- 参数 `bg`

### 调用

```lua
color.NONE
```

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

### 可用于

- 参数 `bg`

### 调用

```lua
color.TRANSPARENT
```

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
