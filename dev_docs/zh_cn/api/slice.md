# slice 库

`slice` 用于创建和管理图层切片对象。

---

# 目录

## 方法

| 方法             | 说明                     | 定位                              |
| ---------------- | ------------------------ | --------------------------------- |
| `create`         | 创建一个图层切片对象     | [create](#create)                 |
| `delete`         | 删除指定图层切片         | [delete](#delete)                 |
| `clear`          | 删除所有图层切片         | [clear](#clear)                   |
| `list`           | 获取所有图层切片信息     | [list](#list)                     |
| `count`          | 返回当前图层切片的总数   | [count](#count)                   |
| `draw`           | 绘制图层切片             | [draw](#draw)                     |
| `set`            | 修改图层切片的参数       | [set](#set)                       |
| `get_info`       | 获取图层切片的完整信息表 | [get_info](#get_info)             |
| `exists`         | 检查图层切片是否存在     | [exists](#exists)                 |

---

# 方法

## `create`

创建一个图层切片对象。

### 限制

最多可同时存在 1024 个图层切片；超出后 `slice.create` 会抛出错误。

### 调用

```lua
slice.create
```

## 参数

### 必填参数

| 参数名 | 类型 | 说明 |
| --- | --- | --- |
| `width` | integer | 图层切片宽度，范围 1～65535 |
| `height` | integer | 图层切片高度，范围 1～65535 |

### 选填参数

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `bg` | string / const-color | `color.NONE` | 图层切片背景 |
| `layer` | integer | 自动向上递增 | 图层层级；正整数，小于 1 会抛出错误 |

## 返回值

返回一个值。

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

## 额外说明

- 图层层级之间不允许空洞图层，参数 `layer` 超过最大层级时自动修正为置顶。
- 插入层级会自动将后面的图层切片层级向上递增。
- 创建只保存图层切片对象及其配置，不会自动将其绘制到画布。
- 图层切片 ID 的格式为 `slice_` 加正整数，例如 `slice_001`。
- 必填参数 `width` 取值范围为 $[1, 65535]$。
- 必填参数 `height` 取值范围为 $[1, 65535]$。
- 选填参数 `layer` 取值范围为 $[1, 2147483647]$。

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
| `id` | string | 图层切片 ID |

## 返回值

返回一个值。

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

## 额外说明

- `"base"` 图层不可删除，传入 `"base"` 时返回 `false`。

---

## `clear`

删除所有图层切片。

### 调用

```lua
slice.clear
```

## 返回值

返回一个值。

| 类型    | 说明        |
| ------- | ----------- |
| boolean | 恒为 `true` |

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

## 额外说明

- 返回值混合表结构如下：

```lua
local slices = {
  {
    id = "slice_001", -- 示例 ID；以 slice.create 的返回值为准
    width = 20, -- integer
    height = 10, -- integer
    layer = 1, -- integer
    bg = "yellow", -- string
  },
  n = 1, -- integer，切片数量
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

返回一个值。

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

无返回值。

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

## 额外说明

- `slice.draw` 的提交仅对当前帧有效。需要持续显示的图层切片应当每帧调用一次。
- `slice.draw` 的位置可以为负数；超出当前 base 的部分会裁剪，绘制仍按切片的局部坐标进行。
- 必填参数 `x` 取值范围为 $[-2147483648, 2147483647]$。
- 必填参数 `y` 取值范围为 $[-2147483648, 2147483647]$。

---

## `set`

修改图层切片的参数。

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

| 参数名 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `width` | integer | 保持原值 | 图层切片宽度，范围 1～65535 |
| `height` | integer | 保持原值 | 图层切片高度，范围 1～65535 |
| `bg` | string / const-color | 保持原值 | 图层切片背景 |
| `layer` | integer | 保持原值 | 图层层级；正整数，小于 1 会抛出错误 |

## 返回值

返回一个值。

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

## 额外说明

- `"base"` 图层不可修改。
- 选填参数 `width` 取值范围为 $[1, 65535]$。
- 选填参数 `height` 取值范围为 $[1, 65535]$。
- 选填参数 `layer` 取值范围为 $[1, 2147483647]$，超过图层总数时自动修正为置顶。

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

**对象存在时**，返回一个表；**不存在时**，返回 `nil`。

| 类型 | 说明 |
| --- | --- |
| table / nil | 切片信息 |

### 示例

```lua
local s = slice.create(20, 10, {bg = color.YELLOW})

debug.print(table.pretty(slice.get_info(s)))
```

**输出：**

```lua
```

## 额外说明

返回表包含以下字段：

| 字段     | 类型    | 说明         |
| -------- | ------- | ------------ |
| `id`     | string  | 图层切片 ID  |
| `width`  | integer | 图层切片宽度 |
| `height` | integer | 图层切片高度 |
| `bg`     | string  | 图层切片背景 |
| `layer`  | integer | 图层层级     |

- 请先判断查询结果是否为 `nil`，再读取字段或参与计算。
- 传入 `"base"` 时返回 base 图层的信息表，其中 `id` 为 `"base"`、`layer` 为 `0`、`bg` 为 `color.TRANSPARENT`，`width` 和 `height` 为 base 图层的尺寸。

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

返回一个值。

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

## 额外说明

- `"base"` 图层始终存在，传入 `"base"` 时返回 `true`。
