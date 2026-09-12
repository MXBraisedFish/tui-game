# align 库

## 基本库说明

`align` 根据当前画布尺寸和相对位置计算对齐坐标。

---

## 目录

### 常量

| 常量名              | 说明         | 索引                                    |
| ------------------- | ------------ | --------------------------------------- |
| `AUTO`              | 自动对齐模式 | [AUTO](#AUTO)                           |
| `LEFT`              | 左对齐模式   | [LEFT](#LEFT)                           |
| `HORIZONTAL_CENTER` | 水平居中模式 | [HORIZONTAL_CENTER](#HORIZONTAL_CENTER) |
| `RIGHT`             | 右对齐模式   | [RIGHT](#RIGHT)                         |
| `TOP`               | 顶部对齐模式 | [TOP](#TOP)                             |
| `VERTICAL_CENTER`   | 垂直居中模式 | [VERTICAL_CENTER](#VERTICAL_CENTER)     |
| `BOTTOM`            | 底部对齐模式 | [BOTTOM](#BOTTOM)                       |
| `CENTER`            | 双向居中模式 | [CENTER](#CENTER)                       |

### 方法

| 方法名         | 说明                                                                                         | 索引                          |
| -------------- | -------------------------------------------------------------------------------------------- | ----------------------------- |
| `resolve_x`    | 根据元素宽度与水平对齐方式，计算文本左边缘的 x 坐标                                          | [resolve_x](#resolve_x)       |
| `resolve_y`    | 根据元素高度与垂直对齐方式，计算文本上边缘的 y 坐标                                          | [resolve_y](#resolve_y)       |
| `resolve_rect` | 根据元素宽度与高度、水平对齐方式和垂直对齐方式，计算文本左边缘的 x 坐标、文本上边缘的 y 坐标 | [resolve_rect](#resolve_rect) |

---

## 常量

## `AUTO`

自动对齐模式。

**可用于**

- 参数 `horizontal_align`
- 参数 `vertical_align`

### 调用

```lua
align.AUTO
```

### 示例

```lua
local rect = align.resolve_rect { width = 4, height = 1, horizontal_align = align.AUTO, vertical_align = align.AUTO, offset_x = 0, offset_y = 0 }
draw.text { x = rect.x, y = rect.y, text = "AUTO", fg = color.BRIGHT_RED }
```

输出：

![align.AUTO示例](../image/align_AUTO_example.png)

### 额外补充

- `align` API 中用于水平对齐时等价于 `align.HORIZONTAL_CENTER`。
- `align` API 中用于垂直对齐时等价于 `align.VERTICAL_CENTER`。
- `draw` 和 `measurement` API 中用于垂直对齐时等价于 `align.LEFT`。

---

## `LEFT`

右对齐模式。

**可用于**

- 参数 `horizontal_align`

### 调用

```lua
align.LEFT
```

### 示例

```lua
local x = align.resolve_x { width = 4, horizontal_align = align.LEFT }
draw.text { x = x, y = 3, text = "LEFT", fg = color.BRIGHT_RED }
```

输出：

![align.LEFT示例](../image/align_LEFT_example.png)

---

## `HORIZONTAL_CENTER`

水平居中模式。

**可用于**

- 参数 `horizontal_align`

### 调用

```lua
align.HORIZONTAL_CENTER
```

### 示例

```lua
local x = align.resolve_x { width = 8, horizontal_align = align.HORIZONTAL_CENTER }
draw.text { x = x, y = 3, text = "H_CENTER", fg = color.BRIGHT_RED }
```

输出：

![align.HORIZONTAL_CENTER示例](../image/align_HORIZONTAL_CENTER_example.png)

---

## `RIGHT`

右对齐模式。

**可用于**

- 参数 `horizontal_align`

### 调用

```lua
align.RIGHT
```

### 示例

```lua
local x = align.resolve_x { width = 5, horizontal_align = align.RIGHT }
draw.text { x = x, y = 3, text = "RIGHT", fg = color.BRIGHT_RED }
```

输出：

![align.RIGHT示例](../image/align_RIGHT_example.png)

---

## `TOP`

顶部对齐模式。

**可用于**

- 参数 `vertical_align`

### 调用

```lua
align.TOP
```

### 示例

```lua
local y = align.resolve_y { height = 3, vertical_align = align.TOP }
draw.text { x = 4, y = y, text = "TOP", fg = color.BRIGHT_RED, max_width = 1 }
```

输出：

![align.TOP示例](../image/align_TOP_example.png)

---

## `VERTICAL_CENTER`

垂直居中模式。

**可用于**

- 参数 `vertical_align`

### 调用

```lua
align.VERTICAL_CENTER
```

### 示例

```lua
local y = align.resolve_y { height = 8, vertical_align = align.VERTICAL_CENTER }
draw.text { x = 4, y = y, text = "V|CENTER", fg = color.BRIGHT_RED, max_width = 1 }
```

输出：

![align.VERTICAL_CENTER示例](../image/align_VERTICAL_CENTER_example.png)

---

## `BOTTOM`

底部对齐模式。

**可用于**

- 参数 `vertical_align`

### 调用

```lua
align.BOTTOM
```

### 示例

```lua
local y = align.resolve_y { height = 7, vertical_align = align.BOTTOM }
draw.text { x = 4, y = y, text = "BOTTOM", fg = color.BRIGHT_RED, max_width = 1 }
```

输出：

![align.BOTTOM示例](../image/align_BOTTOM_example.png)

---

## `CENTER`

双向居中模式。

**可用于**

- 参数 `horizontal_align`
- 参数 `vertical_align`

### 调用

```lua
align.CENTER
```

### 示例

```lua
local rect = align.resolve_rect { width = 6, height = 1, horizontal_align = align.CENTER, vertical_align = align.CENTER, offset_x = 0, offset_y = 0 }
draw.text { x = rect.x, y = rect.y, text = "CENTER", fg = color.BRIGHT_RED }
```

输出：

![align.CENTER示例](../image/align_CENTER_example.png)

### 额外补充

- 水平对齐时等价于 `align.HORIZONTAL_CENTER`。
- 垂直对齐时等价于 `align.VERTICAL_CENTER`。

---

## 方法

## `resolve_x`

根据元素宽度与水平对齐方式，计算文本左边缘的 x 坐标。

### 调用

```lua
-- 表参数
align.resolve_x{}
```

### 参数

| 参数名             | 类型          | 必填 | 默认值   | 说明             |
| ------------------ | ------------- | ---- | -------- | ---------------- |
| `width`            | integer       | 是   | -        | 文本宽度         |
| `horizontal_align` | const-align   | 是   | -        | 水平对齐方式     |
| `offset_x`         | integer       | 否   | `0`      | 锚点上的水平偏移 |
| `relative_x`       | integer / nil | 否   | `nil`    | 自定义水平锚点   |
| `slice_layer`      | string        | 否   | `"base"` | 目标切片图层     |

### 返回

直接返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 绘制起始水平坐标 |

### 示例

```lua
local x1 = align.resolve_x { width = 5, horizontal_align = align.LEFT }
draw.text { x = x1, y = 4, text = "Hello", fg = color.BRIGHT_RED }

local x2 = align.resolve_x { width = 3, horizontal_align = align.CENTER, offset_x = -5 }
draw.text { x = x2, y = 4, text = "TUI", fg = color.BRIGHT_BLUE }

local x3 = align.resolve_x { width = 4, horizontal_align = align.RIGHT, relative_x = 60 }
draw.text { x = x3, y = 4, text = "Game", fg = color.BRIGHT_GREEN }
```

输出：

![align.resolve_x示例](../image/align_resolve_x_example.png)

---

## `resolve_y`

根据元素高度与垂直对齐方式，计算文本上边缘的 y 坐标。

### 调用

```lua
-- 表参数
align.resolve_y{}
```

### 参数

| 参数名           | 类型          | 必填 | 默认值   | 说明             |
| ---------------- | ------------- | ---- | -------- | ---------------- |
| `height`         | integer       | 是   | -        | 文本高度         |
| `vertical_align` | const-align   | 是   | -        | 垂直对齐方式     |
| `offset_y`       | integer       | 否   | `0`      | 锚点上的垂直偏移 |
| `relative_y`     | integer / nil | 否   | `nil`    | 自定义垂直锚点   |
| `slice_layer`    | string        | 否   | `"base"` | 目标切片图层     |

### 返回

直接返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 绘制起始垂直坐标 |

### 示例

```lua
local y1 = align.resolve_y { height = 5, vertical_align = align.TOP }
draw.text { x = 5, y = y1, text = "Hello", fg = color.BRIGHT_RED, max_width = 1 }

local y2 = align.resolve_y { height = 3, vertical_align = align.CENTER, offset_y = -2 }
draw.text { x = 5, y = y2, text = "TUI", fg = color.BRIGHT_BLUE, max_width = 1 }

local y3 = align.resolve_y { height = 4, vertical_align = align.BOTTOM, relative_y = 23 }
draw.text { x = 5, y = y3, text = "Game", fg = color.BRIGHT_GREEN, max_width = 1 }
```

输出：

![align.resolve_y示例](../image/align_resolve_y_example.png)

---

## `resolve_rect`

根据元素宽度与高度、水平对齐方式和垂直对齐方式，计算文本左边缘的 x 坐标、文本上边缘的 y 坐标。

### 调用

```lua
-- 表参数
align.resolve_rect{}
```

### 参数

| 参数名             | 类型          | 必填 | 默认值   | 说明             |
| ------------------ | ------------- | ---- | -------- | ---------------- |
| `width`            | integer       | 是   | -        | 文本宽度         |
| `height`           | integer       | 是   | -        | 文本高度         |
| `horizontal_align` | const-align   | 是   | -        | 水平对齐方式     |
| `vertical_align`   | const-align   | 是   | -        | 垂直对齐方式     |
| `offset_x`         | integer       | 否   | `0`      | 锚点上的水平偏移 |
| `offset_y`         | integer       | 否   | `0`      | 锚点上的垂直偏移 |
| `relative_x`       | integer / nil | 否   | `nil`    | 自定义水平锚点   |
| `relative_y`       | integer / nil | 否   | `nil`    | 自定义垂直锚点   |
| `slice_layer`      | string        | 否   | `"base"` | 目标切片图层     |

### 返回

返回一个对象表。

| 字段 | 类型    | 说明             |
| ---- | ------- | ---------------- |
| `x`  | integer | 绘制起始水平坐标 |
| `y`  | integer | 绘制起始垂直坐标 |

### 示例

```lua
local rect1 = align.resolve_rect { width = 20, height = 4, horizontal_align = align.LEFT, vertical_align = align.CENTER, offset_x = 20, offset_y = -2 }
draw.fill_rect { x = rect1.x, y = rect1.y, width = 20, height = 4, bg = color.BRIGHT_YELLOW }

local rect2 = align.resolve_rect { width = 10, height = 4, horizontal_align = align.CENTER, vertical_align = align.BOTTOM, offset_x = -5, offset_y = 0 }
draw.fill_rect { x = rect2.x, y = rect2.y, width = 10, height = 4, bg = color.BRIGHT_GREEN }
```

输出：

![align.resolve_rect示例](../image/align_resolve_rect_example.png)
