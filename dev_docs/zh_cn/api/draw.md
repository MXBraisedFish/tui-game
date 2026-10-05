# draw 库

`draw` 提供终端画布绘制指令。

---

# 目录

## 方法

| 方法          | 说明                       | 定位                        |
| ------------- | -------------------------- | --------------------------- |
| `text`        | 在指定位置绘制文本         | [text](#text)               |
| `fill_rect`   | 填充一个矩形区域           | [fill_rect](#fill_rect)     |
| `stroke_rect` | 绘制一个矩形边框           | [stroke_rect](#stroke_rect) |
| `erase_rect`  | 擦除指定矩形区域           | [erase_rect](#erase_rect)   |
| `render`      | 请求执行一次 `Render` 回调 | [render](#render)           |

---

# 方法

## `text`

在指定位置绘制文本。

### 调用

```lua
draw.text
```

## 参数

### 必填参数

| 参数名 | 类型    | 说明                  |
| ------ | ------- | --------------------- |
| `x`    | integer | 文本块左上角的 x 坐标 |
| `y`    | integer | 文本块左上角的 y 坐标 |
| `text` | string  | 要绘制的文本          |

### 选填参数

| 参数名             | 类型                 | 默认值        | 说明                                |
| ------------------ | -------------------- | ------------- | ----------------------------------- |
| `fg`               | string / const-color | `color.NONE`  | 前景色                              |
| `bg`               | string / const-color | `color.NONE`  | 背景色                              |
| `horizontal_align` | const-align          | `align.LEFT`  | 各行相对于最长显示行的水平对齐方式  |
| `auto_wrap`        | boolean              | `true`        | 是否自动换行                        |
| `word_wrap`        | boolean              | `true`        | 是否按完整单词换行                  |
| `max_width`        | integer / nil        | `nil`         | 最大绘制宽度 |
| `max_height`       | integer / nil        | `nil`         | 最大绘制高度 |
| `overflow_marker`  | string               | `"..."`       | 文本溢出时使用的省略标记            |
| `text_mode`        | const-string         | `string.AUTO` | 文本解析模式                        |
| `rich_params`      | table / nil          | `nil`         | 富文本参数                          |
| `bold`             | boolean              | `false`       | 粗体                                |
| `italic`           | boolean              | `false`       | 斜体                                |
| `underline`        | boolean              | `false`       | 下划线                              |
| `strike`           | boolean              | `false`       | 删除线                              |
| `blink`            | boolean              | `false`       | 闪烁                                |
| `reverse`          | boolean              | `false`       | 反显                                |
| `hidden`           | boolean              | `false`       | 隐藏                                |
| `dim`              | boolean              | `false`       | 暗淡                                |
| `slice_layer`      | string               | `"base"`      | 绘制目标切片图层                    |

## 返回值

无返回值。

### 示例

```lua
draw.text(2, 1, "Hello TUI GAME", {fg = color.BRIGHT_RED})
draw.text(2, 2, "Hello TUI GAME", {fg = color.WHITE, italic = true})
```

**输出：**

![draw_text_example](../image/draw_text_example.png)

## 额外说明

- 坐标原点为 $(0,0)$。

---

## `fill_rect`

填充一个矩形区域。

### 调用

```lua
draw.fill_rect
```

## 参数

### 必填参数

| 参数名   | 类型    | 说明                    |
| -------- | ------- | ----------------------- |
| `x`      | integer | 矩形左上角的 x 坐标     |
| `y`      | integer | 矩形左上角的 y 坐标     |
| `width`  | integer | 矩形宽度 |
| `height` | integer | 矩形高度 |

### 选填参数

| 参数名        | 类型                 | 默认值       | 说明             |
| ------------- | -------------------- | ------------ | ---------------- |
| `char`        | string / nil         | `nil`        | 填充字符         |
| `fg`          | string / const-color | `color.NONE` | 前景色           |
| `bg`          | string / const-color | `color.NONE` | 背景色           |
| `slice_layer` | string               | `"base"`     | 绘制目标切片图层 |

## 返回值

无返回值。

### 示例

```lua
draw.fill_rect(2, 1, 10, 4, {bg = color.BLUE})

draw.fill_rect(13, 1, 10, 4, {char = "-", fg = color.GREEN})
```

**输出：**

![draw_fill_rect_example](../image/draw_fill_rect_example.png)

## 额外说明

- 选填参数 `char` 填写时必须为宽度为 **1** 的字符。
- 坐标原点为 $(0,0)$。

---

## `stroke_rect`

绘制一个矩形边框。

### 调用

```lua
draw.stroke_rect
```

## 参数

### 必填参数

| 参数名   | 类型    | 说明                    |
| -------- | ------- | ----------------------- |
| `x`      | integer | 矩形左上角的 x 坐标     |
| `y`      | integer | 矩形左上角的 y 坐标     |
| `width`  | integer | 矩形宽度 |
| `height` | integer | 矩形高度 |

### 选填参数

| 参数名        | 类型                 | 默认值       | 说明             |
| ------------- | -------------------- | ------------ | ---------------- |
| `fg`          | string / const-color | `color.NONE` | 边框前景色       |
| `bg`          | string / const-color | `color.NONE` | 边框背景色       |
| `border_char` | const-char / table   | `char.LINE`  | 边框字符         |
| `slice_layer` | string               | `"base"`     | 绘制目标切片图层 |

## 返回值

无返回值。

### 示例

```lua
draw.stroke_rect(2, 1, 12, 5, {fg = color.WHITE, border_char = char.ROUNDED_LINE})

draw.stroke_rect(15, 1, 12, 5, {fg = color.YELLOW, border_char = {
		top = "-",
		left_top = "+",
		left = "|",
		left_bottom = "+",
		bottom = "-",
		right_bottom = "+",
		right = "|",
		right_top = "+",
	}})
```

**输出：**

![draw_stroke_rect_example](../image/draw_stroke_rect_example.png)

## 额外说明

- 参数 `border_char` 表结构：

```lua
local border_char = {
  top = "─",          -- string / const-char
  left_top = "┌",     -- string / const-char
  left = "│",         -- string / const-char
  left_bottom = "└",  -- string / const-char
  bottom = "─",       -- string / const-char
  right_bottom = "┘", -- string / const-char
  right = "│",        -- string / const-char
  right_top = "┐",    -- string / const-char
}
```

- 选填参数 `border_char` 填写时每个字段必须为宽度为 **1** 的字符。
- 坐标原点为 $(0,0)$。

---

## `erase_rect`

擦除指定矩形区域。

### 调用

```lua
draw.erase_rect
```

## 参数

### 必填参数

| 参数名   | 类型    | 说明                    |
| -------- | ------- | ----------------------- |
| `x`      | integer | 矩形左上角的 x 坐标     |
| `y`      | integer | 矩形左上角的 y 坐标     |
| `width`  | integer | 矩形宽度 |
| `height` | integer | 矩形高度 |

### 选填参数

| 参数名        | 类型   | 默认值   | 说明             |
| ------------- | ------ | -------- | ---------------- |
| `slice_layer` | string | `"base"` | 绘制目标切片图层 |

## 返回值

无返回值。

### 示例

```lua
draw.fill_rect(2, 1, 10, 4, {bg = color.BLUE})

draw.erase_rect(3, 2, 8, 2)
```

**输出：**

![draw_erase_rect_example](../image/draw_erase_rect_example.png)

---

## `render`

请求执行一次 `Render` 回调。

### 调用

```lua
draw.render
```

## 返回值

无返回值。

### 示例

```lua
function HandleEvent(event)
	draw.render()
end

function Render()
	-- 绘制逻辑
end
```
## 额外说明

- **不可**在 `Render` 回调中调用。
