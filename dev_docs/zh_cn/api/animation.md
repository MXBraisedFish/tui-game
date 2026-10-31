# animation 库

`animation` 提供插值轨道引擎。（编辑中）

---

# 目录

## 常量

| 常量                        | 说明             | 定位                                                    |
| --------------------------- | ---------------- | ------------------------------------------------------- |
| `STEP`                      | 步进插值函数     | [STEP](#step)                                           |
| `LINEAR`                    | 线性插值函数     | [LINEAR](#linear)                                       |
| `EASE_IN_QUAD`              | 缓入快出插值函数 | [EASE_IN_QUAD](#ease_in_quad)                           |
| `EASE_OUT_QUAD`             | 快入缓出插值函数 | [EASE_OUT_QUAD](#ease_out_quad)                         |
| `EASE_IN_OUT_CUBIC`         | 慢快慢插值函数   | [EASE_IN_OUT_CUBIC](#ease_in_out_cubic)                 |
| `EASE_IN_OUT_REVERSE_CUBIC` | 快慢快插值函数   | [EASE_IN_OUT_REVERSE_CUBIC](#ease_in_out_reverse_cubic) |

## 方法

| 方法       | 说明                             | 定位                  |
| ---------- | -------------------------------- | --------------------- |
| `create`   | 创建一个插值轨道引擎对象         | [create](#create)     |
| `delete`   | 删除指定插值轨道引擎             | [delete](#delete)     |
| `clear`    | 删除所有插值轨道引擎             | [clear](#clear)       |
| `list`     | 获取所有插值轨道引擎信息         | [list](#list)         |
| `count`    | 返回当前插值轨道引擎的总数       | [count](#count)       |
| `sample`   | 获取当前插值计算的结果           | [sample](#sample)     |
| `start`    | 开始或继续插值计算               | [start](#start)       |
| `pause`    | 暂停插值计算，保留已经走过的进度 | [pause](#pause)       |
| `reset`    | 清零插值计算进度，回到未启动状态 | [reset](#reset)       |
| `restart`  | 清零插值计算进度，然后重新开始   | [restart](#restart)   |
| `set`      | 修改插值轨道引擎的参数           | [set](#set)           |
| `get_info` | 获取插值轨道引擎的完整信息表     | [get_info](#get_info) |
| `exists`   | 检查插值轨道引擎是否存在         | [exists](#exists)     |

---

# 常量

## `STEP`

步进插值函数。

### 调用

```lua
effect.STEP
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![widget_animation_STEP_example](../image/widget_animation_STEP_example.png)

### 等值

```text
"step"
```

---

## `LINEAR`

步进插值函数。

### 调用

```lua
effect.LINEAR
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![widget_animation_LINEAR_example](../image/widget_animation_LINEAR_example.png)

### 等值

```text
"step"
```

---

# 方法

## `create`

创建一个插值轨道引擎对象。

### 调用

```lua
effect.create
```

### 参数

#### 必填参数

| 参数名   | 类型    | 说明         |
| -------- | ------- | ------------ |
| `width`  | integer | 作用区域宽度 |
| `height` | integer | 作用区域高度 |

#### 选填参数

| 参数                   | 类型           | 默认值          | 含义                 |
| ---------------------- | -------------- | --------------- | -------------------- |
| `process`              | function / nil | `nil`           | 处理函数             |
| `scope`                | const-effect   | `effect.CANVAS` | 作用层级             |
| `include_continuation` | boolean        | `false`         | 占位符是否命中宽字符 |

### 返回值

返回一个值。

| 类型   | 说明            |
| ------ | --------------- |
| string | 插值轨道引擎 ID |

### 示例

```lua
function Init(ctx)
  ef = effect.create(5, 1, {
  process = function(char)
    local r, g, b = random.randint({ min = 0, max = 255 }), random.randint({ min = 0, max = 255 }), random.randint({ min = 0, max = 255 })
    char.fg = color.rgb(r, g, b)
    return char
  end
})
end

function Render()
  draw.text(5, 2, "@@@@@")
  effect.apply(ef, 5, 2)
end
```

**输出：**

![widget_effect_create_example](../../image/widget_effect_create_example.png)

### 额外说明

- 必填参数 `width` 取值范围为 $[1, 65535]$。
- 必填参数 `height` 取值范围为 $[1, 65535]$。

---

## `delete`

删除指定插值轨道引擎。

### 调用

```lua
effect.delete
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 插值轨道引擎 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
local ef = effect.create(20, 10)
debug.print(ef)

debug.print(effect.delete(ef))

debug.print(table.pretty(effect.list()))
```

**输出：**

```lua
effect_001
true
{
  n = 0
}
```

---

## `clear`

删除所有插值轨道引擎。

### 调用

```lua
effect.clear
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
effect.create(20, 10)
effect.create(30, 8)
effect.create(40, 6)
effect.create(50, 4)

debug.print(effect.clear())

debug.print(table.pretty(effect.list()))
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

获取所有插值轨道引擎信息。

### 调用

```lua
effect.list
```

### 返回值

返回一个值。

| 类型  | 说明                         |
| ----- | ---------------------------- |
| table | 所有插值轨道引擎信息的混合表 |

### 示例

```lua
local ef = effect.create(20, 10)

debug.print(table.pretty(effect.list()))
```

**输出：**

```lua
{
  [1] = {
    id = "effect_001",
    width = 20,
    height = 10,
    process = function,
    scope = "canvas",
    include_continuation = false
  },
  n = 1
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  {
    id = "effect_001",           -- string（插值轨道引擎 ID）
    width = 20,                  -- integer（作用区域宽度）
    height = 10,                 -- integer（作用区域高度）
    process = function,          -- function（处理函数）
    scope = "canvas",            -- string（作用层级）
    include_continuation = false -- boolean（占位符是否命中宽字符）
  },
  n = 1,                         -- integer（插值轨道引擎数量）
}
```

---

## `count`

返回当前插值轨道引擎的总数。

### 调用

```lua
effect.count
```

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 插值轨道引擎总数 |

### 示例

```lua
effect.create(20, 10)
effect.create(30, 8)

debug.print(effect.count())
```

**输出：**

```lua
2
```

---

## `sample`

获取当前插值计算的结果。

### 调用

```lua
timer.sample
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

### 返回值

返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 开始或继续是否成功 |

### 示例

```lua
function Init(ctx)
  local id = timer.create(1, { tip = "Get!" })
  debug.print(timer.start(id))
end

function HandleEvent(event)
  if event.type == "timer" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
true
{
  type = "timer",
  frame = X,    -- 占位符
  sequence = X, -- 占位符
  data = {
    id = X,     -- 占位符
    kind = "finished",
    timer_kind = "timer",
    executed_count = 1,
    tip = "Get!"
  }
}
```

---

## `start`

开始或继续插值计算。

### 调用

```lua
timer.start
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

### 返回值

返回一个值。

| 类型    | 说明               |
| ------- | ------------------ |
| boolean | 开始或继续是否成功 |

### 示例

```lua
function Init(ctx)
  local id = timer.create(1, { tip = "Get!" })
  debug.print(timer.start(id))
end

function HandleEvent(event)
  if event.type == "timer" then
    debug.print(table.pretty(event))
  end
end
```

**输出：**

```lua
true
{
  type = "timer",
  frame = X,    -- 占位符
  sequence = X, -- 占位符
  data = {
    id = X,     -- 占位符
    kind = "finished",
    timer_kind = "timer",
    executed_count = 1,
    tip = "Get!"
  }
}
```

---

## `pause`

暂停插值计算，保留已经走过的进度。

### 调用

```lua
timer.pause
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 暂停是否成功 |

### 示例

```lua
local id = timer.create(1)
timer.start(id)
debug.print(timer.pause(id))
```

**输出：**

```lua
true
```

---

## `reset`

清零插值计算进度，回到未启动状态。

### 调用

```lua
timer.reset
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local id = timer.create(1)
timer.start(id)
debug.print(timer.reset(id))
```

**输出：**

```lua
true
```

---

## `restart`

清零插值计算进度，然后重新开始。

### 调用

```lua
timer.restart
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 设置是否成功 |

### 示例

```lua
local id = timer.create(1)
debug.print(timer.restart(id))
```

**输出：**

```lua
true
```

---

## `set`

修改插值轨道引擎的参数。

### 调用

```lua
effect.set
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 插值轨道引擎 ID |

#### 选填参数

| 参数                   | 类型           | 默认值   | 含义                 |
| ---------------------- | -------------- | -------- | -------------------- |
| `width`                | integer        | 保持原值 | 作用区域宽度         |
| `height`               | integer        | 保持原值 | 作用区域高度         |
| `process`              | function / nil | 保持原值 | 处理函数             |
| `scope`                | const-effect   | 保持原值 | 作用层级             |
| `include_continuation` | boolean        | 保持原值 | 占位符是否命中宽字符 |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 修改是否成功 |

### 示例

```lua
local ef = effect.create(20, 10)

debug.print(effect.set(ef, { width = 25, height = 5 }))

debug.print(table.pretty(effect.get_info(ef)))
```

**输出：**

```lua
true
{
  id = "effect_001",
  width = 25,
  height = 5,
  process = function,
  scope = "canvas",
  include_continuation = false
}
```

### 额外说明

- 选填参数 `width` 取值范围为 $[1, 65535]$。
- 选填参数 `height` 取值范围为 $[1, 65535]$。

---

## `get_info`

获取插值轨道引擎的完整信息表。

### 调用

```lua
effect.get_info
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 插值轨道引擎 ID |

### 返回值

**该插值轨道引擎存在时**，返回一个值。

| 类型  | 说明                     |
| ----- | ------------------------ |
| table | 该插值轨道引擎信息对象表 |

**该插值轨道引擎不存在时**，返回一个值。

| 类型 | 说明                 |
| ---- | -------------------- |
| nil  | 不存在该插值轨道引擎 |

### 示例

```lua
local ef = effect.create(20, 10)

debug.print(table.pretty(effect.get_info(ef)))
```

**输出：**

```lua
{
  id = "effect_001",
  width = 20,
  height = 10,
  process = function,
  scope = "canvas",
  include_continuation = false
}
```

### 额外说明

- 返回对象表包含以下字段：

```lua
{
  id = "effect_001",           -- string（插值轨道引擎 ID）
  width = 20,                  -- integer（作用区域宽度）
  height = 10,                 -- integer（作用区域高度）
  process = function,          -- function（处理函数）
  scope = "canvas",            -- string（作用层级）
  include_continuation = false -- boolean（占位符是否命中宽字符）
}
```

---

## `exists`

检查插值轨道引擎是否存在。

### 调用

```lua
effect.exists
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明            |
| ------ | ------ | --------------- |
| `id`   | string | 插值轨道引擎 ID |

### 返回值

返回一个值。

| 类型    | 说明                   |
| ------- | ---------------------- |
| boolean | 该插值轨道引擎是否存在 |

### 示例

```lua
local ef = effect.create(20, 10)
debug.print(effect.exists(ef))

effect.delete(ef)
debug.print(effect.exists(ef))
```

**输出：**

```lua
true
false
```
