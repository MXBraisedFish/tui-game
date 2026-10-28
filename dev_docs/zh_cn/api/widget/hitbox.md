# widget.hitbox 库

`widget.hitbox` 用于设置鼠标响应区域。

> 鼠标响应区域由绘制组件和交互组件构成。

---

# 目录

## 方法

| 方法       | 说明                         | 定位                  |
| ---------- | ---------------------------- | --------------------- |
| `create`   | 创建一个鼠标响应区域对象     | [create](#create)     |
| `delete`   | 删除指定鼠标响应区域         | [delete](#delete)     |
| `clear`    | 删除所有鼠标响应区域         | [clear](#clear)       |
| `list`     | 获取所有鼠标响应区域信息     | [list](#list)         |
| `count`    | 返回当前鼠标响应区域的总数   | [count](#count)       |
| `draw`     | 绘制鼠标响应区域             | [draw](#draw)         |
| `set`      | 修改鼠标响应区域的参数       | [set](#set)           |
| `get_info` | 获取鼠标响应区域的完整信息表 | [get_info](#get_info) |
| `exists`   | 检查鼠标响应区域是否存在     | [exists](#exists)     |

---

# 方法

## `create`

创建一个鼠标响应区域对象。

### 调用

```lua
widget.hitbox.create
```

### 参数

#### 必填参数

| 参数名   | 类型    | 说明             |
| -------- | ------- | ---------------- |
| `width`  | integer | 鼠标响应区域宽度 |
| `height` | integer | 鼠标响应区域高度 |

#### 选填参数

| 参数         | 类型 | 默认值  | 含义                                          |
| ------------ | ---- | ------- | --------------------------------------------- |
| `hover_move` | bool | `false` | 鼠标在区域内移动时，是否产生 `HoverMove` 事件 |
| `drag`       | bool | `false` | 按住鼠标拖动时，是否产生 `Drag` 事件          |

### 返回值

返回一个值。

| 类型   | 说明            |
| ------ | --------------- |
| string | 鼠标响应区域 ID |

### 示例

```lua
function Init(ctx)
  hitbox = widget.hitbox.create(20, 10)
end

function HandleEvent(event)
  if event.type == "hitbox" then
    debug.print(table.pretty(event))
  end
end

function Render()
  widget.hitbox.draw(hitbox, 5, 2)
end
```

**输出：**

```lua
{
  type = "hitbox"
  frame = X,    -- 占位符
  sequence = X, -- 占位符
  data = {
    id = X,     -- 占位符
    kind = "click",
    button = "left",
    x = 12,
    y = 4
    relative_x = 7,
    relative_y = 2,
  }
}
```

### 额外说明

- 必填参数 `width` 取值范围为 $[1, 65535]$。
- 必填参数 `height` 取值范围为 $[1, 65535]$。

---

## `delete`

删除指定鼠标响应区域。

### 调用

```lua
widget.hitbox.delete
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 鼠标响应区域 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
local hitbox = widget.hitbox.create(20, 10)
debug.print(hitbox)

debug.print(widget.hitbox.delete(hitbox))

debug.print(table.pretty(widget.hitbox.list()))
```

**输出：**

```lua
hitbox_001
true
{
  n = 0
}
```

---

## `clear`

删除所有鼠标响应区域。

### 调用

```lua
widget.hitbox.clear
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
widget.hitbox.create(20, 10)
widget.hitbox.create(30, 8)
widget.hitbox.create(40, 6)
widget.hitbox.create(50, 4)

debug.print(widget.hitbox.clear())

debug.print(table.pretty(widget.hitbox.list()))
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

获取所有鼠标响应区域信息。

### 调用

```lua
widget.hitbox.list
```

### 返回值

返回一个值。

| 类型  | 说明                         |
| ----- | ---------------------------- |
| table | 所有鼠标响应区域信息的混合表 |

### 示例

```lua
local hitbox1 = widget.hitbox.create(20, 10)
local hitbox2 = widget.hitbox.create(30, 8, { hover_move = true })

debug.print(table.pretty(widget.hitbox.list()))
```

**输出：**

```lua
{
  [1] = {
    id = "hitbox_001",
    width = 20,
    height = 10,
    hover_move = false,
    drag = false
  },
  [2] = {
    id = "hitbox_002",
    width = 30,
    height = 8,
    hover_move = true,
    drag = false
  },
  n = 2
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  {
    id = "hitbox_001", -- string（鼠标响应区域 ID）
    width = 20,        -- integer（鼠标响应区域宽度）
    height = 10,       -- integer（鼠标响应区域高度）
    hover_move = true, -- boolean（鼠标响应区域移动事件）
    drag = false       -- boolean（鼠标响应区域拖动事件）
  },
  n = 1,               -- integer（鼠标响应区域数量）
}
```

---

## `count`

返回当前鼠标响应区域的总数。

### 调用

```lua
widget.hitbox.count
```

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 鼠标响应区域总数 |

### 示例

```lua
widget.hitbox.create(20, 10)
widget.hitbox.create(30, 8)

debug.print(widget.hitbox.count())
```

**输出：**

```lua
2
```

---

## `draw`

绘制鼠标响应区域。

### 调用

```lua
widget.hitbox.draw
```

### 参数

#### 必填参数

| 参数名 | 类型    | 说明                            |
| ------ | ------- | ------------------------------- |
| `id`   | string  | 鼠标响应区域 ID                 |
| `x`    | integer | 鼠标响应区域左上角位置的 x 坐标 |
| `y`    | integer | 鼠标响应区域左上角位置的 y 坐标 |

#### 选填参数

| 参数名        | 类型   | 默认值   | 说明             |
| ------------- | ------ | -------- | ---------------- |
| `slice_layer` | string | `"base"` | 绘制目标图层切片 |

### 返回值

无返回值。

### 示例

```lua
function Init(ctx)
  hitbox = widget.hitbox.create(20, 10)
end

function HandleEvent(event)
  if event.type == "hitbox" then
    debug.print(table.pretty(event))
  end
end

function Render()
  widget.hitbox.draw(hitbox, 5, 2)
end
```

**输出：**

```lua
{
  type = "hitbox"
  frame = X,    -- 占位符
  sequence = X, -- 占位符
  data = {
    id = X,     -- 占位符
    kind = "click",
    button = "left",
    x = 12,
    y = 4
    relative_x = 7,
    relative_y = 2,
  }
}
```

### 额外说明

- 必填参数 `x` 取值范围为 $[-2147483648, 2147483647]$。
- 必填参数 `y` 取值范围为 $[-2147483648, 2147483647]$。

---

## `set`

修改鼠标响应区域的参数。

### 调用

```lua
widget.hitbox.set
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 鼠标响应区域 ID |

#### 选填参数

| 参数名       | 类型    | 默认值   | 说明                                          |
| ------------ | ------- | -------- | --------------------------------------------- |
| `width`      | integer | 保持原值 | 鼠标响应区域宽度                              |
| `height`     | integer | 保持原值 | 鼠标响应区域高度                              |
| `hover_move` | bool    | 保持原值 | 鼠标在区域内移动时，是否产生 `HoverMove` 事件 |
| `drag`       | bool    | 保持原值 | 按住鼠标拖动时，是否产生 `Drag` 事件          |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 修改是否成功 |

### 示例

```lua
local hitbox = widget.hitbox.create(20, 10)

debug.print(widget.hitbox.set(hitbox, { width = 25, height = 5 }))

debug.print(table.pretty(widget.hitbox.get_info(hitbox)))
```

**输出：**

```lua
true
{
  id = "hitbox_001",
  width = 25,
  height = 5,
  hover_move = false,
  drag = false
}
```

### 额外说明

- 选填参数 `width` 取值范围为 $[1, 65535]$。
- 选填参数 `height` 取值范围为 $[1, 65535]$。

---

## `get_info`

获取鼠标响应区域的完整信息表。

### 调用

```lua
widget.hitbox.get_info
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 鼠标响应区域 ID |

### 返回值

**该鼠标响应区域存在时**，返回一个值。

| 类型  | 说明                     |
| ----- | ------------------------ |
| table | 该鼠标响应区域信息对象表 |

**该鼠标响应区域不存在时**，返回一个值。

| 类型 | 说明                 |
| ---- | -------------------- |
| nil  | 不存在该鼠标响应区域 |

### 示例

```lua
local hitbox = widget.hitbox.create(20, 10)

debug.print(table.pretty(widget.hitbox.get_info(hitbox)))
```

**输出：**

```lua
{
  id = "hitbox_001",
  width = 20,
  height = 10,
  hover_move = false,
  drag = false
}
```

### 额外说明

- 返回对象表包含以下字段：

```lua
{
  id = "hitbox_001",  -- string（鼠标响应区域 ID）
  width = 20,         -- integer（鼠标响应区域宽度）
  height = 10,        -- integer（鼠标响应区域高度）
  hover_move = false, -- boolean（鼠标响应区域移动事件）
  drag = false        -- boolean（鼠标响应区域拖动事件）
}
```

---

## `exists`

检查鼠标响应区域是否存在。

### 调用

```lua
widget.hitbox.exists
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 鼠标响应区域 ID |

### 返回值

返回一个值。

| 类型    | 说明                   |
| ------- | ---------------------- |
| boolean | 该鼠标响应区域是否存在 |

### 示例

```lua
local hitbox = widget.hitbox.create(20, 10)
debug.print(widget.hitbox.exists(hitbox))

widget.hitbox.delete(hitbox)
debug.print(widget.hitbox.exists(hitbox))
```

**输出：**

```lua
true
false
```
