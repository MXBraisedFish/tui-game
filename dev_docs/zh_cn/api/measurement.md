# measurement 库

## 基本库说明

`measurement` 提供文本尺寸测量辅助。

---

## 目录

### 方法

| 方法名            | 说明                             | 索引                                |
| ----------------- | -------------------------------- | ----------------------------------- |
| `get_text_size`   | 测量给定文本的所占字符宽度与高度 | [get_text_size](#get_text_size)     |
| `get_text_width`  | 测量给定文本的所占字符宽度       | [get_text_width](#get_text_width)   |
| `get_text_height` | 测量给定文本的所占字符高度       | [get_text_height](#get_text_height) |

---

## 方法

## `get_text_size`

测量给定文本的所占字符宽度与高度。

### 调用

```lua
-- 表参数
measurement.get_text_size{}
```

### 参数

| 参数               | 类型          | 必填 | 默认值        | 说明                     |
| ------------------ | ------------- | ---- | ------------- | ------------------------ |
| `text`             | string        | 是   | -             | 要绘制的文本             |
| `horizontal_align` | const-align   | 否   | `align.LEFT`  | 多行文本的水平对齐方式   |
| `auto_wrap`        | boolean       | 否   | `true`        | 是否自动换行             |
| `word_wrap`        | boolean       | 否   | `true`        | 是否按完整单词换行       |
| `max_width`        | integer / nil | 否   | `nil`         | 最大绘制宽度             |
| `max_height`       | integer / nil | 否   | `nil`         | 最大绘制高度             |
| `overflow_marker`  | string        | 否   | `"..."`       | 文本溢出时使用的省略标记 |
| `text_mode`        | const-string  | 否   | `string.AUTO` | 文本解析模式             |
| `rich_params`      | table / nil   | 否   | `nil`         | 富文本参数               |

### 返回

返回一个对象表。

| 字段     | 类型    | 说明         |
| -------- | ------- | ------------ |
| `width`  | integer | 文本显示宽度 |
| `height` | integer | 文本显示高度 |

### 示例

```lua
local size = measurement.get_text_size {
  text = "Hello\nTUI",
  max_width = 10
}
debug.print { message = "width: " .. tostring(size.width) .. ", height: " .. tostring(size.height) }
```

输出：

```text
width: 5, height: 2
```

---

## `get_text_width`

测量给定文本的所占字符宽度。

### 调用

```lua
-- 表参数
measurement.get_text_width{}
```

### 参数

| 参数               | 类型          | 必填 | 默认值        | 说明                     |
| ------------------ | ------------- | ---- | ------------- | ------------------------ |
| `text`             | string        | 是   | -             | 要绘制的文本             |
| `horizontal_align` | const-align   | 否   | `align.LEFT`  | 多行文本的水平对齐方式   |
| `auto_wrap`        | boolean       | 否   | `true`        | 是否自动换行             |
| `word_wrap`        | boolean       | 否   | `true`        | 是否按完整单词换行       |
| `max_width`        | integer / nil | 否   | `nil`         | 最大绘制宽度             |
| `max_height`       | integer / nil | 否   | `nil`         | 最大绘制高度             |
| `overflow_marker`  | string        | 否   | `"..."`       | 文本溢出时使用的省略标记 |
| `text_mode`        | const-string  | 否   | `string.AUTO` | 文本解析模式             |
| `rich_params`      | table / nil   | 否   | `nil`         | 富文本参数               |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 文本显示宽度 |

### 示例

```lua
local width = measurement.get_text_width { text = "Hello TUI", bold = true }
debug.print { message = "width: " .. tostring(width) }
```

输出：

```text
width: 9
```

---

## `get_text_height`

测量给定文本的所占字符高度。

### 调用

```lua
-- 表参数
measurement.get_text_height{}
```

### 参数

| 参数               | 类型          | 必填 | 默认值        | 说明                     |
| ------------------ | ------------- | ---- | ------------- | ------------------------ |
| `text`             | string        | 是   | -             | 要绘制的文本             |
| `horizontal_align` | const-align   | 否   | `align.LEFT`  | 多行文本的水平对齐方式   |
| `auto_wrap`        | boolean       | 否   | `true`        | 是否自动换行             |
| `word_wrap`        | boolean       | 否   | `true`        | 是否按完整单词换行       |
| `max_width`        | integer / nil | 否   | `nil`         | 最大绘制宽度             |
| `max_height`       | integer / nil | 否   | `nil`         | 最大绘制高度             |
| `overflow_marker`  | string        | 否   | `"..."`       | 文本溢出时使用的省略标记 |
| `text_mode`        | const-string  | 否   | `string.AUTO` | 文本解析模式             |
| `rich_params`      | table / nil   | 否   | `nil`         | 富文本参数               |

### 返回

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 文本显示高度 |

### 示例

```lua
local height = measurement.get_text_height {
  text = "Line 1\nLine 2\nLine 3",
  max_width = 10
}
debug.print { message = "height: " .. tostring(height) }
```

输出：

```text
height: 3
```
