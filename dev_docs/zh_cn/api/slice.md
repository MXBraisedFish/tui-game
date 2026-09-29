# slice 库

`slice` 用于创建和管理图层切片对象。

---

# 目录

## 方法

| 方法           | 说明                     | 定位                              |
| ---------------- | ------------------------ | --------------------------------- |
| `create`         | 创建一个图层切片对象     | [create](#create)                 |
| `delete`         | 删除指定图层切片         | [delete](#delete)                 |
| `clear`          | 删除所有图层切片         | [clear](#clear)                   |
| `list`           | 获取所有图层切片信息     | [list](#list)                     |
| `count`          | 返回当前图层切片的总数   | [count](#count)                   |
| `draw`           | 绘制图层切片             | [draw](#draw)                     |
| `set`            | 修改生成器的参数         | [set](#set)                       |
| `set_size`       | 修改图层切片的宽度和高度 | [set_size](#set_size)             |
| `set_width`      | 修改图层切片的宽度       | [set_width](#set_width)           |
| `set_height`     | 修改图层切片的高度       | [set_height](#set_height)         |
| `set_layer`      | 修改图层切片的图层层级   | [set_layer](#set_layer)           |
| `set_background` | 修改图层切片的背景颜色   | [set_background](#set_background) |
| `get_size`       | 获取图层切片的宽度和高度 | [get_size](#get_size)             |
| `get_width`      | 获取图层切片的宽度       | [get_width](#get_width)           |
| `get_height`     | 获取图层切片的高度       | [get_height](#get_height)         |
| `get_layer`      | 获取图层切片的图层层级   | [get_layer](#get_layer)           |
| `get_background` | 获取图层切片的背景颜色   | [get_background](#get_background) |
| `get_info`       | 获取图层切片的完整信息表 | [get_info](#get_info)             |
| `exists`         | 检查图层切片是否存在     | [exists](#exists)                 |

---

# 方法

## `create`

创建一个图层切片对象。

### 调用

```lua
slice.create
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 图层切片宽度 |
| `height` | integer | 图层切片高度 |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `bg` | string / const-color | 默认：`color.NONE`；图层切片背景 |
| `layer` | integer | 默认：自动向上递增；图层层级 |
## 返回值

直接返回一个值。

| 类型   | 说明        |
| ------ | ----------- |
| string | 图层切片 ID |

### 示例

```lua
function Init(ctx)
  s = slice.create(20, 10, {bg = color.YELLOW})
  debug.print(table.pretty(slice.get_info(s)))
end

function Render()
  slice.draw(s, 5, 2)
end
```

**输出：**

```lua
```

### 额外说明

- 图层层级之间不允许空洞图层，参数 `layer` 超过最大层级时自动修正为置顶。
- 插入层级会自动将后面的图层切片层级向上递增。
- 创建只保存图层切片对象及其配置，不会自动将其绘制到画布。
- 图层切片仅在当前帧显式调用 `slice.draw` 后参与该帧合成；下一帧需要再次调用。

---

## `delete`

删除指定图层切片。

### 调用

```lua
slice.delete
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片对象 ID |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否删除成功 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})
debug.print(s)

debug.print(slice.delete(s))

debug.print(table.pretty(slice.list()))
```

**输出：**

```lua
```

---

## `clear`

删除所有图层切片。

### 调用

```lua
slice.clear
```

## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否删除成功 |

### 示例

```lua
slice.create(20, 10, {bg = color.YELLOW})
slice.create(30, 8, {bg = color.RED})
slice.create(40, 6, {bg = color.GREEN})
slice.create(50, 4, {bg = color.BLUE})

debug.print(slice.clear())

debug.print(table.pretty(slice.list()))
```

**输出：**

```lua
```

---

## `list`

获取所有图层切片信息。

### 调用

```lua
slice.list
```

## 返回值

返回一个混合表。

| 类型  | 说明             |
| ----- | ---------------- |
| table | 所有图层切片信息 |

### 示例

```lua
local s1 = slice.create(20, 10, {bg = color.YELLOW})
local s2 = slice.create(30, 8, {bg = color.RED})

debug.print(table.pretty(slice.list()))
```

**输出：**

```lua
```

### 额外说明

- 返回值混合表结构如下：

```lua
local layers = {
  {
    id = "background", -- string
    width = 80, -- integer
    height = 24, -- integer
    layer = 1, -- integer
    bg = "#000000", -- string
  },
  n = 1, -- integer; the number of layers
}
```

- 返回值数组表按照图层切片层级依次排序。
- 返回值数组表不包含 `"base"` 图层。

---

## `count`

返回当前图层切片的总数。

### 调用

```lua
slice.count
```

## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 图层切片总数 |

### 示例

```lua
slice.create(20, 10, {bg = color.YELLOW})
slice.create(30, 8, {bg = color.RED})

debug.print(slice.count())
```

**输出：**

```lua
```

---

## `draw`

绘制图层切片。

### 调用

```lua
slice.draw
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `x` | integer | 图层切片左上角位置的 x 坐标 |
| `y` | integer | 图层切片左上角位置的 y 坐标 |
## 返回值

无。

### 示例

```lua
function Init(ctx)
  s = slice.create(20, 10, {bg = color.YELLOW})
end

function Render()
  slice.draw(s, 5, 2)
end
```

**输出：**

```lua
```

### 额外说明

- `slice.draw` 的提交仅对当前帧有效。需要持续显示的图层切片应当每帧调用一次。

---

## `set`

修改生成器的参数。

### 调用

```lua
slice.set
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |

### 选填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 默认：保持原值；图层切片宽度 |
| `height` | integer | 默认：保持原值；图层切片高度 |
| `bg` | string / const-color | 默认：保持原值；图层切片背景 |
| `layer` | integer | 默认：保持原值；图层层级 |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
local s1 = slice.create(20, 10, {bg = color.YELLOW})
local s2 = slice.create(30, 8, {bg = color.RED})

debug.print(slice.set(s1, {width = 25, height = 5, bg = color.BLUE}))

debug.print(table.pretty(slice.get_info(s1)))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层不可修改。

---

## `set_size`

修改图层切片的宽度和高度。

### 调用

```lua
slice.set_size
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `width` | integer | 默认：保持原值；图层切片宽度 |
| `height` | integer | 默认：保持原值；图层切片高度 |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.set_size(s, 30, 6))

debug.print(table.pretty(slice.get_info(s)))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层不可修改。

---

## `set_width`

修改图层切片的宽度。

### 调用

```lua
slice.set_width
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `width` | integer | 默认：保持原值；图层切片宽度 |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.set_width(s, 30))

debug.print(slice.get_width(s))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层不可修改。

---

## `set_height`

修改图层切片的高度。

### 调用

```lua
slice.set_height
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `height` | integer | 默认：保持原值；图层切片高度 |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.set_height(s, 6))

debug.print(slice.get_height(s))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层不可修改。

---

## `set_layer`

修改图层切片的图层层级。

### 调用

```lua
slice.set_layer
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `layer` | integer | 默认：保持原值；图层层级 |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
function Init(ctx)
  s1 = slice.create(20, 10, {bg = color.YELLOW})
  s2 = slice.create(30, 8, {bg = color.RED})

  debug.print(slice.set_layer(s1, 2))

  debug.print(table.pretty(slice.list()))
end

function Render()
  slice.draw(s1, 0, 0) -- 绘制顺序不影响图层顺序
  slice.draw(s2, 0, 0)
end
```

**输出：**

```lua
```

### 额外说明

- 图层层级之间不允许空洞图层，参数 `layer` 超过最大层级时自动修正为置顶。
- 插入层级会自动将后面的图层切片层级向上递增。
- `"base"` 图层不可修改。

---

## `set_background`

修改图层切片的背景颜色。

### 调用

```lua
slice.set_background
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
| `bg` | string / const-color | 默认：保持原值；图层切片背景 |
## 返回值

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 是否修改成功 |

### 示例

```lua
function Init(ctx)
  s = slice.create(20, 10, {bg = color.YELLOW})

  debug.print(slice.set_background(s, color.RED))

  debug.print(slice.get_background(s))
end

function Render()
  slice.draw(s, 0, 0)
end
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层不可修改。

---

## `get_size`

获取图层切片的宽度和高度。

### 调用

```lua
slice.get_size
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

返回两个值，依次为图层切片宽度和高度；切片不存在时返回一个 nil。

| 值名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 图层切片宽度。 |
| `height` | integer | 图层切片高度。 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})
local width, height = slice.get_size(s)
debug.print(tostring(width) .. ", " .. tostring(height))
```

**输出：**

```lua
```

---

## `get_width`

获取图层切片的宽度。

### 调用

```lua
slice.get_width
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 图层切片宽度 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.get_width(s))
```

**输出：**

```lua
```

---

## `get_height`

获取图层切片的高度。

### 调用

```lua
slice.get_height
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

直接返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| integer | 图层切片高度 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.get_height(s))
```

**输出：**

```lua
```

---

## `get_layer`

获取图层切片的图层层级。

### 调用

```lua
slice.get_layer
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| integer | 图层层级 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.get_layer(s))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层层级为 `0`。

---

## `get_background`

获取图层切片的背景颜色。

### 调用

```lua
slice.get_background
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

直接返回一个值。

| 类型   | 说明     |
| ------ | -------- |
| string | 背景颜色 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(slice.get_background(s))
```

**输出：**

```lua
```

### 额外说明

- `"base"` 图层背景颜色为 `color.TRANSPARENT`。

---

## `get_info`

获取图层切片的完整信息表。

### 调用

```lua
slice.get_info
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

返回一个对象表。

| 字段     | 类型    | 说明         |
| -------- | ------- | ------------ |
| `id`     | string  | 图层切片 ID  |
| `width`  | integer | 图层切片宽度 |
| `height` | integer | 图层切片高度 |
| `bg`     | string  | 图层切片背景 |
| `layer`  | integer | 图层层级     |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(table.pretty(slice.get_info(s)))
```

**输出：**

```lua
```

---

## `exists`

检查图层切片是否存在。

### 调用

```lua
slice.exists
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `id` | string | 图层切片 ID |
## 返回值

直接返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 是否存在 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})
debug.print(slice.exists(s))

slice.delete(s)
debug.print(slice.exists(s))
```

**输出：**

```lua
```
