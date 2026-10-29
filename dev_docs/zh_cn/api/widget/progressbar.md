# widget.progressbar 库

`widget.progressbar` 用于设置进度条。

- 从左向右填充。`completed_ratio` 和 `preview_ratio` 均从左端独立计算，已完成区域覆盖预完成区域。例如 `completed_ratio=0.3`、`preview_ratio=0.5`，显示已完成 30%、额外预完成 20%、剩余 50%。
- 各部分格数按完整宽度乘以比例后向下取整；裁剪不会改变原始宽度或填充比例。
- 比例必须是 `[0, 1]` 内的有限数，非法参数会报错；`set` 验证全部字段后才更新，省略或 nil 保持原值。
- `slice_layer` 可指定普通切片、scrollbox 或 Markdown 文档。进度条遵循所在容器的绘制顺序和裁剪规则，并计入滚动内容范围。


> 进度条属于绘制组件，高度固定为一行，不产生鼠标交互事件。

---

# 目录

## 方法

| 方法       | 说明                   | 定位                  |
| ---------- | ---------------------- | --------------------- |
| `create`   | 创建一个进度条对象     | [create](#create)     |
| `delete`   | 删除指定进度条         | [delete](#delete)     |
| `clear`    | 删除所有进度条         | [clear](#clear)       |
| `list`     | 获取所有进度条信息     | [list](#list)         |
| `count`    | 返回当前进度条的总数   | [count](#count)       |
| `draw`     | 绘制进度条             | [draw](#draw)         |
| `set`      | 修改进度条的参数       | [set](#set)           |
| `get_info` | 获取进度条的完整信息表 | [get_info](#get_info) |
| `exists`   | 检查进度条是否存在     | [exists](#exists)     |

---

# 方法

## `create`

创建一个进度条对象。

### 调用

```lua
widget.progressbar.create
```

### 参数

#### 必填参数

| 参数名  | 类型    | 说明       |
| ------- | ------- | ---------- |
| `width` | integer | 进度条宽度 |

#### 选填参数

| 参数              | 类型                 | 默认值              | 含义                 |
| ----------------- | -------------------- | ------------------- | -------------------- |
| `completed_ratio` | integer / float      | `0`                 | 已完成部分占比       |
| `completed_shape` | string               | `"█"`               | 已完成部分进度条形状 |
| `completed_fg`    | string / const-color | `color.GREEN`       | 已完成部分进度条颜色 |
| `preview_ratio`   | integer / float      | `0`                 | 预完成部分占比       |
| `preview_shape`   | string               | `"█"`               | 预完成部分进度条形状 |
| `preview_fg`      | string / const-color | `color.BRIGHT_BLUE` | 预完成部分进度条颜色 |
| `remaining_shape` | string               | `"─"`               | 未完成部分进度条形状 |
| `remaining_fg`    | string / const-color | `color.WHITE`       | 未完成部分进度条颜色 |

### 返回值

返回一个值。

| 类型   | 说明      |
| ------ | --------- |
| string | 进度条 ID |

### 示例

```lua
function Init(ctx)
  progressbar = widget.progressbar.create(20, { completed_ratio = 0.3, preview_ratio = 0.5 })
end

function Render()
  widget.progressbar.draw(progressbar, 5, 2)
end
```

**输出：**

![widget_progressbar_create_example](../../image/widget_progressbar_create_example.png)

### 额外说明

- 必填参数 `width` 取值范围为 $[1, 65535]$。
- 选填参数 `completed_ratio` 取值范围为 $[0, 1]$。
- 选填参数 `preview_ratio` 取值范围为 $[0, 1]$。
- 选填参数 `completed_shape`、选填参数 `preview_shape` 和选填参数 `remaining_shape` 填写时每个字段必须宽度为 `1` 的字符。

---

## `delete`

删除指定进度条。

### 调用

```lua
widget.progressbar.delete
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 进度条 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
local progressbar = widget.progressbar.create(20)
debug.print(progressbar)

debug.print(widget.progressbar.delete(progressbar))

debug.print(table.pretty(widget.progressbar.list()))
```

**输出：**

```lua
progressbar_001
true
{
  n = 0
}
```

---

## `clear`

删除所有进度条。

### 调用

```lua
widget.progressbar.clear
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
widget.progressbar.create(20)
widget.progressbar.create(30)
widget.progressbar.create(40)
widget.progressbar.create(50)

debug.print(widget.progressbar.clear())

debug.print(table.pretty(widget.progressbar.list()))
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

获取所有进度条信息。

### 调用

```lua
widget.progressbar.list
```

### 返回值

返回一个值。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 所有进度条信息的混合表 |

### 示例

```lua
local progressbar = widget.progressbar.create(20)

debug.print(table.pretty(widget.progressbar.list()))
```

**输出：**

```lua
{
  [1] = {
    id = "progressbar_001",
    width = 20,
    completed_ratio = 0,
    completed_shape = "█",
    completed_fg = "green",
    preview_ratio = 0,
    preview_shape = "█",
    preview_fg = "bright_blue",
    remaining_shape = "─",
    remaining_fg = "white",
  },
  n = 1
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  {
    id = "progressbar_001",     -- string（进度条 ID）
    width = 20,                 -- integer（进度条宽度）
    completed_ratio = 0,        -- integer / float（已完成部分占比）
    completed_shape = "█",      -- string（已完成部分进度条形状）
    completed_fg = "green",     -- string（已完成部分进度条颜色）
    preview_ratio = 0,          -- integer / float（预完成部分占比）
    preview_shape = "█",        -- string（预完成部分进度条形状）
    preview_fg = "bright_blue", -- string（预完成部分进度条颜色）
    remaining_shape = "─",      -- string（未完成部分进度条形状）
    remaining_fg = "white",     -- string（未完成部分进度条颜色）
  },
  n = 1,                        -- integer（进度条数量）
}
```

---

## `count`

返回当前进度条的总数。

### 调用

```lua
widget.progressbar.count
```

### 返回值

返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 进度条总数 |

### 示例

```lua
widget.progressbar.create(20)
widget.progressbar.create(30)

debug.print(widget.progressbar.count())
```

**输出：**

```lua
2
```

---

## `draw`

绘制进度条。

### 调用

```lua
widget.progressbar.draw
```

### 参数

#### 必填参数

| 参数名 | 类型    | 说明                      |
| ------ | ------- | ------------------------- |
| `id`   | string  | 进度条 ID                 |
| `x`    | integer | 进度条左上角位置的 x 坐标 |
| `y`    | integer | 进度条左上角位置的 y 坐标 |

#### 选填参数

| 参数名        | 类型   | 默认值   | 说明             |
| ------------- | ------ | -------- | ---------------- |
| `slice_layer` | string | `"base"` | 绘制目标图层切片 |

### 返回值

无返回值。

### 示例

```lua
function Init(ctx)
  progressbar = widget.progressbar.create(20, { completed_ratio = 0.3, preview_ratio = 0.5 })
end

function Render()
  widget.progressbar.draw(progressbar, 5, 2)
end
```

**输出：**

![widget_progressbar_create_example](../../image/widget_progressbar_create_example.png)

### 额外说明

- 必填参数 `x` 取值范围为 $[-2147483648, 2147483647]$。
- 必填参数 `y` 取值范围为 $[-2147483648, 2147483647]$。

---

## `set`

修改进度条的参数。

### 调用

```lua
widget.progressbar.set
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 进度条 ID |

#### 选填参数

| 参数              | 类型                 | 默认值   | 含义                 |
| ----------------- | -------------------- | -------- | -------------------- |
| `width`           | integer              | 保持原值 | 进度条宽度           |
| `completed_ratio` | integer / float      | 保持原值 | 已完成部分占比       |
| `completed_shape` | string               | 保持原值 | 已完成部分进度条形状 |
| `completed_fg`    | string / const-color | 保持原值 | 已完成部分进度条颜色 |
| `preview_ratio`   | integer / float      | 保持原值 | 预完成部分占比       |
| `preview_shape`   | string               | 保持原值 | 预完成部分进度条形状 |
| `preview_fg`      | string / const-color | 保持原值 | 预完成部分进度条颜色 |
| `remaining_shape` | string               | 保持原值 | 未完成部分进度条形状 |
| `remaining_fg`    | string / const-color | 保持原值 | 未完成部分进度条颜色 |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 修改是否成功 |

### 示例

```lua
local progressbar = widget.progressbar.create(20)

debug.print(widget.progressbar.set(progressbar, { width = 25, completed_ratio = 0.3 }))

debug.print(table.pretty(widget.progressbar.get_info(progressbar)))
```

**输出：**

```lua
true
{
  id = "progressbar_001",
  width = 25,
  completed_ratio = 0.3,
  completed_shape = "█",
  completed_fg = "green",
  preview_ratio = 0,
  preview_shape = "█",
  preview_fg = "bright_blue",
  remaining_shape = "─",
  remaining_fg = "white",
}
```

### 额外说明

- 选填参数 `width` 取值范围为 $[1, 65535]$。
- 选填参数 `completed_ratio` 取值范围为 $[0, 1]$。
- 选填参数 `preview_ratio` 取值范围为 $[0, 1]$。
- 选填参数 `completed_shape`、选填参数 `preview_shape` 和选填参数 `remaining_shape` 填写时每个字段必须宽度为 `1` 的字符。

---

## `get_info`

获取进度条的完整信息表。

### 调用

```lua
widget.progressbar.get_info
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 进度条 ID |

### 返回值

**该进度条存在时**，返回一个值。

| 类型  | 说明               |
| ----- | ------------------ |
| table | 该进度条信息对象表 |

**该进度条不存在时**，返回一个值。

| 类型 | 说明           |
| ---- | -------------- |
| nil  | 不存在该进度条 |

### 示例

```lua
local progressbar = widget.progressbar.create(20)

debug.print(table.pretty(widget.progressbar.get_info(progressbar)))
```

**输出：**

```lua
{
  id = "progressbar_001",
  width = 20,
  completed_ratio = 0,
  completed_shape = "█",
  completed_fg = "green",
  preview_ratio = 0,
  preview_shape = "█",
  preview_fg = "bright_blue",
  remaining_shape = "─",
  remaining_fg = "white",
}
```

### 额外说明

- 返回对象表包含以下字段：

```lua
{
  id = "progressbar_001",     -- string（进度条 ID）
  width = 20,                 -- integer（进度条宽度）
  completed_ratio = 0,        -- integer / float（已完成部分占比）
  completed_shape = "█",      -- string（已完成部分进度条形状）
  completed_fg = "green",     -- string（已完成部分进度条颜色）
  preview_ratio = 0,          -- integer / float（预完成部分占比）
  preview_shape = "█",        -- string（预完成部分进度条形状）
  preview_fg = "bright_blue", -- string（预完成部分进度条颜色）
  remaining_shape = "─",      -- string（未完成部分进度条形状）
  remaining_fg = "white",     -- string（未完成部分进度条颜色）
}
```

---

## `exists`

检查进度条是否存在。

### 调用

```lua
widget.progressbar.exists
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 进度条 ID |

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| boolean | 该进度条是否存在 |

### 示例

```lua
local progressbar = widget.progressbar.create(20)
debug.print(widget.progressbar.exists(progressbar))

widget.progressbar.delete(progressbar)
debug.print(widget.progressbar.exists(progressbar))
```

**输出：**

```lua
true
false
```
