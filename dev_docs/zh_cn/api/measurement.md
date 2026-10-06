# measurement 库

`measurement` 用于测量文本占用的终端显示列数与行数。

---

# 目录

## 方法

| 方法              | 说明                                 | 定位                                |
| ----------------- | ------------------------------------ | ----------------------------------- |
| `get_text_size`   | 测量给定文本占用的终端显示列数与行数 | [get_text_size](#get_text_size)     |
| `get_text_width`  | 测量给定文本占用的终端显示列数       | [get_text_width](#get_text_width)   |
| `get_text_height` | 测量给定文本占用的终端显示行数       | [get_text_height](#get_text_height) |

---

# 方法

## `get_text_size`

测量给定文本占用的终端显示列数与行数。

### 调用

```lua
measurement.get_text_size
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明         |
| ------ | ------ | ------------ |
| `text` | string | 要测量的文本 |

### 选填参数

| 参数名             | 类型          | 默认值        | 说明                               |
| ------------------ | ------------- | ------------- | ---------------------------------- |
| `horizontal_align` | const-align   | `align.LEFT`  | 各行相对于最长显示行的水平对齐方式 |
| `auto_wrap`        | boolean       | `true`        | 是否自动换行                       |
| `word_wrap`        | boolean       | `true`        | 是否按完整单词换行                 |
| `max_height`       | integer / nil | `nil`         | 最大绘制高度                       |
| `max_width`        | integer / nil | `nil`         | 最大绘制宽度                       |
| `overflow_marker`  | string        | `"..."`       | 文本溢出时使用的省略标记           |
| `rich_params`      | table / nil   | `nil`         | 富文本参数                         |
| `text_mode`        | const-string  | `string.AUTO` | 文本解析模式                       |

## 返回值

返回两个值，依次为文本占用的终端显示列数与行数。

| 值名     | 类型    | 说明                   |
| -------- | ------- | ---------------------- |
| `width`  | integer | 文本占用的终端显示列数 |
| `height` | integer | 文本占用的终端显示行数 |

### 示例

```lua
local width, height = measurement.get_text_size("Hello\nTUI", {max_width = 10})
debug.print("width: " .. tostring(width) .. ", height: " .. tostring(height))
```

**输出：**

```lua
width: 5, height: 2
```

## 额外说明

- 选填参数 `max_height` 取值范围为 $[1, 65535]$。
- 选填参数 `max_width` 取值范围为 $[1, 65535]$。

---

## `get_text_width`

测量给定文本占用的终端显示列数。

### 调用

```lua
measurement.get_text_width
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明         |
| ------ | ------ | ------------ |
| `text` | string | 要测量的文本 |

### 选填参数

| 参数名             | 类型          | 默认值        | 说明                               |
| ------------------ | ------------- | ------------- | ---------------------------------- |
| `horizontal_align` | const-align   | `align.LEFT`  | 各行相对于最长显示行的水平对齐方式 |
| `auto_wrap`        | boolean       | `true`        | 是否自动换行                       |
| `word_wrap`        | boolean       | `true`        | 是否按完整单词换行                 |
| `max_height`       | integer / nil | `nil`         | 最大绘制高度                       |
| `max_width`        | integer / nil | `nil`         | 最大绘制宽度                       |
| `overflow_marker`  | string        | `"..."`       | 文本溢出时使用的省略标记           |
| `rich_params`      | table / nil   | `nil`         | 富文本参数                         |
| `text_mode`        | const-string  | `string.AUTO` | 文本解析模式                       |

## 返回值

返回一个值。

| 类型    | 说明                   |
| ------- | ---------------------- |
| integer | 文本占用的终端显示列数 |

### 示例

```lua
local width = measurement.get_text_width("Hello TUI", {max_width = 10})
debug.print("width: " .. tostring(width))
```

**输出：**

```lua
width: 9
```

## 额外说明

- 选填参数 `max_height` 取值范围为 $[1, 65535]$。
- 选填参数 `max_width` 取值范围为 $[1, 65535]$。

---

## `get_text_height`

测量给定文本占用的终端显示行数。

### 调用

```lua
measurement.get_text_height
```

## 参数

### 必填参数

| 参数名 | 类型   | 说明         |
| ------ | ------ | ------------ |
| `text` | string | 要测量的文本 |

### 选填参数

| 参数名             | 类型          | 默认值        | 说明                               |
| ------------------ | ------------- | ------------- | ---------------------------------- |
| `horizontal_align` | const-align   | `align.LEFT`  | 各行相对于最长显示行的水平对齐方式 |
| `auto_wrap`        | boolean       | `true`        | 是否自动换行                       |
| `word_wrap`        | boolean       | `true`        | 是否按完整单词换行                 |
| `max_height`       | integer / nil | `nil`         | 最大绘制高度                       |
| `max_width`        | integer / nil | `nil`         | 最大绘制宽度                       |
| `overflow_marker`  | string        | `"..."`       | 文本溢出时使用的省略标记           |
| `rich_params`      | table / nil   | `nil`         | 富文本参数                         |
| `text_mode`        | const-string  | `string.AUTO` | 文本解析模式                       |

## 返回值

返回一个值。

| 类型    | 说明                   |
| ------- | ---------------------- |
| integer | 文本占用的终端显示行数 |

### 示例

```lua
local height = measurement.get_text_height("Line 1\nLine 2\nLine 3", {max_width = 10})
debug.print("height: " .. tostring(height))
```

**输出：**

```lua
height: 3
```

## 额外说明

- 选填参数 `max_height` 取值范围为 $[1, 65535]$。
- 选填参数 `max_width` 取值范围为 $[1, 65535]$。
