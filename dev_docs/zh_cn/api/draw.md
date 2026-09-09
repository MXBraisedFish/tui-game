# draw 库

## 基本库说明

`draw` 提供终端画布绘制指令。

---

## 目录

### 方法

| 方法名        | 说明                       | 索引                        |
| ------------- | -------------------------- | --------------------------- |
| `text`        | 在指定位置绘制文本         | [text](#text)               |
| `fill_rect`   | 填充一个矩形区域           | [fill_rect](#fill_rect)     |
| `stroke_rect` | 绘制一个矩形边框           | [stroke_rect](#stroke_rect) |
| `erase_rect`  | 擦除指定矩形区域           | [erase_rect](#erase_rect)   |
| `render`      | 请求执行一次 `Render` 回调 | [render](#render)           |

---

## 方法

## `text`

在指定位置绘制文本。

### 调用

```lua
-- 表参数
draw.text{}
```

### 参数

| 参数               | 类型                 | 必填 | 默认值        | 说明                     |
| ------------------ | -------------------- | ---- | ------------- | ------------------------ |
| `x`                | integer              | 是   | -             | 文本起始位置的 x 坐标    |
| `y`                | integer              | 是   | -             | 文本起始位置的 y 坐标    |
| `text`             | string               | 是   | -             | 要绘制的文本             |
| `fg`               | string / const-color | 否   | `color.NONE`  | 前景色                   |
| `bg`               | string / const-color | 否   | `color.NONE`  | 背景色                   |
| `horizontal_align` | const-align          | 否   | `align.LEFT`  | 多行文本的水平对齐方式   |
| `auto_wrap`        | boolean              | 否   | `true`        | 是否自动换行             |
| `word_wrap`        | boolean              | 否   | `true`        | 是否按完整单词换行       |
| `max_width`        | integer / nil        | 否   | `nil`         | 最大绘制宽度             |
| `max_height`       | integer / nil        | 否   | `nil`         | 最大绘制高度             |
| `overflow_marker`  | string               | 否   | `"..."`       | 文本溢出时使用的省略标记 |
| `text_mode`        | const-string         | 否   | `string.AUTO` | 文本解析模式             |
| `rich_params`      | table / nil          | 否   | `nil`         | 富文本参数               |
| `bold`             | boolean              | 否   | `false`       | 粗体                     |
| `italic`           | boolean              | 否   | `false`       | 斜体                     |
| `underline`        | boolean              | 否   | `false`       | 下划线                   |
| `strike`           | boolean              | 否   | `false`       | 删除线                   |
| `blink`            | boolean              | 否   | `false`       | 闪烁                     |
| `reverse`          | boolean              | 否   | `false`       | 反显                     |
| `hidden`           | boolean              | 否   | `false`       | 隐藏                     |
| `dim`              | boolean              | 否   | `false`       | 暗淡                     |
| `slice_layer`      | string               | 否   | `"base"`      | 绘制目标切片图层         |

### 返回

无。

### 示例

```lua
draw.text {
	x = 2,
	y = 1,
	text = "Hello TUI GAME",
	fg = color.BRIGHT_RED
}

draw.text {
	x = 2,
	y = 2,
	text = "Hello TUI GAME",
	fg = color.WHITE,
	italic = true
}
```

输出：

![draw.text示例](../image/draw_text_example.png)

### 额外补充

- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `fill_rect`

填充一个矩形区域。

### 调用

```lua
-- 表参数
draw.fill_rect{}
```

### 参数

| 参数          | 类型                 | 必填 | 默认值       | 说明                |
| ------------- | -------------------- | ---- | ------------ | ------------------- |
| `x`           | integer              | 是   | -            | 矩形左上角的 x 坐标 |
| `y`           | integer              | 是   | -            | 矩形左上角的 y 坐标 |
| `width`       | integer              | 是   | -            | 矩形宽度            |
| `height`      | integer              | 是   | -            | 矩形高度            |
| `char`        | string / nil         | 否   | `nil`        | 填充字符            |
| `fg`          | string / const-color | 否   | `color.NONE` | 前景色              |
| `bg`          | string / const-color | 否   | `color.NONE` | 背景色              |
| `slice_layer` | string               | 否   | `"base"`     | 绘制目标切片图层    |

### 返回

无。

### 示例

```lua
draw.fill_rect {
	x = 2,
	y = 1,
	width = 10,
	height = 4,
	bg = color.BLUE
}

draw.fill_rect {
	x = 13,
	y = 1,
	width = 10,
	height = 4,
	char = "-",
	fg = color.GREEN
}
```

输出：

![draw.fill_rect示例](../image/draw_fill_rect_example.png)

### 额外补充

- 参数 `char` 必须为宽度为 **1** 的字符。
- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `stroke_rect`

绘制一个矩形边框。

### 调用

```lua
-- 表参数
draw.stroke_rect{}
```

### 参数

| 参数          | 类型                 | 必填 | 默认值       | 说明                |
| ------------- | -------------------- | ---- | ------------ | ------------------- |
| `x`           | integer              | 是   | -            | 矩形左上角的 x 坐标 |
| `y`           | integer              | 是   | -            | 矩形左上角的 y 坐标 |
| `width`       | integer              | 是   | -            | 矩形宽度            |
| `height`      | integer              | 是   | -            | 矩形高度            |
| `fg`          | string / const-color | 否   | `color.NONE` | 边框前景色          |
| `bg`          | string / const-color | 否   | `color.NONE` | 边框背景色          |
| `border_char` | const-char / table   | 否   | `char.LINE`  | 边框字符            |
| `slice_layer` | string               | 否   | `"base"`     | 绘制目标切片图层    |

### 返回

无。

### 示例

```lua
draw.stroke_rect {
	x = 2,
	y = 1,
	width = 12,
	height = 5,
	fg = color.WHITE,
	border_char = char.ROUNDED_LINE
}

draw.stroke_rect {
	x = 15,
	y = 1,
	width = 12,
	height = 5,
	fg = color.YELLOW,
	border_char = {
		top = "-",
		left_top = "+",
		left = "|",
		left_bottom = "+",
		bottom = "-",
		right_bottom = "+",
		right = "|",
		right_top = "+",
	}
}
```

输出：

![draw.stroke_rect示例](../image/draw_stroke_rect_example.png)

### 额外补充

- 参数 `border_char` 表结构：

```lua
{
	top          = ..., -- string / const-char
	left_top     = ..., -- string / const-char
	left         = ..., -- string / const-char
	left_bottom  = ..., -- string / const-char
	bottom       = ..., -- string / const-char
	right_bottom = ..., -- string / const-char
	right        = ..., -- string / const-char
	right_top    = ...  -- string / const-char
}
```

- 参数 `border_char` 每个字段必须为宽度为 **1** 的字符。
- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `erase_rect`

擦除指定矩形区域。

### 调用

```lua
-- 表参数
draw.erase_rect{}
```

### 参数

| 参数          | 类型    | 必填 | 默认值   | 说明                |
| ------------- | ------- | ---- | -------- | ------------------- |
| `x`           | integer | 是   | -        | 矩形左上角的 x 坐标 |
| `y`           | integer | 是   | -        | 矩形左上角的 y 坐标 |
| `width`       | integer | 是   | -        | 矩形宽度            |
| `height`      | integer | 是   | -        | 矩形高度            |
| `slice_layer` | string  | 否   | `"base"` | 绘制目标切片图层    |

### 返回

无。

### 示例

```lua
draw.fill_rect {
	x = 2,
	y = 1,
	width = 10,
	height = 4,
	bg = color.BLUE
}

draw.erase_rect {
	x = 3,
	y = 2,
	width = 8,
	height = 2,
}
```

输出：

![draw.erase_rect示例](../image/draw_erase_rect_example.png)

---

## `render`

请求执行一次 `Render` 回调。

### 调用

```lua
-- 单参数
draw.render()
```

### 返回

无。

### 示例

```lua
function HandleEvent(event)
	-- 更新需要显示的内容
	message = "Hello TUI GAME"

	-- 请求重新绘制
	draw.render()
end

function Render()
	-- 绘制逻辑
end
```

### 额外补充

- **不可**在 `Render` 回调中调用。
