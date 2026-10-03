# measurement 库

`measurement` 用于测量文本的宽度和高度。

---

# 目录

## 方法

| 方法                | 说明               | 定位                                  |
| ----------------- | ---------------- | ----------------------------------- |
| `get_text_size`   | 测量给定文本的所占字符宽度与高度 | [get_text_size](#get_text_size)     |
| `get_text_width`  | 测量给定文本的所占字符宽度    | [get_text_width](#get_text_width)   |
| `get_text_height` | 测量给定文本的所占字符高度    | [get_text_height](#get_text_height) |

---

# 方法

## `get_text_size`

测量给定文本的所占字符宽度与高度。

### 调用

```lua
measurement.get_text_size
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 要绘制的文本 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `horizontal_align` | const-align | 默认：`align.LEFT`；各行相对于最长显示行的水平对齐方式 |
| `auto_wrap` | boolean | 默认：`true`；是否自动换行 |
| `word_wrap` | boolean | 默认：`true`；是否按完整单词换行 |
| `max_height` | integer / nil | 默认：`nil`；最大绘制高度；提供时范围为 1～65535 |
| `max_width` | integer / nil | 默认：`nil`；最大绘制宽度；提供时范围为 1～65535 |
| `overflow_marker` | string | 默认：`"..."`；文本溢出时使用的省略标记 |
| `rich_params` | table / nil | 默认：`nil`；富文本参数 |
| `text_mode` | const-string | 默认：`string.AUTO`；文本解析模式 |

## 返回值

返回两个值，依次为文本显示宽度和高度。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 文本显示宽度。 |
| `height` | integer | 文本显示高度。 |

### 示例

```lua
local width, height = measurement.get_text_size("Hello\nTUI", {max_width = 10})
debug.print("width: " .. tostring(width) .. ", height: " .. tostring(height))
```

**输出：**

```lua
```

### 额外说明

- 返回的宽度是换行和省略处理后最长显示行的字符格数，高度是文本显示的行数。`max_width` 是宽度上限，不会为短文本补空白；改变水平对齐方式不会改变测量结果。
- 与 `draw.text` 使用相同选项时，测量得到的宽、高就是绘制文本块的大小。可以直接将其交给 `align.resolve_rect`，将返回坐标作为文本块左上角，无需再减半个宽度。

---

## `get_text_width`

测量给定文本的所占字符宽度。

### 调用

```lua
measurement.get_text_width
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 要绘制的文本 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `horizontal_align` | const-align | 默认：`align.LEFT`；各行相对于最长显示行的水平对齐方式 |
| `auto_wrap` | boolean | 默认：`true`；是否自动换行 |
| `word_wrap` | boolean | 默认：`true`；是否按完整单词换行 |
| `max_width` | integer / nil | 默认：`nil`；最大绘制宽度；提供时范围为 1～65535 |
| `max_height` | integer / nil | 默认：`nil`；最大绘制高度；提供时范围为 1～65535 |
| `overflow_marker` | string | 默认：`"..."`；文本溢出时使用的省略标记 |
| `text_mode` | const-string | 默认：`string.AUTO`；文本解析模式 |
| `rich_params` | table / nil | 默认：`nil`；富文本参数 |

## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 文本显示宽度 |

### 示例

```lua
local width = measurement.get_text_width("Hello TUI", {max_width = 10})
debug.print("width: " .. tostring(width))
```

**输出：**

```lua
```

---

## `get_text_height`

测量给定文本的所占字符高度。

### 调用

```lua
measurement.get_text_height
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `text` | string | 要绘制的文本 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `horizontal_align` | const-align | 默认：`align.LEFT`；各行相对于最长显示行的水平对齐方式 |
| `auto_wrap` | boolean | 默认：`true`；是否自动换行 |
| `word_wrap` | boolean | 默认：`true`；是否按完整单词换行 |
| `max_width` | integer / nil | 默认：`nil`；最大绘制宽度；提供时范围为 1～65535 |
| `max_height` | integer / nil | 默认：`nil`；最大绘制高度；提供时范围为 1～65535 |
| `overflow_marker` | string | 默认：`"..."`；文本溢出时使用的省略标记 |
| `text_mode` | const-string | 默认：`string.AUTO`；文本解析模式 |
| `rich_params` | table / nil | 默认：`nil`；富文本参数 |

## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 文本显示高度 |

### 示例

```lua
local height = measurement.get_text_height("Line 1\nLine 2\nLine 3", {max_width = 10})
debug.print("height: " .. tostring(height))
```

**输出：**

```lua
```
