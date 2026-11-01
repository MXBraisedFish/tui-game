# animation 库

`animation` 提供插值轨道引擎。（编辑中）

---

# 目录

## 常量

| 常量                        | 说明                 | 定位                                                    |
| --------------------------- | -------------------- | ------------------------------------------------------- |
| `STEP`                      | 步进插值函数         | [STEP](#step)                                           |
| `LINEAR`                    | 线性插值函数         | [LINEAR](#linear)                                       |
| `EASE_IN_QUAD`              | 缓入快出插值函数     | [EASE_IN_QUAD](#ease_in_quad)                           |
| `EASE_OUT_QUAD`             | 快入缓出插值函数     | [EASE_OUT_QUAD](#ease_out_quad)                         |
| `EASE_IN_OUT_CUBIC`         | 慢快慢插值函数       | [EASE_IN_OUT_CUBIC](#ease_in_out_cubic)                 |
| `EASE_IN_OUT_REVERSE_CUBIC` | 快慢快插值函数       | [EASE_IN_OUT_REVERSE_CUBIC](#ease_in_out_reverse_cubic) |
| `NORMAL`                    | 正向播放             | [NORMAL](#normal)                                       |
| `REVERSE`                   | 反向播放             | [REVERSE](#reverse)                                     |
| `ALTERNATE`                 | 正反交替播放         | [ALTERNATE](#alternate)                                 |
| `ALTERNATE_REVERSE`         | 反正交替播放         | [ALTERNATE_REVERSE](#alternate_reverse)                 |
| `FORWARDS`                  | 间隔期间取当次尾帧值 | [FORWARDS](#forwards)                                   |
| `BACKWARDS`                 | 间隔期间取下次首帧值 | [BACKWARDS](#backwards)                                 |

## 方法

| 方法       | 说明                             | 定位                  |
| ---------- | -------------------------------- | --------------------- |
| `create`   | 创建一个插值轨道引擎对象         | [create](#create)     |
| `delete`   | 删除指定插值轨道引擎             | [delete](#delete)     |
| `clear`    | 删除所有插值轨道引擎             | [clear](#clear)       |
| `list`     | 获取所有插值轨道引擎信息         | [list](#list)         |
| `count`    | 返回当前插值轨道引擎的总数       | [count](#count)       |
| `sample`   | 获取当前帧插值计算的结果         | [sample](#sample)     |
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
animation.STEP
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_STEP_example](../image/animation_STEP_example.png)

### 等值

```text
"step"
```

---

## `LINEAR`

线性插值函数。

### 调用

```lua
animation.LINEAR
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_LINEAR_example](../image/animation_LINEAR_example.png)

### 等值

```text
"linear"
```

### 额外说明

- 该插值函数要求传递的起始值和结束值必须可参与数学计算。

---

## `EASE_IN_QUAD`

缓入快出插值函数。

### 调用

```lua
animation.EASE_IN_QUAD
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_EASE_IN_QUAD_example](../image/animation_EASE_IN_QUAD_example.png)

### 等值

```text
"ease_in_quad"
```

### 额外说明

- 该插值函数要求传递的起始值和结束值必须可参与数学计算。

---

## `EASE_OUT_QUAD`

快入缓出插值函数。

### 调用

```lua
animation.EASE_OUT_QUAD
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_EASE_OUT_QUAD_example](../image/animation_EASE_OUT_QUAD_example.png)

### 等值

```text
"ease_out_quad"
```

### 额外说明

- 该插值函数要求传递的起始值和结束值必须可参与数学计算。

---

## `EASE_IN_OUT_CUBIC`

慢快慢插值函数。

### 调用

```lua
animation.EASE_IN_OUT_CUBIC
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_EASE_IN_OUT_CUBIC_example](../image/animation_EASE_IN_OUT_CUBIC_example.png)

### 等值

```text
"ease_in_out_cubic"
```

### 额外说明

- 该插值函数要求传递的起始值和结束值必须可参与数学计算。

---

## `EASE_IN_OUT_REVERSE_CUBIC`

快慢快插值函数。

### 调用

```lua
animation.EASE_IN_OUT_REVERSE_CUBIC
```

### 可用于

- 参数 `easing`

### 示例

```lua

```

**输出：**

![animation_EASE_IN_OUT_REVERSE_CUBIC_example](../image/animation_EASE_IN_OUT_REVERSE_CUBIC_example.png)

### 等值

```text
"ease_in_out_reverse_cubic"
```

### 额外说明

- 该插值函数要求传递的起始值和结束值必须可参与数学计算。

---

## `NORMAL`

正向播放。

### 调用

```lua
animation.NORMAL
```

### 可用于

- 参数 `play_direction`

### 示例

```lua

```

**输出：**

![animation_NORMAL_example](../image/animation_NORMAL_example.png)

### 等值

```text
"normal"
```

---

## `REVERSE`

反向播放。

### 调用

```lua
animation.REVERSE
```

### 可用于

- 参数 `play_direction`

### 示例

```lua

```

**输出：**

![animation_REVERSE_example](../image/animation_REVERSE_example.png)

### 等值

```text
"reverse"
```

---

## `ALTERNATE`

正反交替播放。

### 调用

```lua
animation.ALTERNATE
```

### 可用于

- 参数 `play_direction`

### 示例

```lua

```

**输出：**

![animation_ALTERNATE_example](../image/animation_ALTERNATE_example.png)

### 等值

```text
"alternate"
```

### 额外说明

- 奇数次正向播放，偶数次反向播放。

---

## `ALTERNATE_REVERSE`

反正交替播放。

### 调用

```lua
animation.ALTERNATE_REVERSE
```

### 可用于

- 参数 `play_direction`

### 示例

```lua

```

**输出：**

![animation_ALTERNATE_REVERSE_example](../image/animation_ALTERNATE_REVERSE_example.png)

### 等值

```text
"alternate_reverse"
```

### 额外说明

- 偶数次正向播放，奇数次反向播放。

---

## `FORWARDS`

在间隔期间取当次播放尾帧值。

### 调用

```lua
animation.FORWARDS
```

### 可用于

- 参数 `repeat_fill_mode`

### 示例

```lua

```

**输出：**

![animation_FORWARDS_example](../image/animation_FORWARDS_example.png)

### 等值

```text
"forwards"
```

---

## `BACKWARDS`

在间隔期间取下次播放首帧值。

### 调用

```lua
animation.BACKWARDS
```

### 可用于

- 参数 `repeat_fill_mode`

### 示例

```lua

```

**输出：**

![animation_BACKWARDS_example](../image/animation_BACKWARDS_example.png)

### 等值

```text
"forwards"
```

---

# 方法

## `create`

创建一个插值轨道引擎对象。

### 调用

```lua
animation.create
```

### 参数

#### 必填参数

| 参数名     | 类型            | 说明         |
| ---------- | --------------- | ------------ |
| `duration` | integer / float | 总时长（秒） |
| `track`    | table           | 参数轨道     |

#### 选填参数

| 参数名             | 类型                               | 默认值               | 说明                   |
| ------------------ | ---------------------------------- | -------------------- | ---------------------- |
| `easing`           | table / function / const-animation | `animation.STEP`     | 插值函数               |
| `default_easing`   | table / function / const-animation | `animation.STEP`     | 默认回退插值函数       |
| `delay`            | integer / float                    | `0`                  | 首次启动延迟时长（秒） |
| `loop`             | boolean                            | `false`              | 是否循环               |
| `interval`         | integer / float                    | `0`                  | 循环之间间隔时长（秒） |
| `repeat`           | integer / const-math / nil         | `nil`                | 循环次数               |
| `play_direction`   | const-animation                    | `animation.NORMAL`   | 播放方向               |
| `repeat_fill_mode` | const-animation                    | `animation.FORWARDS` | 播放结束后的取值位置   |

### 返回值

返回一个值。

| 类型   | 说明            |
| ------ | --------------- |
| string | 插值轨道引擎 ID |

### 示例

```lua
function Init(ctx)
  local duration = 3
  an = animation.create(duration,
    {
      [1] = {
        timestamp = 0,
        key = {
          pos = 0
        }
      },
      [2] = {
        timestamp = duration,
        key = {
          pos = 10
        }
      }
    },
    {
      easing = animation.EASE_IN_OUT_CUBIC,
      loop = true,
      interval = 0.5,
      play_direction = animation.ALTERNATE
    }
  )
  animation.play(an)
end

function Render()
  local pos = math.floor(animation.sample({ key = "pos" }))
  draw.text(pos, 1, "@", { fg = color.BRIGHT_BLUE })
end
```

**输出：**

![animation_create_example](../image/animation_create_example.gif)

### 额外说明

- 必填参数 `width` 取值范围为 $[1, 65535]$。
- 必填参数 `height` 取值范围为 $[1, 65535]$。
- 选填参数 `repeat_fill_mode` 在选填参数为 `loop` 为 `false` 是恒为 `animation.FORWARDS`，传递 `animation.BACKWARDS` 不会生效。

---

## `delete`

删除指定插值轨道引擎。

### 调用

```lua
animation.delete
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
local an = animation.create(1, {})
debug.print(an)

debug.print(animation.delete(an))

debug.print(table.pretty(animation.list()))
```

**输出：**

```lua
animation_001
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
animation.clear
```

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
animation.create(20, {})
animation.create(30, {})
animation.create(40, {})
animation.create(50, {})

debug.print(animation.clear())

debug.print(table.pretty(animation.list()))
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
animation.list
```

### 返回值

返回一个值。

| 类型  | 说明                         |
| ----- | ---------------------------- |
| table | 所有插值轨道引擎信息的混合表 |

### 示例

```lua
local an = animation.create(1, {})

debug.print(table.pretty(animation.list()))
```

**输出：**

```lua
{
  [1] = {
    id = "animation_001",
    duration = 1,
    track = {},
    easing = "step",
    default_easing = "step",
    delay = 0,
    loop = false,
    interval = 0,
    repeat = nil,
    play_direction = "normal",
    repeat_fill_mode = "forwards"
  },
  n = 1
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  {
    id = "animation_001",         -- string（插值轨道引擎 ID）
    duration = 1,                 -- integer（总时长）
    track = {},                   -- table（参数轨道）
    easing = "step",              -- string / table / function（插值函数）
    default_easing = "step",      -- string / table / function（默认回退插值函数）
    delay = 0,                    -- integer（首次启动延迟时长）
    loop = false,                 -- boolean（是否循环）
    interval = 0,                 -- integer（循环之间间隔时长）
    repeat = nil,                 -- integer / string / nil（循环次数）
    play_direction = "normal",    -- string（播放方向）
    repeat_fill_mode = "forwards" -- string（播放结束后的取值位置）
  },
  n = 1,                          -- integer（插值轨道引擎数量）
}
```

---

## `count`

返回当前插值轨道引擎的总数。

### 调用

```lua
animation.count
```

### 返回值

返回一个值。

| 类型    | 说明             |
| ------- | ---------------- |
| integer | 插值轨道引擎总数 |

### 示例

```lua
animation.create(1, {})
animation.create(1, {})

debug.print(animation.count())
```

**输出：**

```lua
2
```

---

## `sample`

获取当前帧插值计算的结果。

### 调用

```lua
timer.sample
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明          |
| ------ | ------ | ------------- |
| `id`   | string | 插值计算器 ID |

#### 选填参数

| 参数名 | 类型         | 默认值 | 说明               |
| ------ | ------------ | ------ | ------------------ |
| `key`  | string / nil | `nil`  | 获取指定的参数插值 |

### 返回值

**若未填写选填参数 `key`**，返回一个值。

| 类型 | 说明 |
| ---- | ---- |

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
animation.set
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
local ef = animation.create(20, 10)

debug.print(animation.set(ef, { width = 25, height = 5 }))

debug.print(table.pretty(animation.get_info(ef)))
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
animation.get_info
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
local ef = animation.create(20, 10)

debug.print(table.pretty(animation.get_info(ef)))
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
animation.exists
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
local ef = animation.create(20, 10)
debug.print(animation.exists(ef))

animation.delete(ef)
debug.print(animation.exists(ef))
```

**输出：**

```lua
true
false
```
