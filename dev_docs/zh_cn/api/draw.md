# draw 库

`draw` 提供终端画布绘制指令。

---

# 目录

## 方法

| 方法        | 说明                       | 定位                        |
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

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `x` | integer | 文本起始位置的 x 坐标 |
| `y` | integer | 文本起始位置的 y 坐标 |
| `text` | string | 要绘制的文本 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `fg` | string / const-color | 默认：`color.NONE`；前景色 |
| `bg` | string / const-color | 默认：`color.NONE`；背景色 |
| `horizontal_align` | const-align | 默认：`align.LEFT`；多行文本的水平对齐方式 |
| `auto_wrap` | boolean | 默认：`true`；是否自动换行 |
| `word_wrap` | boolean | 默认：`true`；是否按完整单词换行 |
| `max_width` | integer / nil | 默认：`nil`；最大绘制宽度；提供时范围为 1～65535 |
| `max_height` | integer / nil | 默认：`nil`；最大绘制高度；提供时范围为 1～65535 |
| `overflow_marker` | string | 默认：`"..."`；文本溢出时使用的省略标记 |
| `text_mode` | const-string | 默认：`string.AUTO`；文本解析模式 |
| `rich_params` | table / nil | 默认：`nil`；富文本参数 |
| `bold` | boolean | 默认：`false`；粗体 |
| `italic` | boolean | 默认：`false`；斜体 |
| `underline` | boolean | 默认：`false`；下划线 |
| `strike` | boolean | 默认：`false`；删除线 |
| `blink` | boolean | 默认：`false`；闪烁 |
| `reverse` | boolean | 默认：`false`；反显 |
| `hidden` | boolean | 默认：`false`；隐藏 |
| `dim` | boolean | 默认：`false`；暗淡 |
| `slice_layer` | string | 默认：`"base"`；绘制目标切片图层 |

## 返回值

无。

### 示例

```lua
draw.text(2, 1, "Hello TUI GAME", {fg = color.BRIGHT_RED})

draw.text(2, 2, "Hello TUI GAME", {fg = color.WHITE, italic = true})
```

**输出：**

```lua
```

### 额外说明

- 坐标从 0 开始，单位为终端字符格；超出目标切片的内容会裁剪。需要持续显示的内容放在 `Render` 回调中绘制。

- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `fill_rect`

填充一个矩形区域。

### 调用

```lua
draw.fill_rect
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `x` | integer | 矩形左上角的 x 坐标 |
| `y` | integer | 矩形左上角的 y 坐标 |
| `width` | integer | 矩形宽度，范围 1～65535 |
| `height` | integer | 矩形高度，范围 1～65535 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `char` | string / nil | 默认：`nil`；填充字符 |
| `fg` | string / const-color | 默认：`color.NONE`；前景色 |
| `bg` | string / const-color | 默认：`color.NONE`；背景色 |
| `slice_layer` | string | 默认：`"base"`；绘制目标切片图层 |

## 返回值

无。

### 示例

```lua
draw.fill_rect(2, 1, 10, 4, {bg = color.BLUE})

draw.fill_rect(13, 1, 10, 4, {char = "-", fg = color.GREEN})
```

**输出：**

```lua
```

### 额外说明

- 参数 `char` 必须为宽度为 **1** 的字符。
- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `stroke_rect`

绘制一个矩形边框。

### 调用

```lua
draw.stroke_rect
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `x` | integer | 矩形左上角的 x 坐标 |
| `y` | integer | 矩形左上角的 y 坐标 |
| `width` | integer | 矩形宽度，范围 1～65535 |
| `height` | integer | 矩形高度，范围 1～65535 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `fg` | string / const-color | 默认：`color.NONE`；边框前景色 |
| `bg` | string / const-color | 默认：`color.NONE`；边框背景色 |
| `border_char` | const-char / table | 默认：`char.LINE`；边框字符 |
| `slice_layer` | string | 默认：`"base"`；绘制目标切片图层 |

## 返回值

无。

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

```lua
```

### 额外说明

- 参数 `border_char` 表结构：

```lua
local border_char = {
  top = "─", -- string / const-char
  left_top = "┌", -- string / const-char
  left = "│", -- string / const-char
  left_bottom = "└", -- string / const-char
  bottom = "─", -- string / const-char
  right_bottom = "┘", -- string / const-char
  right = "│", -- string / const-char
  right_top = "┐", -- string / const-char
}
```

- 参数 `border_char` 每个字段必须为宽度为 **1** 的字符。
- 参数 `bg` 和参数 `fg` 均支持形如 rgb(r,g,b) 或 \#rrggbb 的颜色代码，字符串类型，无空格

---

## `erase_rect`

擦除指定矩形区域。

### 调用

```lua
draw.erase_rect
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `x` | integer | 矩形左上角的 x 坐标 |
| `y` | integer | 矩形左上角的 y 坐标 |
| `width` | integer | 矩形宽度，范围 1～65535 |
| `height` | integer | 矩形高度，范围 1～65535 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `slice_layer` | string | 默认：`"base"`；绘制目标切片图层 |

## 返回值

无。

### 示例

```lua
draw.fill_rect(2, 1, 10, 4, {bg = color.BLUE})

draw.erase_rect(3, 2, 8, 2)
```

**输出：**

```lua
```

---

## `render`

请求执行一次 `Render` 回调。

### 调用

```lua
draw.render
```

## 返回值

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

**输出：**

```lua
```

### 额外说明

- **不可**在 `Render` 回调中调用。
