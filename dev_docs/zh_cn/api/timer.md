# timer 库

`timer` 提供计时器管理。

---

# 目录

## 方法

| 方法       | 说明                         | 定位                  |
| ---------- | ---------------------------- | --------------------- |
| `create`   | 创建一个计时器               | [create](#create)     |
| `delete`   | 删除一个计时器               | [delete](#delete)     |
| `clear`    | 删除所有计时器               | [clear](#clear)       |
| `list`     | 查看所有计时器的信息         | [list](#list)         |
| `count`    | 查看计时器总数               | [count](#count)       |
| `start`    | 开始或继续计时               | [start](#start)       |
| `pause`    | 暂停计时，保留已经走过的时间 | [pause](#pause)       |
| `reset`    | 清零计时进度，回到未启动状态 | [reset](#reset)       |
| `restart`  | 清零计时进度，然后重新开始   | [restart](#restart)   |
| `set`      | 修改一个计时器的参数         | [set](#set)           |
| `get_info` | 查看一个计时器的参数和状态   | [get_info](#get_info) |
| `exists`   | 检查一个计时器是否还在       | [exists](#exists)     |

---

# 方法

## `create`

创建一个计时器。

### 调用

```lua
timer.create
```

### 参数

#### 必填参数

| 参数名     | 类型            | 说明           |
| ---------- | --------------- | -------------- |
| `duration` | integer / float | 计时时长（秒） |

#### 选填参数

| 参数名     | 类型                       | 默认值  | 说明                   |
| ---------- | -------------------------- | ------- | ---------------------- |
| `delay`    | integer / float            | `0`     | 首次启动延迟时长（秒） |
| `loop`     | boolean                    | `false` | 是否循环               |
| `interval` | integer / float            | `0`     | 循环之间间隔时长（秒） |
| `repeat`   | integer / const-math / nil | `nil`   | 循环次数               |
| `callback` | function / nil / `false`   | `nil`   | 接收事件的回调函数     |
| `tip`      | string / nil / `false`     | `nil`   | 随事件返回的自定义信息 |

### 返回值

返回一个值。

| 类型   | 说明      |
| ------ | --------- |
| string | 计时器 ID |

### 示例

```lua
local id = timer.create(1.5, { delay = 0.2, loop = true, ["repeat"] = 3 })
timer.start(id)
```

**输出：**

```lua

```

### 额外说明

- 必填参数 `duration` 取值范围为 $[0, 18446744073709551616]$；单位秒。
- 选填参数 `delay` 取值范围为 $[0, 18446744073709551616]$；单位秒。
- 选填参数 `interval` 取值范围为 $(-\infty, +\infty)$；必须为有限数，单位秒。
- 选填参数 `repeat` 取值范围为 $[1, 4294967295]$；常量 `math.POSITIVE_INFINITE` 表示无限循环。
- 创建后状态为 idle。

---

## `delete`

删除一个计时器。

### 调用

```lua
timer.delete
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
local id = timer.create(1)

debug.print(timer.delete(id))

debug.print(table.pretty(timer.list()))
```

**输出：**

```lua
true
{
  n = 0
}
```

---

## `clear`

删除所有计时器。

### 调用

```lua
timer.clear
```

### 参数

无。

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 删除是否成功 |

### 示例

```lua
timer.create(1)
timer.create(1)
timer.create(1)
timer.create(1)
timer.create(1)

debug.print(timer.clear())

debug.print(table.pretty(timer.list()))
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

查看所有计时器的信息。

### 调用

```lua
timer.list
```

### 参数

无。

### 返回值

返回一个值。

| 类型  | 说明                   |
| ----- | ---------------------- |
| table | 所有计时器信息的混合表 |

### 示例

```lua
timer.create(1)
local timers = timer.list()
debug.print(table.pretty(timers))
```

**输出：**

```lua
{
  [1] = {
    delay = 0,
    duration = 1,
    elapsed = 0,
    executed_count = 0,
    id = "timer_001",
    interval = 0,
    loop = false,
    remaining = 1,
    state = "idle"
  },
  n = 1
}
```

### 额外说明

- 返回值混合表结构如下：

```lua
{
  [1] = {
    id = "timer_001",   -- string（计时器 ID）
    duration = 1,       -- integer / float（计时时长）
    delay = 0,          -- integer / float（首次启动延迟时长）
    loop = false,       -- boolean（是否循环）
    interval = 0,       -- integer / float（循环之间间隔时长）
    state = "idle"      -- string（当前计时器状态）
    elapsed = 0,        -- integer（当轮计时已过时长）
    executed_count = 0, -- integer（当前计时器已循环次数）
    remaining = 1,      -- integer / float（距离下次触发还剩多久）
    repeat = nil,       -- integer / nil / false（循环次数）
    callback = nil,     -- function / nil / false（接收事件的回调函数）
    tip = nil,          -- string / nil / false（随事件返回的自定义信息）
  },
  n = 1                 -- integer（计时器数量）
}
```

---

## `count`

查看计时器总数。

### 调用

```lua
timer.count
```

### 参数

无。

### 返回值

返回一个值。

| 类型    | 说明       |
| ------- | ---------- |
| integer | 计时器总数 |

### 示例

```lua
timer.create(1)
timer.create(2)
debug.print(timer.count())
```

**输出：**

```lua
2
```

---

## `start`

开始或继续计时。

### 调用

```lua
timer.start
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

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

暂停计时，保留已经走过的时间。

### 调用

```lua
timer.pause
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

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

清零计时进度，回到未启动状态。

### 调用

```lua
timer.reset
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

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

清零计时进度，然后重新开始。

### 调用

```lua
timer.restart
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

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

修改一个计时器的参数。

### 调用

```lua
timer.set
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

#### 选填参数

| 参数名     | 类型                       | 默认值   | 说明                   |
| ---------- | -------------------------- | -------- | ---------------------- |
| `duration` | integer / float            | 保持原值 | 计时时长（秒）         |
| `delay`    | integer / float            | 保持原值 | 首次启动延迟时长（秒） |
| `loop`     | boolean                    | 保持原值 | 是否循环               |
| `interval` | integer / float            | 保持原值 | 循环之间间隔时长（秒） |
| `repeat`   | integer / const-math / nil | 保持原值 | 循环次数               |
| `callback` | function / nil / `false`   | 保持原值 | 接收事件的回调函数     |
| `tip`      | string / nil / `false`     | 保持原值 | 随事件返回的自定义信息 |

### 返回值

返回一个值。

| 类型    | 说明         |
| ------- | ------------ |
| boolean | 修改是否成功 |

### 示例

```lua
local id = timer.create(5, { tip = "tip", loop = true, ["repeat"] = 3 })

debug.print(table.pretty(timer.get_info(id)))

timer.set(id, { duration = 2, tip = nil, ["repeat"] = nil })

debug.print(table.pretty(timer.get_info(id)))
```

**输出：**

```lua
{
  id = "timer_001",
  duration = 5,
  delay = 0,
  loop = true,
  interval = 0,
  state = "idle",
  elapsed = 0,
  executed_count = 0,
  remaining = 5,
  repeat = 3,
  tip = "tip"
}
{
  id = "timer_001",
  duration = 2,
  delay = 0,
  loop = true,
  interval = 0,
  state = "idle",
  elapsed = 0,
  executed_count = 0,
  remaining = 2
}
```

### 额外说明

- 必填参数 `duration` 取值范围为 $[0, 18446744073709551616]$；单位秒。
- 选填参数 `delay` 取值范围为 $[0, 18446744073709551616]$；单位秒。
- 选填参数 `interval` 取值范围为 $(-\infty, +\infty)$；必须为有限数，单位秒。
- 选填参数 `repeat` 取值范围为 $[1, 4294967295]$；常量 `math.POSITIVE_INFINITE` 表示无限循环。
- 修改后计时器状态会强制改为 idle。

---

## `get_info`

查看一个计时器的参数和状态。

### 调用

```lua
timer.get_info
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

### 返回值

**计时器存在时**，返回一个值。

| 类型  | 说明                 |
| ----- | -------------------- |
| table | 计时器配置信息对象表 |

**计时器不存在时**，返回一个值。

| 类型 | 说明           |
| ---- | -------------- |
| nil  | 该计时器不存在 |

### 示例

```lua
local id = timer.create(1)
debug.print(table.pretty(timer.get_info(id)))
```

**输出：**

```lua
{
  id = "timer_001",
  duration = 1,
  delay = 0,
  loop = false,
  interval = 0,
  state = "idle",
  elapsed = 0,
  executed_count = 0,
  remaining = 1
}
```

### 额外说明

- 返回值对象表结构如下：

```lua
{
  id = "timer_001",   -- string（计时器 ID）
  duration = 1,       -- integer / float（计时时长）
  delay = 0,          -- integer / float（首次启动延迟时长）
  loop = false,       -- boolean（是否循环）
  interval = 0,       -- integer / float（循环之间间隔时长）
  state = "idle"      -- string（当前计时器状态）
  elapsed = 0,        -- integer（当轮计时已过时长）
  executed_count = 0, -- integer（当前计时器已循环次数）
  remaining = 1,      -- integer / float（距离下次触发还剩多久）
  repeat = nil,       -- integer / nil / false（循环次数）
  callback = nil,     -- function / nil / false（接收事件的回调函数）
  tip = nil,          -- string / nil / false（随事件返回的自定义信息）
}
```

---

## `exists`

检查一个计时器是否还在。

### 调用

```lua
timer.exists
```

### 参数

#### 必填参数

| 参数名 | 类型   | 说明      |
| ------ | ------ | --------- |
| `id`   | string | 计时器 ID |

### 返回值

返回一个值。

| 类型    | 说明     |
| ------- | -------- |
| boolean | 是否存在 |

### 示例

```lua
local id = timer.create(1)
debug.print(timer.exists(id))
```

**输出：**

```lua
true
```
