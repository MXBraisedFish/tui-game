# color 库

## 基本库说明

`color` 提供基础颜色与标准颜色字符串构造。

---

## 目录

### 常量

| 常量名                        | 说明     | 索引                              |
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

### 方法

| 方法名 | 说明                                          | 索引        |
| ------ | --------------------------------------------- | ----------- |
| `rgb`  | 根据 RGB 分量构造颜色字符串 `rgb(r,g,b)`      | [rgb](#rgb) |
| `hex`  | 根据 RGB 分量构造十六进制颜色字符串 `#rrggbb` | [hex](#hex) |

---

## 常量

## `BLACK`

黑色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BLACK
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.WHITE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BLACK, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BLACK }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.BLACK示例](../image/color_BLACK_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#000000
  rgb(0,0,0)

---

## `RED`

红色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.RED
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.RED, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.RED }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.RED示例](../image/color_RED_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#cc0000
  rgb(204,0,0)

---

## `GREEN`

绿色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.GREEN
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.GREEN, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.GREEN }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.GREEN示例](../image/color_GREEN_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#4e9a06
  rgb(78,154,6)

---

## `YELLOW`

黄色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.YELLOW
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.YELLOW, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.YELLOW }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.YELLOW示例](../image/color_YELLOW_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#c4a000
  rgb(196,160,0)

---

## `BLUE`

蓝色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BLUE
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BLUE, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BLUE }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.BLUE示例](../image/color_BLUE_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#3465a4
  rgb(52,101,164)

---

## `MAGENTA`

品红。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.MAGENTA
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.MAGENTA, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.MAGENTA }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.MAGENTA示例](../image/color_MAGENTA_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#75507b
  rgb(117,80,123)

---

## `CYAN`

青色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.CYAN
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.CYAN, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.CYAN }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.CYAN示例](../image/color_CYAN_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#06989a
  rgb(6,152,154)

---

## `GRAY` / `GREY` {#GRAY}

灰色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.GRAY
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.GRAY, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.GREY }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.GRAY示例](../image/color_GRAY_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#d3d7cf
  rgb(211,215,207)

---

## `BRIGHT_GRAY` / `BRIGHT_GREY` {#BRIGHT_GRAY}

亮灰。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_GRAY
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_GRAY, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_GREY }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_GRAY示例](../image/color_BRIGHT_GRAY_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#eeeeec
  rgb(238,238,236)

---

## `BRIGHT_RED`

亮红。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_RED
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_RED, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_RED }
draw.text { x = 3, y = 4, text = "BG", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_RED示例](../image/color_BRIGHT_RED_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#ef2929
  rgb(239,41,41)

---

## `BRIGHT_GREEN`

亮绿。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_GREEN
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_GREEN, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_GREEN }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_GREEN示例](../image/color_BRIGHT_GREEN_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#8ae234
  rgb(138,226,52)

---

## `BRIGHT_YELLOW`

亮黄。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_YELLOW
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_YELLOW, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_YELLOW }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_YELLOW示例](../image/color_BRIGHT_YELLOW_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#fce94f
  rgb(252,233,79)

---

## `BRIGHT_BLUE`

亮蓝。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_BLUE
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_BLUE, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_BLUE }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_BLUE示例](../image/color_BRIGHT_BLUE_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#729fcf
  rgb(114,159,207)

---

## `BRIGHT_MAGENTA`

亮品红。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_MAGENTA
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_MAGENTA, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_MAGENTA }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_MAGENTA示例](../image/color_BRIGHT_MAGENTA_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#ad7fa8
  rgb(173,127,168)

---

## `BRIGHT_CYAN`

亮青。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.BRIGHT_CYAN
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.BRIGHT_CYAN, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.BRIGHT_CYAN }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.BRIGHT_CYAN示例](../image/color_BRIGHT_CYAN_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#34e2e2
  rgb(52,226,226)

---

## `WHITE`

白色。

**可用于**

- 参数 `fg`
- 参数 `bg`
- 富文本标签

### 调用

```lua
color.WHITE
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 3, bg = color.NONE }
draw.text { x = 3, y = 1, text = "FG", fg = color.WHITE, bg = color.TRANSPARENT }

draw.fill_rect { x = 0, y = 3, width = 8, height = 3, bg = color.WHITE }
draw.text { x = 3, y = 4, text = "BG", fg = color.BLACK, bg = color.TRANSPARENT }
```

输出：

![color.WHITE示例](../image/color_WHITE_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同
- 示例图实际色号：
  \#eeeeec
  rgb(238,238,236)

---

## `NONE`

默认颜色。

**可用于**

- 参数 `fg`
- 参数 `bg`

### 调用

```lua
color.NONE
```

### 示例

```lua
draw.text { x = 3, y = 1, text = "NONE", fg = color.NONE, bg = color.NONE }
```

输出：

![color.NONE示例](../image/color_NONE_example.png)

### 额外补充

- 该参数为相对颜色，实际显示根据每个人的终端设置而不同

---

## `TRANSPARENT`

透明背景。

**可用于**

- 参数 `bg`

### 调用

```lua
color.TRANSPARENT
```

### 示例

```lua
draw.fill_rect { x = 0, y = 0, width = 8, height = 6, bg = color.RED }

draw.text { x = 2, y = 1, text = "NONE", fg = color.WHITE, bg = color.NONE }

draw.text { x = 2, y = 4, text = "TRAN", fg = color.WHITE, bg = color.TRANSPARENT }
```

输出：

![color.TRANSPARENT示例](../image/color_TRANSPARENT_example.png)

---

## 方法

## `rgb`

根据 RGB 分量构造颜色字符串 `rgb(r,g,b)`。

### 调用

```lua
-- 单参数
color.rgb()
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明     |
| ------ | ------- | ---- | ------ | -------- |
| `r`    | integer | 是   | -      | 红色分量 |
| `g`    | integer | 是   | -      | 绿色分量 |
| `b`    | integer | 是   | -      | 蓝色分量 |

### 返回

直接返回一个值。

| 类型   | 说明                  |
| ------ | --------------------- |
| string | 形如 `"rgb(255,0,0)"` |

### 示例

```lua
local rgb = color.rgb { r = 123, g = 128, b = 200 }
draw.text { x = 0, y = 0, text = rgb, fg = rgb }
```

输出：

![color.rgb示例](../image/color_rgb_example.png)

---

## `hex`

根据 RGB 分量构造十六进制颜色字符串 `#rrggbb`。

### 调用

```lua
-- 单参数
color.hex()
```

### 参数

| 参数名 | 类型    | 必填 | 默认值 | 说明     |
| ------ | ------- | ---- | ------ | -------- |
| `r`    | integer | 是   | -      | 红色分量 |
| `g`    | integer | 是   | -      | 绿色分量 |
| `b`    | integer | 是   | -      | 蓝色分量 |

### 返回

直接返回一个值。

| 类型   | 说明             |
| ------ | ---------------- |
| string | 形如 `"#ff0000"` |

### 示例

```lua
local hex = color.hex { r = 176, g = 238, b = 222 }
draw.text { x = 0, y = 0, text = hex, fg = hex }
```

输出：

![color.hex示例](../image/color_hex_example.png)
