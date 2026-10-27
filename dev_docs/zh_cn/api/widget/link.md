# widget.link 库

`widget.link` 用于设置链接内容。

---

# 目录

## 方法

| 方法       | 说明                     | 定位                  |
| ---------- | ------------------------ | --------------------- |
| `create`   | 创建一个链接内容对象     | [create](#create)     |
| `delete`   | 删除指定链接内容         | [delete](#delete)     |
| `clear`    | 删除所有链接内容         | [clear](#clear)       |
| `list`     | 获取所有链接内容信息     | [list](#list)         |
| `count`    | 返回当前链接内容的总数   | [count](#count)       |
| `draw`     | 绘制链接内容             | [draw](#draw)         |
| `click`    | 模拟点击链接内容         | [click](#click)       |
| `set`      | 修改链接内容的参数       | [set](#set)           |
| `get_info` | 获取链接内容的完整信息表 | [get_info](#get_info) |
| `exists`   | 检查链接内容是否存在     | [exists](#exists)     |

---

# 方法

## `create`

创建一个链接内容对象。

### 调用

```lua
widget.link.create
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明         |
| ------ | ------ | ------------ |
| `text` | string | 链接展示文本 |
| `link` | string | 链接目标     |

#### 选填参数

| 参数        | 类型                 | 默认值              | 含义                   |
| ----------- | -------------------- | ------------------- | ---------------------- |
| `fg`        | string / const-color | `color.BRIGHT_BLUE` | 前景色                 |
| `bg`        | string / const-color | `color.NONE`        | 背景色                 |
| `hover_fg`  | string / const-color | `color.BRIGHT_BLUE` | 指针悬浮前景色         |
| `hover_bg`  | string / const-color | `color.NONE`        | 指针悬浮背景色         |
| `mouse`     | boolean              | `true`              | 是否允许鼠标操作       |
| `copy`      | boolean              | `false`             | 执行后是否复制到剪切板 |
| `bold`      | boolean              | `false`             | 粗体                   |
| `italic`    | boolean              | `false`             | 斜体                   |
| `underline` | boolean              | `true`              | 下划线                 |
| `strike`    | boolean              | `false`             | 删除线                 |
| `blink`     | boolean              | `false`             | 闪烁                   |
| `reverse`   | boolean              | `false`             | 反显                   |
| `hidden`    | boolean              | `false`             | 隐藏                   |
| `dim`       | boolean              | `false`             | 暗淡                   |

### 返回值

返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | 链接内容 ID |

### 示例

```lua
function Init(ctx)
  link = widget.link.create("Tui Game", "https://github.com/MXBraisedFish/tui-game")
end

function Render()
  widget.link.draw(link, 5, 2)
end
```

**输出：**

![widget_link_create_example](../../image/widget_link_create_example.png)

### 额外说明

- 必填参数 `link` 在填写文件路径时，路径相对当前包的 `assets/`，当选填参数 `copy` 为 `true` 时，复制的路径为绝对路径。

---

## `delete`

删除指定链接内容。

### 调用

```lua
widget.link.delete
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `id`   | string | 链接内容 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
local link = widget.link.create("test", "./test.md")
debug.print(link)

debug.print(widget.link.delete(link))

debug.print(table.pretty(widget.link.list()))
```

**输出：**

```lua
link_001
true
{
  n = 0
}
```

---

## `clear`

删除所有链接内容。

### 调用

```lua
widget.link.clear
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
widget.link.create("test", "./test.md")
widget.link.create("test", "./test.md")
widget.link.create("test", "./test.md")
widget.link.create("test", "./test.md")

debug.print(widget.link.clear())

debug.print(table.pretty(widget.link.list()))
```

**输出：**

```lua
true
{
  n = 0
}
```

---

## `list`

获取所有链接内容信息。

### 调用

```lua
widget.link.list
```

### 返回值

返回一个值。

| 类型  | 说明                     |
| ----- | ------------------------ |
| table | 所有链接内容信息的混合表 |

### 示例

```lua
local link = widget.link.create("test", "./test.md")

debug.print(table.pretty(widget.link.list()))
```

**输出：**

```lua
{
  [1] = {
    id = "link_001",
    text = "test",
    link = "./test.md",
    fg = "bright_blue",
    bg = "none",
    hover_fg = "bright_blue",
    hover_bg = "none",
    mouse = true,
    copy = false,
    bold = false,
    italic = false,
    underline = true,
    strike = false,
    blink = false,
    reverse = false,
    hidden = false,
    dim = false
  },
  n = 1
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  {
    id = "link_001",          -- string（链接内容 ID）
    text = "test",            -- string（链接展示文本）
    link = "./test.md",       -- string（链接目标）
    fg = "bright_blue",       -- string（前景色）
    bg = "none",              -- string（背景色）
    hover_fg = "bright_blue", -- string（指针悬浮前景色）
    hover_bg = "none",        -- string（指针悬浮背景色）
    mouse = true,             -- boolean（是否允许鼠标操作）
    copy = false,             -- boolean（执行后是否复制到剪切板）
    bold = false,             -- boolean（粗体）
    italic = false,           -- boolean（斜体）
    underline = true,        -- boolean（下划线）
    strike = false,           -- boolean（删除线）
    blink = false,            -- boolean（闪烁）
    reverse = false,          -- boolean（反显）
    hidden = false,           -- boolean（隐藏）
    dim = false               -- boolean（暗淡）
  },
  n = 1,                      -- integer（链接内容数量）
}
```

---

## `count`

返回当前链接内容的总数。

### 调用

```lua
widget.link.count
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 链接内容总数 |

### 示例

```lua
widget.link.create("test", "./test.md")
widget.link.create("test", "./test.md")

debug.print(widget.link.count())
```

**输出：**

```lua
2
```

---

## `draw`

绘制链接内容。

### 调用

```lua
widget.link.draw
```

### 参数

#### 必填参数

| 参数名 | 类型    | 说明                        |
| ------ | ------- | --------------------------- |
| `id`   | string  | 链接内容 ID                 |
| `x`    | integer | 链接内容左上角位置的 x 坐标 |
| `y`    | integer | 链接内容左上角位置的 y 坐标 |

#### 选填参数

| 参数名        | 类型   | 默认值   | 说明             |
| ------------- | ------ | -------- | ---------------- |
| `slice_layer` | string | `"base"` | 绘制目标图层切片 |

### 返回值

无返回值。

### 示例

```lua
function Init(ctx)
  link = widget.link.create("Tui Game", "https://github.com/MXBraisedFish/tui-game")
end

function Render()
  widget.link.draw(link, 5, 2)
end
```

**输出：**

![widget_link_create_example](../../image/widget_link_create_example.png)

### 额外说明

- 必填参数 `x` 取值范围为 $[-2147483648, 2147483647]$。
- 必填参数 `y` 取值范围为 $[-2147483648, 2147483647]$。

---

## `click`

模拟点击链接内容。

### 调用

```lua
widget.link.click
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `id`   | string | 链接内容 ID |

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| boolean | 点击请求是否受理 |

### 示例

```lua
local link = widget.link.create("Tui Game", "https://github.com/MXBraisedFish/tui-game", { copy = true })
debug.print(widget.link.click(link))
```

---

## `set`

修改链接内容的参数。

### 调用

```lua
widget.link.set
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `id`   | string | 链接内容 ID |

#### 选填参数

| 参数        | 类型                 | 默认值   | 含义                   |
| ----------- | -------------------- | -------- | ---------------------- |
| `text`      | string               | 保持原值 | 链接展示文本           |
| `link`      | string               | 保持原值 | 链接目标               |
| `fg`        | string / const-color | 保持原值 | 前景色                 |
| `bg`        | string / const-color | 保持原值 | 背景色                 |
| `hover_fg`  | string / const-color | 保持原值 | 指针悬浮前景色         |
| `hover_bg`  | string / const-color | 保持原值 | 指针悬浮背景色         |
| `mouse`     | boolean              | 保持原值 | 是否允许鼠标操作       |
| `copy`      | boolean              | 保持原值 | 执行后是否复制到剪切板 |
| `bold`      | boolean              | 保持原值 | 粗体                   |
| `italic`    | boolean              | 保持原值 | 斜体                   |
| `underline` | boolean              | 保持原值 | 下划线                 |
| `strike`    | boolean              | 保持原值 | 删除线                 |
| `blink`     | boolean              | 保持原值 | 闪烁                   |
| `reverse`   | boolean              | 保持原值 | 反显                   |
| `hidden`    | boolean              | 保持原值 | 隐藏                   |
| `dim`       | boolean              | 保持原值 | 暗淡                   |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 修改是否成功 |

### 示例

```lua
local link = widget.link.create("test", "./test.md")

debug.print(widget.link.set(link, { fg = color.BRIGHT_GREEN, hover_fg = color.BRIGHT_RED }))

debug.print(table.pretty(widget.link.get_info(link)))
```

**输出：**

```lua
true
{
  id = "link_001",
  text = "test",
  link = "./test.md",
  fg = "bright_green",
  bg = "none",
  hover_fg = "bright_red",
  hover_bg = "none",
  mouse = true,
  copy = false,
  bold = false,
  italic = false,
  underline = true,
  strike = false,
  blink = false,
  reverse = false,
  hidden = false,
  dim = false
}
```

---

## `get_info`

获取链接内容的完整信息表。

### 调用

```lua
widget.link.get_info
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `id`   | string | 链接内容 ID |

### 返回值

**该链接内容存在时**，返回一个值。

| 类型  | 说明                 |
| ----- | -------------------- |
| table | 该链接内容信息对象表 |

**该链接内容不存在时**，返回一个值。

| 类型 | 说明             |
| ---- | ---------------- |
| nil  | 不存在该链接内容 |

### 示例

```lua
local link = widget.link.create("test", "./test.md")

debug.print(table.pretty(widget.link.get_info(link)))
```

**输出：**

```lua
{
  id = "link_001",
  text = "test",
  link = "./test.md",
  fg = "bright_blue",
  bg = "none",
  hover_fg = "bright_blue",
  hover_bg = "none",
  mouse = true,
  copy = false,
  bold = false,
  italic = false,
  underline = true,
  strike = false,
  blink = false,
  reverse = false,
  hidden = false,
  dim = false
}
```

### 额外说明

- 返回对象表包含以下字段：

```lua
{
  id = "link_001",          -- string（链接内容 ID）
  text = "test",            -- string（链接展示文本）
  link = "./test.md",       -- string（链接目标）
  fg = "bright_blue",       -- string（前景色）
  bg = "none",              -- string（背景色）
  hover_fg = "bright_blue", -- string（指针悬浮前景色）
  hover_bg = "none",        -- string（指针悬浮背景色）
  mouse = true,             -- boolean（是否允许鼠标操作）
  copy = false,             -- boolean（执行后是否复制到剪切板）
  bold = false,             -- boolean（粗体）
  italic = false,           -- boolean（斜体）
  underline = true,         -- boolean（下划线）
  strike = false,           -- boolean（删除线）
  blink = false,            -- boolean（闪烁）
  reverse = false,          -- boolean（反显）
  hidden = false,           -- boolean（隐藏）
  dim = false               -- boolean（暗淡）
}
```

---

## `exists`

检查链接内容是否存在。

### 调用

```lua
widget.link.exists
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明        |
| ------ | ------ | ----------- |
| `id`   | string | 链接内容 ID |

### 返回值

返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 该链接内容是否存在 |

### 示例

```lua
local link = widget.link.create("test", "./test.md")
debug.print(widget.link.exists(link))

widget.link.delete(link)
debug.print(widget.link.exists(link))
```

**输出：**

```lua
true
false
```

`mouse` 默认为 `true`。设置为 `false` 后，链接继续绘制，但不参与鼠标命中、不应用悬浮样式，也不响应鼠标点击；关闭时会取消尚未完成的鼠标按下状态。`widget.link.click(id)` 不受此开关影响。`set` 省略 `mouse` 时保留原值。
