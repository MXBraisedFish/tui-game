# align 库

`align` 根据当前画布尺寸和相对位置计算对齐坐标。

---

# 目录

## 常量

| 常量              | 说明         | 定位                                    |
| ------------------- | ------------ | --------------------------------------- |
| `AUTO`              | 自动对齐模式 | [AUTO](#auto)                           |
| `LEFT`              | 左对齐模式   | [LEFT](#left)                           |
| `HORIZONTAL_CENTER` | 水平居中模式 | [HORIZONTAL_CENTER](#horizontal_center) |
| `RIGHT`             | 右对齐模式   | [RIGHT](#right)                         |
| `TOP`               | 顶部对齐模式 | [TOP](#top)                             |
| `VERTICAL_CENTER`   | 垂直居中模式 | [VERTICAL_CENTER](#vertical_center)     |
| `BOTTOM`            | 底部对齐模式 | [BOTTOM](#bottom)                       |
| `CENTER`            | 双向居中模式 | [CENTER](#center)                       |

## 方法

| 方法             | 说明                                                | 定位                            |
| -------------- | ------------------------------------------------- | ----------------------------- |
| `resolve_x`    | 根据元素宽度与水平对齐方式，计算文本左边缘的 x 坐标                       | [resolve_x](#resolve_x)       |
| `resolve_y`    | 根据元素高度与垂直对齐方式，计算文本上边缘的 y 坐标                       | [resolve_y](#resolve_y)       |
| `resolve_rect` | 根据元素宽度与高度、水平对齐方式和垂直对齐方式，计算文本左边缘的 x 坐标、文本上边缘的 y 坐标 | [resolve_rect](#resolve_rect) |

---

# 常量

## `AUTO`

自动对齐模式。

### 调用

```lua
align.AUTO
```

### 可用于

- 参数 `horizontal_align`
- 参数 `vertical_align`

### 示例

```lua
local x, y = align.resolve_rect(4, 1, align.AUTO, align.AUTO, {offset_x = 0, offset_y = 0})
draw.text(x, y, "AUTO", {fg = color.BRIGHT_RED})
```

**输出：**

![align_AUTO_example](../image/align_AUTO_example.png)

### 等值

```text
"auto"
```

### 额外说明

- `align` API 中用于水平对齐时等价于 `align.HORIZONTAL_CENTER`。
- `align` API 中用于垂直对齐时等价于 `align.VERTICAL_CENTER`。
- `draw` 和 `measurement` API 中用于水平对齐时等价于 `align.LEFT`。

---

## `LEFT`

左对齐模式。

### 调用

```lua
align.LEFT
```

### 可用于

- 参数 `horizontal_align`

### 示例

```lua
local x = align.resolve_x(4, align.LEFT)
draw.text(x, 3, "LEFT", {fg = color.BRIGHT_RED})
```

**输出：**

![align_LEFT_example](../image/align_LEFT_example.png)

### 等值

```text
"left"
```

---

## `HORIZONTAL_CENTER`

水平居中模式。

### 调用

```lua
align.HORIZONTAL_CENTER
```

### 可用于

- 参数 `horizontal_align`

### 示例

```lua
local x = align.resolve_x(8, align.HORIZONTAL_CENTER)
draw.text(x, 3, "H_CENTER", {fg = color.BRIGHT_RED})
```

**输出：**

![align_HORIZONTAL_CENTER_example](../image/align_HORIZONTAL_CENTER_example.png)

### 等值

```text
"horizontal_center"
```

### 额外说明

- 用于 `draw.text` 时，各行在最长显示行的宽度内居中。

---

## `RIGHT`

右对齐模式。

### 调用

```lua
align.RIGHT
```

### 可用于

- 参数 `horizontal_align`

### 示例

```lua
local x = align.resolve_x(5, align.RIGHT)
draw.text(x, 3, "RIGHT", {fg = color.BRIGHT_RED})
```

**输出：**

![align_RIGHT_example](../image/align_RIGHT_example.png)

### 等值

```text
"right"
```

### 额外说明

- 用于 `draw.text` 时，各行向最长显示行的右边缘对齐。

---

## `TOP`

顶部对齐模式。

### 调用

```lua
align.TOP
```

### 可用于

- 参数 `vertical_align`

### 示例

```lua
local y = align.resolve_y(3, align.TOP)
draw.text(4, y, "TOP", {fg = color.BRIGHT_RED, max_width = 1})
```

**输出：**

![align_TOP_example](../image/align_TOP_example.png)

### 等值

```text
"top"
```

---

## `VERTICAL_CENTER`

垂直居中模式。

### 调用

```lua
align.VERTICAL_CENTER
```

### 可用于

- 参数 `vertical_align`

### 示例

```lua
local y = align.resolve_y(8, align.VERTICAL_CENTER)
draw.text(4, y, "V|CENTER", {fg = color.BRIGHT_RED, max_width = 1})
```

**输出：**

![align_VERTICAL_CENTER_example](../image/align_VERTICAL_CENTER_example.png)

### 等值

```text
"vertical_center"
```

---

## `BOTTOM`

底部对齐模式。

### 调用

```lua
align.BOTTOM
```

### 可用于

- 参数 `vertical_align`

### 示例

```lua
local y = align.resolve_y(7, align.BOTTOM)
draw.text(4, y, "BOTTOM", {fg = color.BRIGHT_RED, max_width = 1})
```

**输出：**

![align_BOTTOM_example](../image/align_BOTTOM_example.png)

### 等值

```text
"bottom"
```

---

## `CENTER`

双向居中模式。

### 调用

```lua
align.CENTER
```

### 可用于

- 参数 `horizontal_align`
- 参数 `vertical_align`

### 示例

```lua
local x, y = align.resolve_rect(6, 1, align.CENTER, align.CENTER, {offset_x = 0, offset_y = 0})
draw.text(x, y, "CENTER", {fg = color.BRIGHT_RED})
```

**输出：**

![align_CENTER_example](../image/align_CENTER_example.png)

### 等值

```text
"center"
```

### 额外说明

- 水平对齐时等价于 `align.HORIZONTAL_CENTER`。
- 垂直对齐时等价于 `align.VERTICAL_CENTER`。
- 用于 `draw.text` 时，各行在最长显示行的宽度内居中。

---

# 方法

## `resolve_x`

根据元素宽度与水平对齐方式，计算文本左边缘的 x 坐标。

### 调用

```lua
align.resolve_x
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 文本宽度 |
| `horizontal_align` | const-align | 水平对齐方式 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `offset_x` | integer | `0` | 锚点上的水平偏移 |
| `relative_x` | integer / nil | `nil` | 自定义水平锚点 |
| `slice_layer` | string | `"base"` | 目标切片图层 |

## 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 绘制起始水平坐标 |

### 示例

```lua
local x1 = align.resolve_x(5, align.LEFT)
draw.text(x1, 4, "Hello", {fg = color.BRIGHT_RED})

local x2 = align.resolve_x(3, align.CENTER, {offset_x = -5})
draw.text(x2, 4, "TUI", {fg = color.BRIGHT_BLUE})

local x3 = align.resolve_x(4, align.RIGHT, {relative_x = 60})
draw.text(x3, 4, "Game", {fg = color.BRIGHT_GREEN})
```

**输出：**

![align_resolve_x_example](../image/align_resolve_x_example.png)

---

## `resolve_y`

根据元素高度与垂直对齐方式，计算文本上边缘的 y 坐标。

### 调用

```lua
align.resolve_y
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `height` | integer | 文本高度 |
| `vertical_align` | const-align | 垂直对齐方式 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `offset_y` | integer | `0` | 锚点上的垂直偏移 |
| `relative_y` | integer / nil | `nil` | 自定义垂直锚点 |
| `slice_layer` | string | `"base"` | 目标切片图层 |

## 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 绘制起始垂直坐标 |

### 示例

```lua
local y1 = align.resolve_y(5, align.TOP)
draw.text(5, y1, "Hello", {fg = color.BRIGHT_RED, max_width = 1})

local y2 = align.resolve_y(3, align.CENTER, {offset_y = -2})
draw.text(5, y2, "TUI", {fg = color.BRIGHT_BLUE, max_width = 1})

local y3 = align.resolve_y(4, align.BOTTOM, {relative_y = 23})
draw.text(5, y3, "Game", {fg = color.BRIGHT_GREEN, max_width = 1})
```

**输出：**

![align_resolve_y_example](../image/align_resolve_y_example.png)

---

## `resolve_rect`

根据元素宽度与高度、水平对齐方式和垂直对齐方式，计算文本左边缘的 x 坐标、文本上边缘的 y 坐标。

### 调用

```lua
align.resolve_rect
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 文本宽度 |
| `height` | integer | 文本高度 |
| `horizontal_align` | const-align | 水平对齐方式 |
| `vertical_align` | const-align | 垂直对齐方式 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `offset_x` | integer | `0` | 锚点上的水平偏移 |
| `offset_y` | integer | `0` | 锚点上的垂直偏移 |
| `relative_x` | integer / nil | `nil` | 自定义水平锚点 |
| `relative_y` | integer / nil | `nil` | 自定义垂直锚点 |
| `slice_layer` | string | `"base"` | 目标切片图层 |

## 返回值

返回两个值。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `x` | integer | 绘制起始水平坐标 |
| `y` | integer | 绘制起始垂直坐标 |

### 示例

```lua
local x1, y1 = align.resolve_rect(20, 4, align.LEFT, align.CENTER, {offset_x = 20, offset_y = -2})
draw.fill_rect(x1, y1, 20, 4, {bg = color.BRIGHT_YELLOW})

local x2, y2 = align.resolve_rect(10, 4, align.CENTER, align.BOTTOM, {offset_x = -5, offset_y = 0})
draw.fill_rect(x2, y2, 10, 4, {bg = color.BRIGHT_GREEN})
```

**输出：**

![align_resolve_rect_example](../image/align_resolve_rect_example.png)
